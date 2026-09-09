// policy_evaluation_log 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::policy_evaluation_log as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/policy-evaluation-logs", axum::routing::get(h::policy_evaluation_log_list));
    r = r.route("/policy-evaluation-logs/{id}", axum::routing::get(h::policy_evaluation_log_get));
    r
}
