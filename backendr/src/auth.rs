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
///
/// 对齐 Go：access/refresh 令牌**共享同一 `jti`**（Go 端 `UserTokenPayload.Jti`
/// 同时注入两个令牌）。这样会话元数据以该 jti 为键，刷新轮换时可直接用旧
/// refresh token 的 jti 查到旧会话元数据，继承首次 login_at。
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
    jti: &str,
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
        jti: jti.to_string(),
    };
    let refresh = RefreshClaims {
        uid,
        iat: now,
        exp: now + REFRESH_TOKEN_TTL,
        jti: jti.to_string(),
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
        access_jti: jti.to_string(),
        refresh_jti: jti.to_string(),
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
// ===================== 会话元数据与吊销（在线会话模块基础） =====================

/// 单条在线会话元数据（对齐 Go `us:` 键存的内容，另附定位字段）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionMeta {
    pub jti: String,
    #[serde(rename = "clientType")]
    pub client_type: String,
    pub uid: i64,
    pub username: String,
    #[serde(rename = "tenantId")]
    pub tenant_id: i64,
    pub ip: String,
    pub ua: String,
    pub dev: String,
    /// 登录时间（RFC3339）
    #[serde(rename = "loginAt")]
    pub login_at: String,
}

/// 会话元数据 key（v1：按 access jti 一会话一条；刷新轮换后 loginAt 重新计时）。
fn session_meta_key(client_type: &str, uid: i64, jti: &str) -> String {
    format!("gw:session:meta:{client_type}:{uid}:{jti}")
}

/// 令牌黑名单 key（存在即视为已吊销，TTL 覆盖令牌剩余有效期）。
fn blacklist_key(jti: &str) -> String {
    format!("gw:bl:{jti}")
}

async fn redis_conn(state: &AppState) -> Result<redis::aio::MultiplexedConnection, AppError> {
    let client = state.redis.as_ref().ok_or_else(|| AppError::Internal {
        context: "redis not configured".into(),
        source: None,
    })?;
    client
        .get_multiplexed_tokio_connection()
        .await
        .map_err(|e| AppError::Internal {
            context: "redis connect failed".into(),
            source: Some(Box::new(e)),
        })
}

/// 记录会话元数据（失败由调用方决定是否阻断；登录链路中不阻断）。
pub async fn record_session_meta(
    state: &AppState,
    meta: &SessionMeta,
) -> Result<(), AppError> {
    let payload = serde_json::to_string(meta).map_err(|e| AppError::Internal {
        context: "serialize session meta failed".into(),
        source: Some(Box::new(e)),
    })?;
    let mut con = redis_conn(state).await?;
    use redis::AsyncCommands;
    let key = session_meta_key(&meta.client_type, meta.uid, &meta.jti);
    let _: () = con
        .set_ex(&key, payload, REFRESH_TOKEN_TTL as u64)
        .await
        .map_err(|e| AppError::Internal {
            context: "redis set session meta failed".into(),
            source: Some(Box::new(e)),
        })?;
    Ok(())
}

/// 读取单个会话元数据（按 客户端类型+uid+jti）。会话不存在返回 None。
/// 刷新轮换时用于继承首次 login_at（对齐 Go `GetSessionMeta`）。
pub async fn get_session_meta(
    state: &AppState,
    client_type: &str,
    uid: i64,
    jti: &str,
) -> Result<Option<SessionMeta>, AppError> {
    if state.redis.is_none() {
        return Ok(None);
    }
    let mut con = redis_conn(state).await?;
    use redis::AsyncCommands;
    let key = session_meta_key(client_type, uid, jti);
    let raw: Option<String> = con.get(&key).await.map_err(|e| AppError::Internal {
        context: "redis get session meta failed".into(),
        source: Some(Box::new(e)),
    })?;
    match raw {
        Some(v) => serde_json::from_str(&v)
            .map(Some)
            .map_err(|e| AppError::Internal {
                context: "deserialize session meta failed".into(),
                source: Some(Box::new(e)),
            }),
        None => Ok(None),
    }
}

/// 将 jti 加入黑名单（吊销令牌）。
pub async fn blacklist_jti(state: &AppState, jti: &str, ttl_secs: i64) -> Result<(), AppError> {
    let mut con = redis_conn(state).await?;
    use redis::AsyncCommands;
    let _: () = con
        .set_ex(blacklist_key(jti), "1", ttl_secs.max(1) as u64)
        .await
        .map_err(|e| AppError::Internal {
            context: "redis blacklist failed".into(),
            source: Some(Box::new(e)),
        })?;
    Ok(())
}

