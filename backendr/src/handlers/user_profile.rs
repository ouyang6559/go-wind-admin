// user_profile 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn user_profile_get_user(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /me  获取用户资料
    Err(AppError::NotImplemented)
}

pub async fn user_profile_update_user(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /me  更新用户资料
    Err(AppError::NotImplemented)
}

pub async fn user_profile_upload_avatar(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /me/avatar  上传头像
    Err(AppError::NotImplemented)
}

pub async fn user_profile_delete_avatar(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /me/avatar  删除头像
    Err(AppError::NotImplemented)
}

pub async fn user_profile_bind_contact(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /me/contact  绑定手机号码/邮箱
    Err(AppError::NotImplemented)
}

pub async fn user_profile_verify_contact(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /me/contact/verify  验证手机号码/邮箱
    Err(AppError::NotImplemented)
}

pub async fn user_profile_change_password(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /me/password  修改用户密码
    Err(AppError::NotImplemented)
}
