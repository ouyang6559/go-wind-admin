// plan_module 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::plan_module as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/plan-modules", axum::routing::get(h::plan_module_list));
    r = r.route("/plan-modules", axum::routing::post(h::plan_module_create));
    r = r.route("/plan-modules", axum::routing::delete(h::plan_module_delete));
    r = r.route("/plan-modules/{id}", axum::routing::get(h::plan_module_get));
    r = r.route("/plan-modules/{id}", axum::routing::put(h::plan_module_update));
    r
}
