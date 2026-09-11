// api 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、枚举为 proto 名）。
// SyncApis 在 Rust 端无 proto 注册表/OpenAPI 文档可重建，且 sys_apis 是租户闸门依据
// （AGENTS.md：已部署实例勿清空），故实现为幂等 no-op；GetWalkRouteData 无法静态枚举
// axum Router 路径，返回空清单。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::api::{ApiRepo, ApiRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列（本表真实列名，snake_case）
const API_COLUMNS: &[&str] = &[
    "id", "operation", "path", "method", "module", "module_description", "business_module",
    "description", "scope", "status", "created_by", "updated_by", "created_at", "updated_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_description: Option<String>,
    /// proto 枚举名（MODULE_DASHBOARD 等）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// ADMIN | APP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    /// ON | OFF
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
}

/// DB 值（DASHBOARD）→ proto 枚举名（MODULE_DASHBOARD）
fn module_from_db(v: &str) -> String {
    format!("MODULE_{v}")
}

/// proto 枚举名（MODULE_DASHBOARD）→ DB 值（DASHBOARD）；MODULE_UNSPECIFIED/空 → None
fn module_to_db(v: &str) -> Result<Option<String>, AppError> {
    let up = v.to_ascii_uppercase();
    let name = up.strip_prefix("MODULE_").unwrap_or(&up);
    match name {
        "" | "UNSPECIFIED" => Ok(None),
        "DASHBOARD" | "OPM" | "SYSTEM" | "DICT" | "TENANT" | "PERMISSION" | "LOG"
        | "INTERNAL_MESSAGE" | "FILE" | "TASK" => Ok(Some(name.to_string())),
        _ => Err(AppError::Validation(format!("invalid api business module: {v}"))),
    }
}

/// scope 归一：仅接受 ADMIN/APP
fn normalize_scope(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "ADMIN" | "APP" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid api scope: {v}"))),
    }
}

/// status 归一：仅接受 ON/OFF
fn normalize_status(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "ON" | "OFF" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid api status: {v}"))),
    }
}

