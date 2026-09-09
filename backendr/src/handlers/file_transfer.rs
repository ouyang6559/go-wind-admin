// file_transfer 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn file_transfer_download_file(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /file/download  下载文件
    Err(AppError::NotImplemented)
}

pub async fn file_transfer_put_upload_file(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /file/upload  上传文件 方式
    Err(AppError::NotImplemented)
}

pub async fn file_transfer_post_upload_file(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /file/upload  上传文件 方式
    Err(AppError::NotImplemented)
}
