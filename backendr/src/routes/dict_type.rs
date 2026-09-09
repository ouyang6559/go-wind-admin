// dict_type 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::dict_type as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/dict/types", axum::routing::get(h::dict_type_list));
    r = r.route("/dict/types", axum::routing::post(h::dict_type_create));
    r = r.route("/dict/types", axum::routing::delete(h::dict_type_delete));
    r = r.route("/dict/types/code/{code}", axum::routing::get(h::dict_type_get));
    r = r.route("/dict/types/{id}", axum::routing::get(h::dict_type_get_by_types));
    r = r.route("/dict/types/{id}", axum::routing::put(h::dict_type_update));
    r
}
