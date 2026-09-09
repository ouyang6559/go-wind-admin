// task 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::task as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/tasks", axum::routing::get(h::task_list));
    r = r.route("/tasks", axum::routing::post(h::task_create));
    r = r.route("/tasks/type-name/{typeName}", axum::routing::get(h::task_get));
    r = r.route("/tasks/{id}", axum::routing::get(h::task_get_by_tasks));
    r = r.route("/tasks/{id}", axum::routing::put(h::task_update));
    r = r.route("/tasks/{id}", axum::routing::delete(h::task_delete));
    r = r.route("/tasks/control", axum::routing::post(h::task_control_task));
    r = r.route("/tasks/restart", axum::routing::post(h::task_restart_all_task));
    r = r.route("/tasks/start", axum::routing::post(h::task_start_all_task));
    r = r.route("/tasks/stop", axum::routing::post(h::task_stop_all_task));
    r = r.route("/tasks/type-names", axum::routing::get(h::task_list_task_type_name));
    r
}
