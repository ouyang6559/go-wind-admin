// data_access_audit_log 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::data_access_audit_log as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/data-access-audit-logs", axum::routing::get(h::data_access_audit_log_list));
    r = r.route("/data-access-audit-logs/{id}", axum::routing::get(h::data_access_audit_log_get));
    r
}
