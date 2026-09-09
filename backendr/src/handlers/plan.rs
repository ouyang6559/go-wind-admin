// plan 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn plan_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /plans  分页查询套餐列表
    Err(AppError::NotImplemented)
}

pub async fn plan_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /plans  创建套餐
    Err(AppError::NotImplemented)
}

pub async fn plan_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /plans  删除套餐
    Err(AppError::NotImplemented)
}

pub async fn plan_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /plans/{id}  查询套餐详情
    Err(AppError::NotImplemented)
}

pub async fn plan_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /plans/{id}  更新套餐
    Err(AppError::NotImplemented)
}
