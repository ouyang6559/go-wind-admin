// policy_evaluation_log 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn policy_evaluation_log_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /policy-evaluation-logs  查询策略评估日志列表
    Err(AppError::NotImplemented)
}

pub async fn policy_evaluation_log_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /policy-evaluation-logs/{id}  查询策略评估日志详情
    Err(AppError::NotImplemented)
}
