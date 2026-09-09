// dashboard 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::dashboard as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/dashboard/login-status-distribution", axum::routing::get(h::dashboard_get_login_status_distribution));
    r = r.route("/dashboard/login-trend", axum::routing::get(h::dashboard_get_login_trend));
    r = r.route("/dashboard/operation-action-distribution", axum::routing::get(h::dashboard_get_operation_action_distribution));
    r = r.route("/dashboard/overview", axum::routing::get(h::dashboard_get_overview));
    r
}
