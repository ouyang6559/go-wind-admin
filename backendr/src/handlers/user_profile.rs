// user_profile 模块 handlers（/me 自管理：资料读写、头像、绑定联系方式、改密）。
// wire 对齐 Go/Kratos：裸 DTO、camelCase（ChangePasswordRequest 为 oldPassword/newPassword）、
// BindContactRequest/VerifyContactRequest 为 oneof 的 `{"email": {...}}` 结构。
// 对齐 user_profile_service.go / user_profile_contact.go 约束：
//  - GET /me 返回当前用户 DTO（含 Roles 角色码，复用 user 模块聚合）；
//  - PUT /me 强制目标 id=操作人，仅更新资料字段（不改状态/角色/租户）；
//  - /me/contact 仅支持 EMAIL，向新邮箱发验证码；/me/contact/verify 验码后写 EMAIL 凭证；
//  - /me/password 校验旧密码（AES 解密→bcrypt 比对）→ 新密码（AES 解密→bcrypt 落库）→ 撤销全部会话；
//  - /me/avatar：imageUrl 直存；imageBase64 需文件存储（Rust 未接 MinIO/S3），暂不支持。

use axum::Json;
use axum::extract::State;
use axum::response::IntoResponse;
use serde::Deserialize;

use crate::error::AppError;
use crate::handlers::user::{UserData, enrich_and_to_dto};
use crate::middleware::Operator;
use crate::repos::authentication::AuthenticationRepo;
use crate::repos::user::UserRepo;
use crate::repos::user_profile::{UserProfileRepo, save_bind_vcode, verify_bind_vcode};
use crate::response::{json_empty, json_ok};
use crate::state::AppState;

/// GET /me 获取当前用户资料（裸 DTO，含 Roles 角色码）。
pub async fn user_profile_get_user(
    State(state): State<AppState>,
    operator: Operator,
) -> Result<axum::response::Response, AppError> {
    let repo = UserRepo::new(crate::handlers::script::db_of(&state)?);
    let row = repo
        .get(operator.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;
    let dto = enrich_and_to_dto(&repo, &row).await?;
    Ok(json_ok(dto).into_response())
}

/// PUT /me 更新当前用户资料（目标 id 强制为操作人；不更新状态/角色/租户正文）。
pub async fn user_profile_update_user(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<UpdateMeBody>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let repo = UserRepo::new(crate::handlers::script::db_of(&state)?);

    let existing = repo
        .get(operator.user_id)
        .await?
        .ok_or_else(|| AppError::NotFound("user not found".into()))?;

    // username 不可改 / 状态不可自改（对齐：平台管理员改他人状态走 /users/{id}）
    if let Some(u) = data.username.as_deref() {
        if !u.trim().is_empty() && existing.username.as_deref() != Some(u.trim()) {
            return Err(AppError::Validation("username is immutable".into()));
        }
    }

    // email/mobile 掩码跳过写入（对齐 Go 防掩码入库）
    let email = data.email.clone().filter(|v| !v.contains('*'));
    let mobile = data.mobile.clone().filter(|v| !v.contains('*'));

    let gender = data
        .gender
        .as_deref()
        .map(|v| crate::handlers::user::normalize_gender(v))
        .transpose()?;
    let status = data
        .status
        .as_deref()
        .map(|v| crate::handlers::user::normalize_status(v))
        .transpose()?;

    let updated = repo
        .update(
            operator.user_id,
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
            None,
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("user not found".into()));
    }
    Ok(json_empty())
}

// ===================== 头像 =====================

/// POST /me/avatar：imageUrl 直存；imageBase64 依赖 OSS 上传（Rust 未接），返回 501。
pub async fn user_profile_upload_avatar(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<UploadAvatarBody>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let repo = UserRepo::new(crate::handlers::script::db_of(&state)?);

    let avatar = match (body.image_base64.as_deref(), body.image_url.as_deref()) {
        (Some(b64), _) if !b64.trim().is_empty() => {
            // base64 源需要上传到 OSS（MinIO/S3），Rust 侧未接入文件存储，暂不支持
            return Err(AppError::NotImplemented);
        }
        (_, Some(url)) if !url.trim().is_empty() => url.trim().to_string(),
        _ => {
            return Err(AppError::Validation("invalid avatar source".into()));
        }
    };

    let updated = repo
        .update(
            operator.user_id,
            None,
            None,
            None,
            None,
            None,
            Some(&avatar),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("user not found".into()));
    }
    Ok(json_ok(serde_json::json!({ "url": avatar })))
}

/// DELETE /me/avatar 清空头像字段。
pub async fn user_profile_delete_avatar(
    State(state): State<AppState>,
    operator: Operator,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let repo = UserRepo::new(crate::handlers::script::db_of(&state)?);
    let updated = repo
        .update(
            operator.user_id,
            None,
            None,
            None,
            None,
            None,
            Some(""),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("user not found".into()));
    }
    Ok(json_empty())
}

// ===================== 联系方式绑定 =====================

