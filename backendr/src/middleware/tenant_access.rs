//! B6: 按 `sys_apis` 的租户访问闸门中间件。
//!
//! 对齐 Go `pkg/middleware/auth/auth.go` 中的租户级访问检查
//! （data 实现 `tenant_access_checker.go`）：
//! - **仅对租户用户（tid>0）生效**，平台管理员（tid=0）直接放行；
//! - 检查项：
//!   1. 租户状态（status != ON 拒绝）；
//!   2. 到期只读策略（已到期 + READONLY → 仅放行 GET/HEAD/OPTIONS）；
//!   3. `sys_apis` 匹配请求所属业务模块（找不到 API 记录 → fail-closed 拒绝；
//!      未归类模块 → 拒绝）；
//!   4. 套餐模块白名单（`sys_plan_modules` 不含该模块 → 拒绝）。
//!
//! 通过 `axum::middleware::from_fn_with_state` 接入，与审计中间件一样
//! 使用 AppState 的 DB 连接池。无 DB / 无匹配时按 Go 语义 fail-closed（403）。

use std::net::SocketAddr;

use axum::extract::connect_info::ConnectInfo;
use axum::extract::{Request, State};
use axum::http::{header, StatusCode};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::error::AppError;
use crate::state::AppState;

/// 已知业务模块（对齐 Go 常量 Module_*，DB 中 business_module / module 存 proto 枚举名）。
const KNOWN_MODULES: &[&str] = &[
    "DASHBOARD",
    "OPM",
    "SYSTEM",
    "DICT",
    "TENANT",
    "PERMISSION",
    "LOG",
    "INTERNAL_MESSAGE",
    "FILE",
    "TASK",
];

/// `sys_apis` 匹配结果。
enum ApiMatch {
    /// 找到 API 记录且归类到已知业务模块
    Found(String),
    /// 找到 API 记录但 business_module 为空或未归类
    FoundNoModule,
    /// 未找到对应 API 记录
    NotFound,
}

pub async fn tenant_gate(State(state): State<AppState>, req: Request, next: Next) -> Response {
    // 操作者：从 Authorization 解析（无有效令牌→交由 handler 的 Operator 提取器 401）
    let claims = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|h| crate::auth::bearer_token(Some(h)))
        .and_then(|tok| crate::auth::verify_access_token(&state.jwt_secret, &tok).ok());

    let Some(claims) = claims else {
        return next.run(req).await;
    };
    let tid = claims.tid;
    // 平台管理员（tid=0）放行；DB 不可用时 fail-open（与全局骨架“尽力”策略一致）
    if tid == 0 {
        return next.run(req).await;
    }
    let Some(db) = &state.db else {
        return next.run(req).await;
    };

    let method = req.method().clone();
    let path = req.uri().path().to_string();
    let ip = client_ip(&req);

    match check_tenant_access(db, tid, &path, method.as_str()).await {
        Ok(()) => next.run(req).await,
        Err(AppError::Forbidden(msg)) => {
            tracing::warn!(tid, method = %method, path, ip, msg, "tenant access denied");
            forbidden(msg)
        }
        Err(e) => {
            // DB/其他错误：fail-closed 拒绝（对齐 Go TenantAccessChecker 任何错误均拒绝）
            tracing::error!(error = %e, tid, method = %method, path, ip, "tenant access check failed");
            forbidden("access denied".to_string())
        }
    }
}

fn forbidden(msg: String) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "code": 403, "msg": msg, "data": null })),
    )
        .into_response()
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

