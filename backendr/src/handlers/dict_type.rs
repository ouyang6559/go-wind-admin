// dict_type 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn dict_type_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /dict/types  分页查询字典类型列表
    Err(AppError::NotImplemented)
}

pub async fn dict_type_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /dict/types  创建字典类型
    Err(AppError::NotImplemented)
}

pub async fn dict_type_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /dict/types  删除字典类型
    Err(AppError::NotImplemented)
}

pub async fn dict_type_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /dict/types/code/{code}  查询字典类型详情
    Err(AppError::NotImplemented)
}

pub async fn dict_type_get_by_types(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /dict/types/{id}  查询字典类型详情
    Err(AppError::NotImplemented)
}

pub async fn dict_type_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /dict/types/{id}  更新字典类型
    Err(AppError::NotImplemented)
}
