// org_unit 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn org_unit_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /org-units  查询组织单元列表
    Err(AppError::NotImplemented)
}

pub async fn org_unit_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /org-units  创建组织单元
    Err(AppError::NotImplemented)
}

pub async fn org_unit_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /org-units/{id}  查询组织单元详情
    Err(AppError::NotImplemented)
}

pub async fn org_unit_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /org-units/{id}  更新组织单元
    Err(AppError::NotImplemented)
}

pub async fn org_unit_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /org-units/{id}  删除组织单元
    Err(AppError::NotImplemented)
}
