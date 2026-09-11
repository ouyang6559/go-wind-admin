// internal_message_category 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略）。
// code 在 Go 端为租户内唯一（Unique index），Rust 端以全局唯一近似（code_exists）。
// 创建时缺省 isEnabled=true（IsEnabled mixin 默认值）、sortOrder=0。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::internal_message_category::{InternalMessageCategoryRepo, InternalMessageCategoryRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列
const CATEGORY_COLUMNS: &[&str] = &[
    "id", "name", "code", "icon_url", "sort_order", "is_enabled", "tenant_id", "created_by",
    "updated_by", "created_at", "updated_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InternalMessageCategoryDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_name: Option<String>,
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

pub fn to_dto(row: &InternalMessageCategoryRow) -> InternalMessageCategoryDto {
    InternalMessageCategoryDto {
        id: row.id,
        name: row.name.clone(),
        code: row.code.clone(),
        icon_url: row.icon_url.clone(),
        sort_order: row.sort_order,
        is_enabled: row.is_enabled,
        tenant_id: row.tenant_id,
        tenant_name: row.tenant_name.clone(),
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
pub struct InternalMessageCategoryData {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub icon_url: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default)]
    pub is_enabled: Option<bool>,
    #[serde(default)]
    pub tenant_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInternalMessageCategoryBody {
    #[serde(default)]
    pub data: Option<InternalMessageCategoryData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInternalMessageCategoryBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<InternalMessageCategoryData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// Go 端无显式校验（ent name/code NotEmpty 兜底），Rust 端校验必填避免落库 500。
fn validate_data(data: &InternalMessageCategoryData) -> Result<(), AppError> {
    if data.name.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::Validation("category name is required".into()));
    }
    if data.code.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::Validation("category code is required".into()));
    }
    Ok(())
}

pub async fn internal_message_category_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, CATEGORY_COLUMNS)?;
    let repo = InternalMessageCategoryRepo::new(crate::handlers::script::db_of(&state)?);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        let mut filters = lq.filters.clone();
        for f in &mut filters {
            f.column = format!("c.{}", f.column);
        }
        where_clause = format!(" and {}", crate::query::compile_where(&filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, CATEGORY_COLUMNS)?;
        order_parts.push(format!("\"c\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
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

pub async fn internal_message_category_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = InternalMessageCategoryRepo::new(crate::handlers::script::db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("internal message category not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn internal_message_category_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateInternalMessageCategoryBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("data is required".into()))?;
    validate_data(&data)?;
    let name = data.name.clone().unwrap_or_default().trim().to_string();
    let code = data.code.clone().unwrap_or_default().trim().to_string();

    let repo = InternalMessageCategoryRepo::new(crate::handlers::script::db_of(&state)?);
    if repo.code_exists(&code, 0).await? {
        return Err(AppError::Conflict(format!(
            "internal message category code already exists: {code}"
        )));
    }
    let tenant_id = data.tenant_id.or_else(|| {
        if operator.tenant_id > 0 {
            Some(operator.tenant_id)
        } else {
            None
        }
    });
    repo.create(
        &name,
        &code,
        data.icon_url.as_deref(),
        data.sort_order.or(Some(0)),
        Some(data.is_enabled.unwrap_or(true)),
        tenant_id,
        operator.user_id,
    )
    .await?;
    Ok(json_empty())
}

pub async fn internal_message_category_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateInternalMessageCategoryBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("data is required".into()))?;

    let repo = InternalMessageCategoryRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        validate_data(&data)?;
        let name = data.name.clone().unwrap_or_default().trim().to_string();
        let code = data.code.clone().unwrap_or_default().trim().to_string();
        if repo.code_exists(&code, 0).await? {
            return Err(AppError::Conflict(format!(
                "internal message category code already exists: {code}"
            )));
        }
        let tenant_id = data.tenant_id.or_else(|| {
            if operator.tenant_id > 0 {
                Some(operator.tenant_id)
            } else {
                None
            }
        });
        repo.create(
            &name,
            &code,
            data.icon_url.as_deref(),
            data.sort_order.or(Some(0)),
            Some(data.is_enabled.unwrap_or(true)),
            tenant_id,
            operator.user_id,
        )
        .await?;
        return Ok(json_empty());
    }

    // 提供 name/code 时做必填校验 + code 唯一性检查（排除自身）
    let mut name: Option<String> = None;
    let mut code: Option<String> = None;
    if data.name.is_some() || data.code.is_some() {
        validate_data(&data)?;
        name = data.name.clone().map(|v| v.trim().to_string());
        code = data.code.clone().map(|v| v.trim().to_string());
        if let Some(c) = &code {
            if repo.code_exists(c, id).await? {
                return Err(AppError::Conflict(format!(
                    "internal message category code already exists: {c}"
                )));
            }
        }
    }

    let updated = repo
        .update(
            id,
            name.as_deref(),
            code.as_deref(),
            data.icon_url.as_deref(),
            data.sort_order,
            data.is_enabled,
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("internal message category not found".into()));
    }
    Ok(json_empty())
}

pub async fn internal_message_category_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = InternalMessageCategoryRepo::new(crate::handlers::script::db_of(&state)?);
    let deleted = repo.delete(id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("internal message category not found".into()));
    }
    Ok(json_empty())
}
