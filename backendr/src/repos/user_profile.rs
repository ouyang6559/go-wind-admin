// user_profile repo：/me 自管理（资料更新 / 联系方式绑定）。
// 对齐 Go `user_profile_contact.go` 的 VerifyContact：验证码通过后写入
// EMAIL 登录凭证（identifier=邮箱，credential 为占位哈希，不用于密码校验）。
// 绑定验证码走 Redis，键 `gowind:vcode:bind_contact:{contact}`，TTL 10 分钟，消费型（单次有效）。

use sqlx::AnyPool;

use crate::error::AppError;
use crate::state::AppState;

/// 绑定联系方式验证码 Redis 前缀（对齐 Go vcodeCache purpose="bind_contact"）
const BIND_VCODE_PREFIX: &str = "gowind:vcode:bind_contact:";
/// 绑定验证码有效期：10 分钟
const BIND_VCODE_TTL: u64 = 600;

/// 合法 bcrypt 占位哈希（EMAIL 凭证不用于密码校验，对齐 Go dummyPasswordHash）
const DUMMY_PASSWORD_HASH: &str = "$2a$10$1sbpKmhQDpXLHnDnEQ1nLe3oOnYyP2bUJyqHcX2T0Fq1qfyoXOrPm";

pub struct UserProfileRepo {
    pub db: AnyPool,
}

impl UserProfileRepo {
    pub fn new(db: AnyPool) -> Self {
        Self { db }
    }

    /// 邮箱是否已被绑定为某用户的 EMAIL 凭证（排除自身，防重复绑定）。
    pub async fn email_credential_exists(&self, email: &str, exclude_user_id: i64) -> Result<bool, AppError> {
        let sql = "select 1 from sys_user_credentials \
                   where identity_type = 'EMAIL' and identifier = $1 and user_id <> $2 and deleted_at is null \
                   limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(email)
            .bind(exclude_user_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check email credential existence failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 写入 EMAIL 登录凭证（绑定成功后）。已存在同邮箱凭证时静默跳过（幂等）。
    pub async fn bind_email_credential(
        &self,
        tenant_id: i64,
        user_id: i64,
        email: &str,
    ) -> Result<(), AppError> {
        let sql = "insert into sys_user_credentials \
                   (tenant_id, user_id, identity_type, identifier, credential_type, credential, \
                    is_primary, status, created_at, updated_at) \
                   values ($1, $2, 'EMAIL', $3, 'PASSWORD_HASH', $4, false, 'ENABLED', $5::timestamptz, $5::timestamptz) \
                   on conflict do nothing";
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query::<sqlx::Any>(sql)
            .bind(tenant_id)
            .bind(user_id)
            .bind(email)
            .bind(DUMMY_PASSWORD_HASH)
            .bind(now)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "create email credential failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(())
    }
}

/// 生成并保存绑定联系方式验证码（6 位数字，TTL 10 分钟）。
pub async fn save_bind_vcode(state: &AppState, contact: &str) -> Result<String, AppError> {
    let client = state.redis.as_ref().ok_or_else(|| AppError::Internal {
        context: "redis not configured".into(),
        source: None,
    })?;
    use rand::Rng;
    let code = format!("{:06}", rand::thread_rng().gen_range(0..1_000_000u32));
    use redis::AsyncCommands;
    let mut con = client.get_multiplexed_tokio_connection().await.map_err(|e| {
        AppError::Internal {
            context: "redis connect failed".into(),
            source: Some(Box::new(e)),
        }
    })?;
    let key = format!("{BIND_VCODE_PREFIX}{contact}");
    let _: () = con.set_ex(&key, code.as_str(), BIND_VCODE_TTL).await.map_err(|e| {
        AppError::Internal {
            context: "save bind vcode failed".into(),
            source: Some(Box::new(e)),
        }
    })?;
    Ok(code)
}

/// 校验绑定验证码（消费型：匹配即删除，单次有效）。
pub async fn verify_bind_vcode(state: &AppState, contact: &str, code: &str) -> Result<bool, AppError> {
    let client = state.redis.as_ref().ok_or_else(|| AppError::Internal {
        context: "redis not configured".into(),
        source: None,
    })?;
    use redis::AsyncCommands;
    let mut con = client.get_multiplexed_tokio_connection().await.map_err(|e| {
        AppError::Internal {
            context: "redis connect failed".into(),
            source: Some(Box::new(e)),
        }
    })?;
    let key = format!("{BIND_VCODE_PREFIX}{contact}");
    let stored: Option<String> = con.get(&key).await.map_err(|e| AppError::Internal {
        context: "get bind vcode failed".into(),
        source: Some(Box::new(e)),
    })?;
    match stored {
        Some(v) if v == code.trim() => {
            let _: () = con.del(&key).await.map_err(|e| AppError::Internal {
                context: "consume bind vcode failed".into(),
                source: Some(Box::new(e)),
            })?;
            Ok(true)
        }
        _ => Ok(false),
    }
}