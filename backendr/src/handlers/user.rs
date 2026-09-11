// user 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、枚举为 proto 名）。
// 端点：GET/POST /users、GET/PUT/DELETE /users/{id}、GET/DELETE /users/username/{username}。
// 对齐 user_service.go 约束：
//  - Create/Update 必须带角色（role_ids 非空、角色存在且类型匹配租户/平台）；
//  - username 按租户唯一；更新时 email/mobile 掩码（含 '*'）跳过写入；
//  - Delete 禁止删除默认 admin（id=1）与自身；
//  - 创建用户时密码缺省 Abcd@1234（AES 密文传输，需解密后哈希）。
// 列表/详情聚合 roleIds/orgUnitIds/positionIds + Roles/RoleNames/OrgUnitNames/PositionNames/TenantName。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::user::{UserRepo, UserRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列（sys_users 真实列名，snake_case）
const USER_COLUMNS: &[&str] = &[
    "id", "tenant_id", "username", "nickname", "realname", "email", "mobile", "telephone",
    "avatar", "address", "region", "description", "gender", "status", "last_login_ip",
    "created_by", "updated_by", "created_at", "updated_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_name: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub org_unit_ids: Vec<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub org_unit_names: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub position_ids: Vec<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub position_names: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub role_ids: Vec<i64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub role_names: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub realname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub avatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mobile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub telephone: Option<String>,
    /// proto 枚举名（SECRET/MALE/FEMALE）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login_ip: Option<String>,
    /// proto 枚举名（NORMAL/DISABLED/PENDING/LOCKED/EXPIRED/CLOSED）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locked_until: Option<String>,
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

/// 行 → DTO（关联信息由调用方聚合后传入）
pub fn to_dto(
    row: &UserRow,
    role_ids: Vec<i64>,
    roles: Vec<String>,
    role_names: Vec<String>,
    org_unit_ids: Vec<i64>,
    org_unit_names: Vec<String>,
    position_ids: Vec<i64>,
    position_names: Vec<String>,
) -> UserDto {
    UserDto {
        id: row.id,
        tenant_id: row.tenant_id,
        tenant_name: row.tenant_name.clone(),
        org_unit_ids,
        org_unit_names,
        position_ids,
        position_names,
        role_ids,
        roles,
        role_names,
        username: row.username.clone(),
        nickname: row.nickname.clone(),
        realname: row.realname.clone(),
        avatar: row.avatar.clone(),
        email: row.email.clone(),
        mobile: row.mobile.clone(),
        telephone: row.telephone.clone(),
        gender: row.gender.clone(),
        address: row.address.clone(),
        region: row.region.clone(),
        description: row.description.clone(),
        remark: row.remark.clone(),
        last_login_at: row.last_login_at.clone(),
        last_login_ip: row.last_login_ip.clone(),
        status: row.status.clone(),
        locked_until: row.locked_until.clone(),
        created_by: row.created_by,
        updated_by: row.updated_by,
        deleted_by: row.deleted_by,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        deleted_at: row.deleted_at.clone(),
    }
}

/// 聚合用户关联信息并转 DTO（对齐 Go enrichRelations）。
async fn enrich_and_to_dto(repo: &UserRepo, row: &UserRow) -> Result<UserDto, AppError> {
    let role_ids = repo.list_role_ids(row.id).await?;
    let org_unit_ids = repo.list_org_unit_ids(row.id).await?;
    let position_ids = repo.list_position_ids(row.id).await?;

    let mut roles = Vec::new();
    let mut role_names = Vec::new();
    let role_infos = repo.list_roles_by_ids(&role_ids).await?;
    for (_, code, name) in &role_infos {
        if !code.is_empty() {
            roles.push(code.clone());
        }
        if !name.is_empty() {
            role_names.push(name.clone());
        }
    }

    let mut org_unit_names = Vec::new();
    for (_, name) in repo.list_org_unit_names(&org_unit_ids).await? {
        if !name.is_empty() {
            org_unit_names.push(name);
        }
    }

    let mut position_names = Vec::new();
    for (_, name) in repo.list_position_names(&position_ids).await? {
        if !name.is_empty() {
            position_names.push(name);
        }
    }

    Ok(to_dto(
        row,
        role_ids,
        roles,
        role_names,
        org_unit_ids,
        org_unit_names,
        position_ids,
        position_names,
    ))
}

// ===================== 入参 =====================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserData {
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub realname: Option<String>,
    #[serde(default)]
    pub avatar: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub mobile: Option<String>,
    #[serde(default)]
    pub telephone: Option<String>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub remark: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<i64>,
    #[serde(default)]
    pub role_id: Option<i64>,
    #[serde(default)]
    pub role_ids: Option<Vec<i64>>,
    #[serde(default)]
    pub org_unit_ids: Option<Vec<i64>>,
    #[serde(default)]
    pub position_ids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserBody {
    #[serde(default)]
    pub data: Option<UserData>,
    /// 前端 AES 密文（与登录一致）
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<UserData>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// gender 归一：仅接受 proto 枚举名
fn normalize_gender(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "SECRET" | "MALE" | "FEMALE" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid user gender: {v}"))),
    }
}

/// status 归一：仅接受 proto 枚举名
fn normalize_status(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "NORMAL" | "DISABLED" | "PENDING" | "LOCKED" | "EXPIRED" | "CLOSED" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid user status: {v}"))),
    }
}

