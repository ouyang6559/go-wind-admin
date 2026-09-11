// script_log 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::script_log as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/script/logs", axum::routing::get(h::script_log_list));
    r = r.route("/script/logs/count", axum::routing::get(h::script_log_count));
    r = r.route("/script/logs/purge", axum::routing::post(h::script_log_purge));
    r
}
