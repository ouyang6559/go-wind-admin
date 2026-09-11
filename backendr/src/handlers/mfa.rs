// mfa 模块 handlers（TOTP 多因素认证。
// wire 对齐 Go/Kratos + 前端 react MFA 挑战页期望字段名：
// - StartEnroll 返回 {totp:{secret, otpAuthUrl, qrCodeDataUri}, operationId, expiresAt}
// - GetMFAStatus 返回 {enabled, enrolled, enforcement}（enforcement 为枚举字符串）
// - ListEnrolledMethods 返回 {items:[{id, method, display, enabled, createdAt, lastUsedAt}]}
// - Verify 通过返回 LoginResponse（snake_case：access_token/expires_in/refresh_token/refresh_expires_in）
// 除 /mfa/verify（免鉴权，走登录挑战链路）外均需登录态 Operator。

use axum::extract::{ConnectInfo, Path, State};
use axum::http::header::SET_COOKIE;
use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;
use std::net::SocketAddr;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::response::ListResponse;
use crate::services::authentication::TokenIssue;
use crate::services::mfa::{MfaEnrollChallengeContext, MfaService};
use crate::state::AppState;

const REFRESH_TOKEN_COOKIE: &str = "refresh_token";
const REFRESH_EXP_COOKIE: &str = "refresh_exp";
const REFRESH_COOKIE_PATH: &str = "/admin/v1/refresh-token";

