// language 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、total 为字符串）。
// 端点：GET/POST/DELETE /dict/langs、POST /dict/langs/batch、GET/PUT /dict/langs/{id}。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::language::{LanguageRepo, LanguageRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列
const LANGUAGE_COLUMNS: &[&str] = &[
    "id", "language_code", "language_name", "native_name", "is_default", "is_enabled",
    "sort_order", "created_by", "updated_by", "deleted_by", "created_at", "updated_at",
    "deleted_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguageDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_default: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i64>,
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

pub fn to_dto(row: &LanguageRow) -> LanguageDto {
    LanguageDto {
        id: row.id,
        language_code: row.language_code.clone(),
        language_name: row.language_name.clone(),
        native_name: row.native_name.clone(),
        is_default: row.is_default,
        is_enabled: row.is_enabled,
        sort_order: row.sort_order,
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
pub struct LanguageData {
    #[serde(default)]
    pub language_code: Option<String>,
    #[serde(default)]
    pub language_name: Option<String>,
    #[serde(default)]
    pub native_name: Option<String>,
    #[serde(default)]
    pub is_default: Option<bool>,
    #[serde(default)]
    pub is_enabled: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLanguageBody {
    #[serde(default)]
    pub data: Option<LanguageData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldMask {
    #[serde(default)]
    pub paths: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLanguageBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<LanguageData>,
    #[serde(default)]
    pub update_mask: Option<FieldMask>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// 删除入参：前端 DELETE /dict/langs?id=...（Go 端 Delete 为单条删除）
#[derive(Debug, Deserialize)]
pub struct DeleteQuery {
    #[serde(default)]
    pub id: Option<i64>,
}

/// 批量创建入参：POST /dict/langs/batch {items:[...]}（Go 端原为 501，按 Create 语义实现）
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchCreateLanguagesBody {
    #[serde(default)]
    pub items: Vec<LanguageData>,
}

/// Go 端 ent NotEmpty：language_code/language_name/native_name 均必填非空。
fn validate_data(data: &LanguageData) -> Result<(String, String, String), AppError> {
    let code = data.language_code.as_deref().map(str::trim).unwrap_or("");
    let name = data.language_name.as_deref().map(str::trim).unwrap_or("");
    let native = data.native_name.as_deref().map(str::trim).unwrap_or("");
    if code.is_empty() {
        return Err(AppError::Validation("language code is required".into()));
    }
    if name.is_empty() {
        return Err(AppError::Validation("language name is required".into()));
    }
    if native.is_empty() {
        return Err(AppError::Validation("native name is required".into()));
    }
    Ok((code.to_string(), name.to_string(), native.to_string()))
}

pub async fn language_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, LANGUAGE_COLUMNS)?;
    let repo = LanguageRepo::new(crate::handlers::script::db_of(&state)?);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        // 单表查询无 join，列名不加表前缀（compile_where 会加双引号，带前缀会变成非法限定标识符）
        where_clause = format!(" and {}", crate::query::compile_where(&lq.filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, LANGUAGE_COLUMNS)?;
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

pub async fn language_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateLanguageBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let (code, name, native) = validate_data(&data)?;

    let repo = LanguageRepo::new(crate::handlers::script::db_of(&state)?);
    if repo.language_code_exists(&code, 0).await? {
        return Err(AppError::Conflict(format!(
            "language code already exists: {code}"
        )));
    }
    let _ = repo
        .create(&code, &name, &native, data.is_default, data.is_enabled, data.sort_order, operator.user_id)
        .await?;
    Ok(json_empty())
}

pub async fn language_delete(
    State(state): State<AppState>,
    Query(params): Query<DeleteQuery>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let id = params.id.ok_or_else(|| AppError::Validation("invalid parameter".into()))?;
    let repo = LanguageRepo::new(crate::handlers::script::db_of(&state)?);
    // 0 行命中不报错（对齐 Go repository.Delete）
    let _ = repo.delete(id).await?;
    Ok(json_empty())
}

pub async fn language_batch_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<BatchCreateLanguagesBody>,
) -> Result<impl IntoResponse, AppError> {
    if body.items.is_empty() {
        return Err(AppError::Validation("items is required".into()));
    }
    let repo = LanguageRepo::new(crate::handlers::script::db_of(&state)?);
    for item in &body.items {
        let (code, name, native) = validate_data(item)?;
        if repo.language_code_exists(&code, 0).await? {
            return Err(AppError::Conflict(format!(
                "language code already exists: {code}"
            )));
        }
        let _ = repo
            .create(&code, &name, &native, item.is_default, item.is_enabled, item.sort_order, operator.user_id)
            .await?;
    }
    Ok(json_empty())
}

pub async fn language_get(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = LanguageRepo::new(crate::handlers::script::db_of(&state)?);
    // oneof query_by：有 code 参数按 code 查，否则按 path id 查（对齐 Go Get）
    let row = match params
        .get("code")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
    {
        Some(code) => repo.get_by_code(&code).await?,
        None => repo.get(path_id).await?,
    };
    let row = row.ok_or_else(|| AppError::NotFound("language not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn language_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateLanguageBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;

    let repo = LanguageRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        let (code, name, native) = validate_data(&data)?;
        if repo.language_code_exists(&code, 0).await? {
            return Err(AppError::Conflict(format!(
                "language code already exists: {code}"
            )));
        }
        let _ = repo
            .create(&code, &name, &native, data.is_default, data.is_enabled, data.sort_order, operator.user_id)
            .await?;
        return Ok(json_empty());
    }

    // language_code 为 Immutable 不更新（对齐 Go）；name/native 提供时校验非空
    let mut language_name: Option<String> = None;
    if let Some(v) = &data.language_name {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation("language name is required".into()));
        }
        language_name = Some(trimmed.to_string());
    }
    let mut native_name: Option<String> = None;
    if let Some(v) = &data.native_name {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation("native name is required".into()));
        }
        native_name = Some(trimmed.to_string());
    }

    let updated = repo
        .update(
            id,
            language_name.as_deref(),
            native_name.as_deref(),
            data.is_default,
            data.is_enabled,
            data.sort_order,
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("language not found".into()));
    }
    Ok(json_empty())
}
