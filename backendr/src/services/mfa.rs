// mfa service：TOTP 因子注册/验证 + Redis 挑战缓存。
// 对齐 Go 端 mfa_service.go + mfa_challenge_cache.go：
// - enroll 挑战：mfa:enroll:{op_id}（Peek 不消耗，落库成功才删，允许首码重试）
// - enroll 冷却：mfa:enrollcd:{tid}:{uid}（30s）
// - login 挑战：mfa:login:{op_id}（Peek 不消耗，通过或超限才原子消耗）
// - login 失败计数：mfa:loginfail:{op_id}（达上限作废挑战）
// TOTP secret 加密落库用 AES-256-GCM（GOWIND_CRYPTO_KEY 派生），与渠道密码同套 crypto 工具。

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::repos::mfa::MfaRepo;
use crate::state::AppState;

/// 挑战/注册操作上下文有效期（5 分钟，对齐 Go MfaChallengeTTL）
const MFA_CHALLENGE_TTL: u64 = 300;
/// 注册发起冷却（30s，防已登录用户循环 Start 塞 Redis）
const MFA_ENROLL_COOLDOWN: u64 = 30;
/// 单个登录挑战允许的验证失败上限（对齐 Go MaxLoginChallengeFailures）
const MAX_LOGIN_CHALLENGE_FAILURES: i64 = 3;
/// TOTP otpauth URI 发行方标识
const MFA_TOTP_ISSUER: &str = "GoWindAdmin";
/// 允许的时间窗口偏移（±1 个 30s 周期）
const MFA_TOTP_SKEW: u8 = 1;

const MFA_LOGIN_KEY_PREFIX: &str = "mfa:login:";
const MFA_ENROLL_KEY_PREFIX: &str = "mfa:enroll:";
const MFA_LOGIN_FAIL_KEY_PREFIX: &str = "mfa:loginfail:";
const MFA_ENROLL_CD_PREFIX: &str = "mfa:enrollcd:";

/// 登录挑战上下文（密码校验通过、待二次验证阶段由登录链路写入）。
#[derive(Debug, Serialize, Deserialize)]
pub struct MfaLoginChallengeContext {
    pub user_id: i64,
    pub tenant_id: i64,
    pub username: String,
    pub client_id: Option<String>,
    pub device_id: Option<String>,
    pub client_type: String,
    pub role_codes: Vec<String>,
    pub ipa: bool,
    pub ita: bool,
}

/// 注册挑战上下文。
#[derive(Debug, Serialize, Deserialize)]
pub struct MfaEnrollChallengeContext {
    pub secret: String,
    pub tenant_id: i64,
    pub user_id: i64,
}

/// 生成 TOTP 密钥与 otpauth URI、QR data URI。
pub struct TotpEnrollBundle {
    pub secret: String,
    pub otp_auth_url: String,
    pub qr_code_data_uri: String,
}

pub struct MfaService {
    pub repo: MfaRepo,
    pub state: AppState,
}

impl MfaService {
    pub fn from_state(state: &AppState) -> Result<Self, AppError> {
        let db = state
            .db
            .clone()
            .ok_or_else(|| AppError::Internal {
                context: "database not configured; mfa unavailable".into(),
                source: None,
            })?;
        Ok(Self {
            repo: MfaRepo::new(db),
            state: state.clone(),
        })
    }

    /// 生成 TOTP 注册材料：随机 base32 secret（160 位）、otpauth URL、PNG QR data URI。
    /// 账户名用 uid:x 防认证器同名冲突（对齐 Go）。
    pub fn generate_totp(&self, user_id: i64) -> Result<TotpEnrollBundle, AppError> {
        // 随机 20 字节 → base32 无填充 secret（totp-rs Secret::Raw 校验 128 bit+），
        // 用 to_encoded() 生成 base32 展示串（避免额外引入 base32 crate）
        use rand::RngCore;
        let mut raw = [0u8; 20];
        rand::thread_rng().fill_bytes(&mut raw);
        let secret_b32 = totp_rs::Secret::Raw(raw.to_vec())
            .to_encoded()
            .to_string();

        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            MFA_TOTP_SKEW,
            30,
            totp_rs::Secret::Raw(raw.to_vec())
                .to_bytes()
                .map_err(|e| AppError::Internal {
                    context: format!("encode totp secret failed: {e:?}"),
                    source: None,
                })?,
            Some(MFA_TOTP_ISSUER.to_string()),
            // 账户名禁用冒号（totp-rs otpauth 校验；Go 用 "uid:{id}"，这里改成 "uid{id}"）
            format!("uid{user_id}"),
        )
        .map_err(|e| AppError::Internal {
            context: format!("create totp key failed: {e}"),
            source: None,
        })?;

