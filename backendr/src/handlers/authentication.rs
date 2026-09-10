// authentication 模块 handlers。
// 使用精确的 wire DTO（camelCase），不依赖被误生成的 dto（丢 camelCase 映射）。
// refresh token 以 HttpOnly Cookie 下发（对齐 Kratos），不放入响应体。

use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::HeaderValue;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};

use crate::auth;
use crate::error::AppError;
use crate::middleware::Operator;
use crate::response::ApiResponse;
use crate::services::authentication::AuthenticationService;
use crate::state::AppState;

const REFRESH_TOKEN_COOKIE: &str = "refresh_token";
const REFRESH_EXP_COOKIE: &str = "refresh_exp";
const REFRESH_COOKIE_PATH: &str = "/admin/v1/refresh-token";

// ---------- 请求体 ----------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginBody {
    pub grant_type: Option<String>,
    pub client_id: Option<String>,
    #[serde(default)]
    pub client_secret: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
    #[serde(default)]
    pub redirect_uri: Option<String>,
    #[serde(default)]
    pub user_id: Option<i64>,
    pub username: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub mobile: Option<String>,
    pub password: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    pub client_type: Option<String>,
    #[serde(default)]
    pub device_id: Option<String>,
    #[serde(default)]
    pub jti: Option<String>,
    #[serde(default)]
    pub tenant_code: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyCaptchaBody {
    pub captcha_id: String,
    pub user_input: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterBody {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub tenant_code: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub client_type: Option<String>,
}

// ---------- 响应体 ----------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResp {
    pub token_type: String,
    pub access_token: String,
    pub expires_in: String,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
    pub refresh_expires_in: Option<String>,
    pub id_token: Option<String>,
    pub mfa_operation_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptchaResp {
    pub captcha_id: String,
    pub image_base64: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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
    let expires_in = auth::ACCESS_TOKEN_TTL.to_string();
    (
        LoginResp {
            token_type: "Bearer".to_string(),
            access_token: issue.pair.access_token.clone(),
            expires_in: expires_in.clone(),
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
    Ok(Json(ApiResponse::ok(CaptchaResp {
        captcha_id,
        image_base64,
    })))
}

pub async fn authentication_verify_captcha(
    State(state): State<AppState>,
    Json(body): Json<VerifyCaptchaBody>,
) -> Result<impl IntoResponse, AppError> {
    let service = AuthenticationService::from_state(&state)?;
    let valid = service.verify_captcha(&body.captcha_id, &body.user_input).await?;
    Ok(Json(ApiResponse::ok(VerifyCaptchaResp { valid })))
}

pub async fn authentication_login(
    State(state): State<AppState>,
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

    let service = AuthenticationService::from_state(&state)?;
    let issue = service
        .login(
            username.trim(),
            &password,
            body.tenant_code.as_deref(),
            body.client_id.as_deref(),
            body.device_id.as_deref(),
            body.client_type.as_deref(),
            captcha_id.as_deref(),
            captcha_value.as_deref(),
        )
        .await?;

    let (resp, refresh_token) = build_login_response(issue);
    let mut response = Json(ApiResponse::ok(resp)).into_response();
    set_refresh_cookies(&mut response, &headers, &refresh_token, auth::REFRESH_TOKEN_TTL);
    Ok(response)
}

pub async fn authentication_logout(
    State(state): State<AppState>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let service = AuthenticationService::from_state(&state)?;
    service.logout(&operator.client_type, operator.user_id).await?;
    Ok(Json(ApiResponse::ok(serde_json::Value::Null)))
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

    let service = AuthenticationService::from_state(&state)?;
    let issue = service
        .refresh_token(&refresh_token, body.client_type.as_deref())
        .await?;

    let (resp, new_refresh) = build_login_response(issue);
    let mut response = Json(ApiResponse::ok(resp)).into_response();
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
    Ok(Json(ApiResponse::ok(RegisterResp { user_id })))
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