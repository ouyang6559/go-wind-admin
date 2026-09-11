// server_monitor 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::server_monitor as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/server-monitor", axum::routing::get(h::server_monitor_get));
    r
}
