// permission 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::permission as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/permissions", axum::routing::get(h::permission_list));
    r = r.route("/permissions", axum::routing::post(h::permission_create));
    r = r.route("/permissions/sync/perms", axum::routing::post(h::permission_sync_permissions));
    r = r.route("/permissions/{id}", axum::routing::get(h::permission_get));
    r = r.route("/permissions/{id}", axum::routing::put(h::permission_update));
    r = r.route("/permissions/{id}", axum::routing::delete(h::permission_delete));
    r
}
