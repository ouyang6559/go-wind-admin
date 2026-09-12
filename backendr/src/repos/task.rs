// task repo：sys_tasks CRUD SQL（硬删除、type 枚举 PERIODIC/DELAY/WAIT_RESULT、task_options jsonb 透传）。
// 对齐 Go task_repo.go：(tenant_id, type_name) 唯一、type 枚举名落库、task_payload 以文本输出。

use serde_json::Value;
use sqlx::AnyPool;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct TaskRow {
    pub id: i64,
    pub tenant_id: Option<i64>,
    /// PERIODIC | DELAY | WAIT_RESULT（proto 枚举名）
    pub r#type: String,
    pub type_name: Option<String>,
    pub task_payload: Option<String>,
    pub cron_spec: Option<String>,
    pub task_options: Option<Value>,
    pub enable: bool,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

pub struct TaskRepo {
    pub db: AnyPool,
}

type TaskTuple = (
    i64,
    Option<i64>,
    String,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    bool,
    Option<String>,
    Option<i64>,
    Option<i64>,
    Option<String>,
    Option<String>,
);

const SELECT_COLS: &str = "id, tenant_id, type, type_name, task_payload::text, cron_spec, \
                           task_options::text, enable, remark, created_by, updated_by, \
                           to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"'), \
                           to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"')";

fn map_row(r: TaskTuple) -> TaskRow {
    TaskRow {
        id: r.0,
        tenant_id: r.1,
        r#type: r.2,
        type_name: r.3,
        task_payload: r.4,
        cron_spec: r.5,
        task_options: r.6.and_then(|s| serde_json::from_str(&s).ok()),
        enable: r.7,
        remark: r.8,
        created_by: r.9,
        updated_by: r.10,
        created_at: r.11,
        updated_at: r.12,
    }
}

impl TaskRepo {
    pub fn new(db: AnyPool) -> Self {
        Self { db }
    }

    /// 分页 + 列过滤（列已白名单校验，仅限任务自身字段）。
    pub async fn list(
        &self,
        offset: u64,
        limit: u64,
        where_clause: &str,
        params: &[String],
        order_by: &str,
    ) -> Result<(Vec<TaskRow>, u64), AppError> {
        let total_sql = format!("select count(*) from sys_tasks where deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "count tasks failed".into(),
                source: Some(Box::new(e)),
            })?;

        let order = if order_by.is_empty() { "id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} from sys_tasks where deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query_as::<sqlx::Any, TaskTuple>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list tasks failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok((rows.into_iter().map(map_row).collect(), total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<TaskRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} from sys_tasks where id = $1 and deleted_at is null limit 1"
        );
        let row = sqlx::query_as::<sqlx::Any, TaskTuple>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get task failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.map(map_row))
    }

    /// 全量任务（调度器 start_all 用，跨租户，按 typeName 去重依赖上层）。
    pub async fn list_all(&self) -> Result<Vec<TaskRow>, AppError> {
        let sql = format!("select {SELECT_COLS} from sys_tasks where deleted_at is null");
        let rows = sqlx::query_as::<sqlx::Any, TaskTuple>(&sql)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list all tasks failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(rows.into_iter().map(map_row).collect())
    }

    /// 按 type_name 查询（须具名租户上下文，平台上下文在 handler 拦截）。
    pub async fn get_by_type_name(
        &self,
        tenant_id: i64,
        type_name: &str,
    ) -> Result<Option<TaskRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} from sys_tasks \
             where tenant_id = $1 and type_name = $2 and deleted_at is null limit 1"
        );
        let row = sqlx::query_as::<sqlx::Any, TaskTuple>(&sql)
            .bind(tenant_id)
            .bind(type_name)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get task by type name failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.map(map_row))
    }

    /// (tenant_id, type_name) 维度已存在校验（排除自身）。
    pub async fn type_name_exists(&self, tenant_id: i64, type_name: &str, exclude_id: i64) -> Result<bool, AppError> {
        let sql = "select 1 from sys_tasks \
                   where tenant_id = $1 and type_name = $2 and id <> $3 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(tenant_id)
            .bind(type_name)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check task type name failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 创建任务。task_payload/task_options 为 jsonb，直接落 JSON 文本。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        tenant_id: i64,
        r#type: &str,
        type_name: &str,
        task_payload: Option<&Value>,
        cron_spec: Option<&str>,
        task_options: Option<&Value>,
        enable: bool,
        remark: Option<&str>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into sys_tasks \
                   (tenant_id, type, type_name, task_payload, cron_spec, task_options, enable, remark, \
                    created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4::jsonb, $5, $6::jsonb, $7, $8, \
                           $9, $10::timestamptz, $10::timestamptz) returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(tenant_id)
            .bind(r#type)
            .bind(type_name)
            .bind(task_payload.map(|v| v.to_string()))
            .bind(cron_spec)
            .bind(task_options.map(|v| v.to_string()))
            .bind(enable)
            .bind(remark)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert task failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新任务：仅非 None 字段（tenant_id 不可变，对齐 Go TenantID mixin Immutable）。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        r#type: Option<&str>,
        type_name: Option<&str>,
        task_payload: Option<&Value>,
        cron_spec: Option<&str>,
        task_options: Option<&Value>,
        enable: Option<bool>,
        remark: Option<&str>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        macro_rules! push {
            // $cast: "::int8" / "::jsonb" / "::timestamptz" / ""
            ($col:expr, $val:expr, $cast:expr) => {{
                sets.push(format!("{} = ${}{}", $col, sets.len() + 1, $cast));
                params.push($val);
            }};
        }

        if let Some(v) = r#type {
            push!("type", v.to_string(), "");
        }
        if let Some(v) = type_name {
            push!("type_name", v.to_string(), "");
        }
        if let Some(v) = task_payload {
            push!("task_payload", v.to_string(), "::jsonb");
        }
        if let Some(v) = cron_spec {
            push!("cron_spec", v.to_string(), "");
        }
        if let Some(v) = task_options {
            push!("task_options", v.to_string(), "::jsonb");
        }
        if let Some(v) = enable {
            push!("enable", v.to_string(), "");
        }
        if let Some(v) = remark {
            push!("remark", v.to_string(), "");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        let sql = format!(
            "update sys_tasks set {} where id = ${} and deleted_at is null",
            sets.join(", "),
            params.len() + 1
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        q = q.bind(id);
        let res = q
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "update task failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// 硬删除（对齐本仓其他模块约定，Go DeleteOneID 为硬删）。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let res = sqlx::query::<sqlx::Any>("delete from sys_tasks where id = $1 and deleted_at is null")
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete task failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }
}
