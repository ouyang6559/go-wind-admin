// task 模块 handlers。
// Rust 后端未内嵌 asynq 等价调度器，控制类端点对齐 Go「调度器未配置」的降级语义：
// - ListTaskTypeName：返回系统注册的任务类型名（配置元数据，可供任务表单选择）
// - Start/Stop/Restart/ControlTask：500 "task scheduler is not configured"
// sys_tasks 的 CRUD（List/Get/Create/Update/Delete）依赖调度器校验，见 MISSING.md。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::response::{json_empty, json_ok};
use crate::state::AppState;

pub async fn task_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    Err(AppError::NotImplemented)
}

pub async fn task_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    Err(AppError::NotImplemented)
}

pub async fn task_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    Err(AppError::NotImplemented)
}

pub async fn task_get_by_tasks(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    Err(AppError::NotImplemented)
}

pub async fn task_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    Err(AppError::NotImplemented)
}

pub async fn task_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    Err(AppError::NotImplemented)
}

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

/// GET /tasks:type-names —— 对齐 Go asynq mux 注册的系统任务类型（配置元数据）。
pub async fn task_list_task_type_name(_state: State<AppState>) -> Result<impl IntoResponse, AppError> {
    Ok(json_ok(serde_json::json!({
        "typeNames": [
            "broadcast_message",
            "tenant_expiry_scan",
            "backup",
            "script_task",
            "audit_log_archive"
        ]
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
