// position 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、枚举为 proto 名）。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::position::{PositionRepo, PositionRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列
const POSITION_COLUMNS: &[&str] = &[
    "id", "name", "code", "headcount", "sort_order", "status", "type", "remark", "description",
    "job_family", "job_grade", "level", "is_key_position", "tenant_id", "org_unit_id",
    "reports_to_position_id", "start_at", "end_at", "created_by", "updated_by", "created_at",
    "updated_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headcount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_grade: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_key_position: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_unit_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub org_unit_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reports_to_position_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reports_to_position_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_at: Option<String>,
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

pub fn to_dto(row: &PositionRow) -> PositionDto {
    PositionDto {
        id: row.id,
        name: row.name.clone(),
        code: row.code.clone(),
        headcount: row.headcount,
        sort_order: row.sort_order,
        status: row.status.clone(),
        r#type: row.r#type.clone(),
        remark: row.remark.clone(),
        description: row.description.clone(),
        job_family: row.job_family.clone(),
        job_grade: row.job_grade.clone(),
        level: row.level,
        is_key_position: row.is_key_position,
        tenant_id: row.tenant_id,
        tenant_name: row.tenant_name.clone(),
        org_unit_id: row.org_unit_id,
        org_unit_name: row.org_unit_name.clone(),
        reports_to_position_id: row.reports_to_position_id,
        reports_to_position_name: row.reports_to_position_name.clone(),
        start_at: row.start_at.clone(),
        end_at: row.end_at.clone(),
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
pub struct PositionData {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub headcount: Option<i64>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub remark: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub job_family: Option<String>,
    #[serde(default)]
    pub job_grade: Option<String>,
    #[serde(default)]
    pub level: Option<i64>,
    #[serde(default)]
    pub is_key_position: Option<bool>,
    #[serde(default)]
    pub tenant_id: Option<i64>,
    #[serde(default)]
    pub org_unit_id: Option<i64>,
    #[serde(default)]
    pub reports_to_position_id: Option<i64>,
    #[serde(default)]
    pub start_at: Option<String>,
    #[serde(default)]
    pub end_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePositionBody {
    #[serde(default)]
    pub data: Option<PositionData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePositionBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<PositionData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// 枚举归一：type 仅接受 proto 枚举名（含大小写变体），status 仅接受 ON/OFF。
fn normalize_status(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "ON" | "OFF" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid position status: {v}"))),
    }
}

fn normalize_type(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "REGULAR" | "LEADER" | "MANAGER" | "INTERN" | "CONTRACT" | "OTHER" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid position type: {v}"))),
    }
}

/// Go 端 service 校验：data 非空
fn validate_data(data: &PositionData) -> Result<(), AppError> {
    if data.name.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::Validation("position name is required".into()));
    }
    if data.code.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::Validation("position code is required".into()));
    }
    if let Some(s) = &data.status {
        normalize_status(s)?;
    }
    if let Some(t) = &data.r#type {
        normalize_type(t)?;
    }
    Ok(())
}

