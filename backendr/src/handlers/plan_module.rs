// plan_module 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、枚举为 proto 名）。
// module 枚举（identity.service.v1.Module）：DASHBOARD/OPM/SYSTEM/DICT/TENANT/PERMISSION/LOG/
// INTERNAL_MESSAGE/FILE/TASK；proto 零值 MODULE_UNSPECIFIED → 不设置（对齐 Go 跳过）。
// DELETE /plan-modules 为集合路径删除：对齐前端生成 client 的 query `?id=` 行为。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::plan_module::{PlanModuleRepo, PlanModuleRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列
const PLAN_MODULE_COLUMNS: &[&str] = &[
    "id", "plan_id", "module", "created_by", "updated_by", "deleted_by", "created_at",
    "updated_at", "deleted_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanModuleDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
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

pub fn to_dto(row: &PlanModuleRow) -> PlanModuleDto {
    PlanModuleDto {
        id: row.id,
        plan_id: row.plan_id,
        module: row.module.clone(),
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
pub struct PlanModuleData {
    #[serde(default)]
    pub plan_id: Option<i64>,
    #[serde(default)]
    pub module: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlanModuleBody {
    #[serde(default)]
    pub data: Option<PlanModuleData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlanModuleBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<PlanModuleData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// 枚举归一：module 仅接受 proto 枚举名（含大小写变体）。
fn normalize_module(v: &str) -> Result<Option<String>, AppError> {
    match v.to_ascii_uppercase().as_str() {
        "DASHBOARD" => Ok(Some("DASHBOARD".into())),
        "OPM" => Ok(Some("OPM".into())),
        "SYSTEM" => Ok(Some("SYSTEM".into())),
        "DICT" => Ok(Some("DICT".into())),
        "TENANT" => Ok(Some("TENANT".into())),
        "PERMISSION" => Ok(Some("PERMISSION".into())),
        "LOG" => Ok(Some("LOG".into())),
        "INTERNAL_MESSAGE" => Ok(Some("INTERNAL_MESSAGE".into())),
        "FILE" => Ok(Some("FILE".into())),
        "TASK" => Ok(Some("TASK".into())),
        // proto 零值 MODULE_UNSPECIFIED → 不设置（对齐 Go 跳过）
        "MODULE_UNSPECIFIED" | "" => Ok(None),
        other => Err(AppError::Validation(format!("invalid module: {other}"))),
    }
}

/// 归一化 data.module：未提供或零值 → None（不设置/不更新）。
fn normalized_module(data: &PlanModuleData) -> Result<Option<String>, AppError> {
    match data.module.as_deref() {
        Some(v) => normalize_module(v),
        None => Ok(None),
    }
}

pub async fn plan_module_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, PLAN_MODULE_COLUMNS)?;
    let repo = PlanModuleRepo::new(crate::handlers::script::db_of(&state)?);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        where_clause = format!(" and {}", crate::query::compile_where(&lq.filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, PLAN_MODULE_COLUMNS)?;
        order_parts.push(format!("\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let order_by = order_parts.join(", ");
    let (rows, total) = repo
        .list(
            lq.paging.offset(),
            lq.paging.limit(),
            &where_clause,
            &bind,
            &order_by,
        )
        .await?;
    Ok(json_ok(ListResponse::new(
        rows.iter().map(to_dto).collect(),
        total,
    )))
}

pub async fn plan_module_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = PlanModuleRepo::new(crate::handlers::script::db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("plan module not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn plan_module_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreatePlanModuleBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let module = normalized_module(&data)?;
    let _ = PlanModuleRepo::new(crate::handlers::script::db_of(&state)?)
        .create(data.plan_id, module.as_deref(), operator.user_id)
        .await?;
    Ok(json_empty())
}

pub async fn plan_module_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdatePlanModuleBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let module = normalized_module(&data)?;

    let repo = PlanModuleRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        repo.create(data.plan_id, module.as_deref(), operator.user_id)
            .await?;
        return Ok(json_empty());
    }

    let updated = repo
        .update(id, data.plan_id, module.as_deref(), operator.user_id)
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("plan module not found".into()));
    }
    Ok(json_empty())
}

/// DELETE /plan-modules（集合路径）：对齐前端生成 client 的 query `?id=`（Go 端 Delete 单 id）。
/// 空 ids 报校验错误。
pub async fn plan_module_delete(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let mut ids: Vec<i64> = Vec::new();
    if let Some(v) = params.get("id") {
        if let Ok(id) = v.parse::<i64>() {
            ids.push(id);
        }
    }
    ids.sort_unstable();
    ids.dedup();
    if ids.is_empty() {
        return Err(AppError::Validation("ids is required".into()));
    }
    let repo = PlanModuleRepo::new(crate::handlers::script::db_of(&state)?);
    repo.delete_ids(&ids).await?;
    Ok(json_empty())
}
