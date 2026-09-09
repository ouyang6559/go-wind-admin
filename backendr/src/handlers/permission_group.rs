// permission_group 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn permission_group_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /permission-groups  查询权限组列表
    Err(AppError::NotImplemented)
}

pub async fn permission_group_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /permission-groups  创建权限组
    Err(AppError::NotImplemented)
}

pub async fn permission_group_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /permission-groups/{id}  查询权限组详情
    Err(AppError::NotImplemented)
}

pub async fn permission_group_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /permission-groups/{id}  更新权限组
    Err(AppError::NotImplemented)
}

pub async fn permission_group_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /permission-groups/{id}  删除权限组
    Err(AppError::NotImplemented)
}
