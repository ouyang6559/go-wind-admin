// file 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn file_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /files  查询文件列表
    Err(AppError::NotImplemented)
}

pub async fn file_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /files  创建文件
    Err(AppError::NotImplemented)
}

pub async fn file_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /files/{id}  查询文件详情
    Err(AppError::NotImplemented)
}

pub async fn file_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /files/{id}  更新文件
    Err(AppError::NotImplemented)
}

pub async fn file_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /files/{id}  删除文件
    Err(AppError::NotImplemented)
}
