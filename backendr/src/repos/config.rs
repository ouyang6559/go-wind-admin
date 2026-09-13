// config repo：系统参数管理。key 全局唯一；is_built_in 的行可改不可删（对齐 Go）。

use sqlx::AnyPool;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct ConfigRow {
    pub id: i64,
    pub name: Option<String>,
    pub key: Option<String>,
    pub value: Option<String>,
    /// STRING | BOOL | INT（proto 枚举名）
    pub value_type: Option<String>,
    pub is_built_in: Option<bool>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
}

pub struct ConfigRepo {
    pub db: AnyPool,
}

impl ConfigRepo {
    pub fn new(db: AnyPool) -> Self {
        Self { db }
    }

    /// key 唯一冲突检测（对齐 Go 约束错误 → "config key already exists"）。
    /// 唯一约束是全局的（uidx_sys_configs_key，含软删行），预检查同样不滤 deleted_at。
    async fn key_exists(&self, key: &str, exclude_id: Option<i64>) -> Result<bool, AppError> {
        let (n,): (i64,) = match exclude_id {
            Some(id) => {
                sqlx::query_as::<sqlx::Any, (i64,)>(
                    "select count(*) from sys_configs where key = $1 and id <> $2::int8",
                )
                .bind(key)
                .bind(id)
                .fetch_one(&self.db)
                .await
            }
            None => {
                sqlx::query_as::<sqlx::Any, (i64,)>(
                    "select count(*) from sys_configs where key = $1",
                )
                .bind(key)
                .fetch_one(&self.db)
                .await
            }
        }
        .map_err(|e| AppError::Internal {
            context: "check config key failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(n > 0)
    }

    /// 分页列表（列带 t. 前缀）。
    pub async fn list(
        &self,
        offset: u64,
        limit: u64,
        where_clause: &str,
        params: &[String],
        order_by: &str,
    ) -> Result<(Vec<ConfigRow>, u64), AppError> {
        let base = "from sys_configs t where t.deleted_at is null";
        let total_sql = format!("select count(*) {base}{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "count configs failed".into(),
                source: Some(Box::new(e)),
            })?;

        let order = if order_by.is_empty() { "t.id" } else { order_by };
        let sql = format!(
            "select t.id, t.name, t.key, t.value, t.value_type, t.is_built_in, t.created_by, t.updated_by \
             {base}{where_clause} order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query_as::<sqlx::Any, ConfigTuple>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list configs failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok((rows.into_iter().map(map_row).collect(), total.0 as u64))
    }

    /// 按 ID 查详情。
    pub async fn get(&self, id: i64) -> Result<Option<ConfigRow>, AppError> {
        let sql = "select t.id, t.name, t.key, t.value, t.value_type, t.is_built_in, t.created_by, t.updated_by \
                   from sys_configs t where t.id = $1::int8 and t.deleted_at is null";
        let row = sqlx::query_as::<sqlx::Any, ConfigTuple>(sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get config failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.map(map_row))
    }

    /// 创建（key 唯一，冲突 400）。
    pub async fn create(
        &self,
        name: Option<&str>,
        key: &str,
        value: Option<&str>,
        value_type: &str,
        is_built_in: bool,
        created_by: i64,
    ) -> Result<(), AppError> {
        if self.key_exists(key, None).await? {
            return Err(AppError::Validation("config key already exists".into()));
        }
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query::<sqlx::Any>(
            "insert into sys_configs (name, key, value, value_type, is_built_in, created_by, created_at, updated_at) \
             values ($1, $2, $3, $4, $5, $6::int8, $7::timestamptz, $7::timestamptz)",
        )
        .bind(name)
        .bind(key)
        .bind(value)
        .bind(value_type)
        .bind(is_built_in)
        .bind(created_by)
        .bind(now)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "insert config failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(())
    }

    /// 更新（含 key 本身可改；冲突 400）。返回影响行数。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        key: Option<&str>,
        value: Option<&str>,
        value_type: Option<&str>,
        is_built_in: Option<bool>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        if let Some(k) = key {
            if self.key_exists(k, Some(id)).await? {
                return Err(AppError::Validation("config key already exists".into()));
            }
        }
        let now = chrono::Utc::now().to_rfc3339();
        let sql = "update sys_configs set \
                   name = coalesce($1, name), key = coalesce($2, key), value = coalesce($3, value), \
                   value_type = coalesce($4, value_type), is_built_in = coalesce($5, is_built_in), \
                   updated_by = $6::int8, updated_at = $7::timestamptz \
                   where id = $8::int8 and deleted_at is null";
        let res = sqlx::query::<sqlx::Any>(sql)
            .bind(name)
            .bind(key)
            .bind(value)
            .bind(value_type)
            .bind(is_built_in)
            .bind(updated_by)
            .bind(now)
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "update config failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// 删除：内置参数（is_built_in=true）禁删；不存在幂等成功（对齐 Go）。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let row = self.get(id).await?;
        match row {
            None => Ok(0), // 幂等删除：目标不存在视为已删除
            Some(c) if c.is_built_in.unwrap_or(false) => {
                Err(AppError::Validation("built-in config cannot be deleted".into()))
            }
            Some(_) => {
                let now = chrono::Utc::now().to_rfc3339();
                let res = sqlx::query::<sqlx::Any>(
                    "update sys_configs set deleted_at = $1::timestamptz where id = $2::int8 and deleted_at is null",
                )
                .bind(now)
                .bind(id)
                .execute(&self.db)
                .await
                .map_err(|e| AppError::Internal {
                    context: "delete config failed".into(),
                    source: Some(Box::new(e)),
                })?;
                Ok(res.rows_affected())
            }
        }
    }
}

type ConfigTuple = (
    i64,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<bool>,
    Option<i64>,
    Option<i64>,
);

fn map_row(r: ConfigTuple) -> ConfigRow {
    ConfigRow {
        id: r.0,
        name: r.1,
        key: r.2,
        value: r.3,
        value_type: r.4,
        is_built_in: r.5,
        created_by: r.6,
        updated_by: r.7,
    }
}
