// task 模块 handlers。
// CRUD 已实现（sys_tasks 落库管理）；Rust 未内嵌 asynq 等价调度器，控制类端点对齐 Go
// 「调度器未配置」降级语义：
// - ListTaskTypeName：返回系统注册的任务类型名（配置元数据，可供任务表单选择）
// - Start/Stop/Restart/ControlTask：500 "task scheduler is not configured"
// 创建/更新时校验 typeName 已在注册类型清单内（对齐 Go TaskTypeExists 校验语义）。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::task::{TaskRepo, TaskRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列
const TASK_COLUMNS: &[&str] = &[
    "id", "tenant_id", "type", "type_name", "task_payload", "cron_spec", "task_options", "enable",
    "remark", "created_at", "updated_at", "created_by", "updated_by",
];

/// 系统注册的任务类型名（对齐 Go asynq mux 注册表，ListTaskTypeName 数据源）。
const REGISTERED_TYPE_NAMES: &[&str] = &[
    "broadcast_message",
    "tenant_expiry_scan",
    "backup",
    "script_task",
    "audit_log_archive",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<i64>,
    /// PERIODIC | DELAY | WAIT_RESULT（proto 枚举名）
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_payload: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron_spec: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_options: Option<Value>,
    pub enable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

pub fn to_dto(row: &TaskRow) -> TaskDto {
    TaskDto {
        id: row.id,
        tenant_id: row.tenant_id,
        r#type: row.r#type.clone(),
        type_name: row.type_name.clone(),
        task_payload: row.task_payload.clone(),
        cron_spec: row.cron_spec.clone(),
        task_options: row.task_options.clone(),
        enable: row.enable,
        remark: row.remark.clone(),
        created_by: row.created_by,
        updated_by: row.updated_by,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
    }
}

// ===================== 入参 =====================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskData {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub type_name: Option<String>,
    /// proto 为 string；兼容前端传 JSON 字符串或直接传对象（落库统一 jsonb）
    #[serde(default)]
    pub task_payload: Option<Value>,
    #[serde(default)]
    pub cron_spec: Option<String>,
    #[serde(default)]
    pub task_options: Option<Value>,
    #[serde(default)]
    pub enable: Option<bool>,
    #[serde(default)]
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskBody {
    #[serde(default)]
    pub data: Option<TaskData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<TaskData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// type 枚举归一（proto 枚举名，DB 枚举值一致）
fn normalize_type(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "PERIODIC" | "DELAY" | "WAIT_RESULT" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid task type: {v}"))),
    }
}

/// typeName 已注册校验（对齐 Go TaskTypeExists，未注册 → 400）。
fn require_registered_type_name(type_name: &str) -> Result<(), AppError> {
    if type_name.trim().is_empty() || !REGISTERED_TYPE_NAMES.contains(&type_name.trim()) {
        return Err(AppError::Validation(format!(
            "task type [{type_name}] is not registered"
        )));
    }
    Ok(())
}

/// task_payload 落库前归一：JSON 字符串解包为对象/数组；对象/数组直接用（jsonb 不双重编码）。
fn payload_to_json(v: &Value) -> Value {
    match v {
        Value::String(s) => serde_json::from_str(s).unwrap_or_else(|_| v.clone()),
        other => other.clone(),
    }
}

// ===================== CRUD handlers =====================

pub async fn task_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, TASK_COLUMNS)?;
    let db = db_of(&state)?;
    let repo = TaskRepo::new(db);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        where_clause = format!(" and {}", crate::query::compile_where(&lq.filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, TASK_COLUMNS)?;
        order_parts.push(format!("\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let order_by = order_parts.join(", ");
    let (rows, total) = repo
        .list(lq.paging.offset(), lq.paging.limit(), &where_clause, &bind, &order_by)
        .await?;
    Ok(json_ok(ListResponse::new(
        rows.iter().map(to_dto).collect(),
        total,
    )))
}

/// GET /tasks/{id}
pub async fn task_get_by_tasks(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = TaskRepo::new(db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("task not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

/// GET /tasks/type-name/{type_name} —— 须具名租户上下文（对齐 Go：平台上下文 400）。
pub async fn task_get(
    State(state): State<AppState>,
    Path(type_name): Path<String>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    if operator.tenant_id == 0 {
        return Err(AppError::Validation(
            "tenant scope required to query task by type name".into(),
        ));
    }
    let repo = TaskRepo::new(db_of(&state)?);
    let row = repo
        .get_by_type_name(operator.tenant_id, &type_name)
        .await?
        .ok_or_else(|| AppError::NotFound("task not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

/// 创建任务：typeName 注册校验 + (tenant_id, type_name) 唯一 + jsonb 字段落库。
/// Rust 无内嵌调度器，enable=true 仅落库（不实际调度），见 MISSING.md。
pub async fn task_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateTaskBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("invalid parameter".into()))?;
    let type_name = data
        .type_name
        .as_deref()
        .map(str::trim)
        .unwrap_or_default()
        .to_string();
    require_registered_type_name(&type_name)?;

    let r#type = data
        .r#type
        .as_deref()
        .map(normalize_type)
        .transpose()?
        .unwrap_or_else(|| "PERIODIC".into());

    let repo = TaskRepo::new(db_of(&state)?);
    if repo.type_name_exists(operator.tenant_id, &type_name, 0).await? {
        return Err(AppError::Validation(format!(
            "task type [{type_name}] already exists, please use a different type or update the existing task"
        )));
    }

    let task_payload = data.task_payload.as_ref().map(payload_to_json);
    let task_options = data.task_options.as_ref().map(|v| v.clone());
    let _ = repo
        .create(
            operator.tenant_id,
            &r#type,
            &type_name,
            task_payload.as_ref(),
            data.cron_spec.as_deref(),
            task_options.as_ref(),
            data.enable.unwrap_or(false),
            data.remark.as_deref(),
            operator.user_id,
        )
        .await?;
    Ok(json_empty())
}

/// 更新任务：typeName 为空时从库中回填（对齐 Go H8，启停开关只提交 enable 也能过校验）；
/// allowMissing=true 且不存在 → 转 Create。
pub async fn task_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateTaskBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    if id == 0 {
        return Err(AppError::Validation("id is required".into()));
    }
    let data = body.data.ok_or_else(|| AppError::Validation("invalid parameter".into()))?;

    let repo = TaskRepo::new(db_of(&state)?);

    // allowMissing=true 且不存在 → 转 Create（对齐 Go：created_by 沿用操作人）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        let type_name = data
            .type_name
            .as_deref()
            .map(str::trim)
            .unwrap_or_default()
            .to_string();
        require_registered_type_name(&type_name)?;
        let r#type = data
            .r#type
            .as_deref()
            .map(normalize_type)
            .transpose()?
            .unwrap_or_else(|| "PERIODIC".into());
        if repo.type_name_exists(operator.tenant_id, &type_name, 0).await? {
            return Err(AppError::Validation(format!(
                "task type [{type_name}] already exists, please use a different type or update the existing task"
            )));
        }
        let _ = repo
            .create(
                operator.tenant_id,
                &r#type,
                &type_name,
                data.task_payload.as_ref().map(payload_to_json).as_ref(),
                data.cron_spec.as_deref(),
                data.task_options.as_ref(),
                data.enable.unwrap_or(false),
                data.remark.as_deref(),
                operator.user_id,
            )
            .await?;
        return Ok(json_empty());
    }

    // 回填 typeName（仅提交 enable 的启停开关场景），再校验注册
    let type_name = match data.type_name.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        Some(v) => v.to_string(),
        None => {
            let old = repo
                .get(id)
                .await?
                .ok_or_else(|| AppError::NotFound("task not found".into()))?;
            old.type_name.unwrap_or_default()
        }
    };
    require_registered_type_name(&type_name)?;

    // (tenant_id, type_name) 唯一（排除自身）
    if repo.type_name_exists(operator.tenant_id, &type_name, id).await? {
        return Err(AppError::Validation(format!(
            "task type [{type_name}] already exists, please use a different type or update the existing task"
        )));
    }

    let r#type = data
        .r#type
        .as_deref()
        .map(normalize_type)
        .transpose()?;

    let updated = repo
        .update(
            id,
            r#type.as_deref(),
            Some(type_name.as_str()),
            data.task_payload.as_ref().map(payload_to_json).as_ref(),
            data.cron_spec.as_deref(),
            data.task_options.as_ref(),
            data.enable,
            data.remark.as_deref(),
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("task not found".into()));
    }
    Ok(json_empty())
}

/// DELETE /tasks/{id} —— 硬删除，未命中 404（对齐 Go Delete 的 NotFound）。
pub async fn task_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = TaskRepo::new(db_of(&state)?);
    let deleted = repo.delete(id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("task not found".into()));
    }
    Ok(json_empty())
}

