// user_profile 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::user_profile as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/me", axum::routing::get(h::user_profile_get_user));
    r = r.route("/me", axum::routing::put(h::user_profile_update_user));
    r = r.route("/me/avatar", axum::routing::post(h::user_profile_upload_avatar));
    r = r.route("/me/avatar", axum::routing::delete(h::user_profile_delete_avatar));
    r = r.route("/me/contact", axum::routing::post(h::user_profile_bind_contact));
    r = r.route("/me/contact/verify", axum::routing::post(h::user_profile_verify_contact));
    r = r.route("/me/password", axum::routing::post(h::user_profile_change_password));
    r
}
