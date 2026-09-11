// plan_quota 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、枚举为 proto 名）。
// quota_type 枚举（identity.service.v1.PlanQuota.QuotaType）：USER_LIMIT/STORAGE/API_CALL；
// proto 零值 PLAN_QUOTA_TYPE_UNSPECIFIED → 不设置（对齐 Go 跳过）。
// quota_value 为 proto uint64：protojson 输出为字符串，输入兼容数字与字符串。
// DELETE /plan-quotas 为集合路径删除：对齐前端生成 client 的 query `?id=` 行为。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::plan_quota::{PlanQuotaRepo, PlanQuotaRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列
const PLAN_QUOTA_COLUMNS: &[&str] = &[
    "id", "plan_id", "quota_type", "quota_value", "created_by", "updated_by", "deleted_by",
    "created_at", "updated_at", "deleted_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanQuotaDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_type: Option<String>,
    /// uint64 按 protojson 输出字符串
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quota_value: Option<String>,
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

pub fn to_dto(row: &PlanQuotaRow) -> PlanQuotaDto {
    PlanQuotaDto {
        id: row.id,
        plan_id: row.plan_id,
        quota_type: row.quota_type.clone(),
        quota_value: row.quota_value.map(|v| v.to_string()),
        created_by: row.created_by,
        updated_by: row.updated_by,
        deleted_by: row.deleted_by,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        deleted_at: row.deleted_at.clone(),
    }
}

/// quota_value 输入兼容：protojson 接受数字与字符串两种形态。
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum QuotaValueInput {
    Num(i64),
    Str(String),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanQuotaData {
    #[serde(default)]
    pub plan_id: Option<i64>,
    #[serde(default)]
    pub quota_type: Option<String>,
    #[serde(default)]
    pub quota_value: Option<QuotaValueInput>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlanQuotaBody {
    #[serde(default)]
    pub data: Option<PlanQuotaData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlanQuotaBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<PlanQuotaData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// 枚举归一：quota_type 仅接受 proto 枚举名（含大小写变体）。
fn normalize_quota_type(v: &str) -> Result<Option<String>, AppError> {
    match v.to_ascii_uppercase().as_str() {
        "USER_LIMIT" => Ok(Some("USER_LIMIT".into())),
        "STORAGE" => Ok(Some("STORAGE".into())),
        "API_CALL" => Ok(Some("API_CALL".into())),
        // proto 零值 PLAN_QUOTA_TYPE_UNSPECIFIED → 不设置（对齐 Go 跳过）
        "PLAN_QUOTA_TYPE_UNSPECIFIED" | "" => Ok(None),
        other => Err(AppError::Validation(format!("invalid plan quota type: {other}"))),
    }
}

/// 归一化 data.quota_type：未提供或零值 → None（不设置/不更新）。
fn normalized_quota_type(data: &PlanQuotaData) -> Result<Option<String>, AppError> {
    match data.quota_type.as_deref() {
        Some(v) => normalize_quota_type(v),
        None => Ok(None),
    }
}

/// 归一化 data.quota_value：数字直用，字符串按 protojson 解析。
fn normalized_quota_value(data: &PlanQuotaData) -> Result<Option<i64>, AppError> {
    match &data.quota_value {
        Some(QuotaValueInput::Num(n)) => Ok(Some(*n)),
        Some(QuotaValueInput::Str(s)) => s
            .trim()
            .parse::<i64>()
            .map(Some)
            .map_err(|_| AppError::Validation(format!("invalid quota value: {s}"))),
        None => Ok(None),
    }
}

pub async fn plan_quota_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, PLAN_QUOTA_COLUMNS)?;
    let repo = PlanQuotaRepo::new(crate::handlers::script::db_of(&state)?);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        where_clause = format!(" and {}", crate::query::compile_where(&lq.filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, PLAN_QUOTA_COLUMNS)?;
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

pub async fn plan_quota_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreatePlanQuotaBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let quota_type = normalized_quota_type(&data)?;
    let quota_value = normalized_quota_value(&data)?;
    let _ = PlanQuotaRepo::new(crate::handlers::script::db_of(&state)?)
        .create(data.plan_id, quota_type.as_deref(), quota_value, operator.user_id)
        .await?;
    Ok(json_empty())
}

pub async fn plan_quota_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdatePlanQuotaBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let quota_type = normalized_quota_type(&data)?;
    let quota_value = normalized_quota_value(&data)?;

    let repo = PlanQuotaRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        repo.create(data.plan_id, quota_type.as_deref(), quota_value, operator.user_id)
            .await?;
        return Ok(json_empty());
    }

    let updated = repo
        .update(id, data.plan_id, quota_type.as_deref(), quota_value, operator.user_id)
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("plan quota not found".into()));
    }
    Ok(json_empty())
}

/// DELETE /plan-quotas（集合路径）：对齐前端生成 client 的 query `?id=`（Go 端 Delete 单 id）。
/// 空 ids 报校验错误。
pub async fn plan_quota_delete(
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
    let repo = PlanQuotaRepo::new(crate::handlers::script::db_of(&state)?);
    repo.delete_ids(&ids).await?;
    Ok(json_empty())
}
