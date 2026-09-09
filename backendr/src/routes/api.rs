// api 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::api as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/apis", axum::routing::get(h::api_list));
    r = r.route("/apis", axum::routing::post(h::api_create));
    r = r.route("/apis/sync", axum::routing::post(h::api_sync_apis));
    r = r.route("/apis/walk-route", axum::routing::get(h::api_get_walk_route_data));
    r = r.route("/apis/{id}", axum::routing::get(h::api_get));
    r = r.route("/apis/{id}", axum::routing::put(h::api_update));
    r = r.route("/apis/{id}", axum::routing::delete(h::api_delete));
    r
}
