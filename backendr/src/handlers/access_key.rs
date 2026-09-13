// access_key 模块 handlers（wire 对齐 Go protojson：camelCase、枚举为名字、可选字段缺省省略）。
// AK 前缀 ak-(8 字节 hex)、SK 前缀 sk-(32 字节 hex)，SK 只存 SHA-256 hex 摘要，
// 明文仅 Create/ResetSecret 响应返回一次。
// 令牌交换（IssueToken）：免鉴权端点，AK/SK → 短期机器令牌（uid=0、roles=["machine"]、
// 继承凭证租户），复用登录限流器做 IP+AK 防爆破。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use crate::auth::{self, AccessClaims, ACCESS_TOKEN_TTL};
use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::access_key::{AccessKeyRepo, AccessKeyRow};
use crate::repos::login_rate_limiter::LoginRateLimiter;
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

const ACCESS_KEY_COLUMNS: &[&str] = &[
    "id",
    "name",
    "access_key",
    "status",
    "expires_at",
    "last_used_at",
    "tenant_id",
    "created_at",
    "updated_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessKeyDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key: Option<String>,
    /// ON | OFF
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<i64>,
}

fn to_dto(r: &AccessKeyRow) -> AccessKeyDto {
    AccessKeyDto {
        id: r.id,
        name: r.name.clone(),
        access_key: Some(r.access_key.clone()),
        status: Some(r.status.clone()),
        expires_at: r.expires_at.clone(),
        last_used_at: r.last_used_at.clone(),
        tenant_id: Some(r.tenant_id),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccessKeyResponse {
    pub data: AccessKeyDto,
    pub secret: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessKeyData {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAccessKeyBody {
    #[serde(default)]
    pub data: Option<AccessKeyData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccessKeyBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<AccessKeyData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueTokenBody {
    #[serde(default)]
    pub access_key: Option<String>,
    #[serde(default)]
    pub secret: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
    pub token_type: String,
}

/// 状态枚举归一（ON/OFF，proto 枚举名）
fn normalize_status(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "ON" | "OFF" => Ok(up),
        _ => Err(AppError::Validation(format!(
            "invalid access key status: {v}"
        ))),
    }
}

/// 生成随机凭证 token：prefix + n 字节随机数的 hex（对齐 Go randomToken）。
fn random_token(prefix: &str, n_bytes: usize) -> Result<String, AppError> {
    use rand::RngCore;
    let mut buf = vec![0u8; n_bytes];
    rand::thread_rng().fill_bytes(&mut buf);
    Ok(format!("{prefix}{}", hex::encode(buf)))
}

fn hash_secret(secret: &str) -> String {
    let sum = Sha256::digest(secret.as_bytes());
    hex::encode(sum)
}

pub async fn access_key_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, ACCESS_KEY_COLUMNS)?;
    let repo = AccessKeyRepo::new(crate::handlers::task::db_of(&state)?);
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
        let c = crate::query::resolve_column(col, ACCESS_KEY_COLUMNS)?;
        order_parts.push(format!("\"t\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let (rows, total) = repo
        .list(
            lq.paging.offset(),
            lq.paging.limit(),
            &where_clause,
            &bind,
            &order_parts.join(", "),
        )
        .await?;
    Ok(json_ok(ListResponse::new(
        rows.iter().map(to_dto).collect(),
        total,
    )))
}

pub async fn access_key_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = AccessKeyRepo::new(crate::handlers::task::db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("access key not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

/// 创建凭证：生成 AK/SK，SK 只存摘要；归属创建者租户（机器令牌继承该租户闸门）。
pub async fn access_key_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateAccessKeyBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("invalid parameter".into()))?;
    let status = data
        .status
        .as_deref()
        .map(normalize_status)
        .transpose()?
        .unwrap_or_else(|| "ON".into());

    let access_key = random_token("ak-", 8)?;
    let secret = random_token("sk-", 32)?;
    let secret_hash = hash_secret(&secret);

    let repo = AccessKeyRepo::new(crate::handlers::task::db_of(&state)?);
    let id = repo
        .create(
            data.name.as_deref(),
            &access_key,
            &secret_hash,
            &status,
            data.expires_at.as_deref(),
            operator.tenant_id,
            operator.user_id,
        )
        .await?;
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::Internal {
            context: "reload created access key failed".into(),
            source: None,
        })?;
    Ok(json_ok(CreateAccessKeyResponse {
        data: to_dto(&row),
        secret,
    }))
}

/// 更新凭证（仅名称/状态/过期时间；AK 与摘要不可变）。
pub async fn access_key_update(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateAccessKeyBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    if id <= 0 {
        return Err(AppError::Validation("id is required".into()));
    }
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("invalid parameter".into()))?;
    let status = data.status.as_deref().map(normalize_status).transpose()?;
    let expires = data.expires_at.as_deref();

    let repo = AccessKeyRepo::new(crate::handlers::task::db_of(&state)?);
    // allowMissing=true 且不存在 → 转 Create（对齐 go-crud Update 语义）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        let st = status.unwrap_or_else(|| "ON".into());
        let access_key = random_token("ak-", 8)?;
        let secret = random_token("sk-", 32)?;
        repo.create(
            data.name.as_deref(),
            &access_key,
            &hash_secret(&secret),
            &st,
            expires,
            operator.tenant_id,
            operator.user_id,
        )
        .await?;
        return Ok(json_empty());
    }
    let updated = repo
        .update(
            id,
            data.name.as_deref(),
            status.as_deref(),
            expires.map(Some),
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("access key not found".into()));
    }
    Ok(json_empty())
}

