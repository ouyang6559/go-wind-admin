// script_log repo：sys_script_logs 查询与清理。

use sqlx::AnyPool;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct ScriptLogRow {
    pub id: i64,
    pub script_id: i64,
    pub script_name: String,
    /// lua | javascript
    pub language: String,
    /// hook | task | test_run | manual
    pub trigger_type: String,
    pub hook_point: String,
    pub version: i64,
    pub success: bool,
    pub duration_ms: i64,
    pub error: Option<String>,
    pub created_at: Option<String>,
}

const SELECT_COLS: &str = "id, script_id, script_name, language, trigger_type, hook_point, version, \
                           success, duration_ms, error, \
                           to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"')";

type LogTuple = (
    i64,
    i64,
    String,
    String,
    String,
    String,
    i64,
    bool,
    i64,
    Option<String>,
    Option<String>,
);

pub struct ScriptLogRepo {
    pub db: AnyPool,
}

impl ScriptLogRepo {
    pub fn new(db: AnyPool) -> Self {
        Self { db }
    }

    pub async fn list(
        &self,
        offset: u64,
        limit: u64,
        where_clause: &str,
        params: &[String],
        order_by: &str,
    ) -> Result<(Vec<ScriptLogRow>, u64), AppError> {
        let total_sql = format!(
            "select count(*) from sys_script_logs where deleted_at is null{where_clause}"
        );
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "count script logs failed".into(),
                source: Some(Box::new(e)),
            })?;

        let order = if order_by.is_empty() { "id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} from sys_script_logs where deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query_as::<sqlx::Any, LogTuple>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list script logs failed".into(),
                source: Some(Box::new(e)),
            })?;
        let items = rows
            .into_iter()
            .map(|r| ScriptLogRow {
                id: r.0,
                script_id: r.1,
                script_name: r.2,
                language: r.3,
                trigger_type: r.4,
                hook_point: r.5,
                version: r.6,
                success: r.7,
                duration_ms: r.8,
                error: r.9,
                created_at: r.10,
            })
            .collect();
        Ok((items, total.0 as u64))
    }

    /// 对齐 Go CountLog：忽略一切过滤，纯全表 COUNT。
    pub async fn count(&self) -> Result<u64, AppError> {
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(
            "select count(*) from sys_script_logs where deleted_at is null",
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "count script logs failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(row.0 as u64)
    }

    /// 清理 `created_at < before` 的日志；before 为 None 时默认 now - 90 天。
    pub async fn purge(&self, before: Option<chrono::DateTime<chrono::Utc>>) -> Result<u64, AppError> {
        let cutline = before
            .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(90))
            .to_rfc3339();
        let res = sqlx::query::<sqlx::Any>(
            "delete from sys_script_logs where created_at < $1::timestamptz",
        )
        .bind(cutline)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "purge script logs failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }
}
