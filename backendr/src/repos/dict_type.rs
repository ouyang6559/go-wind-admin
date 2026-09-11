// dict_type repo：sys_dict_types CRUD SQL（对齐 Go dict_type_repo.go）。
// 硬删除（Go 端 TimeAt mixin 无软删拦截器）；列表/查询过滤 deleted_at is null；
// Delete 为批量删除（前端 DELETE /dict/types?ids=...），0 行命中不报错（对齐 Go BatchDelete）；
// type_code 在 (tenant_id, type_code) 维度唯一，Rust 端以全局近似（对齐 position.rs code_exists）。

use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct DictTypeRow {
    pub id: i64,
    pub type_code: Option<String>,
    pub type_name: Option<String>,
    pub is_enabled: Option<bool>,
    pub sort_order: Option<i64>,
    pub tenant_id: Option<i64>,
    pub tenant_name: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

/// SELECT 列（t.* + 租户显示名）
const SELECT_COLS: &str = "t.id, t.type_code, t.type_name, t.is_enabled, t.sort_order, \
                           t.tenant_id, tn.name as tenant_name, \
                           t.created_by, t.updated_by, t.deleted_by, \
                           to_char(t.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(t.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(t.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from sys_dict_types t \
                    left join sys_tenants tn on tn.id = t.tenant_id and tn.deleted_at is null";

fn map_row(row: &sqlx::any::AnyRow) -> Result<DictTypeRow, AppError> {
    Ok(DictTypeRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "dict type row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        type_code: row.try_get::<Option<String>, _>("type_code").ok().flatten(),
        type_name: row.try_get::<Option<String>, _>("type_name").ok().flatten(),
        is_enabled: row.try_get::<Option<bool>, _>("is_enabled").ok().flatten(),
        sort_order: row.try_get::<Option<i64>, _>("sort_order").ok().flatten(),
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

pub struct DictTypeRepo {
    pub db: AnyPool,
}

impl DictTypeRepo {
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
    ) -> Result<(Vec<DictTypeRow>, u64), AppError> {
        let total_sql = format!("select count(*) from sys_dict_types t where t.deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "count dict types failed".into(),
            source: Some(Box::new(e)),
        })?;

        let order = if order_by.is_empty() { "t.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where t.deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "list dict types failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<DictTypeRow>, AppError> {
        let sql = format!("select {SELECT_COLS} {FROM} where t.id = $1 and t.deleted_at is null limit 1");
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get dict type failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    pub async fn get_by_code(&self, code: &str) -> Result<Option<DictTypeRow>, AppError> {
        let sql = format!("select {SELECT_COLS} {FROM} where t.type_code = $1 and t.deleted_at is null limit 1");
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(code)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get dict type by code failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// type_code 唯一性检查（exclude_id 排除自身）
    pub async fn type_code_exists(&self, code: &str, exclude_id: i64) -> Result<bool, AppError> {
        let sql = "select 1 from sys_dict_types where type_code = $1 and id <> $2 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(code)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check dict type code failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 创建字典类型（对齐 Go Create：SetNillableTenantID/TypeCode/TypeName/SortOrder/IsEnabled）。
    pub async fn create(
        &self,
        type_code: &str,
        type_name: &str,
        is_enabled: Option<bool>,
        sort_order: Option<i64>,
        tenant_id: Option<i64>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into sys_dict_types \
                   (type_code, type_name, is_enabled, sort_order, tenant_id, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7::timestamptz, $7::timestamptz) \
                   returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(type_code)
            .bind(type_name)
            .bind(is_enabled)
            .bind(sort_order)
            .bind(tenant_id)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert dict type failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新字典类型：仅非 None 字段。type_code 为 Immutable 不可更新（对齐 Go，忽略 data.typeCode）。
    pub async fn update(
        &self,
        id: i64,
        type_name: Option<&str>,
        is_enabled: Option<bool>,
        sort_order: Option<i64>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        macro_rules! push {
            // $cast: "::int8" / "::bool" / ""
            ($col:expr, $val:expr, $cast:expr) => {{
                sets.push(format!("{} = ${}{}", $col, sets.len() + 1, $cast));
                params.push($val);
            }};
        }

        if let Some(v) = type_name {
            push!("type_name", v.to_string(), "");
        }
        if let Some(v) = is_enabled {
            push!("is_enabled", v.to_string(), "::bool");
        }
        if let Some(v) = sort_order {
            push!("sort_order", v.to_string(), "::int8");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let sql = format!(
            "update sys_dict_types set {} where id = ${} and deleted_at is null",
            sets.join(", "),
            params.len() + 1
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        q = q.bind(id);
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "update dict type failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 批量硬删除（对齐 Go BatchDelete：空 ids 由 handler 拦截；0 行命中不报错）。
    pub async fn batch_delete(&self, ids: &[i64]) -> Result<u64, AppError> {
        if ids.is_empty() {
            return Ok(0);
        }
        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${i}")).collect();
        let sql = format!("delete from sys_dict_types where id in ({})", placeholders.join(", "));
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for id in ids {
            q = q.bind(id);
        }
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "batch delete dict types failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }
}
