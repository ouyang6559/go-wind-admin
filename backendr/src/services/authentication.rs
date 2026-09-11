// authentication service：登录/验证码/登出/刷新/注册业务逻辑。
// 对齐 Kratos `authentication_service.go` 的核心链路，但不含 MFA/登录策略/限流（后续模块补齐）。

use base64::Engine;
use rand::Rng;

use crate::auth::{self, TokenPair};
use crate::error::AppError;
use crate::repos::authentication::AuthenticationRepo;
use crate::state::AppState;

/// 平台管理员角色码（用于 token 标志位，值对齐 Kratos constants）
const PLATFORM_ADMIN_ROLE_CODE: &str = "platform_admin";
const TENANT_ADMIN_ROLE_CODE: &str = "tenant_admin";
/// 是否强制校验登录验证码（对齐 Kratos CaptchaEnabled）
const CAPTCHA_ENABLED: bool = true;
/// 验证码答案 Redis 前缀
const CAPTCHA_REDIS_PREFIX: &str = "gowind:captcha:";
/// 验证码有效期（秒）
const CAPTCHA_TTL_SECS: u64 = 300;
/// 密码重置验证码 Redis 前缀（对齐 Go `gowind:vcode:reset_password:{identifier}`）
const RESET_VCODE_PREFIX: &str = "gowind:vcode:reset_password:";
/// 验证码有效期：10 分钟
const RESET_VCODE_TTL: u64 = 600;


pub struct AuthenticationService {
    pub repo: AuthenticationRepo,
    pub state: AppState,
}

impl AuthenticationService {
    pub fn from_state(state: &AppState) -> Result<Self, AppError> {
        let db = state
            .db
            .clone()
            .ok_or_else(|| AppError::Internal {
                context: "database not configured; authentication unavailable".into(),
                source: None,
            })?;
        Ok(Self {
            repo: AuthenticationRepo::new(db),
            state: state.clone(),
        })
    }

    fn client_type(&self, req_client_type: Option<&str>) -> String {
        match req_client_type {
            Some(c) if !c.is_empty() => c.to_string(),
            _ => "admin".to_string(),
        }
    }

    /// 生成验证码：随机 4 字符 + Redis 落答案，返回 SVG data URI 图片。
    pub async fn generate_captcha(&self) -> Result<(String, String), AppError> {
        let code = gen_captcha_code();
        let captcha_id = auth::new_jti();

        // 保存答案到 Redis
        if let Some(client) = &self.state.redis {
            use redis::AsyncCommands;
            let mut con = client.get_multiplexed_tokio_connection().await.map_err(|e| {
                AppError::Internal {
                    context: "redis connect failed".into(),
                    source: Some(Box::new(e)),
                }
            })?;
            let key = format!("{CAPTCHA_REDIS_PREFIX}{captcha_id}");
            let _: () = con
                .set_ex(&key, code.as_str(), CAPTCHA_TTL_SECS)
                .await
                .map_err(|e| AppError::Internal {
                    context: "save captcha to redis failed".into(),
                    source: Some(Box::new(e)),
                })?;
        }

        let svg = render_captcha_svg(&code);
        let image_base64 = format!(
            "data:image/svg+xml;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(svg)
        );
        Ok((captcha_id, image_base64))
    }

