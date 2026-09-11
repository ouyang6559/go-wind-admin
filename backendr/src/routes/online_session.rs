// online_session 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::online_session as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/online-session/sessions", axum::routing::get(h::sessions_list));
    r = r.route("/online-session/my-sessions", axum::routing::get(h::my_sessions_list));
    r = r.route("/online-session/force-logout", axum::routing::post(h::force_logout_session));
    r = r.route("/online-session/my-sessions/revoke", axum::routing::post(h::revoke_my_session));
    r
}
