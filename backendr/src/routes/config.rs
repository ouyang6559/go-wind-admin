// config 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::config as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/configs", axum::routing::get(h::config_list));
    r = r.route("/configs", axum::routing::post(h::config_create));
    r = r.route("/configs/{id}", axum::routing::get(h::config_get));
    r = r.route("/configs/{id}", axum::routing::put(h::config_update));
    r = r.route("/configs/{id}", axum::routing::delete(h::config_delete));
    r
}
