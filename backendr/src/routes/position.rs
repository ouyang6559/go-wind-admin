// position 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::position as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/positions", axum::routing::get(h::position_list));
    r = r.route("/positions", axum::routing::post(h::position_create));
    r = r.route("/positions/{id}", axum::routing::get(h::position_get));
    r = r.route("/positions/{id}", axum::routing::put(h::position_update));
    r = r.route("/positions/{id}", axum::routing::delete(h::position_delete));
    r
}
