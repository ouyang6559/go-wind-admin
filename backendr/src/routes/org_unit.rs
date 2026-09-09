// org_unit 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::org_unit as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/org-units", axum::routing::get(h::org_unit_list));
    r = r.route("/org-units", axum::routing::post(h::org_unit_create));
    r = r.route("/org-units/{id}", axum::routing::get(h::org_unit_get));
    r = r.route("/org-units/{id}", axum::routing::put(h::org_unit_update));
    r = r.route("/org-units/{id}", axum::routing::delete(h::org_unit_delete));
    r
}
