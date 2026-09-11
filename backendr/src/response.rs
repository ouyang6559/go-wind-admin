//! 统一响应约定：**裸 DTO**（与 Go/Kratos wire 格式一致）。
//!
//! Go 端（Kratos 默认编码）成功响应直接返回 proto JSON（无 `{code,message,data}`
//! envelope），`google.protobuf.Empty` 返回 `{}`；列表返回 `{"items":[...],"total":"N"}`
//! （total 为 int64 的 protojson 字符串形式）。本仓所有 handler 遵循同一约定。

use axum::Json;
use serde::Serialize;

/// 成功：直接返回业务 DTO（裸 JSON，无 envelope）
pub fn json_ok<T: Serialize>(data: T) -> Json<T> {
    Json(data)
}

/// 成功：空对象（对齐 google.protobuf.Empty 的 protojson 序列化）
pub fn json_empty() -> Json<serde_json::Value> {
    Json(serde_json::json!({}))
}

/// 列表响应：`{"items":[...],"total":"N"}`（total 为字符串，对齐 protojson int64）
#[derive(Debug, Clone, Serialize)]
pub struct ListResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: String,
}

impl<T: Serialize> ListResponse<T> {
    pub fn new(items: Vec<T>, total: u64) -> Self {
        ListResponse {
            items,
            total: total.to_string(),
        }
    }
}
