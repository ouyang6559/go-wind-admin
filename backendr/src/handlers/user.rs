// user 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn user_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /users  获取用户列表
    Err(AppError::NotImplemented)
}

pub async fn user_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /users  创建用户
    Err(AppError::NotImplemented)
}

pub async fn user_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /users/username/{username}  获取用户数据
    Err(AppError::NotImplemented)
}

pub async fn user_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /users/username/{username}  删除用户
    Err(AppError::NotImplemented)
}

pub async fn user_get_by_users(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /users/{id}  获取用户数据
    Err(AppError::NotImplemented)
}

pub async fn user_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /users/{id}  更新用户
    Err(AppError::NotImplemented)
}

pub async fn user_delete_by_users(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /users/{id}  删除用户
    Err(AppError::NotImplemented)
}

pub async fn user_edit_user_password(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /users/{userId}/password  修改用户密码
    Err(AppError::NotImplemented)
}

pub async fn user_user_exists(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /users/exists  用户是否存在
    Err(AppError::NotImplemented)
}
