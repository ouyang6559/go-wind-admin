// server_monitor 模块 handlers。
// 字段对齐 Go ServerMonitorInfo：go/database/host/collectedAt。
// Go 运行时专有指标在 Rust 侧以进程可观测的真实值映射：
//   version → 编译工具链 rustc 版本（build.rs 注入）；
//   num_goroutine → 当前进程存活 OS 线程数；
//   mem_alloc/mem_sys → 进程 RSS / 虚拟内存（sysinfo 跨平台，Linux + macOS + Windows）；
//   gc_cycles → Rust 运行时无 GC 概念，保留 0（见 MISSING.md）。

use axum::extract::State;
use axum::response::IntoResponse;
use serde::Serialize;

use crate::error::AppError;
use crate::response::json_ok;
use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GoRuntimeInfo {
    pub version: String,
    pub num_goroutine: u32,
    pub mem_alloc_bytes: String,
    pub mem_sys_bytes: String,
    pub gc_cycles: u32,
    pub uptime_seconds: String,
    pub started_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabaseInfo {
    pub driver: String,
    pub ping_ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ping_error: Option<String>,
    pub max_open_connections: u32,
    pub open_connections: u32,
    pub in_use_connections: u32,
    pub idle_connections: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostInfo {
    pub os: std::borrow::Cow<'static, str>,
    pub arch: std::borrow::Cow<'static, str>,
    pub num_cpu: u32,
    pub hostname: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerMonitorInfo {
    pub go: GoRuntimeInfo,
    pub database: DatabaseInfo,
    pub host: HostInfo,
    pub collected_at: String,
}

/// 进程启动时刻（近似对齐 Go repo 构造时刻）
static STARTED_AT: once_cell::sync::Lazy<chrono::DateTime<chrono::Utc>> =
    once_cell::sync::Lazy::new(|| std::time::SystemTime::now().into());

pub async fn server_monitor_get(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let now = chrono::Utc::now();
    let uptime = now.signed_duration_since(*STARTED_AT).num_seconds().max(0);

    // Rust 工具链版本（编译期由 build.rs 注入；缺省给出运行时占位）
    let version = option_env!("GOWIND_RUSTC_VERSION")
        .unwrap_or("rust (backendr)")
        .to_string();

    // 进程存活线程数（numGoroutine 的 Rust 侧真实等价）
    let threads = process_threads();
    // 进程内存（RSS / 虚拟内存，sysinfo 跨平台）
    let (mem_alloc, mem_sys) = read_proc_mem();

    // 数据库探活 + 连接池统计
    let database = match &state.db {
        Some(pool) => {
            let ping = sqlx::query::<sqlx::Any>("select 1").execute(pool).await;
            let (ping_ok, ping_error) = match ping {
                Ok(_) => (true, None),
                Err(e) => (
                    false,
                    Some(format!("database ping failed: {e}")),
                ),
            };
            DatabaseInfo {
                driver: "postgres".into(),
                ping_ok,
                ping_error,
                max_open_connections: pool.size().max(1) as u32,
                open_connections: pool.size() as u32,
                in_use_connections: pool.num_idle() as u32,
                idle_connections: pool.num_idle() as u32,
            }
        }
        None => DatabaseInfo {
            driver: String::new(),
            ping_ok: false,
            ping_error: Some("database client is not configured".into()),
            max_open_connections: 0,
            open_connections: 0,
            in_use_connections: 0,
            idle_connections: 0,
        },
    };

    let hostname = hostname();

    Ok(json_ok(ServerMonitorInfo {
        go: GoRuntimeInfo {
            version,
            // num_goroutine 无 Go 概念，以进程存活 OS 线程数为真实等价
            num_goroutine: threads,
            mem_alloc_bytes: mem_alloc.to_string(),
            mem_sys_bytes: mem_sys.to_string(),
            // Rust 运行时无 GC，保留 0（见 MISSING.md）
            gc_cycles: 0,
            uptime_seconds: uptime.to_string(),
            started_at: STARTED_AT.to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
        },
        database,
        host: HostInfo {
            os: std::borrow::Cow::Borrowed(std::env::consts::OS),
            arch: std::borrow::Cow::Borrowed(std::env::consts::ARCH),
            num_cpu: std::thread::available_parallelism()
                .map(|n| n.get() as u32)
                .unwrap_or(1),
            hostname,
        },
        collected_at: now.to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
    }))
}

/// 进程内存（RSS, 虚拟内存），字节数；sysinfo 跨平台（Linux/macOS/Windows）。
fn read_proc_mem() -> (u64, u64) {
    let Some(pid) = sysinfo::get_current_pid().ok() else {
        return (0, 0);
    };
    let kind = sysinfo::RefreshKind::nothing()
        .with_processes(sysinfo::ProcessRefreshKind::nothing());
    let mut s = sysinfo::System::new_with_specifics(kind);
    s.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    match s.process(pid) {
        Some(p) => (p.memory(), p.virtual_memory()),
        None => (0, 0),
    }
}

/// 当前进程存活 OS 线程数（numGoroutine 的 Rust 侧真实等价）。
/// Linux 读 /proc/self/status；其它平台以可用并行数为下限。
fn process_threads() -> u32 {
    #[cfg(target_os = "linux")]
    {
        if let Ok(data) = std::fs::read_to_string("/proc/self/status") {
            for line in data.lines() {
                if let Some(rest) = line.strip_prefix("Threads:") {
                    return rest.trim().parse().unwrap_or(0);
                }
            }
        }
        0
    }
    #[cfg(not(target_os = "linux"))]
    {
        std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1)
    }
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .ok()
        .or_else(|| {
            std::process::Command::new("hostname")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        })
        .unwrap_or_default()
}
