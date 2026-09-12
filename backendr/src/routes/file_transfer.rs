// file_transfer 模块路由（prefix: /admin/v1）
use axum::Router;
use crate::handlers::file_transfer as h;
use crate::state::AppState;

pub fn build() -> Router<AppState> {
    let mut r = Router::new();
    r = r.route("/file/download", axum::routing::get(h::file_transfer_download_file));
    r = r.route("/file/upload", axum::routing::put(h::file_transfer_put_upload_file));
    r = r.route("/file/upload", axum::routing::post(h::file_transfer_post_upload_file));
    // 签名图片代理：免鉴权（签名即凭证），注册在已鉴权路由之外由调用方决定挂载
    r = r.route("/file/image", axum::routing::get(h::file_transfer_serve_image));
    r
}
