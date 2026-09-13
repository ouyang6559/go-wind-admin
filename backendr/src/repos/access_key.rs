// access_key repo：AK/SK 凭证管理（SK 只存 SHA-256 hex 摘要，明文仅创建/重置时返回一次）。

use sqlx::AnyPool;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct AccessKeyRow {
    pub id: i64,
    pub name: Option<String>,
    pub access_key: String,
    /// ON | OFF
    pub status: String,
    pub expires_at: Option<String>,
    pub last_used_at: Option<String>,
    pub tenant_id: i64,
    pub secret_hash: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
}

pub struct AccessKeyRepo {
    pub db: AnyPool,
}

impl AccessKeyRepo {
    pub fn new(db: AnyPool) -> Self {
        Self { db }
    }

    /// 分页列表（query 过滤走 handler 编译的 where 片段，列带 t. 前缀）。
    pub async fn list(
        &self,
        offset: u64,
        limit: u64,
        where_clause: &str,
        params: &[String],
        order_by: &str,
    ) -> Result<(Vec<AccessKeyRow>, u64), AppError> {
        let base = "from sys_access_keys t where t.deleted_at is null";
        let total_sql = format!("select count(*) {base}{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "count access keys failed".into(),
                source: Some(Box::new(e)),
            })?;

        let order = if order_by.is_empty() { "t.id desc" } else { order_by };
        let sql = format!(
            "select t.id, t.name, t.access_key, t.status, \
             to_char(t.expires_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), \
             to_char(t.last_used_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), \
             t.tenant_id, t.secret_hash, t.created_by, t.updated_by \
             {base}{where_clause} order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query_as::<sqlx::Any, AccessKeyTuple>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list access keys failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok((rows.into_iter().map(map_row).collect(), total.0 as u64))
    }

    /// 按 ID 查详情。
    pub async fn get(&self, id: i64) -> Result<Option<AccessKeyRow>, AppError> {
        let sql = "select t.id, t.name, t.access_key, t.status, \
                    to_char(t.expires_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), \
                    to_char(t.last_used_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), \
                    t.tenant_id, t.secret_hash, t.created_by, t.updated_by \
                    from sys_access_keys t where t.id = $1::int8 and t.deleted_at is null";
        let row = sqlx::query_as::<sqlx::Any, AccessKeyTuple>(sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get access key failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.map(map_row))
    }

    /// 按 AK 查（令牌交换用；系统视角不滤租户）。
    pub async fn get_by_access_key(&self, ak: &str) -> Result<Option<AccessKeyRow>, AppError> {
        let sql = "select t.id, t.name, t.access_key, t.status, \
                    to_char(t.expires_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), \
                    to_char(t.last_used_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), \
                    t.tenant_id, t.secret_hash, t.created_by, t.updated_by \
                    from sys_access_keys t where t.access_key = $1 and t.deleted_at is null";
        let row = sqlx::query_as::<sqlx::Any, AccessKeyTuple>(sql)
            .bind(ak)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get access key by ak failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.map(map_row))
    }

    /// 创建：AK/secret_hash 由服务层生成后落库。返回行 ID。
    pub async fn create(
        &self,
        name: Option<&str>,
        access_key: &str,
        secret_hash: &str,
        status: &str,
        expires_at: Option<&str>,
        tenant_id: i64,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        let sql = "insert into sys_access_keys \
                   (name, access_key, secret_hash, status, expires_at, tenant_id, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5::timestamptz, $6::int8, $7::int8, $8::timestamptz, $8::timestamptz) \
                   returning id";
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(access_key)
            .bind(secret_hash)
            .bind(status)
            .bind(expires_at)
            .bind(tenant_id)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert access key failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新（仅名称/状态/过期时间；AK 与 secret 摘要不可变）。返回影响行数。
    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        status: Option<&str>,
        expires_at: Option<Option<&str>>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        // COALESCE 语义：None = 不改；Some(None) = 清空（对齐 FieldMask 可选路径行为）
        let sql = "update sys_access_keys set \
                   name = coalesce($1, name), \
                   status = coalesce($2, status), \
                   expires_at = case when $3::bool then $4::timestamptz else expires_at end, \
                   updated_by = $5::int8, updated_at = $6::timestamptz \
                   where id = $7::int8 and deleted_at is null";
        let res = sqlx::query::<sqlx::Any>(sql)
            .bind(name)
            .bind(status)
            .bind(expires_at.is_some())
            .bind(expires_at.flatten())
            .bind(updated_by)
            .bind(now)
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "update access key failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// 软删除。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        let res = sqlx::query::<sqlx::Any>(
            "update sys_access_keys set deleted_at = $1::timestamptz where id = $2::int8 and deleted_at is null",
        )
        .bind(now)
        .bind(id)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "delete access key failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 轮换密钥摘要（ResetSecret）。
    pub async fn update_secret_hash(&self, id: i64, secret_hash: &str) -> Result<u64, AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        let res = sqlx::query::<sqlx::Any>(
            "update sys_access_keys set secret_hash = $1, updated_at = $2::timestamptz \
             where id = $3::int8 and deleted_at is null",
        )
        .bind(secret_hash)
        .bind(now)
        .bind(id)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "update access key secret failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 刷新最近使用时间（令牌交换成功后尽力而为）。
    pub async fn touch_last_used(&self, id: i64) {
        let now = chrono::Utc::now().to_rfc3339();
        let _ = sqlx::query::<sqlx::Any>(
            "update sys_access_keys set last_used_at = $1::timestamptz where id = $2::int8",
        )
        .bind(now)
        .bind(id)
        .execute(&self.db)
        .await;
    }
}

type AccessKeyTuple = (
    i64,
    Option<String>,
    String,
    String,
    Option<String>,
    Option<String>,
    i64,
    Option<String>,
    Option<i64>,
    Option<i64>,
);

fn map_row(r: AccessKeyTuple) -> AccessKeyRow {
    AccessKeyRow {
        id: r.0,
        name: r.1,
        access_key: r.2,
        status: r.3,
        expires_at: r.4,
        last_used_at: r.5,
        tenant_id: r.6,
        secret_hash: r.7,
        created_by: r.8,
        updated_by: r.9,
    }
}
