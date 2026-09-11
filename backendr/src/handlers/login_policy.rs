// login_policy 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、枚举为 proto 名）。
// type 枚举 BLACKLIST/WHITELIST、method 枚举 IP/MAC/REGION/TIME/DEVICE；
// 值格式校验对齐 Go ValidateLoginPolicyValue（IP/CIDR、TIME HH:MM-HH:MM、MAC/DEVICE/REGION 长度）。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::login_policy::{LoginPolicyRepo, LoginPolicyRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列
const LOGIN_POLICY_COLUMNS: &[&str] = &[
    "id", "target_id", "type", "method", "value", "reason", "tenant_id", "created_by",
    "updated_by", "created_at", "updated_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginPolicyDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_id: Option<i64>,
    /// BLACKLIST | WHITELIST
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// IP | MAC | REGION | TIME | DEVICE
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
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

pub fn to_dto(row: &LoginPolicyRow) -> LoginPolicyDto {
    LoginPolicyDto {
        id: row.id,
        target_id: row.target_id,
        value: row.value.clone(),
        reason: row.reason.clone(),
        r#type: row.r#type.clone(),
        method: row.method.clone(),
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
pub struct LoginPolicyData {
    #[serde(default)]
    pub target_id: Option<i64>,
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub value: Option<String>,
    #[serde(default)]
    pub reason: Option<String>,
    #[serde(default)]
    pub tenant_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLoginPolicyBody {
    #[serde(default)]
    pub data: Option<LoginPolicyData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateLoginPolicyBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<LoginPolicyData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

fn normalize_type(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "BLACKLIST" | "WHITELIST" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid login policy type: {v}"))),
    }
}

fn normalize_method(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "IP" | "MAC" | "REGION" | "TIME" | "DEVICE" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid login policy method: {v}"))),
    }
}

/// HH:MM 解析（对齐 Go parseHHMM）
fn parse_hhmm(s: &str) -> Option<(u32, u32)> {
    let mut it = s.split(':');
    let h: u32 = it.next()?.trim().parse().ok()?;
    let m: u32 = it.next()?.trim().parse().ok()?;
    if it.next().is_some() || h > 23 || m > 59 {
        return None;
    }
    Some((h, m))
}

/// 值格式校验（对齐 Go ValidateLoginPolicyValue）。
fn validate_value(method: &str, raw: &str) -> Result<(), AppError> {
    let value = raw.trim();
    if value.is_empty() {
        return Err(AppError::Validation("policy value is empty".into()));
    }
    match method {
        "IP" => {
            if value.contains('/') {
                // CIDR：IP/前缀
                let (ip, prefix) = value
                    .split_once('/')
                    .ok_or_else(|| AppError::Validation(format!("invalid CIDR: {value}")))?;
                let parsed: IpAddr = ip
                    .trim()
                    .parse()
                    .map_err(|_| AppError::Validation(format!("invalid CIDR: {value}")))?;
                let max_prefix = match parsed {
                    IpAddr::V4(_) => 32,
                    IpAddr::V6(_) => 128,
                };
                let n: u8 = prefix
                    .trim()
                    .parse()
                    .map_err(|_| AppError::Validation(format!("invalid CIDR: {value}")))?;
                if n > max_prefix {
                    return Err(AppError::Validation(format!("invalid CIDR: {value}")));
                }
                return Ok(());
            }
            if value.parse::<IpAddr>().is_err() {
                return Err(AppError::Validation(format!("invalid IP: {value}")));
            }
            Ok(())
        }
        "TIME" => {
            let mut parts = value.split('-');
            let s1 = parts.next().ok_or_else(|| {
                AppError::Validation("time window must be HH:MM-HH:MM".into())
            })?;
            let s2 = parts.next().ok_or_else(|| {
                AppError::Validation("time window must be HH:MM-HH:MM".into())
            })?;
            if parts.next().is_some() {
                return Err(AppError::Validation(
                    "time window must be HH:MM-HH:MM".into(),
                ));
            }
            let (h1, m1) = parse_hhmm(s1)
                .ok_or_else(|| AppError::Validation(format!("invalid time format: {value}")))?;
            let (h2, m2) = parse_hhmm(s2)
                .ok_or_else(|| AppError::Validation(format!("invalid time format: {value}")))?;
            if h1 * 60 + m1 == h2 * 60 + m2 {
                return Err(AppError::Validation(format!(
                    "time window start equals end: {value}"
                )));
            }
            Ok(())
        }
        "DEVICE" | "MAC" => {
            if value.len() > 128 {
                return Err(AppError::Validation("value too long".into()));
            }
            Ok(())
        }
        "REGION" => {
            if value.len() > 32 {
                return Err(AppError::Validation("region code too long".into()));
            }
            Ok(())
        }
        other => Err(AppError::Validation(format!(
            "unknown policy method: {other}"
        ))),
    }
}

/// 创建/更新时的枚举归一 + 值校验（method 缺失时对齐 Go 枚举 0 名的报错文案）。
fn validate_data(data: &LoginPolicyData) -> Result<(), AppError> {
    if let Some(t) = &data.r#type {
        normalize_type(t)?;
    }
    if let Some(m) = &data.method {
        normalize_method(m)?;
    }
    let method = data
        .method
        .as_deref()
        .map(normalize_method)
        .transpose()?
        .unwrap_or_else(|| "LOGIN_RESTRICTION_METHOD_UNSPECIFIED".into());
    validate_value(&method, data.value.as_deref().unwrap_or(""))?;
    Ok(())
}

/// 创建路径共用的枚举归一（type 缺省 BLACKLIST 对齐 ent Default）。
fn resolve_create_fields(data: &LoginPolicyData) -> Result<(String, String), AppError> {
    let r#type = match data.r#type.as_deref() {
        Some(t) => normalize_type(t)?,
        None => "BLACKLIST".into(),
    };
    let method = data
        .method
        .as_deref()
        .map(normalize_method)
        .transpose()?
        .ok_or_else(|| {
            AppError::Validation("unknown policy method: LOGIN_RESTRICTION_METHOD_UNSPECIFIED".into())
        })?;
    Ok((r#type, method))
}

pub async fn login_policy_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, LOGIN_POLICY_COLUMNS)?;
    let db = crate::handlers::script::db_of(&state)?;
    let repo = LoginPolicyRepo::new(db);
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
        let c = crate::query::resolve_column(col, LOGIN_POLICY_COLUMNS)?;
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

pub async fn login_policy_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = LoginPolicyRepo::new(crate::handlers::script::db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("login policy not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn login_policy_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateLoginPolicyBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("data is required".into()))?;
    validate_data(&data)?;
    let (r#type, method) = resolve_create_fields(&data)?;
    let value = data
        .value
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::Validation("policy value is empty".into()))?;

    let repo = LoginPolicyRepo::new(crate::handlers::script::db_of(&state)?);
    let tenant_id = data.tenant_id.or_else(|| {
        if operator.tenant_id > 0 {
            Some(operator.tenant_id)
        } else {
            None
        }
    });
    if repo
        .conflict_exists(tenant_id, data.target_id, &r#type, &method, 0)
        .await?
    {
        return Err(AppError::Conflict(format!(
            "login policy already exists: type={type} method={method}"
        )));
    }
    repo.create(
        tenant_id,
        data.target_id,
        &r#type,
        &method,
        value,
        data.reason.as_deref(),
        operator.user_id,
    )
    .await?;
    Ok(json_empty())
}

pub async fn login_policy_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateLoginPolicyBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("data is required".into()))?;

    let repo = LoginPolicyRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        validate_data(&data)?;
        let (r#type, method) = resolve_create_fields(&data)?;
        let value = data
            .value
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
            .ok_or_else(|| AppError::Validation("policy value is empty".into()))?;
        let tenant_id = data.tenant_id.or_else(|| {
            if operator.tenant_id > 0 {
                Some(operator.tenant_id)
            } else {
                None
            }
        });
        if repo
            .conflict_exists(tenant_id, data.target_id, &r#type, &method, 0)
            .await?
        {
            return Err(AppError::Conflict(format!(
                "login policy already exists: type={type} method={method}"
            )));
        }
        repo.create(
            tenant_id,
            data.target_id,
            &r#type,
            &method,
            value,
            data.reason.as_deref(),
            operator.user_id,
        )
        .await?;
        return Ok(json_empty());
    }

    // 提供 method/value 时做枚举归一 + 值校验（对齐 Go Update 的固定校验）
    if data.method.is_some() || data.value.is_some() {
        validate_data(&data)?;
    }
    let r#type = data.r#type.as_deref().map(normalize_type).transpose()?;
    let method = data.method.as_deref().map(normalize_method).transpose()?;

    // (tenant_id, target_id, type, method) 唯一性检查（排除自身；tenant 不可变取现有值）
    if method.is_some() {
        let existing = repo.get(id).await?;
        let tenant_id = existing.as_ref().and_then(|r| r.tenant_id);
        let target_id = data.target_id.or_else(|| existing.as_ref().and_then(|r| r.target_id));
        let type_key = r#type
            .clone()
            .or_else(|| existing.as_ref().and_then(|r| r.r#type.clone()))
            .unwrap_or_default();
        if repo
            .conflict_exists(tenant_id, target_id, &type_key, method.as_deref().unwrap_or(""), id)
            .await?
        {
            return Err(AppError::Conflict(format!(
                "login policy already exists: type={type_key} method={}",
                method.as_deref().unwrap_or("")
            )));
        }
    }

    let updated = repo
        .update(
            id,
            data.target_id,
            r#type.as_deref(),
            method.as_deref(),
            data.value.as_deref().map(str::trim),
            data.reason.as_deref(),
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("login policy not found".into()));
    }
    Ok(json_empty())
}

pub async fn login_policy_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = LoginPolicyRepo::new(crate::handlers::script::db_of(&state)?);
    let deleted = repo.delete(id).await?;
    if deleted == 0 {
        return Err(AppError::NotFound("login policy not found".into()));
    }
    Ok(json_empty())
}
