// plan 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、枚举为 proto 名）。
// DELETE /plans 为集合路径删除：对齐前端生成 client 的 query `?id=` 行为。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::plan::{PlanRepo, PlanRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列
const PLAN_COLUMNS: &[&str] = &[
    "id", "name", "version", "expiry_policy", "data_retention_days", "description", "remark",
    "created_by", "updated_by", "created_at", "updated_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_policy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_retention_days: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
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

pub fn to_dto(row: &PlanRow) -> PlanDto {
    PlanDto {
        id: row.id,
        name: row.name.clone(),
        version: row.version.clone(),
        expiry_policy: row.expiry_policy.clone(),
        data_retention_days: row.data_retention_days,
        description: row.description.clone(),
        remark: row.remark.clone(),
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
pub struct PlanData {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub expiry_policy: Option<String>,
    #[serde(default)]
    pub data_retention_days: Option<i64>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePlanBody {
    #[serde(default)]
    pub data: Option<PlanData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePlanBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<PlanData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// 枚举归一：version / expiryPolicy 仅接受 proto 枚举名（含大小写变体）。
fn normalize_version(v: &str) -> Result<Option<String>, AppError> {
    match v.to_ascii_uppercase().as_str() {
        "FREE" => Ok(Some("FREE".into())),
        "STANDARD" => Ok(Some("STANDARD".into())),
        "ENTERPRISE" => Ok(Some("ENTERPRISE".into())),
        // proto 零值 PLAN_VERSION_UNSPECIFIED → 不设置（对齐 Go 跳过）
        "PLAN_VERSION_UNSPECIFIED" | "" => Ok(None),
        other => Err(AppError::Validation(format!("invalid plan version: {other}"))),
    }
}

fn normalize_expiry_policy(v: &str) -> Result<Option<String>, AppError> {
    match v.to_ascii_uppercase().as_str() {
        "READONLY" => Ok(Some("READONLY".into())),
        "BLOCK_LOGIN" => Ok(Some("BLOCK_LOGIN".into())),
        "FREEZE" => Ok(Some("FREEZE".into())),
        // proto 零值 PLAN_EXPIRY_POLICY_UNSPECIFIED → 不设置
        "PLAN_EXPIRY_POLICY_UNSPECIFIED" | "" => Ok(None),
        other => Err(AppError::Validation(format!(
            "invalid plan expiry policy: {other}"
        ))),
    }
}

/// Go 端校验：name 非空（ent NotEmpty）
fn validate_data(data: &PlanData) -> Result<(), AppError> {
    if data.name.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::Validation("plan name is required".into()));
    }
    if let Some(v) = &data.version {
        normalize_version(v)?;
    }
    if let Some(v) = &data.expiry_policy {
        normalize_expiry_policy(v)?;
    }
    Ok(())
}

pub async fn plan_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, PLAN_COLUMNS)?;
    let repo = PlanRepo::new(crate::handlers::script::db_of(&state)?);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        where_clause = format!(" and {}", crate::query::compile_where(&lq.filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, PLAN_COLUMNS)?;
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

pub async fn plan_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = PlanRepo::new(crate::handlers::script::db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("plan not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn plan_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreatePlanBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    validate_data(&data)?;
    let name = data.name.clone().unwrap_or_default();

    let repo = PlanRepo::new(crate::handlers::script::db_of(&state)?);
    if repo.name_exists(name.trim(), 0).await? {
        return Err(AppError::Conflict(format!(
            "plan name already exists: {}",
            name.trim()
        )));
    }
    let version = data.version.as_deref().map(normalize_version).transpose()?.flatten();
    let expiry_policy = data
        .expiry_policy
        .as_deref()
        .map(normalize_expiry_policy)
        .transpose()?
        .flatten();
    let _ = repo
        .create(
            name.trim(),
            version.as_deref(),
            expiry_policy.as_deref(),
            data.data_retention_days,
            data.description.as_deref(),
            data.remark.as_deref(),
            operator.user_id,
        )
        .await?;
    Ok(json_empty())
}

pub async fn plan_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdatePlanBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;

    let repo = PlanRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        validate_data(&data)?;
        let name = data.name.clone().unwrap_or_default();
        if repo.name_exists(name.trim(), 0).await? {
            return Err(AppError::Conflict(format!(
                "plan name already exists: {}",
                name.trim()
            )));
        }
        let version = data.version.as_deref().map(normalize_version).transpose()?.flatten();
        let expiry_policy = data
            .expiry_policy
            .as_deref()
            .map(normalize_expiry_policy)
            .transpose()?
            .flatten();
        repo.create(
            name.trim(),
            version.as_deref(),
            expiry_policy.as_deref(),
            data.data_retention_days,
            data.description.as_deref(),
            data.remark.as_deref(),
            operator.user_id,
        )
        .await?;
        return Ok(json_empty());
    }

    // 提供 name 时校验必填与唯一性
    let mut name: Option<String> = None;
    if data.name.is_some() {
        validate_data(&data)?;
        name = Some(data.name.clone().unwrap_or_default().trim().to_string());
        if let Some(n) = &name {
            if repo.name_exists(n, id).await? {
                return Err(AppError::Conflict(format!("plan name already exists: {n}")));
            }
        }
    }
    let version = data.version.as_deref().map(normalize_version).transpose()?.flatten();
    let expiry_policy = data
        .expiry_policy
        .as_deref()
        .map(normalize_expiry_policy)
        .transpose()?
        .flatten();

    let updated = repo
        .update(
            id,
            name.as_deref(),
            version.as_deref(),
            expiry_policy.as_deref(),
            data.data_retention_days,
            data.description.as_deref(),
            data.remark.as_deref(),
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("plan not found".into()));
    }
    Ok(json_empty())
}

/// DELETE /plans（集合路径）：对齐前端生成 client 的 query `?id=`（Go 端 Delete 单 id）。
/// 空 ids 报校验错误。
pub async fn plan_delete(
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
    let repo = PlanRepo::new(crate::handlers::script::db_of(&state)?);
    repo.delete_ids(&ids).await?;
    Ok(json_empty())
}
