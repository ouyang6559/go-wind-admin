// dict_entry 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn dict_entry_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /dict/entries  分页查询字典条目列表
    Err(AppError::NotImplemented)
}

pub async fn dict_entry_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /dict/entries  创建字典条目
    Err(AppError::NotImplemented)
}

pub async fn dict_entry_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /dict/entries  删除字典条目
    Err(AppError::NotImplemented)
}

pub async fn dict_entry_list_by_type_code(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /dict/entries/by-type-code  查询启用的字典条目
    Err(AppError::NotImplemented)
}

pub async fn dict_entry_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /dict/entries/{id}  更新字典条目
    Err(AppError::NotImplemented)
}