/// POST /me/contact（第一步）：仅支持 EMAIL，向新邮箱发送验证码。
pub async fn user_profile_bind_contact(
    State(state): State<AppState>,
    Json(body): Json<BindContactBody>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let email = body.email.and_then(|e| e.email);
    let contact = email
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Validation("only email binding is supported".into()))?;
    if !contact.contains('@') {
        return Err(AppError::Validation("invalid email address".into()));
    }

    let repo = UserProfileRepo::new(crate::handlers::script::db_of(&state)?);
    // 防重复绑定：该邮箱已被他人绑定 → 400
    if repo.email_credential_exists(&contact, 0).await? {
        return Err(AppError::Conflict(format!("email already bound: {contact}")));
    }

    let code = save_bind_vcode(&state, &contact).await?;

    // 读取首个启用 EMAIL 渠道发送（无渠道仅记日志——sendContactVCode 在无渠道时报错，这里对齐为 500）
    let channel = AuthenticationRepo::new(crate::handlers::script::db_of(&state)?)
        .first_enabled_email_channel()
        .await?;
    let Some((_id, host, port, username, smtp_password, from, tls)) = channel else {
        tracing::error!(contact, "bind-contact: no email channel configured");
        return Err(AppError::Internal {
            context: "email channel is not configured".into(),
            source: None,
        });
    };

    let smtp_password = match smtp_password {
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
    let subject = "GoWind Admin 邮箱绑定验证码";
    let body_txt = format!("您的邮箱绑定验证码是：{code}\n\n10 分钟内有效。若非本人操作请忽略本邮件。\n");
    crate::mailer::send_mail(&cfg, &contact, subject, &body_txt)
        .await
        .map_err(|e| AppError::Internal {
            context: format!("send verification email failed: {e}"),
            source: None,
        })?;
    Ok(json_empty())
}

/// POST /me/contact/verify（第二步）：验码后写入 EMAIL 登录凭证。
pub async fn user_profile_verify_contact(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<VerifyContactBody>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let email_field = body.email.ok_or_else(|| {
        AppError::Validation("only email verification is supported".into())
    })?;
    let contact = email_field
        .email
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Validation("contact and code are required".into()))?;
    let code = email_field
        .code
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .ok_or_else(|| AppError::Validation("contact and code are required".into()))?;

    if !verify_bind_vcode(&state, &contact, &code).await? {
        return Err(AppError::Validation("invalid or expired verification code".into()));
    }

    let repo = UserProfileRepo::new(crate::handlers::script::db_of(&state)?);
    if repo.email_credential_exists(&contact, operator.user_id).await? {
        return Err(AppError::Conflict(format!("email already bound: {contact}")));
    }
    // 写入 EMAIL 登录凭证（占位哈希，不用于密码校验）
    repo.bind_email_credential(operator.tenant_id, operator.user_id, &contact)
        .await?;
    Ok(json_empty())
}

// ===================== 修改密码 =====================

/// POST /me/password：校验旧密码（AES 解密→bcrypt 比对）→ 新密码落库 → 撤销本人全部会话。
pub async fn user_profile_change_password(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<ChangePasswordBody>,
) -> Result<impl axum::response::IntoResponse, AppError> {
    let old_password = body.old_password.ok_or_else(|| AppError::Validation("old_password is required".into()))?;
    let new_password = body.new_password.ok_or_else(|| AppError::Validation("new_password is required".into()))?;

    let username = operator.username.clone();
    if username.is_empty() {
        return Err(AppError::NotFound("user credential not found".into()));
    }

    let auth_repo = AuthenticationRepo::new(crate::handlers::script::db_of(&state)?);

    // 1. 取 USERNAME 凭证，解密旧密码并 bcrypt 比对（解密/比对失败统一 400，防枚举）
    let cred = auth_repo
        .get_credential(operator.tenant_id, &username)
        .await?
        .ok_or_else(|| AppError::NotFound("user credential not found".into()))?;
    let old_plain = crate::crypto::decrypt_transport_secret(&old_password)?;
    let old_ok = if cred.credential_type == "PASSWORD_HASH" {
        bcrypt::verify(&old_plain, &cred.credential).unwrap_or(false)
    } else {
        old_plain == cred.credential
    };
    if !old_ok {
        return Err(AppError::Validation("invalid old password".into()));
    }

    // 2. 解密新密码并哈希落库
    let new_plain = crate::crypto::decrypt_transport_secret(&new_password)?;
    if new_plain.len() < 8 {
        return Err(AppError::Validation("password must be at least 8 characters".into()));
    }
    let hash = bcrypt::hash(&new_plain, bcrypt::DEFAULT_COST).map_err(|e| AppError::Internal {
        context: "hash password failed".into(),
        source: Some(Box::new(e)),
    })?;
    let updated = auth_repo
        .update_password_hash(operator.tenant_id, &username, &hash)
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("user credential not found".into()));
    }

    // 3. 改密成功后吊销本人全部客户端类型的令牌（含当前会话），前端引导重新登录
    for ct in ["admin", "app"] {
        let _ = crate::auth::revoke_all_sessions(&state, ct, operator.user_id, "").await;
    }
    Ok(json_empty())
}

// ===================== 请求体（wire 对齐 proto json_name） =====================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMeBody {
    #[serde(default)]
    pub data: Option<UserData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadAvatarBody {
    #[serde(default)]
    pub image_base64: Option<String>,
    #[serde(default)]
    pub image_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BindContactBody {
    #[serde(default)]
    pub email: Option<BindEmailBody>,
}

#[derive(Debug, Deserialize)]
pub struct BindEmailBody {
    #[serde(default)]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct VerifyContactBody {
    #[serde(default)]
    pub email: Option<EmailVerificationBody>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailVerificationBody {
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordBody {
    #[serde(default)]
    pub old_password: Option<String>,
    #[serde(default)]
    pub new_password: Option<String>,
}