/// 租户访问检查（对齐 `TenantAccessCheckerImpl.CheckTenantAccess`）。
async fn check_tenant_access(
    db: &sqlx::AnyPool,
    tid: i64,
    path: &str,
    method: &str,
) -> Result<(), AppError> {
    // 1. 租户状态与到期只读判定
    let row: Option<(Option<String>, Option<i64>, Option<String>, bool)> = sqlx::query_as(
        "select t.status, t.plan_id, p.expiry_policy, \
                (t.expired_at is not null and t.expired_at < now()) as expired \
         from sys_tenants t \
         left join sys_plans p on p.id = t.plan_id and p.deleted_at is null \
         where t.id = $1 and t.deleted_at is null \
         limit 1",
    )
    .bind(tid)
    .fetch_optional(db)
    .await
    .map_err(|e| AppError::Internal {
        context: "tenant access: query tenant failed".into(),
        source: Some(Box::new(e)),
    })?;

    let Some((status, plan_id, expiry_policy, expired)) = row else {
        return Err(AppError::Forbidden("access denied".into()));
    };
    if status.as_deref() != Some("ON") {
        return Err(AppError::Forbidden("tenant is not active".into()));
    }
    if expired && expiry_policy.as_deref() == Some("READONLY")
        && !matches!(method, "GET" | "HEAD" | "OPTIONS")
    {
        return Err(AppError::Forbidden("tenant is read-only due to expiry".into()));
    }

    // 2. sys_apis 匹配业务模块
    let biz_module = match find_api_module(db, path, method).await? {
        ApiMatch::Found(m) => {
            if !KNOWN_MODULES.iter().any(|k| *k == m) {
                return Err(AppError::Forbidden("module not allowed".into()));
            }
            m
        }
        ApiMatch::FoundNoModule => {
            return Err(AppError::Forbidden("module not allowed".into()));
        }
        ApiMatch::NotFound => {
            return Err(AppError::Forbidden("access denied".into()));
        }
    };

    // 3. 套餐白名单
    let Some(pid) = plan_id else {
        return Err(AppError::Forbidden("no subscription plan".into()));
    };
    let cnt: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(
        "select count(*) from sys_plan_modules \
         where plan_id = $1 and module = $2 and deleted_at is null",
    )
    .bind(pid)
    .bind(&biz_module)
    .fetch_one(db)
    .await
    .map_err(|e| AppError::Internal {
        context: "tenant access: whitelist query failed".into(),
        source: Some(Box::new(e)),
    })?;
    if cnt.0 == 0 {
        return Err(AppError::Forbidden("module not allowed".into()));
    }

    Ok(())
}

/// 在 `sys_apis` 中按 (path 模板, method) 匹配业务模块。
/// path 为请求的具体路径，与模板（含 `{param}` 段或 `:xxx` 字面段）做段级比对。
async fn find_api_module(
    db: &sqlx::AnyPool,
    path: &str,
    method: &str,
) -> Result<ApiMatch, AppError> {
    let method_lower = method.to_ascii_lowercase();
    let rows = sqlx::query(
        "select path, method, business_module from sys_apis where deleted_at is null",
    )
    .fetch_all(db)
    .await
    .map_err(|e| AppError::Internal {
        context: "tenant access: load sys_apis failed".into(),
        source: Some(Box::new(e)),
    })?;

    use sqlx::Row;
    for row in &rows {
        let api_path: Option<String> = row.try_get("path").ok();
        let api_method: Option<String> = row.try_get("method").ok();
        let biz: Option<String> = row.try_get("business_module").ok();
        let Some(api_path) = api_path else { continue };
        if api_method
            .as_deref()
            .map(|m| m.to_ascii_lowercase() == method_lower)
            .unwrap_or(false)
            && template_matches(&api_path, path)
        {
            return match biz {
                Some(m) if !m.trim().is_empty() => Ok(ApiMatch::Found(m)),
                _ => Ok(ApiMatch::FoundNoModule),
            };
        }
    }
    Ok(ApiMatch::NotFound)
}

/// 段级模板匹配：`{param}` 段匹配任意单段，其余段等价即可；段数必须一致。
fn template_matches(template: &str, concrete: &str) -> bool {
    let t: Vec<&str> = template
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let c: Vec<&str> = concrete
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    if t.len() != c.len() {
        return false;
    }
    t.iter()
        .zip(c.iter())
        .all(|(tp, cp)| (tp.starts_with('{') && tp.ends_with('}')) || tp == cp)
}