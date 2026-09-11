// script 模块路由（prefix: /admin/v1；路径对齐 Go proto：Delete 走 query ids）
use axum::Router;
use crate::handlers::script as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/scripts", axum::routing::get(h::script_list));
    r = r.route("/scripts", axum::routing::post(h::script_create));
    r = r.route("/scripts", axum::routing::delete(h::script_delete));
    r = r.route("/scripts/count", axum::routing::get(h::script_count));
    r = r.route("/scripts/name/{name}", axum::routing::get(h::script_get_by_name));
    r = r.route("/scripts/{id}", axum::routing::get(h::script_get));
    r = r.route("/scripts/{id}", axum::routing::put(h::script_update));
    r = r.route("/scripts/test_run", axum::routing::post(h::script_test_run));
    r = r.route("/script/hooks", axum::routing::get(h::script_list_hook_points));
    r
}
