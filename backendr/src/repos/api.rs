// api repo：sys_apis CRUD SQL（对齐 Go api_repo.go）。
// 枚举按 proto 名存储：scope 存 ADMIN/APP，business_module 存 DASHBOARD/OPM/.../TASK
// （输出时 handler 映射回 proto 枚举名 MODULE_*），status 存 ON/OFF。
// 列表/查询过滤 deleted_at is null，Delete 为硬删除（Go 端 TimeAt mixin 无软删拦截器）。

use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct ApiRow {
    pub id: i64,
    pub operation: Option<String>,
    pub path: Option<String>,
    pub method: Option<String>,
    pub module: Option<String>,
    pub module_description: Option<String>,
    /// DB 值（DASHBOARD/OPM/...），handler 输出时映射 proto 枚举名
    pub business_module: Option<String>,
    pub description: Option<String>,
    /// ADMIN | APP
    pub scope: Option<String>,
    /// ON | OFF
    pub status: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

const SELECT_COLS: &str = "a.id, a.operation, a.path, a.method, a.module, a.module_description, \
                           a.business_module, a.description, a.scope, a.status, \
                           a.created_by, a.updated_by, a.deleted_by, \
                           to_char(a.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(a.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(a.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from sys_apis a";

fn map_row(row: &sqlx::any::AnyRow) -> Result<ApiRow, AppError> {
    Ok(ApiRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "api row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        operation: row.try_get::<Option<String>, _>("operation").ok().flatten(),
        path: row.try_get::<Option<String>, _>("path").ok().flatten(),
        method: row.try_get::<Option<String>, _>("method").ok().flatten(),
        module: row.try_get::<Option<String>, _>("module").ok().flatten(),
        module_description: row
            .try_get::<Option<String>, _>("module_description")
            .ok()
            .flatten(),
        business_module: row
            .try_get::<Option<String>, _>("business_module")
            .ok()
            .flatten(),
        description: row.try_get::<Option<String>, _>("description").ok().flatten(),
        scope: row.try_get::<Option<String>, _>("scope").ok().flatten(),
        status: row.try_get::<Option<String>, _>("status").ok().flatten(),
        created_by: row.try_get::<Option<i64>, _>("created_by").ok().flatten(),
        updated_by: row.try_get::<Option<i64>, _>("updated_by").ok().flatten(),
        deleted_by: row.try_get::<Option<i64>, _>("deleted_by").ok().flatten(),
        created_at: row.try_get::<Option<String>, _>("created_at").ok().flatten(),
        updated_at: row.try_get::<Option<String>, _>("updated_at").ok().flatten(),
        deleted_at: row.try_get::<Option<String>, _>("deleted_at").ok().flatten(),
    })
}

pub struct ApiRepo {
    pub db: AnyPool,
}

impl ApiRepo {
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
    ) -> Result<(Vec<ApiRow>, u64), AppError> {
        let total_sql = format!("select count(*) from sys_apis a where a.deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "count apis failed".into(),
            source: Some(Box::new(e)),
        })?;

        let order = if order_by.is_empty() { "a.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where a.deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "list apis failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<ApiRow>, AppError> {
        let sql = format!("select {SELECT_COLS} {FROM} where a.id = $1 and a.deleted_at is null limit 1");
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get api failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// 端点唯一性检查（对齐 sys_apis 唯一索引 module+path+method+scope，
    /// Rust 端以 path+method 全局近似）。
    pub async fn endpoint_exists(&self, path: &str, method: &str, exclude_id: i64) -> Result<bool, AppError> {
        let sql = "select 1 from sys_apis where path = $1 and method = $2 and id <> $3 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(path)
            .bind(method)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check api endpoint failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 创建 API 资源。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        operation: Option<&str>,
        path: Option<&str>,
        method: Option<&str>,
        module: Option<&str>,
        module_description: Option<&str>,
        business_module: Option<&str>,
        description: Option<&str>,
        scope: Option<&str>,
        status: Option<&str>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into sys_apis \
                   (operation, path, method, module, module_description, business_module, \
                    description, scope, status, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11::timestamptz, $11::timestamptz) \
                   returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(operation)
            .bind(path)
            .bind(method)
            .bind(module)
            .bind(module_description)
            .bind(business_module)
            .bind(description)
            .bind(scope)
            .bind(status)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert api failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新 API 资源：仅非 None 字段。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        operation: Option<&str>,
        path: Option<&str>,
        method: Option<&str>,
        module: Option<&str>,
        module_description: Option<&str>,
        business_module: Option<&str>,
        description: Option<&str>,
        scope: Option<&str>,
        status: Option<&str>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        macro_rules! push {
            // $cast: "::int8" / ""（文本列直接绑定）
            ($col:expr, $val:expr, $cast:expr) => {{
                sets.push(format!("{} = ${}{}", $col, sets.len() + 1, $cast));
                params.push($val);
            }};
        }

        if let Some(v) = operation {
            push!("operation", v.to_string(), "");
        }
        if let Some(v) = path {
            push!("path", v.to_string(), "");
        }
        if let Some(v) = method {
            push!("method", v.to_string(), "");
        }
        if let Some(v) = module {
            push!("module", v.to_string(), "");
        }
        if let Some(v) = module_description {
            push!("module_description", v.to_string(), "");
        }
        if let Some(v) = business_module {
            push!("business_module", v.to_string(), "");
        }
        if let Some(v) = description {
            push!("description", v.to_string(), "");
        }
        if let Some(v) = scope {
            push!("scope", v.to_string(), "");
        }
        if let Some(v) = status {
            push!("status", v.to_string(), "");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let sql = format!(
            "update sys_apis set {} where id = ${} and deleted_at is null",
            sets.join(", "),
            params.len() + 1
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        q = q.bind(id);
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "update api failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 硬删除。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let sql = "delete from sys_apis where id = $1 and deleted_at is null";
        let res = sqlx::query::<sqlx::Any>(sql)
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete api failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// 清空全部 API 资源（对齐 Go `repo.Truncate`，用于 SyncApis 全量重建）。
    pub async fn truncate(&self) -> Result<(), AppError> {
        let sql = "delete from sys_apis";
        sqlx::query::<sqlx::Any>(sql)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "truncate apis failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(())
    }
}
