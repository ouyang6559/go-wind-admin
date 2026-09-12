//! B3 脚手架：task 内嵌调度器。
//!
//! 现状：`POST /tasks:start|stop|restart|control` 返回 500 `task scheduler is not
//! configured`（对齐 Go 未配置调度器的行为，见 `src/handlers/task.rs`：
//! `scheduler_not_configured` 约 L377、`task_control_task`/`task_start_all_task`
//! 等约 L384-399）。本脚手架实现一个进程内 tokio-cron 调度器，替换这些占位。
//!
//! 类型注册表（`src/handlers/task.rs` 已有 `REGISTERED_TYPE_NAMES`）：
//! broadcast_message / tenant_expiry_scan / backup / script_task / audit_log_archive。

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// 一次调度执行产生的任务实例 id（用于 start/stop/restart/control 寻址）。
pub type TaskRunId = i64;

/// 系统注册的任务类型 → 执行函数。后续扩展在此注册。
pub type Executor = fn(cron: &str, payload: serde_json::Value) -> BoxFutureStub;

// 占位类型：接入时替换为 `async fn(ctx, payload) -> Result<(), TaskError>`。
pub type BoxFutureStub = std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send>>;

/// 进程内调度器状态。
#[derive(Clone)]
pub struct Scheduler {
    inner: Arc<RwLock<Inner>>,
}

#[derive(Default)]
struct Inner {
    /// typeName → (注册的执行器, 是否允许启动)
    type_names: HashMap<String, Executor>,
    /// taskId → 正在运行的 crontab / 停止句柄
    running: HashMap<i64, tokio::task::JoinHandle<()>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler { inner: Arc::new(RwLock::new(Inner::default())) }
    }

    /// 注册一个任务类型执行器（启动时调用）。
    pub fn register(&self, type_name: &str, exec: Executor) {
        self.inner.blocking_write().type_names.insert(type_name.into(), exec);
    }

    pub fn is_registered(&self, type_name: &str) -> bool {
        self.inner.blocking_read().type_names.contains_key(type_name)
    }

    /// 启动任务：解析 cron（6/5 段）+ 注册运行句柄。返回错误对齐 Go。
    #[allow(unused_variables)]
    pub async fn start(
        &self,
        task_id: i64,
        type_name: &str,
        cron: &str,
        payload: serde_json::Value,
    ) -> Result<(), SchedulerError> {
        if !self.is_registered(type_name) {
            return Err(SchedulerError::Unregistered(type_name.to_string()));
        }
        let Ok(parsed) = parse_cron(cron) else {
            return Err(SchedulerError::BadCron(cron.to_string()));
        };
        // 骨架：用 tokio-cron（或 cron + tokio::time）按 parsed 派发。
        //   例：let scheduler = tokio_cron_scheduler::JobScheduler::new().await?;
        //       Job::new_async(parsed.as_str(), |_, _| exec(payload.clone())).await?;
        // self.inner.write().await.running.insert(task_id, handle);
        let _ = parsed;
        Err(SchedulerError::NotWired)
    }

    /// 停止任务：abort 句柄并从 running 移除。
    #[allow(unused_variables)]
    pub async fn stop(&self, task_id: i64) -> Result<(), SchedulerError> {
        // if let Some(h) = self.inner.write().await.running.remove(&task_id) { h.abort(); }
        Err(SchedulerError::NotWired)
    }

    /// 按 controlType 重启。
    pub async fn control(&self, task_id: i64, control_type: &str, mycron: &str, payload: serde_json::Value) -> Result<(), SchedulerError> {
        match control_type {
            "Start" => self.start(task_id, "", mycron, payload).await,
            "Stop" => self.stop(task_id).await,
            "Restart" => {
                self.stop(task_id).await?;
                self.start(task_id, "", mycron, payload).await
            }
            other => Err(SchedulerError::UnknownControl(other.to_string())),
        }
    }
}

/// 调度器错误（对外映射为 HTTP 4xx/5xx）。
#[derive(Debug)]
pub enum SchedulerError {
    Unregistered(String),
    BadCron(String),
    UnknownControl(String),
    /// 占位：实现未接线，接入后删除此变体。
    NotWired,
}

/// 骨架：解析 cron 表达式（6 段含秒；兼容 5 段）。占位，接入后返回 crate::cron::Schedule。
fn parse_cron(expr: &str) -> Result<(), String> {
    let parts: Vec<&str> = expr.split_whitespace().collect();
    if !matches!(parts.len(), 5 | 6) {
        return Err("cron must have 5 or 6 fields".into());
    }
    let _ = Duration::from_secs(0); // 保留占位以免 unused
    Ok(())
}