// internal_message_category repo：internal_message_categories CRUD SQL（对齐 Go internal_message_category_repo.go）。
// 平铺结构（无 parent_id），列表/查询过滤 deleted_at is null，Delete 为硬删除。

use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct InternalMessageCategoryRow {
    pub id: i64,
    pub name: Option<String>,
    pub code: Option<String>,
    pub icon_url: Option<String>,
    pub sort_order: Option<i64>,
    pub is_enabled: Option<bool>,
    pub tenant_id: Option<i64>,
    pub tenant_name: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

/// SELECT 列（c.* + 租户显示名）
const SELECT_COLS: &str = "c.id, c.name, c.code, c.icon_url, c.sort_order, c.is_enabled, \
                           c.tenant_id, t.name as tenant_name, \
                           c.created_by, c.updated_by, c.deleted_by, \
                           to_char(c.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(c.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(c.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from internal_message_categories c \
                    left join sys_tenants t on t.id = c.tenant_id and t.deleted_at is null";

fn map_row(row: &sqlx::any::AnyRow) -> Result<InternalMessageCategoryRow, AppError> {
    Ok(InternalMessageCategoryRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "internal message category row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        name: row.try_get::<Option<String>, _>("name").ok().flatten(),
        code: row.try_get::<Option<String>, _>("code").ok().flatten(),
        icon_url: row.try_get::<Option<String>, _>("icon_url").ok().flatten(),
        sort_order: row.try_get::<Option<i64>, _>("sort_order").ok().flatten(),
        is_enabled: row.try_get::<Option<bool>, _>("is_enabled").ok().flatten(),
        tenant_id: row.try_get::<Option<i64>, _>("tenant_id").ok().flatten(),
        tenant_name: row.try_get::<Option<String>, _>("tenant_name").ok().flatten(),
        created_by: row.try_get::<Option<i64>, _>("created_by").ok().flatten(),
        updated_by: row.try_get::<Option<i64>, _>("updated_by").ok().flatten(),
        deleted_by: row.try_get::<Option<i64>, _>("deleted_by").ok().flatten(),
        created_at: row.try_get::<Option<String>, _>("created_at").ok().flatten(),
        updated_at: row.try_get::<Option<String>, _>("updated_at").ok().flatten(),
        deleted_at: row.try_get::<Option<String>, _>("deleted_at").ok().flatten(),
    })
}

pub struct InternalMessageCategoryRepo {
    pub db: AnyPool,
}

impl InternalMessageCategoryRepo {
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
    ) -> Result<(Vec<InternalMessageCategoryRow>, u64), AppError> {
        let total_sql = format!(
            "select count(*) from internal_message_categories c where c.deleted_at is null{where_clause}"
        );
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "count internal message categories failed".into(),
                source: Some(Box::new(e)),
            })?;

        let order = if order_by.is_empty() { "c.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where c.deleted_at is null{where_clause} \
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
                context: "list internal message categories failed".into(),
                source: Some(Box::new(e)),
            })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<InternalMessageCategoryRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} {FROM} where c.id = $1 and c.deleted_at is null limit 1"
        );
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get internal message category failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// 编码唯一性检查（Go 端为租户内唯一，Rust 端以 code 全局近似）。
    pub async fn code_exists(&self, code: &str, exclude_id: i64) -> Result<bool, AppError> {
        let sql = "select 1 from internal_message_categories where code = $1 and id <> $2 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(code)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check internal message category code failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 创建分类。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        name: &str,
        code: &str,
        icon_url: Option<&str>,
        sort_order: Option<i64>,
        is_enabled: Option<bool>,
        tenant_id: Option<i64>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into internal_message_categories \
                   (name, code, icon_url, sort_order, is_enabled, tenant_id, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8::timestamptz, $8::timestamptz) returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(code)
            .bind(icon_url)
            .bind(sort_order)
            .bind(is_enabled)
            .bind(tenant_id.unwrap_or(0))
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert internal message category failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新分类：仅非 None 字段。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        code: Option<&str>,
        icon_url: Option<&str>,
        sort_order: Option<i64>,
        is_enabled: Option<bool>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        macro_rules! push {
            // $cast: "::int8" / "::bool" / "::timestamptz" / ""
            ($col:expr, $val:expr, $cast:expr) => {{
                sets.push(format!("{} = ${}{}", $col, sets.len() + 1, $cast));
                params.push($val);
            }};
        }

        if let Some(v) = name {
            push!("name", v.to_string(), "");
        }
        if let Some(v) = code {
            push!("code", v.to_string(), "");
        }
        if let Some(v) = icon_url {
            push!("icon_url", v.to_string(), "");
        }
        if let Some(v) = sort_order {
            push!("sort_order", v.to_string(), "::int8");
        }
        if let Some(v) = is_enabled {
            push!("is_enabled", v.to_string(), "::bool");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let sql = format!(
            "update internal_message_categories set {} where id = ${} and deleted_at is null",
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
                context: "update internal message category failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// 硬删除。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let sql = "delete from internal_message_categories where id = $1 and deleted_at is null";
        let res = sqlx::query::<sqlx::Any>(sql)
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete internal message category failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }
}
