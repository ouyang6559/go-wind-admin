// tenant 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、int64/uint64 字符串化、枚举为 proto 名）。
// 对齐 Go tenant_service.go：
//  - List：分页/过滤 + memberCount（sys_users 按 tenant_id 计数回填）；
//  - Create/Update：data 字段动态写入；Update 支持 allowMissing→Create；
//  - Delete：硬删除（ent DeleteOneID 语义）；
//  - GetUsage：配额 + 用户数 + 存储占用 + API 调用量聚合；
//  - CleanupData：硬删该租户全部业务表数据 + 保留租户记录（status→OFF）续存；
//  - TenantExists：code/name OR 语义；
//  - CreateTenantWithAdminUser：建租户 + 复制模板角色 + 建管理员（明文密码 bcrypt）。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::tenant::{create_tenant_with_admin, TenantNew, TenantRepo, TenantRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列（对应 sys_tenants / 关联别名）
const TENANT_COLUMNS: &[&str] = &[
    "id", "name", "code", "domain", "logo_url", "industry", "admin_user_id", "status", "type",
    "audit_status", "subscription_plan", "plan_id", "created_by", "updated_by", "created_at",
    "updated_at",
];

/// 枚举归一：status 仅接受 ON/OFF/EXPIRED/FREEZE；type 仅 TRIAL/PAID/INTERNAL/PARTNER/CUSTOM；
/// audit_status 仅 PENDING/APPROVED/REJECTED（DB 存 proto 枚举名，对齐 Go converter）。
fn normalize_status(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "ON" | "OFF" | "EXPIRED" | "FREEZE" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid tenant status: {v}"))),
    }
}

fn normalize_type(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "TRIAL" | "PAID" | "INTERNAL" | "PARTNER" | "CUSTOM" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid tenant type: {v}"))),
    }
}

