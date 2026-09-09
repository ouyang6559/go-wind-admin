// api 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn api_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /apis  查询资源列表
    Err(AppError::NotImplemented)
}

pub async fn api_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /apis  创建资源
    Err(AppError::NotImplemented)
}

pub async fn api_sync_apis(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /apis/sync  同步资源
    Err(AppError::NotImplemented)
}

pub async fn api_get_walk_route_data(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /apis/walk-route  查询路由数据
    Err(AppError::NotImplemented)
}

pub async fn api_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /apis/{id}  查询资源详情
    Err(AppError::NotImplemented)
}

pub async fn api_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /apis/{id}  更新资源
    Err(AppError::NotImplemented)
}

pub async fn api_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /apis/{id}  删除资源
    Err(AppError::NotImplemented)
}
