// plan_module 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn plan_module_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /plan-modules  分页查询套餐模块列表
    Err(AppError::NotImplemented)
}

pub async fn plan_module_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /plan-modules  创建套餐模块
    Err(AppError::NotImplemented)
}

pub async fn plan_module_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /plan-modules  删除套餐模块
    Err(AppError::NotImplemented)
}

pub async fn plan_module_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /plan-modules/{id}  查询套餐模块详情
    Err(AppError::NotImplemented)
}

pub async fn plan_module_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /plan-modules/{id}  更新套餐模块
    Err(AppError::NotImplemented)
}
