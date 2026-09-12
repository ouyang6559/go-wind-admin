//! task 内嵌调度器引擎（对齐 Go 的 asynq cron 语义）。
//!
//! 之前 backendr 的控制类端点（Start/Stop/Restart/ControlTask/StartAll/StopAll/
//! RestartAll）一律返回 500「task scheduler is not configured」降级，本模块用内存
//! cron 调度器将它们改为真实调度执行：
//!   - `register_periodic`：按 cron 每轮计算下次触发时刻并 `sleep`，到点派发执行器；
//!   - `start_all`：加载全部启用 PERIODIC 任务（按 typeName 去重，保留首个）+ 注册
//!     2 个系统级常驻任务（租户到期扫描、审计日志归档）；
//!   - `stop`/`stop_all`：abort 对应协程，移除调度项；
//!   - 执行器按 typeName 分派（见 `run_executor`），每类对应一个真实仓库操作。
//!
//! 与 Go 的差异（有意为之，均为环境边界简化）：
//!   - 无需 asynq 的 Redis 队列，调度项仅在进程内存在（重启丢失，靠启动时 start_all 恢复）；
//!   - `audit_log_archive` 导出 JSONL 用 PostgreSQL `to_jsonb`（未做 MySQL/SQLite 分支）；
//!   - `backup` 导出到本地 JSON 文件而非 OSS，`broadcast_message` 仅统计受众并落日志
//!     （消息队列/逐用户投递表未迁移，见 MISSING.md）。

use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use serde_json::Value;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

/// 系统级常驻任务 cron（对齐 Go `pkg/task`）：租户到期扫描每小时整点，
/// 审计日志归档每天 03:30。
const TENANT_EXPIRY_SCAN_CRON: &str = "0 * * * *";
const AUDIT_LOG_ARCHIVE_CRON: &str = "30 3 * * *";

/// 可注册的任务类型名 → 执行器存在（与 handler 常量对齐，仅作登记）。
const REGISTERED_TYPE_NAMES: &[&str] = &[
    "broadcast_message",
    "tenant_expiry_scan",
    "backup",
    "script_task",
    "audit_log_archive",
];

/// 调度器共享状态：持有 DB/Redis 引用 + 活动调度项（typeName → 协程句柄）。
pub struct TaskScheduler {
    db: Option<sqlx::AnyPool>,
    redis: Option<redis::Client>,
    /// 保留 JWT 密钥以便未来黑名单化；现在吊销仅删会话集合键。
    _jwt_secret: String,
    entries: RwLock<HashMap<String, JoinHandle<()>>>,
}

impl TaskScheduler {
    pub fn new(
        db: Option<sqlx::AnyPool>,
        redis: Option<redis::Client>,
        jwt_secret: String,
    ) -> Arc<Self> {
        Arc::new(Self {
            db,
            redis,
            _jwt_secret: jwt_secret,
            entries: RwLock::new(HashMap::new()),
        })
    }

    /// 是否具备 DB（调度与执行的前提）。
    pub fn is_available(&self) -> bool {
        self.db.is_some()
    }

    /// 注册/改写一个 PERIODIC 任务：先停旧的，再按其 cron 起循环调度。
    /// cron_spec 为 5 段 Go 格式，内部补秒转 6 段后校验。
    pub async fn register_periodic(
        self: &Arc<Self>,
        type_name: &str,
        cron_spec: &str,
        payload: &Value,
    ) -> Result<(), AppError> {
        if self.db.is_none() {
            return Err(AppError::Internal {
                context: "database not configured".into(),
                source: None,
            });
        }
        let cron6 = normalize_cron(cron_spec)?;
        let schedule = cron::Schedule::from_str(&cron6).map_err(|e| AppError::Validation(format!("invalid cron {cron_spec:?}: {e}")))?;

        // 同 typeName 已有调度项则先停掉（对齐 Go NewPeriodicTask 覆盖 entry）
        self.remove(type_name).await;

        let payload = payload.clone();
        let me = self.clone();
        let tn = type_name.to_string();
        let handle = tokio::spawn(async move {
            loop {
                let next = schedule.upcoming(chrono::Utc).next();
                let Some(next) = next else { break };
                let wait = (next - chrono::Utc::now()).to_std();
                let Ok(wait) = wait else { break };
                tokio::time::sleep(wait).await;
                run_executor(&me, &tn, &payload).await;
            }
        });
        self.entries.write().await.insert(type_name.to_string(), handle);
        Ok(())
    }

