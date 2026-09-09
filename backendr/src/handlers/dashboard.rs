// dashboard 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn dashboard_get_login_status_distribution(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /dashboard/login-status-distribution  登录审计按 分布
    Err(AppError::NotImplemented)
}

pub async fn dashboard_get_login_trend(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /dashboard/login-trend  获取近 天每日登录次数趋势
    Err(AppError::NotImplemented)
}

pub async fn dashboard_get_operation_action_distribution(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /dashboard/operation-action-distribution  操作审计按 分布
    Err(AppError::NotImplemented)
}

pub async fn dashboard_get_overview(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /dashboard/overview  获取概览统计（用户总数 / 角色总数 / 今日登录次数 / 今日操作审计条数）
    Err(AppError::NotImplemented)
}
