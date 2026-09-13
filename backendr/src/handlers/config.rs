// config 模块 handlers（wire 对齐 Go：camelCase、valueType 枚举 STRING/BOOL/INT）。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::config::{ConfigRepo, ConfigRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

const CONFIG_COLUMNS: &[&str] = &[
    "id",
    "name",
    "key",
    "value",
    "value_type",
    "is_built_in",
    "created_at",
    "updated_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// STRING | BOOL | INT
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_built_in: Option<bool>,
}

fn to_dto(r: &ConfigRow) -> ConfigDto {
    ConfigDto {
        id: r.id,
        name: r.name.clone(),
        key: r.key.clone(),
        value: r.value.clone(),
        value_type: r.value_type.clone(),
        is_built_in: r.is_built_in,
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigData {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub key: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub value_type: Option<String>,
    #[serde(default)]
    pub is_built_in: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConfigBody {
    #[serde(default)]
    pub data: Option<ConfigData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<ConfigData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

fn normalize_value_type(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "STRING" | "BOOL" | "INT" => Ok(up),
        _ => Err(AppError::Validation(format!(
            "invalid config value type: {v}"
        ))),
    }
}

pub async fn config_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, CONFIG_COLUMNS)?;
    let repo = ConfigRepo::new(crate::handlers::task::db_of(&state)?);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        let mut filters = lq.filters.clone();
        for f in &mut filters {
            f.column = format!("t.{}", f.column);
        }
        where_clause = format!(" and {}", crate::query::compile_where(&filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, CONFIG_COLUMNS)?;
        order_parts.push(format!("\"t\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let (rows, total) = repo
        .list(
            lq.paging.offset(),
            lq.paging.limit(),
            &where_clause,
            &bind,
            &order_parts.join(", "),
        )
        .await?;
    Ok(json_ok(ListResponse::new(
        rows.iter().map(to_dto).collect(),
        total,
    )))
}

pub async fn config_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = ConfigRepo::new(crate::handlers::task::db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("config not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn config_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateConfigBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("invalid parameter".into()))?;
    let key = data
        .key
        .clone()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::Validation("key is required".into()))?;
    let value_type = data
        .value_type
        .as_deref()
        .map(normalize_value_type)
        .transpose()?
        .unwrap_or_else(|| "STRING".into());
    let repo = ConfigRepo::new(crate::handlers::task::db_of(&state)?);
    repo.create(
        data.name.as_deref(),
        &key,
        data.value.as_deref(),
        &value_type,
        data.is_built_in.unwrap_or(false),
        operator.user_id,
    )
    .await?;
    Ok(json_empty())
}

pub async fn config_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateConfigBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    if id <= 0 {
        return Err(AppError::Validation("id is required".into()));
    }
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("invalid parameter".into()))?;
    let value_type = data
        .value_type
        .as_deref()
        .map(normalize_value_type)
        .transpose()?;
    let repo = ConfigRepo::new(crate::handlers::task::db_of(&state)?);
    // allowMissing=true 且不存在 → 转 Create
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        let key = data
            .key
            .clone()
            .filter(|v| !v.is_empty())
            .ok_or_else(|| AppError::Validation("key is required".into()))?;
        repo.create(
            data.name.as_deref(),
            &key,
            data.value.as_deref(),
            value_type.as_deref().unwrap_or("STRING"),
            data.is_built_in.unwrap_or(false),
            operator.user_id,
        )
        .await?;
        return Ok(json_empty());
    }
    let updated = repo
        .update(
            id,
            data.name.as_deref(),
            data.key.as_deref().filter(|v| !v.is_empty()),
            data.value.as_deref(),
            value_type.as_deref(),
            data.is_built_in,
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("config not found".into()));
    }
    Ok(json_empty())
}

pub async fn config_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = ConfigRepo::new(crate::handlers::task::db_of(&state)?);
    repo.delete(id).await?;
    Ok(json_empty())
}
