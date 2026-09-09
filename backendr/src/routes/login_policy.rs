// login_policy 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::login_policy as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/login-policies", axum::routing::get(h::login_policy_list));
    r = r.route("/login-policies", axum::routing::post(h::login_policy_create));
    r = r.route("/login-policies/{id}", axum::routing::get(h::login_policy_get));
    r = r.route("/login-policies/{id}", axum::routing::put(h::login_policy_update));
    r = r.route("/login-policies/{id}", axum::routing::delete(h::login_policy_delete));
    r
}
