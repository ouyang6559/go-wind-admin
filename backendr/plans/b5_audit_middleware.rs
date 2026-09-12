//! B5 脚手架：审计日志中间件（tower）。
//!
//! 目标：让 **Rust 自身流量** 产生审计日志——目前 DB 中的审计数据都是 Go 实例写入的
//! （见 `MISSING.md` 三.B5）。需实现 tower middleware：逐请求落库 + 地理解析 + 设备解析
//! + 日志哈希签名链。
//!
//! 现状接线点：
//! - `src/middleware/mod.rs` `layer()`（约 L16）返回 `Identity` 占位。
//! - 路由装配 `src/routes/mod.rs` `build_router()`（约 L41）。
//! 后端起在 `:7666`；旁路响应用 `tower_http` 或手写 `axum::middleware::from_fn_with_state`。
//!
//! 落库目标表（对齐 Go 审计 service，参照 `src/repos/operation_audit_log.rs` 的 schema）：
//! `sys_operation_audit_logs`（action/module/request/response/operator_id/tenant_id/ip/ua/geometry/device/hash/created_at …）。
//! 注意：`CreatedAt` 必须显式 `time.Now()` 非空（仓库既有约定），签名链 `log_hash = hash(prev_hash || payload)`。

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

/// 单条审计记录（优先复用 `src/repos/operation_audit_log.rs` 的行结构）。
#[derive(Debug, Clone)]
pub struct AuditRecord {
    pub action: String,
    pub module: String,
    pub uri: String,
    pub method: String,
    pub status: u16,
    pub operator_id: Option<i64>,
    pub tenant_id: Option<i64>,
    pub ip: String,
    pub user_agent: String,
    pub request_snippet: Option<String>,
    pub prev_hash: Arc<AtomicMethodName>, // 占位：见下
    pub timestamp_ms: u64,
}
// 占位类型：签名链应存上一跳 digest，接入时改为 `String`。
pub struct AtomicMethodName(pub Mutex<String>);

/// 地理/设备解析：接入可调用离线 GeoIP 库（如 `maxminddb`）或占位返回空。
pub struct GeoResolver;
impl GeoResolver {
    pub fn resolve(_ip: &str) -> Option<String> { None }
}

/// 设备解析：UA 字符串 → 设备类型/系统/浏览器。
pub struct DeviceResolver;
impl DeviceResolver {
    pub fn resolve(_ua: &str) -> Option<String> { None }
}

/// 哈希签名链：`hash = sha256(prev_hash || canonical_payload)`。
/// prev_hash 取自全局链尾；写入后更新链尾。
#[allow(unused_variables)]
pub fn sign(prev_hash: &str, payload: &str) -> String {
    // sha2::Sha256 计算 hex，作为本记录 hash 字段；调用方再回写链尾指针。
    String::new()
}

/// tower 中间件层主入口（示意实现，待并入 `layer()`）。
///
/// ```ignore
/// // 在 src/middleware/mod.rs 的 layer() 中：
/// use axum::middleware::from_fn_with_state;
/// pub fn layer(state: Arc<AppState>) -> ... {
///     from_fn_with_state(state, audit_middleware)
/// }
///
/// async fn audit_middleware(
///     State(state): State<Arc<AppState>>,
///     req: Request,
///     next: Next,
/// ) -> Response {
///     let start = SystemTime::now();
///     let resp = next.run(req).await;                 // 读 body 后用 Buffer::new_response 再包
///     let rec = AuditRecord { /* 由 req/resp 填 */ };
///     // best-effort 落库（不吞原始错误；若 DB 失败至少 eprintln!）
///     resp
/// }
/// ```
#[allow(dead_code)]
fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

/// 链尾全局指针（进程内单例）。接入后落到 Redis 或内存原子变量，跨进程保持一致。
static CHAIN_TAIL: once_cell_lazy_tail!(());

// 占位宏：接入时替换为 once_cell::sync::Lazy<std::sync::Mutex<String>>。
macro_rules! once_cell_lazy_tail {
    () => {
        Arc::new(AtomicU64::new(0))
    };
}