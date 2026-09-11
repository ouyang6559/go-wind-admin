// internal_message_recipient 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::internal_message_recipient as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/internal-message/inbox", axum::routing::get(h::internal_message_recipient_list_user_inbox));
    r = r.route("/internal-message/inbox/delete", axum::routing::post(h::internal_message_recipient_delete_notification_from_inbox));
    r = r.route("/internal-message/read", axum::routing::post(h::internal_message_recipient_mark_notification_as_read));
    r = r.route("/internal-message/status", axum::routing::post(h::internal_message_recipient_mark_notifications_status));
    r
}
