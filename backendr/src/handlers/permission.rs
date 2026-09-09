// permission 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn permission_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /permissions  查询权限点列表
    Err(AppError::NotImplemented)
}

pub async fn permission_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /permissions  创建权限点
    Err(AppError::NotImplemented)
}

pub async fn permission_sync_permissions(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /permissions/sync/perms  同步权限点
    Err(AppError::NotImplemented)
}

pub async fn permission_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /permissions/{id}  查询权限点详情
    Err(AppError::NotImplemented)
}

pub async fn permission_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /permissions/{id}  更新权限点
    Err(AppError::NotImplemented)
}

pub async fn permission_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /permissions/{id}  删除权限点
    Err(AppError::NotImplemented)
}