pub async fn position_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, POSITION_COLUMNS)?;
    let db = crate::handlers::script::db_of(&state)?;
    let repo = PositionRepo::new(db);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        // 编译时把列名加 p. 前缀，避免与 join 表的同名列歧义
        let mut filters = lq.filters.clone();
        for f in &mut filters {
            f.column = format!("p.{}", f.column);
        }
        where_clause = format!(" and {}", crate::query::compile_where(&filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, POSITION_COLUMNS)?;
        order_parts.push(format!("\"p\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
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

pub async fn position_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = PositionRepo::new(crate::handlers::script::db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("position not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn position_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreatePositionBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    validate_data(&data)?;
    let name = data.name.clone().unwrap_or_default();
    let code = data.code.clone().unwrap_or_default();

    let repo = PositionRepo::new(crate::handlers::script::db_of(&state)?);
    if repo.name_exists(name.trim(), 0).await? {
        return Err(AppError::Conflict(format!(
            "position name already exists: {}",
            name.trim()
        )));
    }
    if repo.code_exists(code.trim(), 0).await? {
        return Err(AppError::Conflict(format!(
            "position code already exists: {}",
            code.trim()
        )));
    }
    let tenant_id = data.tenant_id.or_else(|| {
        if operator.tenant_id > 0 {
            Some(operator.tenant_id)
        } else {
            None
        }
    });
    let status = data.status.as_deref().map(|s| normalize_status(s)).transpose()?;
    let r#type = data.r#type.as_deref().map(|t| normalize_type(t)).transpose()?;
    let _ = repo
        .create(
            name.trim(),
            code.trim(),
            tenant_id,
            data.org_unit_id,
            data.reports_to_position_id,
            data.sort_order,
            status.as_deref(),
            r#type.as_deref(),
            data.job_family.as_deref(),
            data.job_grade.as_deref(),
            data.level,
            data.is_key_position,
            data.headcount,
            data.description.as_deref(),
            data.remark.as_deref(),
            data.start_at.as_deref(),
            data.end_at.as_deref(),
            operator.user_id,
        )
        .await?;
    Ok(json_empty())
}

pub async fn position_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdatePositionBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;

    let repo = PositionRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        validate_data(&data)?;
        let name = data.name.clone().unwrap_or_default();
        let code = data.code.clone().unwrap_or_default();
        if repo.name_exists(name.trim(), 0).await? {
            return Err(AppError::Conflict(format!(
                "position name already exists: {}",
                name.trim()
            )));
        }
        if repo.code_exists(code.trim(), 0).await? {
            return Err(AppError::Conflict(format!(
                "position code already exists: {}",
                code.trim()
            )));
        }
        let tenant_id = data.tenant_id.or_else(|| {
            if operator.tenant_id > 0 {
                Some(operator.tenant_id)
            } else {
                None
            }
        });
        let status = data.status.as_deref().map(|s| normalize_status(s)).transpose()?;
        let r#type = data.r#type.as_deref().map(|t| normalize_type(t)).transpose()?;
        repo.create(
            name.trim(),
            code.trim(),
            tenant_id,
            data.org_unit_id,
            data.reports_to_position_id,
            data.sort_order,
            status.as_deref(),
            r#type.as_deref(),
            data.job_family.as_deref(),
            data.job_grade.as_deref(),
            data.level,
            data.is_key_position,
            data.headcount,
            data.description.as_deref(),
            data.remark.as_deref(),
            data.start_at.as_deref(),
            data.end_at.as_deref(),
            operator.user_id,
        )
        .await?;
        return Ok(json_empty());
    }

    // 提供 name/code 时校验必填与唯一性
    let mut name: Option<String> = None;
    let mut code: Option<String> = None;
    if data.name.is_some() || data.code.is_some() {
        validate_data(&data)?;
        name = data.name.clone().map(|v| v.trim().to_string());
        code = data.code.clone().map(|v| v.trim().to_string());
        if let Some(n) = &name {
            if repo.name_exists(n, id).await? {
                return Err(AppError::Conflict(format!(
                    "position name already exists: {n}"
                )));
            }
        }
        if let Some(c) = &code {
            if repo.code_exists(c, id).await? {
                return Err(AppError::Conflict(format!(
                    "position code already exists: {c}"
                )));
            }
        }
    }
    let status = data.status.as_deref().map(|s| normalize_status(s)).transpose()?;
    let r#type = data.r#type.as_deref().map(|t| normalize_type(t)).transpose()?;

    let updated = repo
        .update(
            id,
            name.as_deref(),
            code.as_deref(),
            data.org_unit_id,
            data.reports_to_position_id,
            data.sort_order,
            status.as_deref(),
            r#type.as_deref(),
            data.job_family.as_deref(),
            data.job_grade.as_deref(),
            data.level,
            data.is_key_position,
            data.headcount,
            data.description.as_deref(),
            data.remark.as_deref(),
            data.start_at.as_deref(),
            data.end_at.as_deref(),
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("position not found".into()));
    }
    Ok(json_empty())
}

pub async fn position_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = PositionRepo::new(crate::handlers::script::db_of(&state)?);
    let deleted = repo.delete(id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("position not found".into()));
    }
    Ok(json_empty())
}
