// admin_portal 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::admin_portal as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/initial-context", axum::routing::get(h::admin_portal_get_initial_context));
    r = r.route("/perm-codes", axum::routing::get(h::admin_portal_get_my_permission_code));
    r = r.route("/routes", axum::routing::get(h::admin_portal_get_navigation));
    r
}
