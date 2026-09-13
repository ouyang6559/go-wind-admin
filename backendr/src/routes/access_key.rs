// access_key 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::access_key as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/access-keys", axum::routing::get(h::access_key_list));
    r = r.route("/access-keys", axum::routing::post(h::access_key_create));
    r = r.route("/access-keys/{id}", axum::routing::get(h::access_key_get));
    r = r.route("/access-keys/{id}", axum::routing::put(h::access_key_update));
    r = r.route("/access-keys/{id}", axum::routing::delete(h::access_key_delete));
    // 重置密钥（轮换）：新 SK 明文仅本次返回
    r = r.route("/access-keys/{id}/secret", axum::routing::put(h::access_key_reset_secret));
    // 令牌交换：免鉴权端点（本身即认证）；路由顺序上须先于 Operator 提取层——
    // 该 handler 不声明 Operator 提取器，中间件层对未认证请求放行至此
    r = r.route("/access-keys/token", axum::routing::post(h::access_key_issue_token));
    r
}
