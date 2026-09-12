// file repository：files 表 CRUD（对齐 Go data/file_repo.go）。
//  - List：分页/过滤 + LEFT JOIN sys_tenants 回填 tenantName；
//  - Get：按 id；
//  - Create/Update/Delete：硬删除；Update 仅非 None 字段；
//  - provider 存 proto 枚举名（MINIO/...）；size 为 int8 读取后按 protojson uint64 字符串输出。

use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct FileRow {
    pub id: i64,
    pub provider: Option<String>,
    pub bucket_name: Option<String>,
    pub file_directory: Option<String>,
    pub file_guid: Option<String>,
    pub save_file_name: Option<String>,
    pub file_name: Option<String>,
    pub extension: Option<String>,
    /// 原始大小（字节），DTO 层按 protojson uint64 转字符串
    pub size: Option<i64>,
    pub size_format: Option<String>,
    pub link_url: Option<String>,
    pub content_hash: Option<String>,
    pub tenant_id: Option<i64>,
    pub tenant_name: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

const SELECT_COLS: &str = "f.id, f.provider, f.bucket_name, f.file_directory, f.file_guid, \
                           f.save_file_name, f.file_name, f.extension, f.size, f.size_format, \
                           f.link_url, f.content_hash, f.tenant_id, \
                           t.name as tenant_name, f.created_by, f.updated_by, f.deleted_by, \
                           to_char(f.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(f.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(f.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from files f left join sys_tenants t on t.id = f.tenant_id";

fn map_row(row: &sqlx::any::AnyRow) -> Result<FileRow, AppError> {
    let id: i64 = row.try_get("id").map_err(|e| AppError::Internal {
        context: "file row id decode failed".into(),
        source: Some(Box::new(e)),
    })?;
    Ok(FileRow {
        id,
        provider: row.try_get("provider").ok().flatten(),
        bucket_name: row.try_get("bucket_name").ok().flatten(),
        file_directory: row.try_get("file_directory").ok().flatten(),
        file_guid: row.try_get("file_guid").ok().flatten(),
        save_file_name: row.try_get("save_file_name").ok().flatten(),
        file_name: row.try_get("file_name").ok().flatten(),
        extension: row.try_get("extension").ok().flatten(),
        size: row.try_get("size").ok().flatten(),
        size_format: row.try_get("size_format").ok().flatten(),
        link_url: row.try_get("link_url").ok().flatten(),
        content_hash: row.try_get("content_hash").ok().flatten(),
        tenant_id: row.try_get("tenant_id").ok().flatten(),
        tenant_name: row.try_get("tenant_name").ok().flatten(),
        created_by: row.try_get("created_by").ok().flatten(),
        updated_by: row.try_get("updated_by").ok().flatten(),
        deleted_by: row.try_get("deleted_by").ok().flatten(),
        created_at: row.try_get("created_at").ok().flatten(),
        updated_at: row.try_get("updated_at").ok().flatten(),
        deleted_at: row.try_get("deleted_at").ok().flatten(),
    })
}

pub struct FileRepo {
    pub db: AnyPool,
}

impl FileRepo {
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
    ) -> Result<(Vec<FileRow>, u64), AppError> {
        let total_sql =
            format!("select count(*) from files f where f.deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "count files failed".into(),
                source: Some(Box::new(e)),
            })?;

        let order = if order_by.is_empty() { "f.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where f.deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list files failed".into(),
                source: Some(Box::new(e)),
            })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<FileRow>, AppError> {
        let sql = format!("select {SELECT_COLS} {FROM} where f.id = $1 and f.deleted_at is null limit 1");
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get file failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        provider: &str,
        bucket_name: Option<&str>,
        file_directory: Option<&str>,
        file_guid: Option<&str>,
        save_file_name: Option<&str>,
        file_name: Option<&str>,
        extension: Option<&str>,
        size: Option<i64>,
        size_format: Option<&str>,
        link_url: Option<&str>,
        content_hash: Option<&str>,
        tenant_id: Option<i64>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        let sql = "insert into files \
                   (provider, bucket_name, file_directory, file_guid, save_file_name, file_name, \
                    extension, size, size_format, link_url, content_hash, tenant_id, created_by, \
                    created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8::int8, $9, $10, $11, $12::int8, \
                           $13::int8, $14::timestamptz, $14::timestamptz) \
                   returning id";
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(provider)
            .bind(bucket_name)
            .bind(file_directory)
            .bind(file_guid)
            .bind(save_file_name)
            .bind(file_name)
            .bind(extension)
            .bind(size)
            .bind(size_format)
            .bind(link_url)
            .bind(content_hash)
            .bind(tenant_id)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert file failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        provider: Option<&str>,
        bucket_name: Option<&str>,
        file_directory: Option<&str>,
        file_guid: Option<&str>,
        save_file_name: Option<&str>,
        file_name: Option<&str>,
        extension: Option<&str>,
        size: Option<i64>,
        size_format: Option<&str>,
        link_url: Option<&str>,
        content_hash: Option<&str>,
        tenant_id: Option<i64>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        macro_rules! push {
            ($col:expr, $val:expr, $cast:expr) => {{
                sets.push(format!("{} = ${}{}", $col, sets.len() + 1, $cast));
                params.push($val);
            }};
        }

        if let Some(v) = provider {
            push!("provider", v.to_string(), "");
        }
        if let Some(v) = bucket_name {
            push!("bucket_name", v.to_string(), "");
        }
        if let Some(v) = file_directory {
            push!("file_directory", v.to_string(), "");
        }
        if let Some(v) = file_guid {
            push!("file_guid", v.to_string(), "");
        }
        if let Some(v) = save_file_name {
            push!("save_file_name", v.to_string(), "");
        }
        if let Some(v) = file_name {
            push!("file_name", v.to_string(), "");
        }
        if let Some(v) = extension {
            push!("extension", v.to_string(), "");
        }
        if let Some(v) = size {
            push!("size", v.to_string(), "::int8");
        }
        if let Some(v) = size_format {
            push!("size_format", v.to_string(), "");
        }
        if let Some(v) = link_url {
            push!("link_url", v.to_string(), "");
        }
        if let Some(v) = content_hash {
            push!("content_hash", v.to_string(), "");
        }
        if let Some(v) = tenant_id {
            push!("tenant_id", v.to_string(), "::int8");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let sql = format!(
            "update files set {} where id = ${} and deleted_at is null",
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
                context: "update file failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// 硬删除（对齐 Go FileRepo.Delete）。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let res = sqlx::query::<sqlx::Any>("delete from files where id = $1")
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete file failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }
}