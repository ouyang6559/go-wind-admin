// notification_channel 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::notification_channel as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/notification-channels", axum::routing::get(h::notification_channel_list));
    r = r.route("/notification-channels", axum::routing::post(h::notification_channel_create));
    r = r.route("/notification-channels/{id}", axum::routing::get(h::notification_channel_get));
    r = r.route("/notification-channels/{id}", axum::routing::put(h::notification_channel_update));
    r = r.route("/notification-channels/{id}", axum::routing::delete(h::notification_channel_delete));
    r = r.route("/notification-channels/{id}/send-test-email", axum::routing::post(h::notification_channel_send_test_email));
    r
}