    /// 一次性任务（DELAY/WAIT_RESULT）：短暂延迟执行一次，不纳入可停调度项
    /// （对齐 Go：此类任务不可 `stop`）。
    pub async fn enqueue_once(self: &Arc<Self>, type_name: &str, delay: std::time::Duration, payload: &Value) {
        let payload = payload.clone();
        let me = self.clone();
        let tn = type_name.to_string();
        tokio::spawn(async move {
            tokio::time::sleep(delay).await;
            run_executor(&me, &tn, &payload).await;
        });
    }

    /// 停止单个已调度项。返回是否曾存在。
    pub async fn remove(&self, type_name: &str) -> bool {
        let had = self.entries.write().await.remove(type_name);
        if let Some(h) = had {
            h.abort();
            return true;
        }
        false
    }

    /// 停止全部调度项。
    pub async fn stop_all(&self) {
        let mut w = self.entries.write().await;
        for (_, h) in w.drain() {
            h.abort();
        }
    }

    /// 启动全部：加载 sys_tasks 中启用且 PERIODIC 的任务（typeName 去重保留首个），
    /// 并注册 2 个系统级常驻任务。返回成功注册数（对齐 Go startAllTask count）。
    pub async fn start_all(self: &Arc<Self>) -> Result<u32, AppError> {
        let Some(db) = self.db.as_ref() else {
            return Ok(0);
        };
        let rows = crate::repos::task::TaskRepo::new(db.clone()).list_all().await?;
        let mut seen: HashMap<String, bool> = HashMap::new();
        let mut count = 0u32;
        for t in &rows {
            if !t.enable || t.r#type != "PERIODIC" {
                continue;
            }
            let Some(tn) = t.type_name.as_deref() else { continue };
            if seen.contains_key(tn) {
                continue;
            }
            seen.insert(tn.to_string(), true);
            let cron = t.cron_spec.as_deref().unwrap_or("0 * * * *");
            let payload = t
                .task_payload
                .as_ref()
                .and_then(|s| serde_json::from_str::<Value>(s).ok())
                .unwrap_or_else(|| Value::Object(Default::default()));
            match self.register_periodic(tn, cron, &payload).await {
                Ok(()) => count += 1,
                Err(e) => tracing::warn!(type_name = tn, error = %e, "start_all: register task failed"),
            }
        }
        // 系统级常驻任务（不依赖 sys_tasks 表）
        let _ = self
            .register_periodic("tenant_expiry_scan", TENANT_EXPIRY_SCAN_CRON, &Value::Object(Default::default()))
            .await;
        let _ = self
            .register_periodic("audit_log_archive", AUDIT_LOG_ARCHIVE_CRON, &Value::Object(Default::default()))
            .await;
        tracing::info!(count, "task scheduler start_all completed");
        Ok(count)
    }
}

/// 5 段 Go cron → 6 段秒优先（cron crate 默认），统一校验并通过长度判断。
fn normalize_cron(raw: &str) -> Result<String, crate::error::AppError> {
    let raw = raw.trim();
    let parts: Vec<&str> = raw.split_whitespace().collect();
    if parts.is_empty() {
        return Err(crate::error::AppError::Validation("cron spec is empty".into()));
    }
    let six = if parts.len() >= 6 { raw.to_string() } else { format!("0 {raw}") };
    if cron::Schedule::from_str(&six).is_err() {
        return Err(crate::error::AppError::Validation(format!(
            "invalid cron spec: {raw}"
        )));
    }
    Ok(six)
}

use crate::error::AppError;

/// 任务执行器分派：typeName → 真实仓库操作。执行失败仅记日志，不中断调度循环。
async fn run_executor(self_: &Arc<TaskScheduler>, type_name: &str, payload: &Value) {
    let started_at = chrono::Utc::now();
    let res = match type_name {
        "tenant_expiry_scan" => exec_tenant_expiry_scan(self_).await,
        "audit_log_archive" => exec_audit_log_archive(self_).await,
        "backup" => exec_backup(self_).await,
        "script_task" => exec_script_task(self_, payload).await,
        "broadcast_message" => exec_broadcast_message(self_, payload).await,
        other => {
            tracing::warn!(type_name = other, "no executor for task type; skipped");
            Ok(())
        }
    };
    match res {
        Ok(()) => tracing::info!(
            type_name,
            elapsed_ms = (chrono::Utc::now() - started_at).num_milliseconds(),
            "task executed"
        ),
        Err(e) => tracing::error!(type_name, error = %e, "task execution failed"),
    }
}

fn db(self_: &TaskScheduler) -> Result<sqlx::AnyPool, String> {
    self_.db.clone().ok_or_else(|| "database not configured".to_string())
}