// ===================== 调度控制（对齐 Go「调度器未配置」降级） =====================

fn scheduler_not_configured() -> AppError {
    AppError::Internal {
        context: "task scheduler is not configured".into(),
        source: None,
    }
}

pub async fn task_control_task(_state: State<AppState>, Json(_body): Json<Value>) -> Result<axum::response::Response, AppError> {
    // ControlTaskRequest { controlType: Start|Stop|Restart, typeName }
    Err(scheduler_not_configured())
}

pub async fn task_restart_all_task(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    Err(scheduler_not_configured())
}

pub async fn task_start_all_task(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    Err(scheduler_not_configured())
}

pub async fn task_stop_all_task(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    Err(scheduler_not_configured())
}

/// GET /tasks:type-names —— 系统注册的任务类型（配置元数据）。
pub async fn task_list_task_type_name(_state: State<AppState>) -> Result<impl IntoResponse, AppError> {
    Ok(json_ok(serde_json::json!({
        "typeNames": REGISTERED_TYPE_NAMES,
    })))
}

// ===================== users:exists / tenants:exists / EditUserPassword =====================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditUserPasswordBody {
    #[serde(default)]
    pub user_id: Option<i64>,
    #[serde(default)]
    pub password: Option<String>,
}

/// POST /users/{user_id}/password —— 管理员改用户密码。
/// 对齐 Go EditUserPassword：AES 解密新密码 → 更新 USERNAME 凭证 → 撤销该用户全部会话。
pub async fn user_edit_password(
    State(state): State<AppState>,
    Path(path_user_id): Path<i64>,
    _operator: Operator,
    Json(body): Json<EditUserPasswordBody>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = body.user_id.filter(|v| *v != 0).unwrap_or(path_user_id);
    if user_id == 0 {
        return Err(AppError::Validation("userId is required".into()));
    }
    let password = body
        .password
        .clone()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| AppError::Validation("password is required".into()))?;

    let db = db_of(&state)?;
    let repo = crate::repos::authentication::AuthenticationRepo::new(db);

    let plain = crate::crypto::decrypt_transport_secret(&password)?;
    let hash = bcrypt::hash(&plain, bcrypt::DEFAULT_COST).map_err(|e| AppError::Internal {
        context: "hash password failed".into(),
        source: Some(Box::new(e)),
    })?;

    let user = repo
        .get_user_by_id(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;
    let username_ident = repo
        .get_username_identifier(user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user credential not found".into()))?;

    let updated = repo
        .update_password_hash(user.tenant_id, &username_ident, &hash)
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("user credential not found".into()));
    }

    for ct in ["admin", "app"] {
        let _ = crate::auth::revoke_all_sessions(&state, ct, user_id, "").await;
    }
    Ok(json_empty())
}