        let otp_auth_url = totp.get_url();
        let png = totp
            .get_qr_png()
            .map_err(|e| AppError::Internal {
                context: format!("render totp qr failed: {e}"),
                source: None,
            })?;
        let qr_code_data_uri = format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(png)
        );

        Ok(TotpEnrollBundle {
            secret: secret_b32,
            otp_auth_url,
            qr_code_data_uri,
        })
    }

    /// 校验 TOTP 码（±1 窗口，SHA1，6 位，30s），对齐 Go otpTotp.ValidateCustom。
    pub fn verify_totp(&self, code: &str, plain_secret: &str) -> bool {
        let Ok(bytes) = totp_rs::Secret::Encoded(plain_secret.to_string()).to_bytes() else {
            return false;
        };
        let Ok(totp) = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            MFA_TOTP_SKEW,
            30,
            bytes,
            None,
            String::new(),
        ) else {
            return false;
        };
        totp.check_current(code).unwrap_or(false)
    }

    // ===================== Redis 挑战缓存 =====================

    async fn redis_conn(&self) -> Result<redis::aio::MultiplexedConnection, AppError> {
        let client = self.state.redis.as_ref().ok_or_else(|| AppError::Internal {
            context: "redis not configured; mfa unavailable".into(),
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

    fn new_operation_id(&self) -> String {
        crate::auth::new_jti()
    }

    /// 写入登录挑战上下文，返回 operation_id。
    pub async fn set_login_challenge(
        &self,
        ctx: &MfaLoginChallengeContext,
    ) -> Result<String, AppError> {
        let op_id = self.new_operation_id();
        let raw = serde_json::to_string(ctx).map_err(|e| AppError::Internal {
            context: "serialize login challenge failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut con = self.redis_conn().await?;
        use redis::AsyncCommands;
        let key = format!("{MFA_LOGIN_KEY_PREFIX}{op_id}");
        let _: () = con
            .set_ex(&key, &raw, MFA_CHALLENGE_TTL)
            .await
            .map_err(|e| AppError::Internal {
                context: "set login challenge failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(op_id)
    }

    /// 只读登录挑战上下文（不删除，供失败重试）。
    pub async fn peek_login_challenge(
        &self,
        op_id: &str,
    ) -> Result<Option<MfaLoginChallengeContext>, AppError> {
        let mut con = self.redis_conn().await?;
        use redis::AsyncCommands;
        let key = format!("{MFA_LOGIN_KEY_PREFIX}{op_id}");
        let raw: Option<String> = con.get(&key).await.map_err(|e| AppError::Internal {
            context: "peek login challenge failed".into(),
            source: Some(Box::new(e)),
        })?;
        let Some(raw) = raw else {
            return Ok(None);
        };
        let ctx = serde_json::from_str(&raw).map_err(|e| AppError::Internal {
            context: "unmarshal login challenge failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(Some(ctx))
    }

    /// 记录一次验证失败并返回挑战是否已达失败上限。达上限顺带作废挑战。
    pub async fn record_login_failure(&self, op_id: &str) -> bool {
        let Ok(mut con) = self.redis_conn().await else {
            return true; // Redis 异常 fail-closed 按已达上限处理
        };
        use redis::AsyncCommands;
        let fail_key = format!("{MFA_LOGIN_FAIL_KEY_PREFIX}{op_id}");
        let n: Result<i64, _> = con.incr(&fail_key, 1).await;
        let n = match n {
            Ok(n) => n,
            Err(_) => return true,
        };
        let _: Result<(), _> = con.expire(&fail_key, MFA_CHALLENGE_TTL as i64).await;
        if n >= MAX_LOGIN_CHALLENGE_FAILURES {
            let _: Result<(), _> = con
                .del(format!("{MFA_LOGIN_KEY_PREFIX}{op_id}"))
                .await;
            return true;
        }
        false
    }

    /// 原子消耗登录挑战（GET+DEL 脚本）：返回是否抢到（裁决权）。并发同 opId 仅先抢到者发 token。
    pub async fn take_login_challenge_atomic(&self, op_id: &str) -> bool {
        let Ok(mut con) = self.redis_conn().await else {
            return false;
        };
        use redis::AsyncCommands;
        // Lua 脚本保证 GET+DEL 原子性（redis-rs 无 getdel 命令封装，用脚本等价）
        let key = format!("{MFA_LOGIN_KEY_PREFIX}{op_id}");
        let script = redis::Script::new("local v = redis.call('GET', KEYS[1]); if v then redis.call('DEL', KEYS[1]); return v; end; return false");
        let got: Result<Option<String>, _> = script.arg(1).key(&key).invoke_async(&mut con).await;
        match got {
            Ok(Some(v)) if !v.is_empty() => {
                let _: Result<(), _> = con
                    .del(format!("{MFA_LOGIN_FAIL_KEY_PREFIX}{op_id}"))
                    .await;
                true
            }
            _ => false,
        }
    }

    /// 消耗登录挑战（非裁决路径：失败上限/因子缺失等），对齐 Go ConsumeLoginChallenge。
    pub async fn consume_login_challenge(&self, op_id: &str) {
        let Ok(mut con) = self.redis_conn().await else {
            return;
        };
        use redis::AsyncCommands;
        let _: Result<(), _> = con
            .del(vec![
                format!("{MFA_LOGIN_KEY_PREFIX}{op_id}"),
                format!("{MFA_LOGIN_FAIL_KEY_PREFIX}{op_id}"),
            ])
            .await;
    }

    /// 注册频控：同一用户冷却期内只允许发起一次 StartEnroll（防循环 Start 塞 Redis）。
    pub async fn try_acquire_enroll_cooldown(
        &self,
        tenant_id: i64,
        user_id: i64,
    ) -> bool {
        let Ok(mut con) = self.redis_conn().await else {
            return true; // Redis 异常放行（频控是防御性优化）
        };
        use redis::AsyncCommands;
        let key = format!("{MFA_ENROLL_CD_PREFIX}{tenant_id}:{user_id}");
        let ok: Result<bool, _> = con.set_nx(&key, "1").await;
        let ok = match ok {
            Ok(true) => {
                let _: Result<(), _> = con.expire(&key, MFA_ENROLL_COOLDOWN as i64).await;
                true
            }
            Ok(false) => false,
            Err(_) => true,
        };
        ok
    }

    /// 写入注册挑战上下文（含 TOTP secret），返回 operation_id。
    pub async fn set_enroll_challenge(
        &self,
        ctx: &MfaEnrollChallengeContext,
    ) -> Result<String, AppError> {
        let op_id = self.new_operation_id();
        let raw = serde_json::to_string(ctx).map_err(|e| AppError::Internal {
            context: "serialize enroll challenge failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut con = self.redis_conn().await?;
        use redis::AsyncCommands;
        let key = format!("{MFA_ENROLL_KEY_PREFIX}{op_id}");
        let _: () = con
            .set_ex(&key, &raw, MFA_CHALLENGE_TTL)
            .await
            .map_err(|e| AppError::Internal {
                context: "set enroll challenge failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(op_id)
    }

    /// 只读注册挑战上下文（不删除，允许首码输错重试）。
    pub async fn peek_enroll_challenge(
        &self,
        op_id: &str,
    ) -> Result<Option<MfaEnrollChallengeContext>, AppError> {
        let mut con = self.redis_conn().await?;
        use redis::AsyncCommands;
        let key = format!("{MFA_ENROLL_KEY_PREFIX}{op_id}");
        let raw: Option<String> = con.get(&key).await.map_err(|e| AppError::Internal {
            context: "peek enroll challenge failed".into(),
            source: Some(Box::new(e)),
        })?;
        let Some(raw) = raw else {
            return Ok(None);
        };
        let ctx = serde_json::from_str(&raw).map_err(|e| AppError::Internal {
            context: "unmarshal enroll challenge failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(Some(ctx))
    }

    /// 删除注册挑战（注册成功后消耗）。
    pub async fn delete_enroll_challenge(&self, op_id: &str) {
        let Ok(mut con) = self.redis_conn().await else {
            return;
        };
        use redis::AsyncCommands;
        let _: Result<(), _> = con.del(format!("{MFA_ENROLL_KEY_PREFIX}{op_id}")).await;
    }
}