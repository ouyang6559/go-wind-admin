//! 统一响应体。
//!
//! 所有接口按 `{ code, message, data }` 返回；`code == 0` 表示成功。

use serde::Serialize;

use crate::error::CODE_OK;

/// 统一 API 响应
#[derive(Debug, Clone, Serialize)]
pub struct ApiResponse<T> {
    pub code: i64,
    pub message: String,
    pub data: T,
}

impl<T> ApiResponse<T> {
    pub fn ok(data: T) -> Self {
        ApiResponse {
            code: CODE_OK,
            message: "ok".into(),
            data,
        }
    }
}

/// JSON 成功响应（data 为业务对象）
pub fn json_ok<T: Serialize>(data: T) -> axum::Json<ApiResponse<T>> {
    axum::Json(ApiResponse::ok(data))
}

/// JSON 空成功响应
pub fn json_ok_null() -> axum::Json<ApiResponse<serde_json::Value>> {
    axum::Json(ApiResponse::ok(serde_json::Value::Null))
}