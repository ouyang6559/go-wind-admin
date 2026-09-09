// language 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn language_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /dict/langs  分页查询语言列表
    Err(AppError::NotImplemented)
}

pub async fn language_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /dict/langs  创建语言
    Err(AppError::NotImplemented)
}

pub async fn language_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /dict/langs  删除语言
    Err(AppError::NotImplemented)
}

pub async fn language_batch_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /dict/langs/batch  批量创建语言
    Err(AppError::NotImplemented)
}

pub async fn language_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /dict/langs/{id}  查询语言详情
    Err(AppError::NotImplemented)
}

pub async fn language_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /dict/langs/{id}  更新语言
    Err(AppError::NotImplemented)
}
