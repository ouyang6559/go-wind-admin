//! 中间件与扩展提取器。
//!
//! 当前提供：全局中间件装配（跟踪 ID / CORS / 日志）与一个
//! `Operator` 提取器（从 `Authorization: Bearer <access_token>` 解析当前用户，
//! HS256 验签，字段对齐 Kratos `pkg/jwt`：`uid`/`tid`/`sub`/`roc`）。

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;

use crate::auth::bearer_token;
use crate::error::AppError;
use crate::state::AppState;

/// 全局中间件：CORS + 请求跟踪 + 访问日志。
pub fn layer(
) -> tower::layer::util::Identity {
    // 骨架阶段先用 Identity 占位，避免引入泛型粘合复杂度；
    // 正式接入时改为 tower_http::trace + cors + propagation。
    tower::layer::util::Identity::new()
}

/// 当前操作者（由 Authorization 头解析并验签 JWT）。
#[derive(Debug, Clone)]
pub struct Operator {
    pub user_id: i64,
    pub tenant_id: i64,
    pub username: String,
    pub roles: Vec<String>,
    /// 当前仅支持 admin 客户端（令牌未携带 client_type，登录/登出固定 admin）
    pub client_type: String,
}

impl<S> FromRequestParts<S> for Operator
where
    S: Send + Sync + AsRef<AppState>,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());
        let token = bearer_token(header).ok_or(AppError::Unauthorized)?;
        let claims = crate::auth::verify_access_token(&state.as_ref().jwt_secret, &token)?;
        Ok(Operator {
            user_id: claims.uid,
            tenant_id: claims.tid,
            username: claims.sub,
            roles: claims.roc.unwrap_or_default(),
            client_type: "admin".to_string(),
        })
    }
}

/// 供骨架使用的占位路由（HTTP 层拒绝未实现调用）。
pub async fn not_implemented_fallback() -> Result<StatusCode, AppError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}