// role 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::role as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/roles", axum::routing::get(h::role_list));
    r = r.route("/roles", axum::routing::post(h::role_create));
    r = r.route("/roles/{id}", axum::routing::get(h::role_get));
    r = r.route("/roles/{id}", axum::routing::put(h::role_update));
    r = r.route("/roles/{id}", axum::routing::delete(h::role_delete));
    r
}
