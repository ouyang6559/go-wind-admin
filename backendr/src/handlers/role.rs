// role 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、枚举为 proto 名）。
// 对齐 role_service.go 的约束：
//  - 租户操作者（tenant_id>0）创建/更新强制 type=TENANT；
//  - SYSTEM 角色仅平台操作者可修改（租户操作者改 → Forbidden）；
//  - 保护角色（is_protected）不可改 is_protected/type/status/code，且不可删除；
//  - permissions 为关联字段（sys_role_permissions）：Some 即整体替换（含清空），None 不动。
// 列表/详情输出时聚合权限ID（对齐 Go fillPermissionIDs）。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::role::{RoleRepo, RoleRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列（sys_roles 真实列名，snake_case）
const ROLE_COLUMNS: &[&str] = &[
    "id", "name", "code", "sort_order", "status", "type", "is_protected", "description",
    "tenant_id", "created_by", "updated_by", "created_at", "updated_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i64>,
    /// proto 枚举名（ON/OFF）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_protected: Option<bool>,
    /// proto 枚举名（SYSTEM/TEMPLATE/TENANT）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// 绑定的权限点ID列表（空列表省略，对齐 protojson）
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<i64>,
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

/// 角色行 → DTO（permissions 由调用方从 sys_role_permissions 聚合后传入）
pub fn to_dto(row: &RoleRow, permissions: Vec<i64>) -> RoleDto {
    RoleDto {
        id: row.id,
        name: row.name.clone(),
        code: row.code.clone(),
        sort_order: row.sort_order,
        status: row.status.clone(),
        description: row.description.clone(),
        is_protected: row.is_protected,
        r#type: row.r#type.clone(),
        permissions,
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
pub struct RoleData {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub is_protected: Option<bool>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub permissions: Option<Vec<i64>>,
    #[serde(default)]
    pub tenant_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleBody {
    #[serde(default)]
    pub data: Option<RoleData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<RoleData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// status 归一：仅接受 ON/OFF
fn normalize_status(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "ON" | "OFF" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid role status: {v}"))),
    }
}

/// type 归一：仅接受 proto 枚举名（SYSTEM/TEMPLATE/TENANT）
fn normalize_type(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "SYSTEM" | "TEMPLATE" | "TENANT" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid role type: {v}"))),
    }
}

/// 创建共用逻辑（role_create 与 update-allowMissing 转为 Create 共用）：
/// 校验 name/code 必填、code 唯一，租户操作者强制 TENANT，执行事务创建。
async fn create_role(
    repo: &RoleRepo,
    data: &RoleData,
    operator: &Operator,
) -> Result<(), AppError> {
    let name = data.name.clone().unwrap_or_default();
    let code = data.code.clone().unwrap_or_default();
    if name.trim().is_empty() || code.trim().is_empty() {
        return Err(AppError::Validation("role name and code are required".into()));
    }
    if repo.code_exists(code.trim(), 0).await? {
        return Err(AppError::Conflict(format!(
            "role code already exists: {}",
            code.trim()
        )));
    }
    let mut r#type = data.r#type.as_deref().map(normalize_type).transpose()?;
    // 租户操作者只能创建租户角色（对齐 Go Create）
    if operator.tenant_id > 0 {
        r#type = Some("TENANT".to_string());
    }
    let status = data.status.as_deref().map(normalize_status).transpose()?;
    let tenant_id = data.tenant_id.or_else(|| {
        if operator.tenant_id > 0 {
            Some(operator.tenant_id)
        } else {
            None
        }
    });
    let permissions = data.permissions.clone().unwrap_or_default();
    repo.create(
        name.trim(),
        code.trim(),
        data.sort_order,
        data.is_protected,
        r#type.as_deref(),
        status.as_deref(),
        data.description.as_deref(),
        tenant_id,
        operator.user_id,
        &permissions,
    )
    .await?;
    Ok(())
}

pub async fn role_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, ROLE_COLUMNS)?;
    let db = crate::handlers::script::db_of(&state)?;
    let repo = RoleRepo::new(db);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        let mut filters = lq.filters.clone();
        for f in &mut filters {
            f.column = format!("r.{}", f.column);
        }
        where_clause = format!(" and {}", crate::query::compile_where(&filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, ROLE_COLUMNS)?;
        order_parts.push(format!("\"r\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let order_by = order_parts.join(", ");
    let (rows, total) = repo
        .list(lq.paging.offset(), lq.paging.limit(), &where_clause, &bind, &order_by)
        .await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in &rows {
        let pids = repo.list_permission_ids(row.id).await?;
        items.push(to_dto(row, pids));
    }
    Ok(json_ok(ListResponse::new(items, total)))
}

pub async fn role_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = RoleRepo::new(crate::handlers::script::db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("role not found".into()))?;
    let pids = repo.list_permission_ids(id).await?;
    Ok(json_ok(to_dto(&row, pids)))
}

pub async fn role_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateRoleBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let repo = RoleRepo::new(crate::handlers::script::db_of(&state)?);
    create_role(&repo, &data, &operator).await?;
    Ok(json_empty())
}

pub async fn role_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateRoleBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let repo = RoleRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        create_role(&repo, &data, &operator).await?;
        return Ok(json_empty());
    }

    // 读取现有角色，用于保护约束
    let existing = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("role not found".into()))?;

    // 非平台操作者禁止修改系统角色（对齐 Go Update）
    if existing.r#type.as_deref() == Some("SYSTEM") && operator.tenant_id > 0 {
        return Err(AppError::Forbidden(
            "no permission to update system role".into(),
        ));
    }

    // 保护角色：is_protected/type/status/code 不可修改（对齐 Go FilterBlacklist）
    let mut r#type = data.r#type.as_deref().map(normalize_type).transpose()?;
    let mut status = data.status.as_deref().map(normalize_status).transpose()?;
    let mut code = data.code.clone().map(|v| v.trim().to_string());
    let mut is_protected = data.is_protected;
    if existing.is_protected.unwrap_or(false) {
        r#type = None;
        status = None;
        code = None;
        is_protected = None;
    }
    // 租户操作者强制 TENANT（对齐 Go Update）
    if operator.tenant_id > 0 && r#type.as_deref() != Some("TENANT") {
        r#type = Some("TENANT".to_string());
    }

    // name/code 提供时校验必填与唯一性
    let name = data.name.clone().map(|v| v.trim().to_string());
    if let Some(n) = &name {
        if n.is_empty() {
            return Err(AppError::Validation("role name is required".into()));
        }
    }
    if let Some(c) = &code {
        if c.is_empty() {
            return Err(AppError::Validation("role code is required".into()));
        }
        if repo.code_exists(c, id).await? {
            return Err(AppError::Conflict(format!(
                "role code already exists: {c}"
            )));
        }
    }

    let updated = repo
        .update(
            id,
            name.as_deref(),
            code.as_deref(),
            data.sort_order,
            is_protected,
            r#type.as_deref(),
            status.as_deref(),
            data.description.as_deref(),
            operator.user_id,
            data.permissions.as_deref(),
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("role not found".into()));
    }
    Ok(json_empty())
}

pub async fn role_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = RoleRepo::new(crate::handlers::script::db_of(&state)?);
    let existing = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("role not found".into()))?;
    // 保护角色禁止删除（对齐 Go Delete）
    if existing.is_protected.unwrap_or(false) {
        return Err(AppError::Forbidden(
            "protected role cannot be deleted".into(),
        ));
    }
    let deleted = repo.delete(id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("role not found".into()));
    }
    Ok(json_empty())
}
