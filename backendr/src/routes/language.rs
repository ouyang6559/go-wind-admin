// language 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::language as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/dict/langs", axum::routing::get(h::language_list));
    r = r.route("/dict/langs", axum::routing::post(h::language_create));
    r = r.route("/dict/langs", axum::routing::delete(h::language_delete));
    r = r.route("/dict/langs/batch", axum::routing::post(h::language_batch_create));
    r = r.route("/dict/langs/{id}", axum::routing::get(h::language_get));
    r = r.route("/dict/langs/{id}", axum::routing::put(h::language_update));
    r
}
