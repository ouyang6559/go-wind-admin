// plan_quota 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::plan_quota as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/plan-quotas", axum::routing::get(h::plan_quota_list));
    r = r.route("/plan-quotas", axum::routing::post(h::plan_quota_create));
    r = r.route("/plan-quotas", axum::routing::delete(h::plan_quota_delete));
    r = r.route("/plan-quotas/{id}", axum::routing::put(h::plan_quota_update));
    r
}