/// 撤销某用户全部在线令牌：删除 admin/app 客户端会话集合键（对齐 auth::revoke_sessions）。
async fn revoke_user(self_: &TaskScheduler, uid: i64) {
    let Some(c) = self_.redis.as_ref() else { return };
    let mut con = match c.get_multiplexed_tokio_connection().await {
        Ok(c) => c,
        Err(_) => return,
    };
    use redis::AsyncCommands;
    for ct in ["admin", "app"] {
        let _: Result<(), _> = con.del(format!("gw:sessions:{ct}:{uid}")).await;
    }
}

/// 租户到期扫描：status=ON 且已到期的租户按套餐 expiry_policy 改状态并吊销其用户令牌。
/// 对齐 Go `AsyncTenantExpiryScan` / `TenantUsageRepo.EnforceExpiryPolicies`。
async fn exec_tenant_expiry_scan(self_: &Arc<TaskScheduler>) -> Result<(), String> {
    let db = db(self_)?;
    let rows: Vec<(i64, Option<String>)> = sqlx::query_as(
        "select t.id, p.expiry_policy \
         from sys_tenants t \
         left join sys_plans p on p.id = t.plan_id and p.deleted_at is null \
         where t.status = 'ON' and t.deleted_at is null \
           and t.expired_at is not null and t.expired_at <= now()",
    )
    .fetch_all(&db)
    .await
    .map_err(|e| format!("query expired tenants failed: {e}"))?;

    let mut enforced = 0i64;
    let mut revoked_users = 0usize;
    for (tid, policy) in rows {
        let new_status = match policy.as_deref() {
            Some("BLOCK_LOGIN") => Some("EXPIRED"),
            Some("FREEZE") => Some("FREEZE"),
            // READONLY：保持 ON，读写拦截交给租户闸门
            _ => None,
        };
        let Some(new_status) = new_status else {
            tracing::info!(tenant = tid, policy = policy.as_deref().unwrap_or("(no plan)"), "expired tenant kept as-is");
            continue;
        };
        if let Err(e) = sqlx::query("update sys_tenants set status = $1, updated_at = now() where id = $2")
            .bind(new_status)
            .bind(tid)
            .execute(&db)
            .await
        {
            tracing::error!(tenant = tid, error = %e, "expiry scan: update tenant status failed");
            continue;
        }
        // 吊销该租户全部用户令牌
        let uids: Vec<(i64,)> = sqlx::query_as("select id from sys_users where tenant_id = $1 and deleted_at is null")
            .bind(tid)
            .fetch_all(&db)
            .await
            .unwrap_or_default();
        for (uid,) in &uids {
            revoke_user(self_, *uid).await;
        }
        revoked_users += uids.len();
        enforced += 1;
        tracing::info!(tenant = tid, status = new_status, users = uids.len(), "expiry scan enforced");
    }
    tracing::info!(enforced, revoked_users, "tenant expiry scan completed");
    Ok(())
}

/// 审计日志归档：把 created_at < before（保留期）的 6 张审计表行导出 JSONL 后删除。
/// 对齐 Go `AsyncAuditLogArchive` / `AuditLogArchiveRepo.ArchiveExpired`。
async fn exec_audit_log_archive(self_: &Arc<TaskScheduler>) -> Result<(), String> {
    let db = db(self_)?;

    let retention_days = std::env::var("AUDIT_RETENTION_DAYS")
        .ok()
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(180);
    let out_dir = std::env::var("AUDIT_ARCHIVE_DIR").unwrap_or_else(|_| "./data/audit-archive".into());
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("create archive dir failed: {e}"))?;

    let before = chrono::Utc::now() - chrono::Duration::days(retention_days);
    let before_s = before.to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true);
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let tables = [
        "sys_api_audit_logs",
        "sys_login_audit_logs",
        "sys_operation_audit_logs",
        "sys_permission_audit_logs",
        "sys_data_access_audit_logs",
        "sys_policy_evaluation_logs",
    ];

    use std::io::Write;
    let mut total = 0u64;
    for tbl in tables {
        let path = format!("{out_dir}/{tbl}-{stamp}.jsonl");
        let mut total_tbl = 0u64;
        loop {
            // 单批 5000：导出后再按 id 删除，避免无限循环
            let rows: Vec<(i64, String)> = sqlx::query_as(&format!(
                "select t.id, to_jsonb(t)::text as j from {tbl} t \
                 where t.created_at < $1 order by t.created_at asc limit 5000"
            ))
            .bind(&before_s)
            .fetch_all(&db)
            .await
            .map_err(|e| format!("archive {tbl}: query failed: {e}"))?;
            if rows.is_empty() {
                break;
            }
            let mut f = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .map_err(|e| format!("archive {tbl}: open file failed: {e}"))?;
            for (_, j) in &rows {
                let _ = writeln!(f, "{j}");
            }
            f.flush().ok();
            // 删除本批 id
            let placeholders: Vec<String> = (1..=rows.len()).map(|i| format!("${i}")).collect();
            let sql = format!("delete from {tbl} where id in ({})", placeholders.join(","));
            let mut q = sqlx::query(&sql);
            for (id, _) in &rows {
                q = q.bind(*id);
            }
            if let Err(e) = q.execute(&db).await {
                tracing::error!(table = tbl, error = %e, "archive: delete batch failed");
                break;
            }
            total_tbl += rows.len() as u64;
            if rows.len() < 5000 {
                break;
            }
        }
        total += total_tbl;
        if total_tbl > 0 {
            tracing::info!(table = tbl, rows = total_tbl, path, "audit log archived");
        }
    }
    tracing::info!(total, "audit log archive completed");
    Ok(())
}

