// admin_portal 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn admin_portal_get_initial_context(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /initial-context  一次性获取进入后台所需的所有上下文
    Err(AppError::NotImplemented)
}

pub async fn admin_portal_get_my_permission_code(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /perm-codes  查询权限码列表
    Err(AppError::NotImplemented)
}

pub async fn admin_portal_get_navigation(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /routes  查询前端路由表
    Err(AppError::NotImplemented)
}
