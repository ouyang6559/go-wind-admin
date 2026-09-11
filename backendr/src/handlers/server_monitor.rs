// server_monitor 模块 handlers。
// 字段对齐 Go ServerMonitorInfo：go/database/host/collectedAt。
// Go 运行时专有指标（goroutine/GC 次数）在 Rust 侧以进程可观测的等价/占位值填充。

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

    // 进程内存（Linux /proc 可用；其它平台为 0 占位）
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
            // Rust 运行时无 goroutine 概念，tokio worker 数以 0 占位（见 MISSING.md）
            num_goroutine: 0,
            mem_alloc_bytes: mem_alloc.to_string(),
            mem_sys_bytes: mem_sys.to_string(),
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

fn read_proc_mem() -> (u64, u64) {
    #[cfg(target_os = "linux")]
    {
        if let Ok(data) = std::fs::read_to_string("/proc/self/statm") {
            let mut it = data.split_whitespace();
            let _total = it.next();
            let resident_pages = it.next().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
            let page = 4096u64;
            return (resident_pages * page, resident_pages * page);
        }
        (0, 0)
    }
    #[cfg(not(target_os = "linux"))]
    {
        (0, 0)
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
