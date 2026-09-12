//! B5: 操作审计中间件。
//!
//! 让 backendr 自身流量产生 `sys_operation_audit_logs` 记录，对齐 Go
//! `pkg/middleware/logging/operation_audit_log.go` 语义：
//! - 仅写操作（POST/PUT/PATCH/DELETE）落库；
//! - 排除会话维护端点（login/logout/refresh/mfa/captcha 等，交登录审计处理）；
//! - 由 `Authorization` 解析当前操作者（user_id/tenant_id/username）；
//! - 写成功标志、IP、log_hash（SHA256）。
//!
//! 落库为 best-effort：DB 不可用或写入失败只 `tracing::error!`（不吞原始错误），
//! 绝不影响业务响应。

use std::net::SocketAddr;

use axum::extract::connect_info::ConnectInfo;
use axum::extract::{Request, State};
use axum::http::header;
use axum::middleware::Next;
use axum::response::Response;

use crate::auth::bearer_token;
use crate::state::AppState;

/// 会话维护端点（跳过操作审计，交登录审计）——对齐 Go sessionOnlyOperations。
const SESSION_ONLY: &[&str] = &[
    "/login",
    "/logout",
    "/refresh-token",
    "/captcha",
    "/mfa",
    "/forgot-password",
    "/reset-password-by-code",
    "/register",
];

fn is_session_only(path: &str) -> bool {
    SESSION_ONLY.iter().any(|s| path.contains(s))
}

/// 由 HTTP 方法 + 路径推导动作；资源名词取路径中最后一个非数字/非 `batch` 段并去复数。
/// 资源 ID 取路径中最后一个纯数字段（对齐 Go `lastNumericPathSegment`）。
fn action_of(method: &str, path: &str) -> &'static str {
    if path.contains("/unassign") {
        "UNASSIGN"
    } else if path.contains("/assign") {
        "ASSIGN"
    } else if path.contains("/export") {
        "EXPORT"
    } else if path.contains("/import") {
        "IMPORT"
    } else {
        match method {
            "POST" => "CREATE",
            "PUT" | "PATCH" => "UPDATE",
            "DELETE" => "DELETE",
            _ => "OTHER",
        }
    }
}

/// 分类写操作。返回 (resource_type, action, resource_id)。
fn classify(method: &str, path: &str) -> Option<(String, &'static str, Option<String>)> {
    // 仅写操作
    if !matches!(method, "POST" | "PUT" | "PATCH" | "DELETE") {
        return None;
    }
    if is_session_only(path) {
        return None;
    }
    let segs: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let mut resource: Option<&str> = None;
    let mut resource_id: Option<String> = None;
    for seg in &segs {
        if seg.parse::<i64>().is_ok() {
            resource_id = Some(seg.to_string());
        } else if *seg != "batch" {
            resource = Some(seg);
        }
    }
    let resource_type = resource
        .map(|s| s.trim_end_matches('s').to_string())
        .unwrap_or_else(|| "other".to_string());
    Some((resource_type, action_of(method, path), resource_id))
}

/// 客户端 IP：优先 ConnectInfo，其次 X-Forwarded-For 首段。
fn client_ip(req: &Request) -> String {
    if let Some(ConnectInfo(addr)) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
        return addr.ip().to_string();
    }
    if let Some(v) = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
    {
        if let Some(first) = v.split(',').next() {
            return first.trim().to_string();
        }
    }
    String::new()
}

pub async fn audit(State(state): State<AppState>, req: Request, next: Next) -> Response {
    // 先取请求元信息（Request 非 Clone，须在 move 进 next.run 前读完）
    let method = req.method().as_str().to_string();
    let path = req.uri().path().to_string();
    let Some((resource_type, action, resource_id)) = classify(&method, &path) else {
        return next.run(req).await;
    };

    // 操作者：从 Authorization 解析（记录用，不校验黑名单）
    let operator = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| bearer_token(Some(h)))
        .and_then(|tok| crate::auth::verify_access_token(&state.jwt_secret, &tok).ok());
    // 未认证（无有效操作者）请求不写操作审计——对齐 Go：操作审计中间件挂在
    // 认证后的路由组，未认证请求先被 401 拦下，故 Go 的操作审计行身份字段恒定非空。
    // 若这里也为未认证请求落库，会产生 tenant_id/user_id/username 全 NULL 的行，
    // 与 Go wire 不一致（前端列表缺身份列）。
    let Some(operator) = operator else {
        return next.run(req).await;
    };
    let ip = client_ip(&req);

    let resp = next.run(req).await;

    let success = resp.status().is_success();
    let status_code = resp.status().as_u16();
    let username = operator.sub.clone();
    let user_id = operator.uid;
    let tenant_id = operator.tid;
    // CreatedAt 显式写非空（仓库既有约定：mixin 无默认值会落 NULL）
    let created_at = chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Nanos, true);
    let request_id = uuid::Uuid::new_v4().to_string();

    // log_hash：对 canonical payload 做 SHA256（十六进制小写）
    let payload = serde_json::json!({
        "created_at": created_at,
        "tenant_id": tenant_id,
        "user_id": user_id,
        "username": username,
        "resource_type": resource_type,
        "resource_id": resource_id,
        "action": action,
        "request_id": request_id,
        "success": success,
        "status_code": status_code,
        "ip_address": ip,
    });
    let log_hash = {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(payload.to_string().as_bytes());
        format!("{:x}", h.finalize())
    };

    if let Some(db) = &state.db {
        // best-effort 落库：失败必须打日志，绝不静默吞错
        let sql = "insert into sys_operation_audit_logs \
                   (created_at, tenant_id, user_id, username, resource_type, resource_id, \
                    action, request_id, success, failure_reason, ip_address, log_hash) \
                   values ($1::timestamptz, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)";
        let res = sqlx::query(sql)
            .bind(created_at.as_str())
            .bind(tenant_id)
            .bind(user_id)
            .bind(username)
            .bind(resource_type.clone())
            .bind(resource_id.clone())
            .bind(action)
            .bind(request_id)
            .bind(success)
            .bind::<Option<String>>(None)
            .bind(ip.clone())
            .bind(log_hash.as_str())
            .execute(db)
            .await;
        if let Err(e) = res {
            tracing::error!(error = %e, %method, %path, "operation audit write failed");
        }
    }

    resp
}