/// 查询 jti 是否已被吊销（redis 不可用时跳过检查，返回 false）。
pub async fn is_jti_blacklisted(state: &AppState, jti: &str) -> bool {
    if state.redis.is_none() {
        return false;
    }
    let Ok(mut con) = redis_conn(state).await else {
        return false;
    };
    use redis::AsyncCommands;
    let exists: Result<bool, _> = con.exists(blacklist_key(jti)).await;
    exists.unwrap_or(false)
}

/// 强制下线单个会话：删 SET 成员 + 删元数据 + 黑名单两个 jti。
pub async fn revoke_session_by_jti(
    state: &AppState,
    client_type: &str,
    uid: i64,
    access_jti: &str,
    refresh_jti: Option<&str>,
) -> Result<(), AppError> {
    let mut con = redis_conn(state).await?;
    use redis::AsyncCommands;
    let set_key = session_key(client_type, uid);
    let _: () = con.srem(&set_key, access_jti).await.map_err(|e| {
        AppError::Internal {
            context: "redis srem session failed".into(),
            source: Some(Box::new(e)),
        }
    })?;
    if let Some(rj) = refresh_jti {
        let _: () = con.srem(&set_key, rj).await.map_err(|e| {
            AppError::Internal {
                context: "redis srem refresh session failed".into(),
                source: Some(Box::new(e)),
            }
        })?;
    }
    let meta_key = session_meta_key(client_type, uid, access_jti);
    let _: () = con.del(&meta_key).await.map_err(|e| AppError::Internal {
        context: "redis del session meta failed".into(),
        source: Some(Box::new(e)),
    })?;
    drop(con);
    blacklist_jti(state, access_jti, ACCESS_TOKEN_TTL).await?;
    if let Some(rj) = refresh_jti {
        blacklist_jti(state, rj, REFRESH_TOKEN_TTL).await?;
    }
    Ok(())
}

/// 枚举全部在线会话元数据（SCAN `gw:session:meta:*`）。
pub async fn scan_session_metas(state: &AppState) -> Result<Vec<SessionMeta>, AppError> {
    let mut con = redis_conn(state).await?;
    let mut metas = Vec::new();
    let mut cursor: u64 = 0;
    loop {
        let (next, keys): (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg("gw:session:meta:*")
            .arg("COUNT")
            .arg(200)
            .query_async(&mut con)
            .await
            .map_err(|e| AppError::Internal {
                context: "redis scan session metas failed".into(),
                source: Some(Box::new(e)),
            })?;
        if !keys.is_empty() {
            let values: Vec<Option<String>> = redis::cmd("MGET")
                .arg(&keys)
                .query_async(&mut con)
                .await
                .map_err(|e| AppError::Internal {
                    context: "redis mget session metas failed".into(),
                    source: Some(Box::new(e)),
                })?;
            for v in values.into_iter().flatten() {
                if let Ok(meta) = serde_json::from_str::<SessionMeta>(&v) {
                    metas.push(meta);
                }
            }
        }
        cursor = next;
        if cursor == 0 {
            break;
        }
    }
    Ok(metas)
}

/// 撤销某用户某客户端的全部会话：黑名单集合中的全部 jti、删除集合与元数据。
/// `except_jti` 用于刷新链路（新会话已在调用后写入，不会被本函数覆盖——
/// 调用顺序为先撤销再记录）。
pub async fn revoke_all_sessions(
    state: &AppState,
    client_type: &str,
    uid: i64,
    except_jti: &str,
) -> Result<(), AppError> {
    let mut con = redis_conn(state).await?;
    use redis::AsyncCommands;
    let set_key = session_key(client_type, uid);
    let members: Vec<String> = con.smembers(&set_key).await.unwrap_or_default();
    drop(con);
    for jti in &members {
        if jti == except_jti {
            continue;
        }
        // 集合里混合 access/refresh jti，统一按最长 TTL 拉黑以稳妥覆盖
        let _ = blacklist_jti(state, jti, REFRESH_TOKEN_TTL).await;
        // 顺带清理元数据
        if let Ok(mut con2) = redis_conn(state).await {
            use redis::AsyncCommands;
            let meta_key = session_meta_key(client_type, uid, jti);
            let _: Result<(), _> = con2.del(&meta_key).await;
        }
    }
    let mut con = redis_conn(state).await?;
    let _: () = con.del(&set_key).await.map_err(|e| AppError::Internal {
        context: "redis del sessions failed".into(),
        source: Some(Box::new(e)),
    })?;
    Ok(())
}
