//! 中间件与扩展提取器。
//!
//! 当前提供：全局中间件装配（跟踪 ID / CORS / 日志）与一个
//! `Operator` 占位提取器（后续从 JWT 解析当前用户）。
//! 认证与鉴权链路当前编排在 `routes` 层待接入，这里保证骨架可编译。

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;

use crate::error::AppError;

/// 全局中间件：CORS + 请求跟踪 + 访问日志。
pub fn layer(
) -> tower::layer::util::Identity {
    // 骨架阶段先用 Identity 占位，避免引入泛型粘合复杂度；
    // 正式接入时改为 tower_http::trace + cors + propagation。
    tower::layer::util::Identity::new()
}

/// 当前操作者（由 Authorization 头解析 JWT；骨架阶段仅做非空校验）。
pub struct Operator {
    pub user_id: i64,
    pub tenant_id: i64,
}

impl<S> FromRequestParts<S> for Operator
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        match parts.headers.get(axum::http::header::AUTHORIZATION) {
            Some(h) if !h.is_empty() => {
                // TODO: 解析 JWT，填充 user_id / tenant_id
                tracing::debug!("auth header present, tenant-scoped filter pending");
                Ok(Operator {
                    user_id: 0,
                    tenant_id: 1,
                })
            }
            _ => Err(AppError::Internal {
                context: "认证上下文占位未实现".into(),
                source: None,
            }),
        }
    }
}

/// 供骨架使用的占位路由（HTTP 层拒绝未实现调用）。
pub async fn not_implemented_fallback() -> Result<StatusCode, AppError> {
    Ok(StatusCode::NOT_IMPLEMENTED)
}