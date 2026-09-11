// script repo：sys_scripts CRUD SQL（硬删除、version 自增等对齐 Go script_repo.go）。

use sqlx::AnyPool;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct ScriptRow {
    pub id: i64,
    pub name: String,
    /// LUA | JAVASCRIPT
    pub language: String,
    pub hook_point: Option<String>,
    pub source: String,
    pub priority: i64,
    pub description: Option<String>,
    pub critical: bool,
    pub version: i64,
    pub is_enabled: bool,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

pub struct ScriptRepo {
    pub db: AnyPool,
}

type ScriptTuple = (
    i64,
    String,
    String,
    Option<String>,
    String,
    i64,
    Option<String>,
    bool,
    i64,
    bool,
    Option<i64>,
    Option<i64>,
    Option<String>,
    Option<String>,
);

const SELECT_COLS: &str = "id, name, language, hook_point, source, priority, description, critical, \
                           version, is_enabled, created_by, updated_by, \
                           to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), \
                           to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"')";

fn map_row(r: ScriptTuple) -> ScriptRow {
    ScriptRow {
        id: r.0,
        name: r.1,
        language: r.2,
        hook_point: r.3,
        source: r.4,
        priority: r.5,
        description: r.6,
        critical: r.7,
        version: r.8,
        is_enabled: r.9,
        created_by: r.10,
        updated_by: r.11,
        created_at: r.12,
        updated_at: r.13,
    }
}

impl ScriptRepo {
    pub fn new(db: AnyPool) -> Self {
        Self { db }
    }

    /// 分页 + 列过滤（列已白名单校验，仅限脚本自身字段）。
    pub async fn list(
        &self,
        offset: u64,
        limit: u64,
        where_clause: &str,
        params: &[String],
        order_by: &str,
    ) -> Result<(Vec<ScriptRow>, u64), AppError> {
        let total_sql = format!(
            "select count(*) from sys_scripts where deleted_at is null{where_clause}"
        );
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "count scripts failed".into(),
                source: Some(Box::new(e)),
            })?;

        let order = if order_by.is_empty() { "id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} from sys_scripts where deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query_as::<sqlx::Any, ScriptTuple>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list scripts failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok((rows.into_iter().map(map_row).collect(), total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<ScriptRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} from sys_scripts where id = $1 and deleted_at is null limit 1"
        );
        let row = sqlx::query_as::<sqlx::Any, ScriptTuple>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get script failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.map(map_row))
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Option<ScriptRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} from sys_scripts where name = $1 and deleted_at is null limit 1"
        );
        let row = sqlx::query_as::<sqlx::Any, ScriptTuple>(&sql)
            .bind(name)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get script by name failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.map(map_row))
    }

    /// 名称唯一性检查（exclude_id 排除自身）。
    pub async fn name_exists(&self, name: &str, exclude_id: i64) -> Result<bool, AppError> {
        let sql = "select 1 from sys_scripts where name = $1 and id <> $2 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check script name failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 创建脚本。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        name: &str,
        language: &str,
        hook_point: Option<&str>,
        source: &str,
        priority: i64,
        description: Option<&str>,
        critical: bool,
        is_enabled: bool,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into sys_scripts \
                   (name, language, hook_point, source, priority, description, critical, is_enabled, \
                    version, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8, 1, $9, $10::timestamptz, $10::timestamptz) returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(language)
            .bind(hook_point)
            .bind(source)
            .bind(priority)
            .bind(description)
            .bind(critical)
            .bind(is_enabled)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert script failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新脚本：version 每次自增（热更新指纹），仅非 None 字段。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        language: Option<&str>,
        hook_point: Option<&str>,
        source: Option<&str>,
        priority: Option<i64>,
        description: Option<&str>,
        critical: Option<bool>,
        is_enabled: Option<bool>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        macro_rules! push {
            // $cast: "::int8" / "::timestamptz" / ""
            ($col:expr, $val:expr, $cast:expr) => {{
                sets.push(format!("{} = ${}{}", $col, sets.len() + 1, $cast));
                params.push($val);
            }};
        }

        if let Some(v) = name {
            push!("name", v.to_string(), "");
        }
        if let Some(v) = language {
            push!("language", v.to_string(), "");
        }
        if let Some(v) = hook_point {
            push!("hook_point", v.to_string(), "");
        }
        if let Some(v) = source {
            push!("source", v.to_string(), "");
        }
        if let Some(v) = priority {
            push!("priority", v.to_string(), "::int8");
        }
        if let Some(v) = description {
            push!("description", v.to_string(), "");
        }
        if let Some(v) = critical {
            push!("critical", v.to_string(), "");
        }
        if let Some(v) = is_enabled {
            push!("is_enabled", v.to_string(), "");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");
        sets.push("version = version + 1".to_string());

        let sql = format!(
            "update sys_scripts set {} where id = ${} and deleted_at is null",
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
                context: "update script failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// 硬删除（ids 批量，对齐 Go DELETE /scripts?ids=）。
    pub async fn delete_ids(&self, ids: &[i64]) -> Result<u64, AppError> {
        if ids.is_empty() {
            return Ok(0);
        }
        let placeholders = (1..=ids.len())
            .map(|i| format!("${i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("delete from sys_scripts where id in ({placeholders})");
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for id in ids {
            q = q.bind(id);
        }
        let res = q
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete scripts failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    pub async fn count(&self) -> Result<u64, AppError> {
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(
            "select count(*) from sys_scripts where deleted_at is null",
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "count scripts failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(row.0 as u64)
    }
}
