// login_policy 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn login_policy_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /login-policies  查询登录策略列表
    Err(AppError::NotImplemented)
}

pub async fn login_policy_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /login-policies  创建登录策略
    Err(AppError::NotImplemented)
}

pub async fn login_policy_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /login-policies/{id}  查询登录策略详情
    Err(AppError::NotImplemented)
}

pub async fn login_policy_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /login-policies/{id}  更新登录策略
    Err(AppError::NotImplemented)
}

pub async fn login_policy_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /login-policies/{id}  删除登录策略
    Err(AppError::NotImplemented)
}
