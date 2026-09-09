// internal_message_category 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn internal_message_category_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /internal-message/categories  查询站内信消息分类列表
    Err(AppError::NotImplemented)
}

pub async fn internal_message_category_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /internal-message/categories  创建站内信消息分类
    Err(AppError::NotImplemented)
}

pub async fn internal_message_category_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /internal-message/categories/{id}  查询站内信消息分类详情
    Err(AppError::NotImplemented)
}

pub async fn internal_message_category_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /internal-message/categories/{id}  更新站内信消息分类
    Err(AppError::NotImplemented)
}

pub async fn internal_message_category_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /internal-message/categories/{id}  删除站内信消息分类
    Err(AppError::NotImplemented)
}
