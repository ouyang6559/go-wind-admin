// org_unit 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、枚举为 proto 名）。
// List 对齐 Go：分页/过滤查询后按 sort_order 稳定排序，再在内存构建 parentId→children 树，
// 只返回根节点（children 嵌套）；total 为过滤后总数（字符串）。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::org_unit::{OrgUnitRepo, OrgUnitRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列（仅本表真实列，snake_case）
const ORG_UNIT_COLUMNS: &[&str] = &[
    "id", "name", "code", "leader_id", "type", "business_scopes", "external_id",
    "is_legal_entity", "registration_number", "tax_id", "legal_entity_org_id", "address",
    "phone", "email", "timezone", "country", "latitude", "longitude", "start_at", "end_at",
    "contact_user_id", "permission_tags", "status", "sort_order", "tenant_id", "remark",
    "description", "parent_id", "path", "created_by", "updated_by", "created_at", "updated_at",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgUnitDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leader_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leader_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub business_scopes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_legal_entity: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub legal_entity_org_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_user_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contact_user_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i64>,
    /// 树子节点（叶子省略，对齐 protojson 空 repeated）
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<OrgUnitDto>,
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

pub fn to_dto(row: &OrgUnitRow) -> OrgUnitDto {
    OrgUnitDto {
        id: row.id,
        name: row.name.clone(),
        code: row.code.clone(),
        r#type: row.r#type.clone(),
        path: row.path.clone(),
        status: row.status.clone(),
        sort_order: row.sort_order,
        leader_id: row.leader_id,
        leader_name: row.leader_name.clone(),
        tenant_id: row.tenant_id,
        tenant_name: row.tenant_name.clone(),
        remark: row.remark.clone(),
        description: row.description.clone(),
        business_scopes: row.business_scopes.clone(),
        external_id: row.external_id.clone(),
        is_legal_entity: row.is_legal_entity,
        registration_number: row.registration_number.clone(),
        tax_id: row.tax_id.clone(),
        legal_entity_org_id: row.legal_entity_org_id,
        address: row.address.clone(),
        phone: row.phone.clone(),
        email: row.email.clone(),
        timezone: row.timezone.clone(),
        country: row.country.clone(),
        latitude: row.latitude,
        longitude: row.longitude,
        start_at: row.start_at.clone(),
        end_at: row.end_at.clone(),
        contact_user_id: row.contact_user_id,
        contact_user_name: row.contact_user_name.clone(),
        permission_tags: row.permission_tags.clone(),
        parent_id: row.parent_id,
        children: Vec::new(),
        created_by: row.created_by,
        updated_by: row.updated_by,
        deleted_by: row.deleted_by,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        deleted_at: row.deleted_at.clone(),
    }
}

/// 构建树：parent_id 为 NULL/0 的为根；子节点挂到父的 children。
/// 孤儿节点（父不在结果集内）跳过（对齐 Go pagination.BuildTree）。
fn build_tree(dtos: Vec<OrgUnitDto>) -> Vec<OrgUnitDto> {
    let mut map: HashMap<i64, OrgUnitDto> = HashMap::with_capacity(dtos.len());
    for dto in &dtos {
        map.insert(dto.id, dto.clone());
    }
    let mut roots: Vec<OrgUnitDto> = Vec::new();
    for dto in dtos {
        match dto.parent_id {
            Some(pid) if pid > 0 => {
                if let Some(parent) = map.get_mut(&pid) {
                    parent.children.push(dto);
                }
                // 父不在结果集 → 孤儿节点跳过
            }
            _ => roots.push(dto),
        }
    }
    roots
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrgUnitData {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default)]
    pub leader_id: Option<i64>,
    #[serde(default)]
    pub tenant_id: Option<i64>,
    #[serde(default)]
    pub remark: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub business_scopes: Option<Vec<String>>,
    #[serde(default)]
    pub external_id: Option<String>,
    #[serde(default)]
    pub is_legal_entity: Option<bool>,
    #[serde(default)]
    pub registration_number: Option<String>,
    #[serde(default)]
    pub tax_id: Option<String>,
    #[serde(default)]
    pub legal_entity_org_id: Option<i64>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub timezone: Option<String>,
    #[serde(default)]
    pub country: Option<String>,
    #[serde(default)]
    pub latitude: Option<f64>,
    #[serde(default)]
    pub longitude: Option<f64>,
    #[serde(default)]
    pub start_at: Option<String>,
    #[serde(default)]
    pub end_at: Option<String>,
    #[serde(default)]
    pub contact_user_id: Option<i64>,
    #[serde(default)]
    pub permission_tags: Option<Vec<String>>,
    #[serde(default)]
    pub parent_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrgUnitBody {
    #[serde(default)]
    pub data: Option<OrgUnitData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrgUnitBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<OrgUnitData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// 枚举归一：type 仅接受 proto 枚举名（含大小写变体），status 仅接受 ON/OFF。
fn normalize_status(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "ON" | "OFF" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid org unit status: {v}"))),
    }
}

fn normalize_type(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "COMPANY" | "DIVISION" | "DEPARTMENT" | "TEAM" | "PROJECT" | "COMMITTEE" | "REGION"
        | "SUBSIDIARY" | "BRANCH" | "OTHER" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid org unit type: {v}"))),
    }
}

