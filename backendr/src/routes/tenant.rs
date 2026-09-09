// tenant 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::tenant as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/tenants", axum::routing::get(h::tenant_list));
    r = r.route("/tenants", axum::routing::post(h::tenant_create));
    r = r.route("/tenants/{id}", axum::routing::get(h::tenant_get));
    r = r.route("/tenants/{id}", axum::routing::put(h::tenant_update));
    r = r.route("/tenants/{id}", axum::routing::delete(h::tenant_delete));
    r = r.route("/tenants/{id}/cleanup", axum::routing::post(h::tenant_cleanup_data));
    r = r.route("/tenants/{id}/usage", axum::routing::get(h::tenant_get_usage));
    r = r.route("/tenants/exists", axum::routing::get(h::tenant_tenant_exists));
    r = r.route("/tenants/with-admin", axum::routing::post(h::tenant_create_tenant_with_admin_user));
    r
}