/// GET /users:exists?username=xxx 或 ?id=1 → { exist: bool }
pub async fn user_exists(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let db = db_of(&state)?;
    let id: Option<i64> = params.get("id").and_then(|v| v.parse().ok());
    let username = params.get("username").cloned().unwrap_or_default();

    if id.is_none() && username.trim().is_empty() {
        return Err(AppError::Validation("id or username is required".into()));
    }

    let exist = if let Some(id) = id {
        let row: Option<(i64,)> =
            sqlx::query_as::<sqlx::Any, (i64,)>(
                "select 1 from sys_users where id = $1 and deleted_at is null limit 1",
            )
            .bind(id)
            .fetch_optional(&db)
            .await
            .map_err(|e| AppError::Internal {
                context: "query user existence failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.is_some()
    } else {
        let row: Option<(i64,)> =
            sqlx::query_as::<sqlx::Any, (i64,)>(
                "select 1 from sys_users where username = $1 and deleted_at is null limit 1",
            )
            .bind(username.trim())
            .fetch_optional(&db)
            .await
            .map_err(|e| AppError::Internal {
                context: "query user existence failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.is_some()
    };

    Ok(json_ok(serde_json::json!({ "exist": exist })))
}

/// GET /tenants:exists?code=xxx 或 ?name=xxx → { exist: bool }
pub async fn tenant_exists(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let db = db_of(&state)?;
    let code = params.get("code").cloned().unwrap_or_default();
    let name = params.get("name").cloned().unwrap_or_default();

    if code.trim().is_empty() && name.trim().is_empty() {
        return Err(AppError::Validation("code or name is required".into()));
    }

    let exist = if !code.trim().is_empty() {
        let row: Option<(i64,)> =
            sqlx::query_as::<sqlx::Any, (i64,)>(
                "select 1 from sys_tenants where code = $1 and deleted_at is null limit 1",
            )
            .bind(code.trim())
            .fetch_optional(&db)
            .await
            .map_err(|e| AppError::Internal {
                context: "query tenant existence failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.is_some()
    } else {
        let row: Option<(i64,)> =
            sqlx::query_as::<sqlx::Any, (i64,)>(
                "select 1 from sys_tenants where name = $1 and deleted_at is null limit 1",
            )
            .bind(name.trim())
            .fetch_optional(&db)
            .await
            .map_err(|e| AppError::Internal {
                context: "query tenant existence failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.is_some()
    };

    Ok(json_ok(serde_json::json!({ "exist": exist })))
}

pub fn db_of(state: &AppState) -> Result<sqlx::AnyPool, AppError> {
    state.db.clone().ok_or_else(|| AppError::Internal {
        context: "database not configured".into(),
        source: None,
    })
}