pub async fn access_key_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = AccessKeyRepo::new(crate::handlers::task::db_of(&state)?);
    repo.delete(id).await?;
    Ok(json_empty())
}

/// 重置密钥（轮换）：新 SK 明文仅本次返回；旧 SK 的交换立即失效（摘要已换）。
pub async fn access_key_reset_secret(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let secret = random_token("sk-", 32)?;
    let repo = AccessKeyRepo::new(crate::handlers::task::db_of(&state)?);
    let updated = repo.update_secret_hash(id, &hash_secret(&secret)).await?;
    if updated == 0 {
        return Err(AppError::NotFound("access key not found".into()));
    }
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("access key not found".into()))?;
    Ok(json_ok(CreateAccessKeyResponse {
        data: to_dto(&row),
        secret,
    }))
}

/// 令牌交换（免鉴权端点，本身即认证）：AK/SK → 短期机器令牌。
/// 对齐 Go IssueToken：IP+AK 限流防爆破、不区分"不存在/密钥错"防枚举、
/// 禁用/过期拒绝、常量时间摘要比对、成功后清计数并刷新 last_used_at。
pub async fn access_key_issue_token(
    State(state): State<AppState>,
    Json(body): Json<IssueTokenBody>,
) -> Result<impl IntoResponse, AppError> {
    let ak = body
        .access_key
        .clone()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::Validation("access key and secret are required".into()))?;
    let secret = body
        .secret
        .clone()
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AppError::Validation("access key and secret are required".into()))?;

    let ip = "unknown".to_string(); // 交换端点暂无 ConnectInfo；限流按 AK 维度仍然有效
    let limiter = state.redis.clone().map(LoginRateLimiter::new);
    if let Some(l) = &limiter {
        if let Ok(true) = l.is_locked(&ip, &ak).await {
            return Err(AppError::Validation(
                "too many attempts, try again later".into(),
            ));
        }
    }

    let repo = AccessKeyRepo::new(crate::handlers::task::db_of(&state)?);
    let Some(row) = repo.get_by_access_key(&ak).await? else {
        if let Some(l) = &limiter {
            let _ = l.check_and_incr(&ip, &ak).await;
        }
        return Err(AppError::Validation("invalid access key or secret".into()));
    };
    if row.status != "ON" {
        return Err(AppError::Validation("access key is disabled".into()));
    }
    if let Some(exp) = row.expires_at.as_deref() {
        if let Ok(t) = chrono::DateTime::parse_from_rfc3339(exp) {
            if chrono::Utc::now() > t {
                return Err(AppError::Validation("access key is expired".into()));
            }
        }
    }
    let given = hash_secret(&secret);
    let stored = row.secret_hash.clone().unwrap_or_default();
    let ok = constant_time_eq(given.as_bytes(), stored.as_bytes());
    if !ok {
        if let Some(l) = &limiter {
            let _ = l.check_and_incr(&ip, &ak).await;
        }
        return Err(AppError::Validation("invalid access key or secret".into()));
    }
    if let Some(l) = &limiter {
        let _ = l.reset(&ip, &ak).await;
    }

    // 签发机器令牌：uid=0（机器身份）、username="ak:{AK}"、roles=["machine"]、
    // 继承凭证租户（下游租户隔离/Api 闸门照常生效）。仅 access、无 refresh、不进在线会话。
    let now = chrono::Utc::now().timestamp();
    let claims = AccessClaims {
        sub: format!("ak:{ak}"),
        uid: 0,
        tid: row.tenant_id,
        cid: Some("admin".into()),
        did: None,
        roc: Some(vec!["machine".into()]),
        ipa: Some(false),
        ita: Some(false),
        iat: now,
        exp: now + ACCESS_TOKEN_TTL,
        jti: auth::new_jti(),
    };
    let header = jsonwebtoken::Header::default(); // HS256
    let token = jsonwebtoken::encode(&header, &claims, &jsonwebtoken::EncodingKey::from_secret(
        state.jwt_secret.as_bytes(),
    ))
    .map_err(|e| AppError::Internal {
        context: "issue machine token failed".into(),
        source: Some(Box::new(e)),
    })?;
    // 机器令牌入缓存（对齐 Go：未入缓存的令牌会被校验器判为无效）
    auth::cache_machine_token(&state, &claims.jti, &token, ACCESS_TOKEN_TTL).await;

    // 尽力而为刷新最近使用时间
    repo.touch_last_used(row.id).await;

    Ok(json_ok(IssueTokenResponse {
        access_token: token,
        expires_in: ACCESS_TOKEN_TTL,
        token_type: "bearer".into(),
    }))
}

/// 常量时间字节比对（对齐 Go subtle.ConstantTimeCompare，防时序侧信道）。
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
