// authentication 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn authentication_generate_captcha(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /captcha  生成验证码
    Err(AppError::NotImplemented)
}

pub async fn authentication_verify_captcha(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /captcha/verify  验证验证码
    Err(AppError::NotImplemented)
}

pub async fn authentication_login(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /login  登录
    Err(AppError::NotImplemented)
}

pub async fn authentication_logout(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /logout  登出
    Err(AppError::NotImplemented)
}

pub async fn authentication_refresh_token(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /refresh-token  刷新认证令牌
    Err(AppError::NotImplemented)
}

pub async fn authentication_register_user(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /register  
    Err(AppError::NotImplemented)
}
