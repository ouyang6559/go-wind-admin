// file_transfer 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::file_transfer as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/file/download", axum::routing::get(h::file_transfer_download_file));
    r = r.route("/file/upload", axum::routing::put(h::file_transfer_put_upload_file));
    r = r.route("/file/upload", axum::routing::post(h::file_transfer_post_upload_file));
    r
}
