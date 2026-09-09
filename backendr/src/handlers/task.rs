// task 模块 handlers（骨架，业务逻辑待填充）
use axum::extract::State;
use crate::state::AppState;
use crate::error::AppError;

pub async fn task_list(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /tasks  查询调度任务列表
    Err(AppError::NotImplemented)
}

pub async fn task_create(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /tasks  创建调度任务
    Err(AppError::NotImplemented)
}

pub async fn task_get(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /tasks/type-name/{typeName}  查询调度任务详情
    Err(AppError::NotImplemented)
}

pub async fn task_get_by_tasks(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /tasks/{id}  查询调度任务详情
    Err(AppError::NotImplemented)
}

pub async fn task_update(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // PUT /tasks/{id}  更新调度任务
    Err(AppError::NotImplemented)
}

pub async fn task_delete(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // DELETE /tasks/{id}  删除调度任务
    Err(AppError::NotImplemented)
}

pub async fn task_control_task(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /tasks/control  控制调度任务
    Err(AppError::NotImplemented)
}

pub async fn task_restart_all_task(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /tasks/restart  重启所有的调度任务
    Err(AppError::NotImplemented)
}

pub async fn task_start_all_task(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /tasks/start  启动所有的调度任务
    Err(AppError::NotImplemented)
}

pub async fn task_stop_all_task(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // POST /tasks/stop  停止所有的调度任务
    Err(AppError::NotImplemented)
}

pub async fn task_list_task_type_name(_state: State<AppState>) -> Result<axum::response::Response, AppError> {
    // GET /tasks/type-names  任务类型名称列表
    Err(AppError::NotImplemented)
}
