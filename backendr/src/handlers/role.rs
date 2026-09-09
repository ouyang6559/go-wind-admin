// role 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn role_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /roles  查询角色列表
    Err(AppError::NotImplemented)
}

pub async fn role_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /roles  创建角色
    Err(AppError::NotImplemented)
}

pub async fn role_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /roles/{id}  查询角色详情
    Err(AppError::NotImplemented)
}

pub async fn role_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /roles/{id}  更新角色
    Err(AppError::NotImplemented)
}

pub async fn role_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /roles/{id}  删除角色
    Err(AppError::NotImplemented)
}
