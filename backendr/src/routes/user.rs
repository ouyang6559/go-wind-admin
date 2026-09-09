// user 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::user as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/users", axum::routing::get(h::user_list));
    r = r.route("/users", axum::routing::post(h::user_create));
    r = r.route("/users/username/{username}", axum::routing::get(h::user_get));
    r = r.route("/users/username/{username}", axum::routing::delete(h::user_delete));
    r = r.route("/users/{id}", axum::routing::get(h::user_get_by_users));
    r = r.route("/users/{id}", axum::routing::put(h::user_update));
    r = r.route("/users/{id}", axum::routing::delete(h::user_delete_by_users));
    r = r.route("/users/{userId}/password", axum::routing::post(h::user_edit_user_password));
    r = r.route("/users/exists", axum::routing::get(h::user_user_exists));
    r
}
