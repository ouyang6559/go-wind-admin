// menu 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn menu_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /menus  查询菜单列表
    Err(AppError::NotImplemented)
}

pub async fn menu_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /menus  创建菜单
    Err(AppError::NotImplemented)
}

pub async fn menu_sync_menus(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /menus/sync  同步菜单
    Err(AppError::NotImplemented)
}

pub async fn menu_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /menus/{id}  查询菜单详情
    Err(AppError::NotImplemented)
}

pub async fn menu_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /menus/{id}  更新菜单
    Err(AppError::NotImplemented)
}

pub async fn menu_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /menus/{id}  删除菜单
    Err(AppError::NotImplemented)
}
