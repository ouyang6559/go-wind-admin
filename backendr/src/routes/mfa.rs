// mfa 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::mfa as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/mfa/disable", axum::routing::post(h::mfa_disable_m_f_a));
    r = r.route("/mfa/enroll/confirm", axum::routing::post(h::mfa_confirm_enroll_method));
    r = r.route("/mfa/enroll/start", axum::routing::post(h::mfa_start_enroll_method));
    r = r.route("/mfa/methods", axum::routing::get(h::mfa_list_enrolled_methods));
    r = r.route("/mfa/status", axum::routing::get(h::mfa_get_m_f_a_status));
    r = r.route("/mfa/verify", axum::routing::post(h::mfa_verify_m_f_a_challenge));
    r = r.route("/mfa/{credentialId}", axum::routing::delete(h::mfa_revoke_m_f_a_device));
    r
}
