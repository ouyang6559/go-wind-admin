// login_audit_log 模块路由（prefix: /admin/v1）—— 实现统一挂在 handlers::audit_logs
use axum::Router;
use crate::handlers::audit_logs as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/login-audit-logs", axum::routing::get(h::login_audit_log_list));
    r = r.route("/login-audit-logs/{id}", axum::routing::get(h::login_audit_log_get));
    r
}
