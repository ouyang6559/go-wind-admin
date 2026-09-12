// permission_group 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、枚举为 proto 名）。
// List 对齐 Go（treeTravel=true）：分页/过滤查询后在内存构建 parentId→children 树，
// 只返回根节点（children 嵌套），total 为过滤后总数（字符串）。
// status 枚举 ON/OFF（SwitchStatus mixin 存储值）；创建时缺省 status=ON、sortOrder=0。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::permission_group::{PermissionGroupRepo, PermissionGroupRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列
const PERMISSION_GROUP_COLUMNS: &[&str] = &[
    "id", "name", "path", "module", "sort_order", "status", "description", "parent_id",
    "created_by", "updated_by", "created_at", "updated_at",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionGroupDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i64>,
    /// ON | OFF
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<i64>,
    /// 树子节点（protojson repeated 恒输出 []，不 skip）
    pub children: Vec<PermissionGroupDto>,
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

pub fn to_dto(row: &PermissionGroupRow) -> PermissionGroupDto {
    PermissionGroupDto {
        id: row.id,
        name: Some(row.name.clone()),
        path: row.path.clone(),
        module: row.module.clone(),
        sort_order: row.sort_order,
        status: row.status.clone(),
        description: row.description.clone(),
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
/// 孤儿节点（父不在结果集内，连同其子树）跳过（对齐 Go pagination.BuildTree）。
fn build_tree(dtos: Vec<PermissionGroupDto>) -> Vec<PermissionGroupDto> {
    fn assemble(
        id: i64,
        by_id: &HashMap<i64, PermissionGroupDto>,
        child_ids: &HashMap<i64, Vec<i64>>,
        visiting: &mut Vec<i64>,
    ) -> Option<PermissionGroupDto> {
        if visiting.contains(&id) {
            return None; // 环保护
        }
        let mut node = by_id.get(&id)?.clone();
        visiting.push(id);
        if let Some(kids) = child_ids.get(&id) {
            for k in kids {
                if let Some(c) = assemble(*k, by_id, child_ids, visiting) {
                    node.children.push(c);
                }
            }
        }
        visiting.pop();
        Some(node)
    }

    let by_id: HashMap<i64, PermissionGroupDto> = dtos.iter().map(|d| (d.id, d.clone())).collect();
    let mut child_ids: HashMap<i64, Vec<i64>> = HashMap::new();
    let mut root_ids: Vec<i64> = Vec::new();
    for d in &dtos {
        let pid = d.parent_id.unwrap_or(0);
        if pid == 0 {
            root_ids.push(d.id);
        } else if by_id.contains_key(&pid) {
            child_ids.entry(pid).or_default().push(d.id);
        }
        // 父不在结果集 → 孤儿跳过
    }
    root_ids
        .into_iter()
        .map(|id| assemble(id, &by_id, &child_ids, &mut Vec::new()))
        .flatten()
        .collect()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionGroupData {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub module: Option<String>,
    #[serde(default)]
    pub sort_order: Option<i64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parent_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePermissionGroupBody {
    #[serde(default)]
    pub data: Option<PermissionGroupData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePermissionGroupBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<PermissionGroupData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// 枚举归一：status 仅接受 ON/OFF（SwitchStatus mixin 存储值）。
fn normalize_status(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "ON" | "OFF" => Ok(up),
        _ => Err(AppError::Validation(format!(
            "invalid permission group status: {v}"
        ))),
    }
}

/// Go 端无显式校验（ent name NotEmpty 兜底），Rust 端校验必填避免落库 500。
fn validate_data(data: &PermissionGroupData) -> Result<(), AppError> {
    if data.name.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::Validation("permission group name is required".into()));
    }
    if let Some(s) = &data.status {
        normalize_status(s)?;
    }
    Ok(())
}

pub async fn permission_group_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, PERMISSION_GROUP_COLUMNS)?;
    let repo = PermissionGroupRepo::new(crate::handlers::script::db_of(&state)?);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        let mut filters = lq.filters.clone();
        for f in &mut filters {
            f.column = format!("g.{}", f.column);
        }
        where_clause = format!(" and {}", crate::query::compile_where(&filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, PERMISSION_GROUP_COLUMNS)?;
        order_parts.push(format!("\"g\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let order_by = order_parts.join(", ");
    let (rows, total) = repo
        .list(lq.paging.offset(), lq.paging.limit(), &where_clause, &bind, &order_by)
        .await?;
    let dtos: Vec<PermissionGroupDto> = rows.iter().map(to_dto).collect();
    Ok(json_ok(ListResponse::new(build_tree(dtos), total)))
}

pub async fn permission_group_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = PermissionGroupRepo::new(crate::handlers::script::db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("permission group not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn permission_group_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreatePermissionGroupBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("data is required".into()))?;
    validate_data(&data)?;
    let name = data.name.clone().unwrap_or_default().trim().to_string();

    let repo = PermissionGroupRepo::new(crate::handlers::script::db_of(&state)?);
    let status = data
        .status
        .as_deref()
        .map(normalize_status)
        .transpose()?
        .unwrap_or_else(|| "ON".into());
    repo.create(
        &name,
        data.module.as_deref(),
        data.sort_order.or(Some(0)),
        Some(&status),
        data.description.as_deref(),
        data.parent_id,
        operator.user_id,
    )
    .await?;
    Ok(json_empty())
}

pub async fn permission_group_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdatePermissionGroupBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("data is required".into()))?;

    let repo = PermissionGroupRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        validate_data(&data)?;
        let name = data.name.clone().unwrap_or_default().trim().to_string();
        let status = data
            .status
            .as_deref()
            .map(normalize_status)
            .transpose()?
            .unwrap_or_else(|| "ON".into());
        repo.create(
            &name,
            data.module.as_deref(),
            data.sort_order.or(Some(0)),
            Some(&status),
            data.description.as_deref(),
            data.parent_id,
            operator.user_id,
        )
        .await?;
        return Ok(json_empty());
    }

    // 提供 name 时做必填校验（status 提供时做枚举归一）
    let mut name: Option<String> = None;
    if data.name.is_some() {
        validate_data(&data)?;
        name = Some(data.name.clone().unwrap_or_default().trim().to_string());
    }
    let status = data.status.as_deref().map(normalize_status).transpose()?;

    let updated = repo
        .update(
            id,
            name.as_deref(),
            data.module.as_deref(),
            data.sort_order,
            status.as_deref(),
            data.description.as_deref(),
            data.parent_id,
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("permission group not found".into()));
    }
    Ok(json_empty())
}

pub async fn permission_group_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = PermissionGroupRepo::new(crate::handlers::script::db_of(&state)?);
    let deleted = repo.delete(id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("permission group not found".into()));
    }
    Ok(json_empty())
}
