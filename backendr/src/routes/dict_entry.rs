// dict_entry 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::dict_entry as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/dict/entries", axum::routing::get(h::dict_entry_list));
    r = r.route("/dict/entries", axum::routing::post(h::dict_entry_create));
    r = r.route("/dict/entries", axum::routing::delete(h::dict_entry_delete));
    r = r.route("/dict/entries/by-type-code", axum::routing::get(h::dict_entry_list_by_type_code));
    r = r.route("/dict/entries/{id}", axum::routing::put(h::dict_entry_update));
    r
}