fn normalize_audit_status(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "PENDING" | "APPROVED" | "REJECTED" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid tenant audit status: {v}"))),
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub industry: Option<String>,
    /// proto 枚举名（TRIAL/PAID/...）
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_user_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_user_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsubscribe_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expired_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription_plan: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<i64>,
    /// List 时由 sys_users 计数回填；详情为 0 时省略
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member_count: Option<i64>,
    /// proto 枚举名（ON/OFF/EXPIRED/FREEZE）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// proto 枚举名（PENDING/APPROVED/REJECTED）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_status: Option<String>,
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

pub fn to_dto(row: &TenantRow, member_count: Option<i64>) -> TenantDto {
    TenantDto {
        id: row.id,
        name: row.name.clone(),
        code: row.code.clone(),
        domain: row.domain.clone(),
        logo_url: row.logo_url.clone(),
        industry: row.industry.clone(),
        r#type: row.r#type.clone(),
        admin_user_id: row.admin_user_id,
        admin_user_name: row.admin_user_name.clone(),
        subscription_at: row.subscription_at.clone(),
        unsubscribe_at: row.unsubscribe_at.clone(),
        expired_at: row.expired_at.clone(),
        subscription_plan: row.subscription_plan.clone(),
        plan_id: row.plan_id,
        member_count: member_count.filter(|c| *c > 0).or(member_count).filter(|c| *c != 0),
        status: row.status.clone(),
        audit_status: row.audit_status.clone(),
        created_by: row.created_by,
        updated_by: row.updated_by,
        deleted_by: row.deleted_by,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        deleted_at: row.deleted_at.clone(),
    }
}

// ===================== 请求体（wire 对齐 proto json_name） =====================
// Tenant message 字段（camelCase）；CreateTenantRequest{ data }, UpdateTenantRequest{ id, data, updateMask, allowMissing }。

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantData {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub logo_url: Option<String>,
    #[serde(default)]
    pub industry: Option<String>,
    #[serde(rename = "type", default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub audit_status: Option<String>,
    #[serde(default)]
    pub subscription_plan: Option<String>,
    #[serde(default)]
    pub expired_at: Option<String>,
    #[serde(default)]
    pub subscription_at: Option<String>,
    #[serde(default)]
    pub unsubscribe_at: Option<String>,
    #[serde(default)]
    pub plan_id: Option<i64>,
}

impl TenantData {
    fn normalize_enums(&self) -> Result<(), AppError> {
        if let Some(s) = &self.status {
            normalize_status(s)?;
        }
        if let Some(t) = &self.r#type {
            normalize_type(t)?;
        }
        if let Some(a) = &self.audit_status {
            normalize_audit_status(a)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTenantBody {
    #[serde(default)]
    pub data: Option<TenantData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTenantBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<TenantData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

/// CreateTenantWithAdminUserRequest{ tenant, user, password } 中的 user 子集
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminUserData {
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub realname: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub mobile: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTenantWithAdminBody {
    #[serde(default)]
    pub tenant: Option<TenantData>,
    #[serde(default)]
    pub user: Option<AdminUserData>,
    #[serde(default)]
    pub password: Option<String>,
}

fn validate_create_data(data: &TenantData) -> Result<(), AppError> {
    data.normalize_enums()?;
    if data.name.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(AppError::Validation("tenant name is required".into()));
    }
    Ok(())
}

fn db(state: &AppState) -> Result<sqlx::AnyPool, AppError> {
    crate::handlers::script::db_of(state)
}

// ===================== List =====================
pub async fn tenant_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, TENANT_COLUMNS)?;
    let repo = TenantRepo::new(db(&state)?);

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
        let c = crate::query::resolve_column(col, TENANT_COLUMNS)?;
        order_parts.push(format!("\"t\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let order_by = order_parts.join(", ");

    let (rows, total) = repo
        .list(lq.paging.offset(), lq.paging.limit(), &where_clause, &bind, &order_by)
        .await?;
    // 聚合 memberCount（仅当存在行时查询，避免空 IN）
    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let counts = if ids.is_empty() {
        HashMap::new()
    } else {
        repo.count_users_by_tenant_ids(&ids).await?
    };
    let dtos: Vec<TenantDto> = rows
        .iter()
        .map(|r| to_dto(r, counts.get(&r.id).copied()))
        .collect();
    Ok(json_ok(ListResponse::new(dtos, total)))
}

// ===================== Get =====================
pub async fn tenant_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = TenantRepo::new(db(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("tenant not found".into()))?;
    Ok(json_ok(to_dto(&row, None)))
}

// ===================== Create =====================
pub async fn tenant_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateTenantBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("data is required".into()))?;
    validate_create_data(&data)?;

    let repo = TenantRepo::new(db(&state)?);
    // code/name 已存在 → 400（对齐 Go 唯一约束预检，避免落库 500）
    if repo
        .exists(
            data.code.as_deref(),
            Some(data.name.as_deref().unwrap_or("")),
        )
        .await?
    {
        return Err(AppError::Conflict("tenant code or name already exists".into()));
    }

    repo.create(
        data.name.as_deref(),
        data.code.as_deref(),
        data.logo_url.as_deref(),
        data.domain.as_deref(),
        data.industry.as_deref(),
        None,
        data.status.as_deref(),
        data.r#type.as_deref(),
        data.audit_status.as_deref(),
        data.subscription_plan.as_deref(),
        data.expired_at.as_deref(),
        data.subscription_at.as_deref(),
        data.unsubscribe_at.as_deref(),
        data.plan_id,
        operator.user_id,
    )
    .await?;
    Ok(json_empty())
}

// ===================== Update =====================
pub async fn tenant_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateTenantBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("data is required".into()))?;
    let repo = TenantRepo::new(db(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go updateMask 忽略）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        validate_create_data(&data)?;
        repo.create(
            data.name.as_deref(),
            data.code.as_deref(),
            data.logo_url.as_deref(),
            data.domain.as_deref(),
            data.industry.as_deref(),
            None,
            data.status.as_deref(),
            data.r#type.as_deref(),
            data.audit_status.as_deref(),
            data.subscription_plan.as_deref(),
            data.expired_at.as_deref(),
            data.subscription_at.as_deref(),
            data.unsubscribe_at.as_deref(),
            data.plan_id,
            operator.user_id,
        )
        .await?;
        return Ok(json_empty());
    }

    data.normalize_enums()?;
    let updated = repo
        .update(
            id,
            data.name.as_deref(),
            data.code.as_deref(),
            data.logo_url.as_deref(),
            data.domain.as_deref(),
            data.industry.as_deref(),
            None,
            data.status.as_deref(),
            data.r#type.as_deref(),
            data.audit_status.as_deref(),
            data.subscription_plan.as_deref(),
            data.expired_at.as_deref(),
            data.subscription_at.as_deref(),
            data.unsubscribe_at.as_deref(),
            data.plan_id,
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("tenant not found".into()));
    }
    Ok(json_empty())
}

