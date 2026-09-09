// permission_group 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::permission_group as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/permission-groups", axum::routing::get(h::permission_group_list));
    r = r.route("/permission-groups", axum::routing::post(h::permission_group_create));
    r = r.route("/permission-groups/{id}", axum::routing::get(h::permission_group_get));
    r = r.route("/permission-groups/{id}", axum::routing::put(h::permission_group_update));
    r = r.route("/permission-groups/{id}", axum::routing::delete(h::permission_group_delete));
    r
}
