//! 统一错误类型。
//!
//! `AppError` 全仓统一：handler 返回 `Result<impl IntoResponse, AppError>`，
//! 通过 `IntoResponse` 转成统一 JSON 响应体。

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

/// 对外错误码（与响应体 `code` 字段对应）
pub const CODE_OK: i64 = 0;
pub const CODE_VALIDATION: i64 = 40000;
pub const CODE_UNAUTHORIZED: i64 = 40100;
pub const CODE_FORBIDDEN: i64 = 40300;
pub const CODE_NOT_FOUND: i64 = 40400;
pub const CODE_INTERNAL: i64 = 50000;

#[derive(Debug, Error)]
pub enum AppError {
    /// 骨架占位：业务逻辑未实现
    #[error("接口未实现")]
    NotImplemented,

    /// 参数/校验失败
    #[error("参数校验失败: {0}")]
    Validation(String),

    /// 未认证
    #[error("未认证")]
    Unauthorized,

    /// 无权限
    #[error("无权限: {0}")]
    Forbidden(String),

    /// 资源不存在
    #[error("资源不存在: {0}")]
    NotFound(String),

    /// 内部错误（携带原始源便于排查，不吞错）
    #[error("内部错误: {context}")]
    Internal {
        context: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// 数据库错误透传
    #[error("数据库错误: {0}")]
    Db(#[from] sqlx::Error),

    /// Redis 错误透传
    #[error("Redis 错误: {0}")]
    Redis(#[from] redis::RedisError),
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            AppError::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Db(_) | AppError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn code(&self) -> i64 {
        match self {
            AppError::NotImplemented => 50100,
            AppError::Validation(_) => CODE_VALIDATION,
            AppError::Unauthorized => CODE_UNAUTHORIZED,
            AppError::Forbidden(_) => CODE_FORBIDDEN,
            AppError::NotFound(_) => CODE_NOT_FOUND,
            AppError::Internal { .. } | AppError::Db(_) | AppError::Redis(_) => CODE_INTERNAL,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let code = self.code();
        let message = self.to_string();

        // 内部错误必须带出原始错误对象，便于排查（全仓铁律：不吞错）
        if let AppError::Internal { source, .. } = &self {
            if let Some(src) = source {
                tracing::error!(error = %src, %code, %message, "internal error");
            }
        }

        let body = crate::response::ApiResponse {
            code,
            message,
            data: serde_json::Value::Null,
        };
        (status, axum::Json(body)).into_response()
    }
}