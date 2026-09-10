//! JWT 令牌（签发/验签）与 Redis 在线会话。
//!
//! Claims 的字段名对齐 Kratos `pkg/jwt`：
//! - **access token**：`sub`(用户名)、`uid`(用户ID)、`tid`(租户ID)、`iat/exp/jti`、
//!   `cid`(客户端ID)、`did`(设备ID)、`roc`(角色码数组)、`ipa`(平台管理员)、`ita`(租户管理员)
//! - **refresh token**：仅 `uid`、`jti`、`iat`、`exp`
//! 签名算法 HS256，密钥为 `GW_ADMIN_JWT_SECRET`。

use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation,
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::state::AppState;

/// access token 有效期：2 小时（对齐 Kratos DefaultTokenExpiration）
pub const ACCESS_TOKEN_TTL: i64 = 2 * 60 * 60;
/// refresh token 有效期：7 天
pub const REFRESH_TOKEN_TTL: i64 = 7 * 24 * 60 * 60;
/// 令牌时间容差 60s（jsonwebtoken 的 `leeway` 为 u64）
const TOKEN_LEEWAY: u64 = 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessClaims {
    pub sub: String,
    pub uid: i64,
    pub tid: i64,
    pub cid: Option<String>,
    pub did: Option<String>,
    pub roc: Option<Vec<String>>,
    pub ipa: Option<bool>,
    pub ita: Option<bool>,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshClaims {
    pub uid: i64,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,
}

/// 登录成功后签发的令牌对
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_jti: String,
    pub refresh_jti: String,
}

/// 签发 access + refresh 令牌对。
pub fn issue_token_pair(
    secret: &str,
    uid: i64,
    tid: i64,
    username: &str,
    client_id: Option<String>,
    device_id: Option<String>,
    role_codes: Vec<String>,
    ipa: bool,
    ita: bool,
    access_jti: &str,
    refresh_jti: &str,
) -> Result<TokenPair, AppError> {
    let now = chrono::Utc::now().timestamp();
    let access = AccessClaims {
        sub: username.to_string(),
        uid,
        tid,
        cid: client_id,
        did: device_id,
        roc: if role_codes.is_empty() {
            None
        } else {
            Some(role_codes)
        },
        ipa: if ipa { Some(true) } else { None },
        ita: if ita { Some(true) } else { None },
        iat: now,
        exp: now + ACCESS_TOKEN_TTL,
        jti: access_jti.to_string(),
    };
    let refresh = RefreshClaims {
        uid,
        iat: now,
        exp: now + REFRESH_TOKEN_TTL,
        jti: refresh_jti.to_string(),
    };

    let enc = EncodingKey::from_secret(secret.as_bytes());
    let header = Header::new(Algorithm::HS256);
    let access_token =
        encode(&header, &access, &enc).map_err(|e| AppError::Internal {
            context: "sign access token failed".into(),
            source: Some(Box::new(e)),
        })?;
    let refresh_token = encode(&header, &refresh, &enc).map_err(|e| AppError::Internal {
        context: "sign refresh token failed".into(),
        source: Some(Box::new(e)),
    })?;

    Ok(TokenPair {
        access_token,
        refresh_token,
        access_jti: access_jti.to_string(),
        refresh_jti: refresh_jti.to_string(),
    })
}

fn decoding_key(secret: &str) -> DecodingKey {
    DecodingKey::from_secret(secret.as_bytes())
}

fn validation() -> Validation {
    let mut v = Validation::new(Algorithm::HS256);
    v.leeway = TOKEN_LEEWAY;
    v
}

/// 校验并解析 access token，返回 claims。
pub fn verify_access_token(secret: &str, token: &str) -> Result<AccessClaims, AppError> {
    decode::<AccessClaims>(token, &decoding_key(secret), &validation())
        .map(|td| td.claims)
        .map_err(|e| match e.kind().clone() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => {
                AppError::Unauthorized
            }
            _ => AppError::Unauthorized,
        })
}

/// 校验并解析 refresh token（独立验签，不依赖 access token）。
pub fn verify_refresh_token(secret: &str, token: &str) -> Result<RefreshClaims, AppError> {
    decode::<RefreshClaims>(token, &decoding_key(secret), &validation())
        .map(|td| td.claims)
        .map_err(|_| AppError::Validation("invalid refresh token".into()))
}

/// 从 Authorization 头提取 Bearer token（返回 None 表示无 token）。
pub fn bearer_token(header: Option<&str>) -> Option<String> {
    let h = header?.trim();
    let token = h.strip_prefix("Bearer ")?.trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

/// 生成会话 jti（UUID v4）。
pub fn new_jti() -> String {
    uuid::Uuid::new_v4().to_string()
}

// ===================== Redis 在线会话 =====================

/// 会话集合 key（一个用户一次登录存 access/refresh 两个 jti）。
fn session_key(client_type: &str, uid: i64) -> String {
    format!("gw:sessions:{client_type}:{uid}")
}

/// 登录成功后记录会话 jti（TTL 对齐 refresh token 有效期）。
pub async fn record_session(
    state: &AppState,
    client_type: &str,
    uid: i64,
    jtis: &[String],
) -> Result<(), AppError> {
    use redis::AsyncCommands;
    let client = state.redis.as_ref().ok_or_else(|| AppError::Internal {
        context: "redis not configured (session record skipped)".into(),
        source: None,
    })?;
    let mut con = client
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|e| AppError::Internal {
            context: "redis connect failed".into(),
            source: Some(Box::new(e)),
        })?;
    let key = session_key(client_type, uid);
    let _: () = con
        .sadd(&key, jtis)
        .await
        .map_err(|e| AppError::Internal {
            context: "redis sadd failed".into(),
            source: Some(Box::new(e)),
        })?;
    let _: () = con
        .expire(&key, REFRESH_TOKEN_TTL)
        .await
        .map_err(|e| AppError::Internal {
            context: "redis expire failed".into(),
            source: Some(Box::new(e)),
        })?;
    Ok(())
}

/// 登出：删除该用户该客户端的全部会话记录（等效于撤销所有令牌）。
pub async fn revoke_sessions(state: &AppState, client_type: &str, uid: i64) -> Result<(), AppError> {
    use redis::AsyncCommands;
    let client = state.redis.as_ref().unwrap(); // AppState 保证 redis 存在；不存在时函数不调用
    let mut con = client
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|e| AppError::Internal {
            context: "redis connect failed".into(),
            source: Some(Box::new(e)),
        })?;
    let key = session_key(client_type, uid);
    let _: () = con.del(&key).await.map_err(|e| AppError::Internal {
        context: "redis del failed".into(),
        source: Some(Box::new(e)),
    })?;
    Ok(())
}