// internal_message_category 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::internal_message_category as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/internal-message/categories", axum::routing::get(h::internal_message_category_list));
    r = r.route("/internal-message/categories", axum::routing::post(h::internal_message_category_create));
    r = r.route("/internal-message/categories/{id}", axum::routing::get(h::internal_message_category_get));
    r = r.route("/internal-message/categories/{id}", axum::routing::put(h::internal_message_category_update));
    r = r.route("/internal-message/categories/{id}", axum::routing::delete(h::internal_message_category_delete));
    r
}
