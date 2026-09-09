// permission_audit_log 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn permission_audit_log_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /permission-audit-logs  查询权限变更审计日志列表
    Err(AppError::NotImplemented)
}

pub async fn permission_audit_log_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /permission-audit-logs/{id}  查询权限变更审计日志详情
    Err(AppError::NotImplemented)
}
