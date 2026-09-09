// position 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn position_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /positions  查询职位列表
    Err(AppError::NotImplemented)
}

pub async fn position_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /positions  创建职位
    Err(AppError::NotImplemented)
}

pub async fn position_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /positions/{id}  查询职位详情
    Err(AppError::NotImplemented)
}

pub async fn position_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /positions/{id}  更新职位
    Err(AppError::NotImplemented)
}

pub async fn position_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /positions/{id}  删除职位
    Err(AppError::NotImplemented)
}
