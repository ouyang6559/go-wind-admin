// login_audit_log 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn login_audit_log_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /login-audit-logs  查询登录审计日志列表
    Err(AppError::NotImplemented)
}

pub async fn login_audit_log_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /login-audit-logs/{id}  查询登录审计日志详情
    Err(AppError::NotImplemented)
}
