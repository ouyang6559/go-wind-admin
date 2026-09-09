// menu 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::menu as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/menus", axum::routing::get(h::menu_list));
    r = r.route("/menus", axum::routing::post(h::menu_create));
    r = r.route("/menus/sync", axum::routing::post(h::menu_sync_menus));
    r = r.route("/menus/{id}", axum::routing::get(h::menu_get));
    r = r.route("/menus/{id}", axum::routing::put(h::menu_update));
    r = r.route("/menus/{id}", axum::routing::delete(h::menu_delete));
    r
}