/// 收集角色 ID（role_id + role_ids 并集去重）
fn collect_role_ids(data: &UserData) -> Vec<i64> {
    let mut ids = Vec::new();
    if let Some(v) = data.role_id {
        if v > 0 {
            ids.push(v);
        }
    }
    if let Some(v) = data.role_ids.as_deref() {
        for id in v {
            if *id > 0 && !ids.contains(id) {
                ids.push(*id);
            }
        }
    }
    ids
}

/// 校验角色：非空 + 全部存在且类型匹配（租户操作者只能绑 TENANT，平台绑 SYSTEM）。
async fn validate_role_ids(repo: &UserRepo, role_ids: &[i64], tenant_id: i64) -> Result<(), AppError> {
    if role_ids.is_empty() {
        return Err(AppError::Validation("role_ids is required".into()));
    }
    let matched = repo.validate_roles(role_ids, tenant_id).await?;
    if matched != role_ids.len() {
        return Err(AppError::Validation("some roles not found".into()));
    }
    Ok(())
}

// ===================== handlers =====================

pub async fn user_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, USER_COLUMNS)?;
    let db = crate::handlers::script::db_of(&state)?;
    let repo = UserRepo::new(db);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        let mut filters = lq.filters.clone();
        for f in &mut filters {
            f.column = format!("u.{}", f.column);
        }
        where_clause = format!(" and {}", crate::query::compile_where(&filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, USER_COLUMNS)?;
        order_parts.push(format!("\"u\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let order_by = order_parts.join(", ");
    let (rows, total) = repo
        .list(lq.paging.offset(), lq.paging.limit(), &where_clause, &bind, &order_by)
        .await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in &rows {
        items.push(enrich_and_to_dto(&repo, row).await?);
    }
    Ok(json_ok(ListResponse::new(items, total)))
}

/// GET /users/{id} 与 GET /users/username/{username} 共用。
/// 路径段为纯数字按 id 查；否则按 username 查（须具名租户上下文，对齐 Go）。
pub async fn user_get(
    State(state): State<AppState>,
    Path(seg): Path<String>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = UserRepo::new(crate::handlers::script::db_of(&state)?);
    let row = if let Ok(id) = seg.parse::<i64>() {
        repo.get(id).await?
    } else {
        if operator.tenant_id <= 0 {
            return Err(AppError::Validation(
                "tenant scope required to query user by username".into(),
            ));
        }
        repo.get_by_username(operator.tenant_id, &seg).await?
    };
    let row = row.ok_or_else(|| AppError::NotFound("user not found".into()))?;
    let dto = enrich_and_to_dto(&repo, &row).await?;
    Ok(json_ok(dto))
}

/// GET /users/{id}（数字段走这里）——与 user_get 共用同一函数，无需重复实现。

pub async fn user_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateUserBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let repo = UserRepo::new(crate::handlers::script::db_of(&state)?);

    let username = data.username.clone().unwrap_or_default();
    if username.trim().is_empty() {
        return Err(AppError::Validation("username is required".into()));
    }
    let username = username.trim().to_string();

    // 租户归属：租户操作者强制本租户
    let tenant_id = if operator.tenant_id > 0 {
        operator.tenant_id
    } else {
        data.tenant_id.unwrap_or(0)
    };

    if repo.username_exists(tenant_id, &username, 0).await? {
        return Err(AppError::Conflict(format!(
            "username already exists: {username}"
        )));
    }

    let role_ids = collect_role_ids(&data);
    validate_role_ids(&repo, &role_ids, tenant_id).await?;

    // 密码：AES 密文 → 明文 → bcrypt；缺省默认密码
    let plain = if let Some(p) = body.password.as_deref() {
        if p.trim().is_empty() {
            "Abcd@1234".to_string()
        } else {
            crate::crypto::decrypt_transport_secret(p)?
        }
    } else {
        "Abcd@1234".to_string()
    };
    let hash = bcrypt::hash(&plain, bcrypt::DEFAULT_COST).map_err(|e| AppError::Internal {
        context: "hash password failed".into(),
        source: Some(Box::new(e)),
    })?;

    let gender = data.gender.as_deref().map(normalize_gender).transpose()?;
    let status = data.status.as_deref().map(normalize_status).transpose()?;

    let _ = repo
        .create(
            &username,
            tenant_id,
            data.nickname.as_deref(),
            data.realname.as_deref(),
            data.email.as_deref(),
            data.mobile.as_deref(),
            data.telephone.as_deref(),
            data.avatar.as_deref(),
            data.address.as_deref(),
            data.region.as_deref(),
            data.description.as_deref(),
            gender.as_deref(),
            status.as_deref(),
            data.remark.as_deref(),
            &role_ids,
            Some(&hash),
            operator.user_id,
        )
        .await?;
    Ok(json_empty())
}

/// PUT /users/{id}：字段更新 + 角色整体替换 + 可选重置密码。
pub async fn user_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateUserBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let repo = UserRepo::new(crate::handlers::script::db_of(&state)?);

    let existing = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;

    // 租户归属：租户操作者强制本租户
    let tenant_id = if operator.tenant_id > 0 {
        operator.tenant_id
    } else {
        data.tenant_id.or(existing.tenant_id).unwrap_or(0)
    };

    // username 不可改（immutable）：如提供且不同则报错
    if let Some(u) = data.username.as_deref() {
        if !u.trim().is_empty() && existing.username.as_deref() != Some(u.trim()) {
            return Err(AppError::Validation("username is immutable".into()));
        }
    }

    // 角色校验（对齐 Go Update：role_ids 必填）
    let role_ids = collect_role_ids(&data);
    validate_role_ids(&repo, &role_ids, tenant_id).await?;

    // email/mobile 掩码跳过写入（对齐 Go 防掩码入库）
    let email = data.email.clone().filter(|v| !v.contains('*'));
    let mobile = data.mobile.clone().filter(|v| !v.contains('*'));

    let gender = data.gender.as_deref().map(normalize_gender).transpose()?;
    let status = data.status.as_deref().map(normalize_status).transpose()?;

    let updated = repo
        .update(
            id,
            data.nickname.as_deref(),
            data.realname.as_deref(),
            email.as_deref(),
            mobile.as_deref(),
            data.telephone.as_deref(),
            data.avatar.as_deref(),
            data.address.as_deref(),
            data.region.as_deref(),
            data.description.as_deref(),
            gender.as_deref(),
            status.as_deref(),
            data.remark.as_deref(),
            Some(&role_ids),
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("user not found".into()));
    }

    // 可选重置密码：AES 解密 → bcrypt 落 USERNAME 凭证 → 吊销目标全部会话
    if let Some(p) = body.password.as_deref() {
        if !p.trim().is_empty() {
            let target_username = existing
                .username
                .clone()
                .ok_or_else(|| AppError::NotFound("user credential not found".into()))?;
            let plain = crate::crypto::decrypt_transport_secret(p)?;
            let hash = bcrypt::hash(&plain, bcrypt::DEFAULT_COST).map_err(|e| AppError::Internal {
                context: "hash password failed".into(),
                source: Some(Box::new(e)),
            })?;
            let auth_repo = crate::repos::authentication::AuthenticationRepo::new(
                crate::handlers::script::db_of(&state)?,
            );
            let updated_cred = auth_repo
                .update_password_hash(tenant_id, &target_username, &hash)
                .await?;
            if updated_cred == 0 {
                return Err(AppError::NotFound("user credential not found".into()));
            }
            for ct in ["admin", "app"] {
                let _ = crate::auth::revoke_all_sessions(&state, ct, id, "").await;
            }
        }
    }

    Ok(json_empty())
}

/// DELETE /users/{id} 与 DELETE /users/username/{username} 共用。
pub async fn user_delete(
    State(state): State<AppState>,
    Path(seg): Path<String>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = UserRepo::new(crate::handlers::script::db_of(&state)?);
    let row = if let Ok(id) = seg.parse::<i64>() {
        repo.get(id).await?
    } else {
        if operator.tenant_id <= 0 {
            return Err(AppError::Validation(
                "tenant scope required to delete user by username".into(),
            ));
        }
        repo.get_by_username(operator.tenant_id, &seg).await?
    };
    let target = row.ok_or_else(|| AppError::NotFound("user not found".into()))?;

    // 禁止删除默认 admin（id=1 或 admin/tenant=0）
    if target.id == 1
        || (target.username.as_deref() == Some("admin") && target.tenant_id.unwrap_or(0) == 0)
    {
        return Err(AppError::Validation("default admin cannot be deleted".into()));
    }
    // 禁止删除自己
    if target.id == operator.user_id {
        return Err(AppError::Validation("cannot delete yourself".into()));
    }

    let deleted = repo.delete(target.id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("user not found".into()));
    }
    Ok(json_empty())
}
