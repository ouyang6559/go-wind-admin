//! GoWind Admin — Rust/axum rewrite 主库入口。
//!
//! 分层约定（middleware -> handler -> service -> repo -> db）：
//! - `handlers` 只做 HTTP 解析与统一响应包装，不写业务逻辑；
//! - `services` 承载业务规则，可组合多个 repo；
//! - `repos` 封装数据访问（sqlx AnyPool / redis）；
//! - `dto` 为 OpenAPI 生成的实体与请求/响应模型；
//! - `routes` 负责把每个业务模块的 Router 聚合挂载到 `/admin/v1`。

pub mod config;
pub mod crypto;
pub mod dto;
pub mod error;
pub mod handlers;
pub mod mailer;
pub mod middleware;
pub mod query;
pub mod repos;
pub mod response;
pub mod routes;
pub mod scheduler;
pub mod scripting;
pub mod services;
pub mod state;
pub mod storage;
pub mod api_manifest;
pub mod auth;