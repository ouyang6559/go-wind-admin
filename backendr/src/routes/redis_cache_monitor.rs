// redis_cache_monitor 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::redis_cache_monitor as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/redis-cache-monitor", axum::routing::get(h::redis_cache_monitor_get));
    r
}
