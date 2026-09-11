// script 模块 handlers（wire 对齐 Go：裸 DTO，language 为枚举名 LUA/JAVASCRIPT）。
// TestRun 需要脚本执行引擎（gopher-lua/goja 的 Rust 等价物），见 MISSING.md。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::script::{ScriptRepo, ScriptRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列
const SCRIPT_COLUMNS: &[&str] = &[
    "id", "name", "language", "hook_point", "priority", "critical", "is_enabled",
    "version", "created_at", "updated_at", "created_by", "updated_by", "description", "source",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptDto {
    pub id: i64,
    pub name: String,
    pub language: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hook_point: Option<String>,
    pub source: String,
    pub priority: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub critical: bool,
    pub version: i64,
    pub is_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

pub fn to_dto(row: &ScriptRow) -> ScriptDto {
    ScriptDto {
        id: row.id,
        name: row.name.clone(),
        language: row.language.clone(),
        hook_point: row.hook_point.clone(),
        source: row.source.clone(),
        priority: row.priority,
        description: row.description.clone(),
        critical: row.critical,
        version: row.version,
        is_enabled: row.is_enabled,
        created_by: row.created_by,
        updated_by: row.updated_by,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptData {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub hook_point: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub priority: Option<i64>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub critical: Option<bool>,
    #[serde(default)]
    pub is_enabled: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateScriptBody {
    #[serde(default)]
    pub data: Option<ScriptData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateScriptBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<ScriptData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

fn normalize_language(lang: &str) -> Result<String, AppError> {
    match lang.to_ascii_uppercase().as_str() {
        "LUA" => Ok("LUA".into()),
        "JAVASCRIPT" | "JS" => Ok("JAVASCRIPT".into()),
        other => Err(AppError::Validation(format!(
            "unsupported script language: {other}"
        ))),
    }
}

/// Go validateDraft：name/source 非空 + language 引擎支持
fn validate_draft(data: &ScriptData) -> Result<(), AppError> {
    if data.name.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::Validation("script name is required".into()));
    }
    if data.source.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::Validation("script source is required".into()));
    }
    let lang = data.language.as_deref().unwrap_or("LUA");
    normalize_language(lang)?;
    Ok(())
}

pub async fn script_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, SCRIPT_COLUMNS)?;
    let db = db_of(&state)?;
    let repo = ScriptRepo::new(db);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        where_clause = format!(" and {}", crate::query::compile_where(&lq.filters, &mut bind));
    }
    // orderBy 编译（列已白名单校验）
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, SCRIPT_COLUMNS)?;
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

pub async fn script_count(
    State(state): State<AppState>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = ScriptRepo::new(db_of(&state)?);
    let count = repo.count().await?;
    Ok(json_ok(serde_json::json!({ "count": count.to_string() })))
}

pub async fn script_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = ScriptRepo::new(db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("script not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn script_get_by_name(
    State(state): State<AppState>,
    Path(name): Path<String>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = ScriptRepo::new(db_of(&state)?);
    let row = repo
        .get_by_name(&name)
        .await?
        .ok_or_else(|| AppError::NotFound("script not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn script_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateScriptBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    validate_draft(&data)?;
    let name = data.name.clone().unwrap_or_default();
    let lang = normalize_language(data.language.as_deref().unwrap_or("LUA"))?;

    let repo = ScriptRepo::new(db_of(&state)?);
    if repo.name_exists(name.trim(), 0).await? {
        return Err(AppError::Conflict(format!(
            "script name already exists: {}",
            name.trim()
        )));
    }
    let id = repo
        .create(
            name.trim(),
            &lang,
            data.hook_point.as_deref(),
            data.source.as_deref().unwrap_or_default().trim(),
            data.priority.unwrap_or(0),
            data.description.as_deref(),
            data.critical.unwrap_or(false),
            data.is_enabled.unwrap_or(true),
            operator.user_id,
        )
        .await?;
    // 对齐 Go：Empty 响应（{}），不在响应体回传实体
    let _ = id;
    Ok(json_empty())
}

pub async fn script_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateScriptBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;

    let repo = ScriptRepo::new(db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        validate_draft(&data)?;
        let name = data.name.clone().unwrap_or_default();
        let lang = normalize_language(data.language.as_deref().unwrap_or("LUA"))?;
        if repo.name_exists(name.trim(), 0).await? {
            return Err(AppError::Conflict(format!(
                "script name already exists: {}",
                name.trim()
            )));
        }
        repo.create(
            name.trim(),
            &lang,
            data.hook_point.as_deref(),
            data.source.as_deref().unwrap_or_default().trim(),
            data.priority.unwrap_or(0),
            data.description.as_deref(),
            data.critical.unwrap_or(false),
            data.is_enabled.unwrap_or(true),
            operator.user_id,
        )
        .await?;
        return Ok(json_empty());
    }

    // 仅当提供 name 才做 draft 校验 + 唯一性检查（对齐 Go validate on name only）
    let mut name: Option<String> = None;
    let mut lang: Option<String> = None;
    if data.name.is_some() {
        validate_draft(&data)?;
        name = Some(data.name.clone().unwrap_or_default().trim().to_string());
        if repo.name_exists(name.as_deref().unwrap_or(""), id).await? {
            return Err(AppError::Conflict(format!(
                "script name already exists: {}",
                name.as_deref().unwrap_or("")
            )));
        }
    }
    if data.language.is_some() {
        lang = Some(normalize_language(data.language.as_deref().unwrap())?);
    }

    let updated = repo
        .update(
            id,
            name.as_deref(),
            lang.as_deref(),
            data.hook_point.as_deref(),
            data.source.as_deref(),
            data.priority,
            data.description.as_deref(),
            data.critical,
            data.is_enabled,
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("script not found".into()));
    }
    Ok(json_empty())
}

#[derive(Debug, Deserialize)]
pub struct ScriptDeleteQuery {
    #[serde(default)]
    pub ids: Option<String>,
}

pub async fn script_delete(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    // DELETE /scripts?ids=1&ids=2（repeated query）；HashMap 只留最后一个，
    // 兼容逗号分隔的 ids=1,2
    let raw = params.get("ids").cloned().unwrap_or_default();
    let ids: Vec<i64> = raw
        .split(',')
        .filter_map(|s| s.trim().parse::<i64>().ok())
        .collect();
    if ids.is_empty() {
        return Err(AppError::Validation("ids is required".into()));
    }
    let repo = ScriptRepo::new(db_of(&state)?);
    repo.delete_ids(&ids).await?;
    Ok(json_empty())
}

/// POST /scripts/test_run —— 需要脚本执行引擎（Lua/JS 沙箱），Rust 端暂未实现。
/// 见 MISSING.md（gin 等价引擎选型：mlua / rquickjs）。
pub async fn script_test_run(
    State(_state): State<AppState>,
    _operator: Operator,
    Json(_body): Json<Value>,
) -> Result<axum::response::Response, AppError> {
    Err(AppError::NotImplemented)
}

/// GET /script/hooks —— Go 端为运行时注册表动态聚合；Rust 端无脚本引擎常驻
/// 注册表，返回空集合 + 语言列表（与 Go 空库实例行为一致）。
pub async fn script_list_hook_points(
    State(_state): State<AppState>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    Ok(json_ok(serde_json::json!({
        "items": [],
        "languages": ["lua", "javascript"],
    })))
}

pub fn db_of(state: &AppState) -> Result<sqlx::AnyPool, AppError> {
    state.db.clone().ok_or_else(|| AppError::Internal {
        context: "database not configured".into(),
        source: None,
    })
}
