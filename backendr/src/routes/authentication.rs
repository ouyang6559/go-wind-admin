// authentication 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::authentication as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/captcha", axum::routing::get(h::authentication_generate_captcha));
    r = r.route("/captcha/verify", axum::routing::post(h::authentication_verify_captcha));
    r = r.route("/login", axum::routing::post(h::authentication_login));
    r = r.route("/logout", axum::routing::post(h::authentication_logout));
    r = r.route("/refresh-token", axum::routing::post(h::authentication_refresh_token));
    r = r.route("/register", axum::routing::post(h::authentication_register_user));
    // 找回/重置密码（免鉴权）
    r = r.route("/forgot-password", axum::routing::post(h::authentication_forgot_password));
    r = r.route("/reset-password-by-code", axum::routing::post(h::authentication_reset_password_by_code));
    r
}
