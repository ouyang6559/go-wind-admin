//! B6 脚手架：租户闸门中间件（按 `sys_apis` fail-closed）。
//!
//! Go 端 Kratos 的路由级鉴权：中间件查 `sys_apis`（含 path/method/tenant_scoped/
//! auth_required），对未登记端点 fail-closed 返回 403。Rust 现只做 JWT + 黑名单。
//! 依赖 B2 的 `permissions/sync:perms`（`src/handlers/permission.rs`）把端点清单写入
//! `sys_apis` 后才生效；因此以下签名的数据源就是那张表。
//!
//! 接线点：与 B5 同为 `src/middleware/mod.rs` `layer()` + `src/routes/mod.rs` `build_router()`。

use std::collections::HashMap;
use axum::http::Request;

/// `sys_apis` 中对一个端点登记的可见性规则。
#[derive(Debug, Clone)]
pub struct ApiRule {
    /// 匹配路径（含 `/admin/v1` 前缀）；支持 `/users/{id}` 模板。
    pub path: String,
    pub method: String,
    /// 端点是否要求鉴权。
    pub auth_required: bool,
    /// 端点在多租户下是否按租户过滤（Go 中间件核心开关）。
    pub tenant_scoped: bool,
}

/// 路由注册表查询（返回该端点的规则）。数据源 = `sys_apis` 表。
#[allow(unused_variables)]
pub async fn lookup_rule(
    state: &crate::state::AppState,
    method: &str,
    path: &str,
) -> Option<ApiRule> {
    // SELECT path, method, tenant_scoped, auth_required
    //   FROM sys_apis
    //  WHERE tenant_id = <default> AND method = $1 AND path = $2;
    // 路径模板需归一（{id} ≡ {`id`}），可缓存 HashMap<(method, normalized_path), ApiRule>。
    None
}

/// 端点清单缓存（启动/`sync:perms` 后重建），避免每次请求查表。
#[derive(Default)]
pub struct RuleCache {
    inner: HashMap<(String, String), ApiRule>, // (METHOD, normalized_path)
}

impl RuleCache {
    pub fn build(rows: Vec<ApiRule>) -> Self {
        RuleCache { inner: rows.into_iter().map(|r| ((r.method.clone(), r.path.clone()), r)).collect() }
    }
    pub fn get(&self, method: &str, path: &str) -> Option<&ApiRule> {
        self.inner.get(&(method.to_string(), path.to_string()))
    }
}

/// 租户闸门中间件（示意）。对 `auth_required=false` 的端点放行；其余按规则 fail-closed。
///
/// ```ignore
/// async fn tenant_gate_middleware<B>(
///     State(state): State<Arc<AppState>>,
///     req: Request<B>,
///     next: Next,
/// ) -> Result<Response, StatusCode> {
///     let path = req.uri().path().to_string();
///     let method = req.method().as_str().to_string();
///     let cache = state.rule_cache.read().await;         // RuleCache
///     match cache.get(&method, &path) {
///         None => Err(StatusCode::FORBIDDEN),            // 未登记 → 403（fail-closed）
///         Some(rule) if !rule.auth_required => Ok(next.run(req).await),
///         Some(_) => {
///             // 已由 Operator 提取器完成 JWT+黑名单；
///             // 此处可补充：rule.tenant_scoped 时校验 operator.tenant_id 是否有访问权
///             Ok(next.run(req).await)
///         }
///     }
/// }
/// ```
#[allow(dead_code)]
pub async fn gate<B>(req: Request<B>, state: &crate::state::AppState) -> Result<(), ()> {
    let _ = (req, state);
    Ok(())
}