pub fn to_dto(row: &ApiRow) -> ApiDto {
    ApiDto {
        id: row.id,
        operation: row.operation.clone(),
        path: row.path.clone(),
        method: row.method.clone(),
        module: row.module.clone(),
        module_description: row.module_description.clone(),
        business_module: row.business_module.as_deref().map(module_from_db),
        description: row.description.clone(),
        scope: row.scope.clone(),
        status: row.status.clone(),
        created_by: row.created_by,
        updated_by: row.updated_by,
        deleted_by: row.deleted_by,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        deleted_at: row.deleted_at.clone(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiData {
    #[serde(default)]
    pub operation: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub module: Option<String>,
    #[serde(default)]
    pub module_description: Option<String>,
    #[serde(default)]
    pub business_module: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiBody {
    #[serde(default)]
    pub data: Option<ApiData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApiBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<ApiData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// Go 端 service 校验：data 非空；枚举归一（scope/status/businessModule）
fn validate_data(data: &ApiData) -> Result<(), AppError> {
    if let Some(s) = &data.scope {
        normalize_scope(s)?;
    }
    if let Some(s) = &data.status {
        normalize_status(s)?;
    }
    if let Some(m) = &data.business_module {
        module_to_db(m)?;
    }
    Ok(())
}

pub async fn api_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, API_COLUMNS)?;
    let db = crate::handlers::script::db_of(&state)?;
    let repo = ApiRepo::new(db);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        let mut filters = lq.filters.clone();
        for f in &mut filters {
            f.column = format!("a.{}", f.column);
        }
        where_clause = format!(" and {}", crate::query::compile_where(&filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, API_COLUMNS)?;
        order_parts.push(format!("\"a\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
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

pub async fn api_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = ApiRepo::new(crate::handlers::script::db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("api not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn api_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateApiBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    validate_data(&data)?;
    let path = data.path.clone().unwrap_or_default();
    let method = data.method.clone().unwrap_or_default();
    if path.trim().is_empty() || method.trim().is_empty() {
        return Err(AppError::Validation("path and method are required".into()));
    }

    let repo = ApiRepo::new(crate::handlers::script::db_of(&state)?);
    if repo.endpoint_exists(path.trim(), method.trim(), 0).await? {
        return Err(AppError::Conflict(format!(
            "api endpoint already exists: {method} {path}"
        )));
    }
    let business_module = match &data.business_module {
        Some(m) => module_to_db(m)?,
        None => None,
    };
    let scope = data.scope.as_deref().map(normalize_scope).transpose()?;
    let status = data.status.as_deref().map(normalize_status).transpose()?;
    repo.create(
        data.operation.as_deref(),
        Some(path.trim()),
        Some(method.trim()),
        data.module.as_deref(),
        data.module_description.as_deref(),
        business_module.as_deref(),
        data.description.as_deref(),
        scope.as_deref().or(Some("ADMIN")),
        status.as_deref().or(Some("ON")),
        operator.user_id,
    )
    .await?;
    Ok(json_empty())
}

pub async fn api_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateApiBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;

    let repo = ApiRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        validate_data(&data)?;
        let path = data.path.clone().unwrap_or_default();
        let method = data.method.clone().unwrap_or_default();
        if path.trim().is_empty() || method.trim().is_empty() {
            return Err(AppError::Validation("path and method are required".into()));
        }
        if repo.endpoint_exists(path.trim(), method.trim(), 0).await? {
            return Err(AppError::Conflict(format!(
                "api endpoint already exists: {method} {path}"
            )));
        }
        let business_module = match &data.business_module {
            Some(m) => module_to_db(m)?,
            None => None,
        };
        let scope = data.scope.as_deref().map(normalize_scope).transpose()?;
        let status = data.status.as_deref().map(normalize_status).transpose()?;
        repo.create(
            data.operation.as_deref(),
            Some(path.trim()),
            Some(method.trim()),
            data.module.as_deref(),
            data.module_description.as_deref(),
            business_module.as_deref(),
            data.description.as_deref(),
            scope.as_deref().or(Some("ADMIN")),
            status.as_deref().or(Some("ON")),
            operator.user_id,
        )
        .await?;
        return Ok(json_empty());
    }

    validate_data(&data)?;
    let mut path: Option<String> = None;
    let mut method: Option<String> = None;
    if let Some(p) = &data.path {
        let p = p.trim();
        if p.is_empty() {
            return Err(AppError::Validation("path is required".into()));
        }
        path = Some(p.to_string());
    }
    if let Some(m) = &data.method {
        let m = m.trim();
        if m.is_empty() {
            return Err(AppError::Validation("method is required".into()));
        }
        method = Some(m.to_string());
    }
    if let (Some(p), Some(m)) = (&path, &method) {
        if repo.endpoint_exists(p, m, id).await? {
            return Err(AppError::Conflict(format!(
                "api endpoint already exists: {m} {p}"
            )));
        }
    }
    let business_module = match &data.business_module {
        Some(m) => module_to_db(m)?,
        None => None,
    };
    let scope = data.scope.as_deref().map(normalize_scope).transpose()?;
    let status = data.status.as_deref().map(normalize_status).transpose()?;

    let updated = repo
        .update(
            id,
            data.operation.as_deref(),
            path.as_deref(),
            method.as_deref(),
            data.module.as_deref(),
            data.module_description.as_deref(),
            business_module.as_deref(),
            data.description.as_deref(),
            scope.as_deref(),
            status.as_deref(),
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("api not found".into()));
    }
    Ok(json_empty())
}

pub async fn api_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = ApiRepo::new(crate::handlers::script::db_of(&state)?);
    let deleted = repo.delete(id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("api not found".into()));
    }
    Ok(json_empty())
}

/// POST /apis/sync：Rust 端无 proto/OpenAPI 注册表可重建，且 sys_apis 为租户闸门依据，
/// 实现为幂等 no-op（保留现有数据），返回 google.protobuf.Empty。
pub async fn api_sync_apis(
    State(_state): State<AppState>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    Ok(json_empty())
}

/// GET /apis/walk-route：axum Router 无法静态枚举路径，返回空清单（对齐 Go 返回 ListApiResponse）。
pub async fn api_get_walk_route_data(
    State(_state): State<AppState>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    Ok(json_ok(ListResponse::new(Vec::<ApiDto>::new(), 0)))
}
