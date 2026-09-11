// dict_type 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、total 为字符串）。
// 端点：GET/POST/DELETE /dict/types、GET /dict/types/code/{code}、GET/PUT /dict/types/{id}。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::dict_type::{DictTypeRepo, DictTypeRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列
const DICT_TYPE_COLUMNS: &[&str] = &[
    "id", "type_code", "type_name", "is_enabled", "sort_order", "tenant_id", "created_by",
    "updated_by", "created_at", "updated_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DictTypeDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i64>,
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

pub fn to_dto(row: &DictTypeRow) -> DictTypeDto {
    DictTypeDto {
        id: row.id,
        type_code: row.type_code.clone(),
        type_name: row.type_name.clone(),
        is_enabled: row.is_enabled,
        sort_order: row.sort_order,
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
pub struct DictTypeData {
    #[serde(default)]
    pub type_code: Option<String>,
    #[serde(default)]
    pub type_name: Option<String>,
    #[serde(default)]
    pub is_enabled: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default)]
    pub tenant_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDictTypeBody {
    #[serde(default)]
    pub data: Option<DictTypeData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDictTypeBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<DictTypeData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// 删除入参：前端 DELETE /dict/types?ids=1&ids=2（repeated query param）
#[derive(Debug, Deserialize)]
pub struct DeleteIdsQuery {
    #[serde(default)]
    pub ids: Vec<i64>,
}

/// Go 端 service 校验：data 非空；type_code/type_name 非空（ent NotEmpty）。
fn validate_data(data: &DictTypeData) -> Result<(), AppError> {
    if data.type_code.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::Validation("dict type code is required".into()));
    }
    if data.type_name.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::Validation("dict type name is required".into()));
    }
    Ok(())
}

pub async fn dict_type_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, DICT_TYPE_COLUMNS)?;
    let repo = DictTypeRepo::new(crate::handlers::script::db_of(&state)?);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        // 编译时把列名加 t. 前缀，避免与 join 表的同名列歧义
        let mut filters = lq.filters.clone();
        for f in &mut filters {
            f.column = format!("t.{}", f.column);
        }
        where_clause = format!(" and {}", crate::query::compile_where(&filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, DICT_TYPE_COLUMNS)?;
        order_parts.push(format!("\"t\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
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

pub async fn dict_type_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateDictTypeBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    validate_data(&data)?;
    let type_code = data.type_code.clone().unwrap_or_default();
    let type_name = data.type_name.clone().unwrap_or_default();

    let repo = DictTypeRepo::new(crate::handlers::script::db_of(&state)?);
    if repo.type_code_exists(type_code.trim(), 0).await? {
        return Err(AppError::Conflict(format!(
            "dict type code already exists: {}",
            type_code.trim()
        )));
    }
    let tenant_id = data.tenant_id.or_else(|| {
        if operator.tenant_id > 0 {
            Some(operator.tenant_id)
        } else {
            None
        }
    });
    let _ = repo
        .create(
            type_code.trim(),
            type_name.trim(),
            data.is_enabled,
            data.sort_order,
            tenant_id,
            operator.user_id,
        )
        .await?;
    Ok(json_empty())
}

pub async fn dict_type_delete(
    State(state): State<AppState>,
    Query(params): Query<DeleteIdsQuery>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    if params.ids.is_empty() {
        return Err(AppError::Validation("invalid parameter".into()));
    }
    let repo = DictTypeRepo::new(crate::handlers::script::db_of(&state)?);
    let _ = repo.batch_delete(&params.ids).await?;
    Ok(json_empty())
}

pub async fn dict_type_get(
    State(state): State<AppState>,
    Path(code): Path<String>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    // 对齐 Go：type_code 仅在 (tenant_id, type_code) 维度唯一，平台上下文(tid=0)下按 code 查询
    // 会跨租户匹配多行且泄露其他租户字典类型，仅允许具名租户上下文查询。
    if operator.tenant_id <= 0 {
        return Err(AppError::Validation(
            "tenant scope required to query dict type by code".into(),
        ));
    }
    let repo = DictTypeRepo::new(crate::handlers::script::db_of(&state)?);
    let row = repo
        .get_by_code(&code)
        .await?
        .ok_or_else(|| AppError::NotFound("dict type not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn dict_type_get_by_types(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = DictTypeRepo::new(crate::handlers::script::db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("dict type not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn dict_type_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateDictTypeBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;

    let repo = DictTypeRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        validate_data(&data)?;
        let type_code = data.type_code.clone().unwrap_or_default();
        let type_name = data.type_name.clone().unwrap_or_default();
        if repo.type_code_exists(type_code.trim(), 0).await? {
            return Err(AppError::Conflict(format!(
                "dict type code already exists: {}",
                type_code.trim()
            )));
        }
        let tenant_id = data.tenant_id.or_else(|| {
            if operator.tenant_id > 0 {
                Some(operator.tenant_id)
            } else {
                None
            }
        });
        let _ = repo
            .create(
                type_code.trim(),
                type_name.trim(),
                data.is_enabled,
                data.sort_order,
                tenant_id,
                operator.user_id,
            )
            .await?;
        return Ok(json_empty());
    }

    // type_code 为 Immutable 不更新（对齐 Go）；提供 type_name 时校验非空
    let mut type_name: Option<String> = None;
    if let Some(v) = &data.type_name {
        let trimmed = v.trim();
        if trimmed.is_empty() {
            return Err(AppError::Validation("dict type name is required".into()));
        }
        type_name = Some(trimmed.to_string());
    }

    let updated = repo
        .update(id, type_name.as_deref(), data.is_enabled, data.sort_order, operator.user_id)
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("dict type not found".into()));
    }
    Ok(json_empty())
}
