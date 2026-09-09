// file 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::file as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/files", axum::routing::get(h::file_list));
    r = r.route("/files", axum::routing::post(h::file_create));
    r = r.route("/files/{id}", axum::routing::get(h::file_get));
    r = r.route("/files/{id}", axum::routing::put(h::file_update));
    r = r.route("/files/{id}", axum::routing::delete(h::file_delete));
    r
}
