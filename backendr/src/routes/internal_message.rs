// internal_message 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::internal_message as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/internal-message/messages", axum::routing::get(h::internal_message_list_message));
    r = r.route("/internal-message/messages/{id}", axum::routing::get(h::internal_message_get_message));
    r = r.route("/internal-message/messages/{id}", axum::routing::put(h::internal_message_update_message));
    r = r.route("/internal-message/messages/{id}", axum::routing::delete(h::internal_message_delete_message));
    r = r.route("/internal-message/revoke", axum::routing::post(h::internal_message_revoke_message));
    r = r.route("/internal-message/send", axum::routing::post(h::internal_message_send_message));
    r
}