    /// 校验验证码（verify-and-delete 单次有效）。
    pub async fn verify_captcha(&self, captcha_id: &str, user_input: &str) -> Result<bool, AppError> {
        let client = self.state.redis.as_ref().ok_or_else(|| AppError::Internal {
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
        let key = format!("{CAPTCHA_REDIS_PREFIX}{captcha_id}");
        let stored: Option<String> = con.get(&key).await.map_err(|e| AppError::Internal {
            context: "get captcha from redis failed".into(),
            source: Some(Box::new(e)),
        })?;
        let _: () = con.del(&key).await.map_err(|e| AppError::Internal {
            context: "del captcha from redis failed".into(),
            source: Some(Box::new(e)),
        })?;
        match stored {
            Some(answer) => Ok(answer.eq_ignore_ascii_case(user_input.trim())),
            None => Ok(false),
        }
    }

    /// 登录（password 授权）。
    #[allow(clippy::too_many_arguments)]
    pub async fn login(
        &self,
        username: &str,
        password: &str,
        tenant_code: Option<&str>,
        client_id: Option<&str>,
        device_id: Option<&str>,
        req_client_type: Option<&str>,
        captcha_id: Option<&str>,
        captcha_value: Option<&str>,
        login_ip: &str,
        login_ua: &str,
    ) -> Result<TokenIssue, AppError> {
        // 验证码闸门：Redis 已配置时强制校验（verify-and-delete 单次有效）
        if CAPTCHA_ENABLED && self.state.redis.is_some() {
            let cid = captcha_id.map(str::trim).unwrap_or("");
            let cv = captcha_value.map(str::trim).unwrap_or("");
            if cid.is_empty() || cv.is_empty() {
                return Err(AppError::Validation("invalid or missing captcha".into()));
            }
            let ok = self.verify_captcha(cid, cv).await?;
            if !ok {
                return Err(AppError::Validation("invalid or missing captcha".into()));
            }
        }

        // 解密传输密码（base64 + AES-128-CBC）
        let plain_password = crate::crypto::decrypt_transport_secret(password)?;

        // 租户解析：tenant_code 留空视为平台（tenant 0）
        let mut tenant_id: i64 = 0;
        if let Some(code) = tenant_code {
            let code = code.trim();
            if !code.is_empty() {
                let resolved = self.repo.get_tenant_id_by_code(code).await?;
                tenant_id = resolved.ok_or_else(|| {
                    AppError::Validation("invalid tenant".into())
                })?;
            }
        }

        // identifier 智能解析（含 @ 按 email、纯数字按 mobile 反查 username）
        let mut cred_username = username.to_string();
        if let Some(resolved) = self
            .repo
            .find_username_by_identifier(tenant_id, username)
            .await?
        {
            cred_username = resolved;
        }

        // 凭证校验
        let cred = self
            .repo
            .get_credential(tenant_id, &cred_username)
            .await?
            .ok_or_else(|| {
                // 恒定时间防护：用户不存在时也跑一次 bcrypt 校验
                let _ = bcrypt::verify(&plain_password, DUMMY_BCRYPT_HASH);
                AppError::Validation("invalid username or password".into())
            })?;

        if cred.status != "ENABLED" {
            let _ = bcrypt::verify(&plain_password, DUMMY_BCRYPT_HASH);
            return Err(AppError::Validation("invalid username or password".into()));
        }

        let password_ok = if cred.credential_type == "PASSWORD_HASH" {
            bcrypt::verify(&plain_password, &cred.credential).unwrap_or(false)
        } else {
            plain_password == cred.credential
        };
        if !password_ok {
            return Err(AppError::Validation("invalid username or password".into()));
        }

        // 载入用户
        let user = self
            .repo
            .get_user_by_id(cred.user_id)
            .await?
            .ok_or_else(|| AppError::Unauthorized)?;
        if user.status != "NORMAL" {
            return Err(AppError::Forbidden("user is disabled".into()));
        }
        // 纵深防御：凭证租户必须与用户租户一致
        if user.tenant_id != tenant_id {
            return Err(AppError::Validation("invalid tenant".into()));
        }

        // 角色码 + 管理员标志
        let role_codes = self.repo.list_role_codes(user.id).await?;
        let ipa = role_codes.iter().any(|c| c == PLATFORM_ADMIN_ROLE_CODE);
        let ita = role_codes.iter().any(|c| c == TENANT_ADMIN_ROLE_CODE);

        let client_type = self.client_type(req_client_type);
        let access_jti = auth::new_jti();
        let refresh_jti = auth::new_jti();
        let pair = auth::issue_token_pair(
            &self.state.jwt_secret,
            user.id,
            user.tenant_id,
            &user.username,
            client_id.map(str::to_string),
            device_id.map(str::to_string),
            role_codes,
            ipa,
            ita,
            &access_jti,
            &refresh_jti,
        )?;

        // 记录会话 + 会话元数据（在线会话列表用）+ 更新最近登录
        let _ = auth::record_session(
            &self.state,
            &client_type,
            user.id,
            &[pair.access_jti.clone(), pair.refresh_jti.clone()],
        )
        .await;
        let _ = auth::record_session_meta(
            &self.state,
            &auth::SessionMeta {
                jti: pair.access_jti.clone(),
                client_type: client_type.clone(),
                uid: user.id,
                username: user.username.clone(),
                tenant_id: user.tenant_id,
                ip: login_ip.to_string(),
                ua: login_ua.to_string(),
                dev: device_id.unwrap_or("").to_string(),
                login_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            },
        )
        .await;
        let _ = self.repo.update_last_login(user.id, login_ip).await;

        Ok(TokenIssue {
            pair,
            client_type,
        })
    }

    /// 登出：撤销该用户该客户端的全部会话（黑名单全部 jti 含当前令牌）。
    pub async fn logout(&self, client_type: &str, user_id: i64) -> Result<(), AppError> {
        auth::revoke_all_sessions(&self.state, client_type, user_id, "").await
    }

    /// 刷新令牌（自描述 refresh token 独立鉴权）。
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
        req_client_type: Option<&str>,
    ) -> Result<TokenIssue, AppError> {
        let claims = auth::verify_refresh_token(&self.state.jwt_secret, refresh_token)?;
        let user = self
            .repo
            .get_user_by_id(claims.uid)
            .await?
            .ok_or_else(|| AppError::Validation("invalid refresh token".into()))?;
        if user.status != "NORMAL" {
            return Err(AppError::Forbidden("user is disabled".into()));
        }

        let role_codes = self.repo.list_role_codes(user.id).await?;
        let ipa = role_codes.iter().any(|c| c == PLATFORM_ADMIN_ROLE_CODE);
        let ita = role_codes.iter().any(|c| c == TENANT_ADMIN_ROLE_CODE);

        let client_type = self.client_type(req_client_type);
        let access_jti = auth::new_jti();
        let refresh_jti = auth::new_jti();
        let pair = auth::issue_token_pair(
            &self.state.jwt_secret,
            user.id,
            user.tenant_id,
            &user.username,
            None,
            None,
            role_codes,
            ipa,
            ita,
            &access_jti,
            &refresh_jti,
        )?;

        // 撤销旧会话（黑名单旧 jti，旧 refresh token 立即失效）并记录新会话
        let _ = auth::revoke_all_sessions(&self.state, &client_type, user.id, "").await;
        let _ = auth::record_session(
            &self.state,
            &client_type,
            user.id,
            &[pair.access_jti.clone(), pair.refresh_jti.clone()],
        )
        .await;
        let _ = auth::record_session_meta(
            &self.state,
            &auth::SessionMeta {
                jti: pair.access_jti.clone(),
                client_type: client_type.clone(),
                uid: user.id,
                username: user.username.clone(),
                tenant_id: user.tenant_id,
                ip: "-".to_string(),
                ua: "-".to_string(),
                dev: String::new(),
                login_at: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            },
        )
        .await;

        Ok(TokenIssue {
            pair,
            client_type,
        })
    }

    /// 注册前台用户（需归属租户）。
    pub async fn register_user(
        &self,
        username: &str,
        password: &str,
        tenant_code: &str,
        email: Option<&str>,
    ) -> Result<i64, AppError> {
        let plain = crate::crypto::decrypt_transport_secret(password)?;

        let code = tenant_code.trim();
        if code.is_empty() {
            return Err(AppError::Validation("tenant code required".into()));
        }
        let tenant_id = self
            .repo
            .get_tenant_id_by_code(code)
            .await?
            .ok_or_else(|| AppError::Validation("invalid tenant".into()))?;

        if self.repo.username_exists(tenant_id, username).await? {
            return Err(AppError::Validation("username already exists".into()));
        }
        if plain.len() < 6 {
            return Err(AppError::Validation("password too weak".into()));
        }

        let user_id = self
            .repo
            .create_user(tenant_id, username, email)
            .await?;
        let hash = bcrypt::hash(&plain, bcrypt::DEFAULT_COST).map_err(|e| {
            AppError::Internal {
                context: "hash password failed".into(),
                source: Some(Box::new(e)),
            }
        })?;
        self.repo
            .create_credential(tenant_id, user_id, username, &hash)
            .await?;
        Ok(user_id)
    }

// ===================== 找回/重置密码 =====================

/// 忘记密码：签发 6 位重置验证码并发送邮件。
/// 防枚举：identifier 无 EMAIL 凭证或无可用邮件渠道时静默成功（仅记日志）。
pub async fn forgot_password(&self, identifier: &str) -> Result<(), AppError> {
    let identifier = identifier.trim();
    if identifier.is_empty() {
        return Err(AppError::Validation("identifier is required".into()));
    }

    let cred = self.repo.find_user_id_by_email_credential(identifier).await?;
    let Some((_tenant_id, user_id)) = cred else {
        tracing::warn!(identifier, "forgot-password: no EMAIL credential, silently ok");
        return Ok(());
    };

    // 生成 6 位数字验证码（密码学随机，单次有效 + 10 分钟 TTL）
    use rand::Rng;
    let code = format!("{:06}", rand::thread_rng().gen_range(0..1_000_000u32));

    if let Some(client) = &self.state.redis {
        use redis::AsyncCommands;
        let mut con = client.get_multiplexed_tokio_connection().await.map_err(|e| {
            AppError::Internal {
                context: "redis connect failed".into(),
                source: Some(Box::new(e)),
            }
        })?;
        let key = format!("{RESET_VCODE_PREFIX}{identifier}");
        let _: () = con.set_ex(&key, code.as_str(), RESET_VCODE_TTL).await.map_err(|e| {
            AppError::Internal {
                context: "save reset vcode failed".into(),
                source: Some(Box::new(e)),
            }
        })?;
    } else {
        return Err(AppError::Internal {
            context: "redis not configured; cannot issue reset code".into(),
            source: None,
        });
    }

    // 读取首个启用 EMAIL 渠道发送（无渠道仅记日志，静默成功防枚举）
    let channel = self.repo.first_enabled_email_channel().await?;
    let Some((_id, host, port, username, password, from, tls)) = channel else {
        tracing::warn!(identifier, "forgot-password: no enabled EMAIL channel configured");
        return Ok(());
    };

    let smtp_password = match password {
        Some(p) if !p.is_empty() => crate::crypto::decrypt_channel_secret(&p)?,
        _ => String::new(),
    };
    let cfg = crate::mailer::SmtpConfig {
        host,
        port: port as u16,
        username,
        password: smtp_password,
        from,
        tls,
    };
    let subject = "GoWind Admin 密码重置验证码";
    let body = format!("您的密码重置验证码是：{code}，10 分钟内有效。");
    if let Err(e) = crate::mailer::send_mail(&cfg, identifier, subject, &body).await {
        // 发送失败：验证码已签发但邮件不可达。对外静默成功（防枚举），日志带出原始错误。
        tracing::error!(error = %e, identifier, "forgot-password: send mail failed");
        return Ok(());
    }
    let _ = user_id;
    Ok(())
}

/// 按验证码重置密码：验码（消费型）→ 解密新密码 → 更新 USERNAME 凭证 → 撤销全部会话。
pub async fn reset_password_by_code(
    &self,
    identifier: &str,
    code: &str,
    new_password: &str,
) -> Result<(), AppError> {
    let identifier = identifier.trim();
    if identifier.is_empty() || code.trim().is_empty() {
        return Err(AppError::Validation("invalid or expired verification code".into()));
    }

    // 消费型验码：GET 比对成功即 DEL（单次有效）
    let client = self.state.redis.as_ref().ok_or_else(|| AppError::Internal {
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
    let key = format!("{RESET_VCODE_PREFIX}{identifier}");
    let stored: Option<String> = con.get(&key).await.map_err(|e| AppError::Internal {
        context: "get reset vcode failed".into(),
        source: Some(Box::new(e)),
    })?;
    match stored {
        Some(v) if v == code.trim() => {
            let _: () = con.del(&key).await.map_err(|e| AppError::Internal {
                context: "consume reset vcode failed".into(),
                source: Some(Box::new(e)),
            })?;
        }
        _ => {
            return Err(AppError::Validation("invalid or expired verification code".into()));
        }
    }

    // 定位用户与 USERNAME 凭证
    let Some((tenant_id, user_id)) = self.repo.find_user_id_by_email_credential(identifier).await? else {
        return Err(AppError::Validation("invalid or expired verification code".into()));
    };
    let Some(username_ident) = self.repo.get_username_identifier(user_id).await? else {
        return Err(AppError::NotFound("user credential not found".into()));
    };

    // 解密传输密码（AES，与登录同规）并落新哈希
    let plain = crate::crypto::decrypt_transport_secret(new_password)?;
    let hash = bcrypt::hash(&plain, bcrypt::DEFAULT_COST).map_err(|e| AppError::Internal {
        context: "hash password failed".into(),
        source: Some(Box::new(e)),
    })?;
    let updated = self
        .repo
        .update_password_hash(tenant_id, &username_ident, &hash)
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("user credential not found".into()));
    }

    // 撤销该用户全部客户端类型的会话（admin/app）
    for ct in ["admin", "app"] {
        let _ = auth::revoke_all_sessions(&self.state, ct, user_id, "").await;
    }
    Ok(())
}
}


/// 登录成功的结果（handler 负责把 refresh token 以 HttpOnly cookie 下发）
pub struct TokenIssue {
    pub pair: TokenPair,
    pub client_type: String,
}

/// 恒定时间防护用假哈希（用户不存在时报错前也跑一次 bcrypt 校验）
const DUMMY_BCRYPT_HASH: &str =
    "$2a$10$1sbpKmhQDpXLHnDnEQ1nLe3oOnYyP2bUJyqHcX2T0Fq1qfyoXOrPm";

fn gen_captcha_code() -> String {
    const CHARS: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";
    let mut rng = rand::thread_rng();
    let mut code = String::with_capacity(4);
    for _ in 0..4 {
        code.push(CHARS[rng.gen_range(0..CHARS.len())] as char);
    }
    code
}

fn render_captcha_svg(code: &str) -> String {
    let mut rng = rand::thread_rng();
    let mut noise = String::new();
    for _ in 0..4 {
        let (x1, y1, x2, y2) = (
            rng.gen_range(0..110),
            rng.gen_range(0..44),
            rng.gen_range(0..110),
            rng.gen_range(0..44),
        );
        noise.push_str(&format!(
            "<line x1='{x1}' y1='{y1}' x2='{x2}' y2='{y2}' stroke='#b0b0b0' stroke-width='1'/>"
        ));
    }
    format!(
        "<svg xmlns='http://www.w3.org/2000/svg' width='110' height='44' viewBox='0 0 110 44'>\
         <rect width='110' height='44' fill='#f3f4f6'/>\
         {noise}\
         <text x='55' y='31' font-size='26' fill='#374151' text-anchor='middle' \
           font-family='monospace' font-weight='bold'>{code}</text>\
         </svg>"
    )
}