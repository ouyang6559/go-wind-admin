// task 模块路由（prefix: /admin/v1）
// 控制类端点同时注册 proto 冒号风格（tasks:restart）与斜杠别名（tasks/restart）：
// 前端按 proto 生成调用冒号路径；斜杠形式仅为兼容早期 axum 路由。
use axum::Router;
use crate::handlers::task as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/tasks", axum::routing::get(h::task_list));
    r = r.route("/tasks", axum::routing::post(h::task_create));
    r = r.route("/tasks/type-name/{type_name}", axum::routing::get(h::task_get));
    r = r.route("/tasks/{id}", axum::routing::get(h::task_get_by_tasks));
    r = r.route("/tasks/{id}", axum::routing::put(h::task_update));
    r = r.route("/tasks/{id}", axum::routing::delete(h::task_delete));
    // proto 冒号风格
    r = r.route("/tasks:control", axum::routing::post(h::task_control_task));
    r = r.route("/tasks:restart", axum::routing::post(h::task_restart_all_task));
    r = r.route("/tasks:start", axum::routing::post(h::task_start_all_task));
    r = r.route("/tasks:stop", axum::routing::post(h::task_stop_all_task));
    r = r.route("/tasks:type-names", axum::routing::get(h::task_list_task_type_name));
    // 斜杠别名
    r = r.route("/tasks/control", axum::routing::post(h::task_control_task));
    r = r.route("/tasks/restart", axum::routing::post(h::task_restart_all_task));
    r = r.route("/tasks/start", axum::routing::post(h::task_start_all_task));
    r = r.route("/tasks/stop", axum::routing::post(h::task_stop_all_task));
    r = r.route("/tasks/type-names", axum::routing::get(h::task_list_task_type_name));
    r
}
