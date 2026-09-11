// authentication 模块 handlers。
// wire 格式对齐 Go/Kratos：**裸 DTO**（无 envelope）。
// - LoginRequest/LoginResponse：proto json_name 全部 snake_case（grant_type/access_token/expires_in…）
// - RegisterUserResponse：userId（camelCase json_name）
// - Captcha：captchaId/imageBase64/userInput（camelCase json_name）
// - ForgotPassword：identifier；ResetPasswordByCode：identifier/code/new_password（snake_case）
// refresh token 以 HttpOnly Cookie 下发（对齐 Kratos），不放入响应体。

use axum::extract::connect_info::ConnectInfo;
use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::auth;
use crate::error::AppError;
use crate::middleware::Operator;
use crate::response::{json_empty, json_ok};
use crate::services::authentication::{AuthenticationService, LoginOutcome};
use crate::state::AppState;

const REFRESH_TOKEN_COOKIE: &str = "refresh_token";
const REFRESH_EXP_COOKIE: &str = "refresh_exp";
const REFRESH_COOKIE_PATH: &str = "/admin/v1/refresh-token";

// ---------- 请求体 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LoginBody {
    pub grant_type: Option<String>,
    #[serde(default, alias = "clientId")]
    pub client_id: Option<String>,
    #[serde(default, alias = "clientSecret")]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default, alias = "redirectUri")]
    pub redirect_uri: Option<String>,
    #[serde(default, alias = "userId")]
    pub user_id: Option<i64>,
    #[serde(default)]
    pub username: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub mobile: Option<String>,
    #[serde(default)]
    pub password: Option<String>,
    #[serde(default, alias = "refreshToken")]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default, alias = "clientType")]
    pub client_type: Option<String>,
    #[serde(default, alias = "deviceId")]
    pub device_id: Option<String>,
    #[serde(default)]
    pub jti: Option<String>,
    #[serde(default, alias = "tenantCode")]
    pub tenant_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyCaptchaBody {
    pub captcha_id: String,
    pub user_input: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterBody {
    pub username: String,
    pub password: String,
    #[serde(default, alias = "tenant_code")]
    pub tenant_code: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default, alias = "clientType")]
    pub client_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordBody {
    pub identifier: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ResetPasswordByCodeBody {
    pub identifier: String,
    pub code: String,
    pub new_password: String,
}

// ---------- 响应体 ----------

/// 对齐 LoginResponse：非零值才输出（protojson 省略零值/未设置字段）。
#[derive(Debug, Serialize)]
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

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptchaResp {
    pub captcha_id: String,
    pub image_base64: String,
}

#[derive(Debug, Serialize)]
pub struct VerifyCaptchaResp {
    pub valid: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterResp {
    pub user_id: i64,
}

// ---------- cookie 辅助 ----------

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

fn build_login_response(
    issue: crate::services::authentication::TokenIssue,
) -> (LoginResp, String) {
    (
        LoginResp {
            token_type: "bearer".to_string(),
            access_token: issue.pair.access_token.clone(),
            expires_in: Some(auth::ACCESS_TOKEN_TTL.to_string()),
            refresh_token: None,
            scope: None,
            refresh_expires_in: None,
            id_token: None,
            mfa_operation_id: None,
        },
        issue.pair.refresh_token,
    )
}

// ---------- handlers ----------

pub async fn authentication_generate_captcha(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let service = AuthenticationService::from_state(&state)?;
    let (captcha_id, image_base64) = service.generate_captcha().await?;
    Ok(json_ok(CaptchaResp {
        captcha_id,
        image_base64,
    }))
}

pub async fn authentication_verify_captcha(
    State(state): State<AppState>,
    Json(body): Json<VerifyCaptchaBody>,
) -> Result<impl IntoResponse, AppError> {
    let service = AuthenticationService::from_state(&state)?;
    let valid = service.verify_captcha(&body.captcha_id, &body.user_input).await?;
    Ok(json_ok(VerifyCaptchaResp { valid }))
}

pub async fn authentication_login(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    Json(body): Json<LoginBody>,
) -> Result<Response, AppError> {
    let grant_type = body.grant_type.clone().unwrap_or_else(|| "password".into());
    if grant_type != "password" {
        return Err(AppError::Validation("invalid grant type".into()));
    }
    let username = body
        .username
        .clone()
        .ok_or_else(|| AppError::Validation("username is required".into()))?;
    let password = body
        .password
        .clone()
        .ok_or_else(|| AppError::Validation("password is required".into()))?;

    let captcha_id = headers
        .get("X-Captcha-Id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let captcha_value = headers
        .get("X-Captcha-Value")
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let ua = headers
        .get(axum::http::header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let service = AuthenticationService::from_state(&state)?;
    let outcome = service
        .login(
            username.trim(),
            &password,
            body.tenant_code.as_deref(),
            body.client_id.as_deref(),
            body.device_id.as_deref(),
            body.client_type.as_deref(),
            captcha_id.as_deref(),
            captcha_value.as_deref(),
            &peer.ip().to_string(),
            &ua,
        )
        .await?;

    // MFA 闸门：用户绑定 TOTP 时登录返回操作 ID（不发 token、不下发 cookie），
    // 前端跳转 MFA 挑战页（提交到 /mfa/verify）。
    let (resp, refresh_token) = match outcome {
        LoginOutcome::MfaChallenge(op_id) => {
            let resp = LoginResp {
                token_type: "bearer".to_string(),
                access_token: String::new(),
                expires_in: None,
                refresh_token: None,
                scope: None,
                refresh_expires_in: None,
                id_token: None,
                mfa_operation_id: Some(op_id),
            };
            return Ok(json_ok(resp).into_response());
        }
        LoginOutcome::Token(issue) => build_login_response(issue),
    };
    let mut response = json_ok(resp).into_response();
    set_refresh_cookies(&mut response, &headers, &refresh_token, auth::REFRESH_TOKEN_TTL);
    Ok(response)
}

pub async fn authentication_logout(
    State(state): State<AppState>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let service = AuthenticationService::from_state(&state)?;
    service
        .logout(&operator.client_type, operator.user_id)
        .await?;
    Ok(json_empty())
}

pub async fn authentication_refresh_token(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
    Json(body): Json<LoginBody>,
) -> Result<Response, AppError> {
    let grant_type = body.grant_type.clone().unwrap_or_default();
    if grant_type != "refresh_token" {
        return Err(AppError::Validation("invalid grant type".into()));
    }
    let cookie = headers
        .get(axum::http::header::COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let refresh_token = extract_refresh_cookie(cookie);

    // 旧 refresh token 若已被吊销（强制下线/改密），拒绝刷新
    let service = AuthenticationService::from_state(&state)?;
    let claims = auth::verify_refresh_token(&state.jwt_secret, &refresh_token)?;
    if auth::is_jti_blacklisted(&state, &claims.jti).await {
        return Err(AppError::Unauthorized);
    }
    let issue = service
        .refresh_token(&refresh_token, body.client_type.as_deref())
        .await?;

    let (resp, new_refresh) = build_login_response(issue);
    let mut response = json_ok(resp).into_response();
    set_refresh_cookies(&mut response, &headers, &new_refresh, auth::REFRESH_TOKEN_TTL);
    Ok(response)
}

pub async fn authentication_register_user(
    State(state): State<AppState>,
    Json(body): Json<RegisterBody>,
) -> Result<impl IntoResponse, AppError> {
    let service = AuthenticationService::from_state(&state)?;
    let user_id = service
        .register_user(
            body.username.trim(),
            &body.password,
            &body.tenant_code,
            body.email.as_deref(),
        )
        .await?;
    Ok(json_ok(RegisterResp { user_id }))
}

pub async fn authentication_forgot_password(
    State(state): State<AppState>,
    Json(body): Json<ForgotPasswordBody>,
) -> Result<impl IntoResponse, AppError> {
    let service = AuthenticationService::from_state(&state)?;
    service.forgot_password(&body.identifier).await?;
    Ok(json_empty())
}

pub async fn authentication_reset_password_by_code(
    State(state): State<AppState>,
    Json(body): Json<ResetPasswordByCodeBody>,
) -> Result<impl IntoResponse, AppError> {
    let service = AuthenticationService::from_state(&state)?;
    service
        .reset_password_by_code(&body.identifier, &body.code, &body.new_password)
        .await?;
    Ok(json_empty())
}

fn extract_refresh_cookie(cookie_header: &str) -> String {
    cookie_header
        .split(';')
        .filter_map(|kv| {
            let mut it = kv.trim().splitn(2, '=');
            match (it.next(), it.next()) {
                (Some(name), Some(value)) if name == REFRESH_TOKEN_COOKIE => Some(value.to_string()),
                _ => None,
            }
        })
        .next()
        .unwrap_or_default()
}
