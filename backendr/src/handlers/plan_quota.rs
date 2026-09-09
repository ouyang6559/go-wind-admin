// plan_quota 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn plan_quota_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /plan-quotas  分页查询套餐配额列表
    Err(AppError::NotImplemented)
}

pub async fn plan_quota_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /plan-quotas  创建套餐配额
    Err(AppError::NotImplemented)
}

pub async fn plan_quota_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /plan-quotas  删除套餐配额
    Err(AppError::NotImplemented)
}

pub async fn plan_quota_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /plan-quotas/{id}  更新套餐配额
    Err(AppError::NotImplemented)
}