// ===================== Delete =====================
pub async fn tenant_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = TenantRepo::new(db(&state)?);
    let deleted = repo.delete(id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("tenant not found".into()));
    }
    Ok(json_empty())
}

// ===================== CleanupData =====================
pub async fn tenant_cleanup_data(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = TenantRepo::new(db(&state)?);
    if repo.get(id).await?.is_none() {
        return Err(AppError::NotFound("tenant not found".into()));
    }
    let _ = operator; // 操作人已注入 token；清理事务由 repo 内部处理
    repo.cleanup_data(id, &state).await?;
    Ok(json_empty())
}

// ===================== GetUsage =====================
pub async fn tenant_get_usage(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = TenantRepo::new(db(&state)?);
    if repo.get(id).await?.is_none() {
        return Err(AppError::NotFound("tenant not found".into()));
    }
    let usage = repo.get_usage(id).await?;
    Ok(json_ok(usage))
}

// ===================== TenantExists =====================
pub async fn tenant_tenant_exists(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = TenantRepo::new(db(&state)?);
    let exist = repo
        .exists(
            params.get("code").map(String::as_str),
            params.get("name").map(String::as_str),
        )
        .await?;
    Ok(json_ok(serde_json::json!({ "exist": exist })))
}

// ===================== CreateTenantWithAdminUser =====================
pub async fn tenant_create_tenant_with_admin_user(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateTenantWithAdminBody>,
) -> Result<impl IntoResponse, AppError> {
    let tenant = body
        .tenant
        .ok_or_else(|| AppError::Validation("tenant is required".into()))?;
    let user = body
        .user
        .ok_or_else(|| AppError::Validation("user is required".into()))?;
    let password = body
        .password
        .unwrap_or_default()
        .trim()
        .to_string();

    validate_create_data(&tenant)?;
    let username = user
        .username
        .unwrap_or_default()
        .trim()
        .to_string();
    if username.is_empty() {
        return Err(AppError::Validation("username is required".into()));
    }
    if password.len() < 8 {
        return Err(AppError::Validation("password must be at least 8 characters".into()));
    }

    let repo = TenantRepo::new(db(&state)?);
    // code/name 存在即 400（OR 语义，对齐 Go CreateTenantWithAdminUser）
    if repo
        .exists(
            tenant.code.as_deref(),
            Some(tenant.name.as_deref().unwrap_or("")),
        )
        .await?
    {
        return Err(AppError::Conflict("tenant code or name already exists".into()));
    }

    // 明文密码 bcrypt（对齐 Go CreateWithTx 无 NeedDecrypt）
    let hash = bcrypt::hash(&password, bcrypt::DEFAULT_COST).map_err(|e| AppError::Internal {
        context: "hash password failed".into(),
        source: Some(Box::new(e)),
    })?;

    let new = TenantNew {
        name: tenant.name.clone(),
        code: tenant.code.clone(),
        domain: tenant.domain.clone(),
        logo_url: tenant.logo_url.clone(),
        industry: tenant.industry.clone(),
        status: tenant.status.clone(),
        r#type: tenant.r#type.clone(),
        audit_status: tenant.audit_status.clone(),
        subscription_plan: tenant.subscription_plan.clone(),
        expired_at: tenant.expired_at.clone(),
        plan_id: tenant.plan_id,
        admin_name: user.nickname,
        admin_realname: user.realname,
        admin_email: user.email,
        admin_mobile: user.mobile,
    };

    let db = db(&state)?;
    create_tenant_with_admin(&db, &new, &username, &hash, operator.user_id).await?;
    Ok(json_empty())
}