// ---------- 请求体（camelCase，对齐 proto json_name） ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartEnrollBody {
    pub method: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmEnrollBody {
    pub method: Option<String>,
    pub operation_id: String,
    #[serde(default)]
    pub totp_code: Option<String>,
    #[serde(default)]
    pub display: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyChallengeBody {
    pub operation_id: String,
    #[serde(default)]
    pub totp_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisableMfaBody {
    #[serde(default)]
    pub credential_id: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub user_id: Option<i64>,
    #[serde(default)]
    pub reason: Option<String>,
}

// ---------- 响应体 ----------

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartEnrollResp {
    pub totp: TotpResult,
    pub operation_id: String,
    pub expires_at: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TotpResult {
    pub secret: String,
    pub otp_auth_url: String,
    pub qr_code_data_uri: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmEnrollResp {
    pub success: bool,
    pub credential_id: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MfaStatusResp {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub enrolled: Vec<EnrolledMethodResp>,
    pub enforcement: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnrolledMethodResp {
    pub id: String,
    pub method: String,
    pub display: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<String>,
}

/// 对齐 LoginResponse（proto json_name snake_case，且 int64 序列化为字符串）
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct LoginResp {
    pub token_type: String,
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_expires_in: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfa_operation_id: Option<String>,
}

fn method_name(m: &str) -> String {
    match m.trim().to_uppercase().as_str() {
        "SMS" => "SMS".to_string(),
        "EMAIL" => "EMAIL".to_string(),
        "WEBAUTHN" | "U2F" => "WEBAUTHN".to_string(),
        _ => "TOTP".to_string(),
    }
}

fn to_enrolled_vec(
    infos: Vec<crate::repos::mfa::EnrolledFactorInfo>,
) -> Vec<EnrolledMethodResp> {
    infos
        .into_iter()
        .map(|i| EnrolledMethodResp {
            id: i.id.to_string(),
            method: i.method.clone(),
            display: i.display_name,
            enabled: i.enabled,
            created_at: i.created_at,
            last_used_at: i.last_used_at,
        })
        .collect()
}

fn cookie_secure(headers: &axum::http::HeaderMap) -> bool {
    headers
        .get("X-Forwarded-Proto")
        .and_then(|v| v.to_str().ok())
        .map(|p| p.eq_ignore_ascii_case("https"))
        .unwrap_or(false)
}

fn set_refresh_cookies(
    resp: &mut Response,
    headers: &axum::http::HeaderMap,
    refresh_token: &str,
    refresh_expires_in_seconds: i64,
) {
    let secure = cookie_secure(headers);
    let exp_unix = chrono::Utc::now().timestamp() + refresh_expires_in_seconds;
    let rt = format!(
        "{REFRESH_TOKEN_COOKIE}={refresh_token}; Path={REFRESH_COOKIE_PATH}; Max-Age={refresh_expires_in_seconds}; HttpOnly; SameSite=Lax{}",
        if secure { "; Secure" } else { "" }
    );
    let exp = format!(
        "{REFRESH_EXP_COOKIE}={exp_unix}; Path=/; Max-Age={refresh_expires_in_seconds}; SameSite=Lax{}",
        if secure { "; Secure" } else { "" }
    );
    if let (Ok(a), Ok(b)) = (HeaderValue::from_str(&rt), HeaderValue::from_str(&exp)) {
        resp.headers_mut().append(SET_COOKIE, a);
        resp.headers_mut().append(SET_COOKIE, b);
    }
}

fn build_login_response(issue: &TokenIssue) -> LoginResp {
    LoginResp {
        token_type: "bearer".to_string(),
        access_token: issue.pair.access_token.clone(),
        expires_in: Some(crate::auth::ACCESS_TOKEN_TTL.to_string()),
        refresh_token: None,
        scope: None,
        refresh_expires_in: None,
        id_token: None,
        mfa_operation_id: None,
    }
}

// ---------- handlers ----------

/// GET /mfa/status 查询当前登录用户 MFA 总览（对齐 Go：仅绑定 TOTP 且 ENABLED 才算启用）。
pub async fn mfa_get_m_f_a_status(
    State(state): State<AppState>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let service = MfaService::from_state(&state)?;
    let infos = service.repo.list_by_user(operator.tenant_id, operator.user_id).await?;
    let has_totp = infos.iter().any(|i| i.method == "TOTP" && i.enabled);
    let (enrolled, enforcement) = if has_totp {
        (to_enrolled_vec(infos), "MFA_REQUIRED")
    } else {
        (Vec::new(), "MFA_NOT_REQUIRED")
    };
    Ok(axum::Json(MfaStatusResp {
        enabled: has_totp,
        enrolled,
        enforcement: enforcement.to_string(),
    }))
}

/// GET /mfa/methods 列出当前登录用户的 MFA 因子（不含 secret）。
pub async fn mfa_list_enrolled_methods(
    State(state): State<AppState>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let service = MfaService::from_state(&state)?;
    let infos = service.repo.list_by_user(operator.tenant_id, operator.user_id).await?;
    let items = to_enrolled_vec(infos);
    let total = items.len().to_string();
    Ok(axum::Json(ListResponse {
        items,
        total,
    }))
}

/// POST /mfa/enroll/start 开始注册 TOTP（返回 secret/otpauth URL/QR data URI + operation_id）。
pub async fn mfa_start_enroll_method(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<StartEnrollBody>,
) -> Result<impl IntoResponse, AppError> {
    let method = method_name(body.method.as_deref().unwrap_or("TOTP"));
    if method != "TOTP" {
        return Err(AppError::Validation("only TOTP is supported".into()));
    }
    let service = MfaService::from_state(&state)?;

    // 频控：同一用户 30s 冷却期内只允许发起一次注册
    if !service
        .try_acquire_enroll_cooldown(operator.tenant_id, operator.user_id)
        .await
    {
        return Err(AppError::Validation(
            "enroll request too frequent, retry later".into(),
        ));
    }

    // 预检：已绑定 TOTP 时拒绝重复注册
    if service
        .repo
        .has_enabled_totp(operator.tenant_id, operator.user_id)
        .await?
    {
        return Err(AppError::Validation(
            "totp already enrolled, disable it first".into(),
        ));
    }

    let bundle = service.generate_totp(operator.user_id)?;
    let op_id = service
        .set_enroll_challenge(&MfaEnrollChallengeContext {
            secret: bundle.secret.clone(),
            tenant_id: operator.tenant_id,
            user_id: operator.user_id,
        })
        .await?;

    Ok(axum::Json(StartEnrollResp {
        totp: TotpResult {
            secret: bundle.secret,
            otp_auth_url: bundle.otp_auth_url,
            qr_code_data_uri: bundle.qr_code_data_uri,
        },
        operation_id: op_id,
        expires_at: chrono::Utc::now()
            .checked_add_signed(chrono::Duration::seconds(300))
            .map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
            .unwrap_or_default(),
    }))
}

/// POST /mfa/enroll/confirm 确认注册：首码校验通过则落库因子。
pub async fn mfa_confirm_enroll_method(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<ConfirmEnrollBody>,
) -> Result<impl IntoResponse, AppError> {
    if method_name(body.method.as_deref().unwrap_or("TOTP")) != "TOTP" {
        return Err(AppError::Validation("only TOTP is supported".into()));
    }
    let service = MfaService::from_state(&state)?;

    let Some(ctx) = service.peek_enroll_challenge(&body.operation_id).await? else {
        return Err(AppError::Validation("invalid or expired enroll operation".into()));
    };
    // 防 operation_id 跨用户劫持：注册上下文绑定的人必须等于当前 operator
    if ctx.tenant_id != operator.tenant_id || ctx.user_id != operator.user_id {
        return Err(AppError::Forbidden("enroll operation user mismatch".into()));
    }

    let code = body.totp_code.as_deref().unwrap_or("");
    if !service.verify_totp(code, &ctx.secret) {
        return Ok(axum::Json(ConfirmEnrollResp {
            success: false,
            credential_id: String::new(),
        }));
    }

    let factor_id = service
        .repo
        .create_totp_factor(
            operator.tenant_id,
            operator.user_id,
            &ctx.secret,
            body.display.as_deref().unwrap_or(""),
        )
        .await?;
    service.delete_enroll_challenge(&body.operation_id).await;
    Ok(axum::Json(ConfirmEnrollResp {
        success: true,
        credential_id: factor_id.to_string(),
    }))
}

/// POST /mfa/verify 验证登录 MFA 挑战。通过则签发真 token（含刷新 cookie）。
/// 免鉴权：operation_id 由登录流程在密码校验通过、待二次验证阶段签发。
pub async fn mfa_verify_m_f_a_challenge(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    Json(body): Json<VerifyChallengeBody>,
) -> Result<Response, AppError> {
    let service = MfaService::from_state(&state)?;

    // Peek 不消耗：允许失败重试（上限见 record_login_failure），通过或超限才原子消耗
    let Some(ctx) = service.peek_login_challenge(&body.operation_id).await? else {
        return Err(AppError::Validation("invalid or expired mfa operation".into()));
    };

    // 取该用户 ENABLED 的 TOTP 因子并解密 secret
    let Some((factor_id, plain_secret)) = service
        .repo
        .find_enabled_totp(ctx.tenant_id, ctx.user_id)
        .await?
    else {
        // 因子缺失（验证期间被解绑/管理员重置）：作废挑战走重新登录
        service.consume_login_challenge(&body.operation_id).await;
        return Err(AppError::Forbidden("mfa verification failed".into()));
    };

    let code = body.totp_code.as_deref().unwrap_or("");
    if !service.verify_totp(code, &plain_secret) {
        // 错码：记失败计数，达上限作废挑战
        if service.record_login_failure(&body.operation_id).await {
            return Err(AppError::Forbidden(
                "too many invalid mfa attempts, please login again".into(),
            ));
        }
        return Err(AppError::Forbidden("invalid mfa code".into()));
    }

    // 通过：原子消耗挑战做最终裁决（并发同 opId 仅先抢到者发 token）
    if !service
        .take_login_challenge_atomic(&body.operation_id)
        .await
    {
        return Err(AppError::Forbidden("mfa challenge already consumed".into()));
    }
    let _ = service
        .repo
        .update_last_used(ctx.tenant_id, ctx.user_id, factor_id)
        .await;

    // 签发真 token（与登录链路同套逻辑：角色码/管理员标志重新解析）
    let issue = crate::services::authentication::AuthenticationService::from_state(&state)?
        .issue_tokens_for_payload(&ctx, &peer.ip().to_string())
        .await?;

    let resp = build_login_response(&issue);
    let mut response = axum::Json(resp).into_response();
    set_refresh_cookies(
        &mut response,
        &headers,
        &issue.pair.refresh_token,
        crate::auth::REFRESH_TOKEN_TTL,
    );
    Ok(response)
}

/// DELETE /mfa/{credentialId} 撤销指定 MFA 凭证（强制归属校验）。
pub async fn mfa_revoke_m_f_a_device(
    State(state): State<AppState>,
    Path(credential_id): Path<String>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let service = MfaService::from_state(&state)?;
    let factor_id: i64 = credential_id.trim().parse().map_err(|_| {
        AppError::Validation("invalid credential id".into())
    })?;
    let ok = service
        .repo
        .delete_for_user(operator.tenant_id, operator.user_id, factor_id)
        .await?;
    if !ok {
        return Err(AppError::NotFound("mfa credential not found".into()));
    }
    Ok(axum::Json(serde_json::json!({})))
}

/// POST /mfa/disable 禁用/移除 MFA 凭证。
/// - 不传 user_id：操作当前登录用户本人（按 credential_id 精确解绑或按 method 清空）。
/// - 传 user_id 指定他人：仅平台管理员允许（救援重置）。
pub async fn mfa_disable_m_f_a(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<DisableMfaBody>,
) -> Result<impl IntoResponse, AppError> {
    let service = MfaService::from_state(&state)?;

    // 管理端救援路径：目标用户由因子行定位，或按 user_id+method 清空
    if let Some(target) = body.user_id {
        if target != operator.user_id {
            let is_platform_admin = operator.roles.iter().any(|c| c == "platform_admin");
            if !is_platform_admin {
                return Err(AppError::Forbidden(
                    "only platform admin can reset mfa for others".into(),
                ));
            }
            if let Some(cid) = body.credential_id.as_deref().filter(|s| !s.trim().is_empty()) {
                let factor_id: i64 = cid.trim().parse().map_err(|_| {
                    AppError::Validation("invalid credential id".into())
                })?;
                let Some((tid, uid)) = service.repo.get_factor_by_id(factor_id).await? else {
                    return Err(AppError::NotFound("mfa credential not found".into()));
                };
                if uid != target {
                    return Err(AppError::NotFound("mfa credential not found".into()));
                }
                if !service.repo.delete_for_user(tid, uid, factor_id).await? {
                    return Err(AppError::Internal {
                        context: "disable mfa failed".into(),
                        source: None,
                    });
                }
            } else {
                let method = method_name(body.method.as_deref().unwrap_or(""));
                if method != "TOTP" {
                    return Err(AppError::Validation(
                        "method required when resetting by user".into(),
                    ));
                }
                let Some((tid, uid)) = service.repo.find_first_by_user(target, &method).await?
                else {
                    return Err(AppError::NotFound("mfa credential not found".into()));
                };
                service
                    .repo
                    .delete_all_by_user_method(tid, uid, &method)
                    .await?;
            }
            tracing::warn!(
                operator = operator.user_id,
                target,
                reason = body.reason.as_deref().unwrap_or(""),
                "admin reset mfa for user"
            );
            return Ok(axum::Json(serde_json::json!({})));
        }
    }

    // 本人路径：未传 credential_id 时按 method 清空本人该方法全部因子
    if body.credential_id.as_deref().map(str::trim).unwrap_or("").is_empty() {
        let method = method_name(body.method.as_deref().unwrap_or(""));
        if method != "TOTP" {
            return Err(AppError::Validation(
                "credential_id or method required".into(),
            ));
        }
        let n = service
            .repo
            .delete_all_by_user_method(operator.tenant_id, operator.user_id, &method)
            .await?;
        if n == 0 {
            return Err(AppError::NotFound("mfa credential not found".into()));
        }
        return Ok(axum::Json(serde_json::json!({})));
    }

    let cid = body.credential_id.as_deref().unwrap_or("");
    let factor_id: i64 = cid.trim().parse().map_err(|_| {
        AppError::Validation("invalid credential id".into())
    })?;
    let ok = service
        .repo
        .delete_for_user(operator.tenant_id, operator.user_id, factor_id)
        .await?;
    if !ok {
        return Err(AppError::NotFound("mfa credential not found".into()));
    }
    Ok(axum::Json(serde_json::json!({})))
}