/// 备份：把核心业务表导出为本地 JSON 文件（对齐 Go AsyncBackup，但落盘非 OSS）。
async fn exec_backup(self_: &Arc<TaskScheduler>) -> Result<(), String> {
    let db = db(self_)?;
    let out_dir = std::env::var("BACKUP_DIR").unwrap_or_else(|_| "./data/backup".into());
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("create backup dir failed: {e}"))?;

    let tables = [
        "sys_tenants",
        "sys_users",
        "sys_roles",
        "sys_user_roles",
        "sys_menus",
        "sys_role_menus",
        "sys_permissions",
        "sys_permission_groups",
        "sys_user_permissions",
        "sys_languages",
        "sys_apis",
        "sys_tasks",
        "sys_scripts",
    ];
    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    for tbl in tables {
        let row: Option<(Option<String>,)> = sqlx::query_as(&format!(
            "select coalesce(json_agg(t), '[]'::json)::text from {tbl} t"
        ))
        .fetch_optional(&db)
        .await
        .map_err(|e| format!("backup {tbl}: export failed: {e}"))?;
        let content = row.and_then(|v| v.0).unwrap_or_else(|| "[]".into());
        let path = format!("{out_dir}/{tbl}-{stamp}.json");
        std::fs::write(&path, content).map_err(|e| format!("backup {tbl}: write failed: {e}"))?;
        tracing::info!(table = tbl, path, "backup table dumped");
    }
    tracing::info!(dir = out_dir, "backup completed");
    Ok(())
}

/// script 任务：按 payload.scriptId 载入脚本并用隔离引擎执行（对齐 B1 scripting）。
async fn exec_script_task(self_: &Arc<TaskScheduler>, payload: &Value) -> Result<(), String> {
    if self_.db.is_none() {
        return Err("database not configured".into());
    }
    let Some(db) = self_.db.clone() else { return Ok(()) };
    let script_id = payload
        .get("scriptId")
        .and_then(|v| v.as_i64())
        .or_else(|| payload.get("script_id").and_then(|v| v.as_i64()));

    let script = match script_id {
        Some(id) => crate::repos::script::ScriptRepo::new(db).get(id).await.map_err(|e| e.to_string())?,
        None => {
            return Err("script task payload missing scriptId".into());
        }
    };
    let Some(script) = script else {
        return Err("script task: script not found".into());
    };
    if !script.is_enabled {
        return Ok(());
    }

    // 执行上下文：以 payload（去掉 scriptId）为输入 ctx
    let mut input: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
    if let Value::Object(map) = payload {
        for (k, v) in map {
            if k != "scriptId" && k != "script_id" {
                input.insert(k.clone(), v.clone());
            }
        }
    }
    let out = crate::scripting::run(&script.language, &script.name, &script.source, &input);
    match out {
        Ok(ctx) => {
            let keys: Vec<String> = ctx.keys().cloned().collect();
            tracing::info!(script_id = script.id, script = %script.name, keys = ?keys, "script_task executed");
            Ok(())
        }
        Err(e) => Err(format!("script_task execute failed: {e}")),
    }
}

/// 广播任务（best-effort）：载入消息、统计受众并落日志。
/// 逐用户投递队列/映射表尚未迁移，见 MISSING.md。
async fn exec_broadcast_message(self_: &Arc<TaskScheduler>, payload: &Value) -> Result<(), String> {
    let db = db(self_)?;
    let message_id = payload
        .get("messageId")
        .and_then(|v| v.as_i64())
        .or_else(|| payload.get("message_id").and_then(|v| v.as_i64()));
    let cnt: (i64,) = sqlx::query_as("select count(*) from sys_users where deleted_at is null")
        .fetch_one(&db)
        .await
        .map_err(|e| format!("broadcast: count users failed: {e}"))?;
    tracing::info!(
        message_id,
        audience = cnt.0,
        "broadcast_message executed (best-effort; per-user delivery queue pending)"
    );
    Ok(())
}