/// Go 端校验：name 非空（ent NotEmpty）
fn validate_data(data: &OrgUnitData) -> Result<(), AppError> {
    if data.name.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::Validation("org unit name is required".into()));
    }
    if let Some(s) = &data.status {
        normalize_status(s)?;
    }
    if let Some(t) = &data.r#type {
        normalize_type(t)?;
    }
    Ok(())
}

pub async fn org_unit_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, ORG_UNIT_COLUMNS)?;
    let repo = OrgUnitRepo::new(crate::handlers::script::db_of(&state)?);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        // 列加 ou. 前缀，避免与 join 表同名列歧义
        let mut filters = lq.filters.clone();
        for f in &mut filters {
            f.column = format!("ou.{}", f.column);
        }
        where_clause = format!(" and {}", crate::query::compile_where(&filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, ORG_UNIT_COLUMNS)?;
        order_parts.push(format!("\"ou\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let order_by = order_parts.join(", ");
    let (mut rows, total) = repo
        .list(
            lq.paging.offset(),
            lq.paging.limit(),
            &where_clause,
            &bind,
            &order_by,
        )
        .await?;
    // 对齐 Go：分页结果按 sort_order 稳定排序后构建树（nil 视为 0）
    rows.sort_by_key(|r| r.sort_order.unwrap_or(0));
    let mut dtos: Vec<OrgUnitDto> = rows.iter().map(to_dto).collect();
    dtos = build_tree(dtos);
    Ok(json_ok(ListResponse::new(dtos, total)))
}

pub async fn org_unit_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = OrgUnitRepo::new(crate::handlers::script::db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("org unit not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn org_unit_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateOrgUnitBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    validate_data(&data)?;
    let name = data.name.clone().unwrap_or_default();

    let repo = OrgUnitRepo::new(crate::handlers::script::db_of(&state)?);
    if repo.name_exists(name.trim(), 0).await? {
        return Err(AppError::Conflict(format!(
            "org unit name already exists: {}",
            name.trim()
        )));
    }
    if let Some(code) = data.code.as_deref().map(str::trim).filter(|c| !c.is_empty()) {
        if repo.code_exists(code, 0).await? {
            return Err(AppError::Conflict(format!(
                "org unit code already exists: {code}"
            )));
        }
    }
    let tenant_id = data.tenant_id.or_else(|| {
        if operator.tenant_id > 0 {
            Some(operator.tenant_id)
        } else {
            None
        }
    });
    let status = data.status.as_deref().map(normalize_status).transpose()?;
    let r#type = data.r#type.as_deref().map(normalize_type).transpose()?;
    let _ = repo
        .create(
            name.trim(),
            data.code.as_deref(),
            data.leader_id,
            r#type.as_deref(),
            data.business_scopes.as_ref(),
            data.external_id.as_deref(),
            data.is_legal_entity,
            data.registration_number.as_deref(),
            data.tax_id.as_deref(),
            data.legal_entity_org_id,
            data.address.as_deref(),
            data.phone.as_deref(),
            data.email.as_deref(),
            data.timezone.as_deref(),
            data.country.as_deref(),
            data.latitude,
            data.longitude,
            data.start_at.as_deref(),
            data.end_at.as_deref(),
            data.contact_user_id,
            data.permission_tags.as_ref(),
            status.as_deref(),
            data.sort_order,
            tenant_id,
            data.remark.as_deref(),
            data.description.as_deref(),
            data.parent_id,
            operator.user_id,
        )
        .await?;
    Ok(json_empty())
}

pub async fn org_unit_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateOrgUnitBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;

    let repo = OrgUnitRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        validate_data(&data)?;
        let name = data.name.clone().unwrap_or_default();
        if repo.name_exists(name.trim(), 0).await? {
            return Err(AppError::Conflict(format!(
                "org unit name already exists: {}",
                name.trim()
            )));
        }
        if let Some(code) = data.code.as_deref().map(str::trim).filter(|c| !c.is_empty()) {
            if repo.code_exists(code, 0).await? {
                return Err(AppError::Conflict(format!(
                    "org unit code already exists: {code}"
                )));
            }
        }
        let tenant_id = data.tenant_id.or_else(|| {
            if operator.tenant_id > 0 {
                Some(operator.tenant_id)
            } else {
                None
            }
        });
        let status = data.status.as_deref().map(normalize_status).transpose()?;
        let r#type = data.r#type.as_deref().map(normalize_type).transpose()?;
        repo.create(
            name.trim(),
            data.code.as_deref(),
            data.leader_id,
            r#type.as_deref(),
            data.business_scopes.as_ref(),
            data.external_id.as_deref(),
            data.is_legal_entity,
            data.registration_number.as_deref(),
            data.tax_id.as_deref(),
            data.legal_entity_org_id,
            data.address.as_deref(),
            data.phone.as_deref(),
            data.email.as_deref(),
            data.timezone.as_deref(),
            data.country.as_deref(),
            data.latitude,
            data.longitude,
            data.start_at.as_deref(),
            data.end_at.as_deref(),
            data.contact_user_id,
            data.permission_tags.as_ref(),
            status.as_deref(),
            data.sort_order,
            tenant_id,
            data.remark.as_deref(),
            data.description.as_deref(),
            data.parent_id,
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
                    "org unit name already exists: {n}"
                )));
            }
        }
        if let Some(c) = &code {
            if !c.is_empty() && repo.code_exists(c, id).await? {
                return Err(AppError::Conflict(format!(
                    "org unit code already exists: {c}"
                )));
            }
        }
    }
    let status = data.status.as_deref().map(normalize_status).transpose()?;
    let r#type = data.r#type.as_deref().map(normalize_type).transpose()?;

    let updated = repo
        .update(
            id,
            name.as_deref(),
            code.as_deref(),
            data.leader_id,
            r#type.as_deref(),
            data.business_scopes.as_ref(),
            data.external_id.as_deref(),
            data.is_legal_entity,
            data.registration_number.as_deref(),
            data.tax_id.as_deref(),
            data.legal_entity_org_id,
            data.address.as_deref(),
            data.phone.as_deref(),
            data.email.as_deref(),
            data.timezone.as_deref(),
            data.country.as_deref(),
            data.latitude,
            data.longitude,
            data.start_at.as_deref(),
            data.end_at.as_deref(),
            data.contact_user_id,
            data.permission_tags.as_ref(),
            status.as_deref(),
            data.sort_order,
            data.remark.as_deref(),
            data.description.as_deref(),
            data.parent_id,
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("org unit not found".into()));
    }
    Ok(json_empty())
}

pub async fn org_unit_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = OrgUnitRepo::new(crate::handlers::script::db_of(&state)?);
    // 递归删除后代（对齐 Go QueryAllChildrenIds）
    let deleted = repo.delete(id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("org unit not found".into()));
    }
    Ok(json_empty())
}
