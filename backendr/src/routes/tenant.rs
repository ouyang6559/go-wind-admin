// tenant 模块路由（prefix: /admin/v1）
// proto 冒号风格与斜杠别名并存；/tenants:with-admin 为创建租户+管理员事务，
// Rust 端暂未实现（见 MISSING.md），先移除 501 骨架路由避免误调用。
use axum::Router;
use crate::handlers::task as h;
use crate::handlers::tenant as ht;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/tenants:exists", axum::routing::get(h::tenant_exists));
    r = r.route("/tenants/exists", axum::routing::get(h::tenant_exists));
    r = r.route("/tenants", axum::routing::get(ht::tenant_list));
    r = r.route("/tenants", axum::routing::post(ht::tenant_create));
    r = r.route("/tenants/{id}", axum::routing::get(ht::tenant_get));
    r = r.route("/tenants/{id}", axum::routing::put(ht::tenant_update));
    r = r.route("/tenants/{id}", axum::routing::delete(ht::tenant_delete));
    r = r.route("/tenants/{id}/cleanup", axum::routing::post(ht::tenant_cleanup_data));
    r = r.route("/tenants/{id}/usage", axum::routing::get(ht::tenant_get_usage));
    r = r.route("/tenants:with-admin", axum::routing::post(ht::tenant_create_tenant_with_admin_user));
    r
}
