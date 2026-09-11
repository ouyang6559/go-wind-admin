// user 模块路由（prefix: /admin/v1）
// proto 冒号风格与斜杠别名并存（/users:exists 与 /users/exists 等价）
use axum::Router;
use crate::handlers::task as h;
use crate::handlers::user as hu;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/users:exists", axum::routing::get(h::user_exists));
    r = r.route("/users/exists", axum::routing::get(h::user_exists));
    r = r.route("/users/{user_id}/password", axum::routing::post(h::user_edit_password));
    // 其余 user 端点（CRUD/MFA 关联等）见 MISSING.md
    r = r.route("/users", axum::routing::get(hu::user_list));
    r = r.route("/users", axum::routing::post(hu::user_create));
    r = r.route("/users/{id}", axum::routing::get(hu::user_get));
    r = r.route("/users/{id}", axum::routing::put(hu::user_update));
    r = r.route("/users/{id}", axum::routing::delete(hu::user_delete));
    r = r.route("/users/username/{username}", axum::routing::get(hu::user_get));
    r = r.route("/users/username/{username}", axum::routing::delete(hu::user_delete));
    r
}
