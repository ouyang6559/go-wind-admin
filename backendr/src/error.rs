//! 统一错误类型。
//!
//! `AppError` 全仓统一：handler 返回 `Result<impl IntoResponse, AppError>`，
//! 通过 `IntoResponse` 转成与 Go/Kratos 完全一致的错误响应：
//! HTTP 状态码 + `{"code":<http状态码>,"reason":"<REASON>","message":"...","metadata":{}}`。
//! 前端 i18n 依赖 `reason` 字段做错误文案翻译，不可缺失。

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    /// 骨架占位：业务逻辑未实现
    #[error("接口未实现")]
    NotImplemented,

    /// 参数/校验失败（message 直接对齐 Go 端的英文文案，供 wire 比对）
    #[error("{0}")]
    Validation(String),

    /// 未认证
    #[error("missing bearer token")]
    Unauthorized,

    /// 无权限
    #[error("{0}")]
    Forbidden(String),

    /// 资源不存在
    #[error("{0}")]
    NotFound(String),

    /// 资源冲突（唯一键等）
    #[error("{0}")]
    Conflict(String),

    /// 内部错误（携带原始源便于排查，不吞错）
    #[error("{context}")]
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

/// Kratos 错误响应体（对齐 `{"code","reason","message","metadata"}`）
#[derive(Debug, Serialize)]
pub struct KratosErrorBody {
    pub code: u16,
    pub reason: &'static str,
    pub message: String,
    pub metadata: serde_json::Value,
}

impl AppError {
    fn status(&self) -> StatusCode {
        match self {
            AppError::NotImplemented => StatusCode::NOT_IMPLEMENTED,
            AppError::Validation(_) => StatusCode::BAD_REQUEST,
            AppError::Unauthorized => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::Internal { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Db(_) | AppError::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Kratos reason（对齐 proto admin_error/audit_error 枚举名）
    fn reason(&self) -> &'static str {
        match self {
            AppError::NotImplemented => "NOT_IMPLEMENTED",
            AppError::Validation(_) => "BAD_REQUEST",
            AppError::Unauthorized => "UNAUTHORIZED",
            AppError::Forbidden(_) => "FORBIDDEN",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Conflict(_) => "CONFLICT",
            AppError::Internal { .. } => "INTERNAL_SERVER_ERROR",
            AppError::Db(_) | AppError::Redis(_) => "INTERNAL_SERVER_ERROR",
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let reason = self.reason();
        let message = self.to_string();

        // 内部错误必须带出原始错误对象，便于排查（全仓铁律：不吞错）
        if let AppError::Internal { source, .. } = &self {
            if let Some(src) = source {
                tracing::error!(error = %src, %reason, %message, "internal error");
            }
        }

        let body = KratosErrorBody {
            code: status.as_u16(),
            reason,
            message,
            metadata: serde_json::json!({}),
        };
        (status, axum::Json(body)).into_response()
    }
}
