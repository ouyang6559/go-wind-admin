// internal_message 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn internal_message_list_message(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /internal-message/messages  查询站内信消息列表
    Err(AppError::NotImplemented)
}

pub async fn internal_message_get_message(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /internal-message/messages/{id}  查询站内信消息详情
    Err(AppError::NotImplemented)
}

pub async fn internal_message_update_message(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /internal-message/messages/{id}  更新站内信消息
    Err(AppError::NotImplemented)
}

pub async fn internal_message_delete_message(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /internal-message/messages/{id}  删除站内信消息
    Err(AppError::NotImplemented)
}

pub async fn internal_message_revoke_message(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /internal-message/revoke  撤销某条消息
    Err(AppError::NotImplemented)
}

pub async fn internal_message_send_message(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /internal-message/send  发送消息
    Err(AppError::NotImplemented)
}
