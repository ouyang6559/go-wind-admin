// plan 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::plan as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/plans", axum::routing::get(h::plan_list));
    r = r.route("/plans", axum::routing::post(h::plan_create));
    r = r.route("/plans", axum::routing::delete(h::plan_delete));
    r = r.route("/plans/{id}", axum::routing::get(h::plan_get));
    r = r.route("/plans/{id}", axum::routing::put(h::plan_update));
    r
}
