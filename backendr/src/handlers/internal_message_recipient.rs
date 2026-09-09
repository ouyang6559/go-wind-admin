// internal_message_recipient 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn internal_message_recipient_list_user_inbox(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /internal-message/inbox  获取用户的收件箱列表 (通知类)
    Err(AppError::NotImplemented)
}

pub async fn internal_message_recipient_delete_notification_from_inbox(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /internal-message/inbox/delete  删除用户收件箱中的通知记录
    Err(AppError::NotImplemented)
}

pub async fn internal_message_recipient_mark_notification_as_read(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /internal-message/read  将通知标记为已读
    Err(AppError::NotImplemented)
}
