// dict_entry repo：sys_dict_entries + sys_dict_entry_i18n CRUD SQL（对齐 Go dict_entry_repo.go / dict_entry_i18n_repo.go）。
// 硬删除（Go 端 TimeAt mixin 无软删拦截器）；列表/查询过滤 deleted_at is null；
// Delete 为批量删除（前端 DELETE /dict/entries?ids=...），0 行命中不报错（对齐 Go BatchDelete）；
// i18n 替换语义：ReplaceByEntryID = 先删后插；更新仅在 updateMask 含 i18n 时替换（handler 侧判定）。
// 说明：Go 端 Create/Update 在事务内完成 entry + i18n，Rust 端顺序执行（position.rs 同款简化）。

use sqlx::AnyPool;
use sqlx::Row;
use std::collections::HashMap;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct DictEntryRow {
    pub id: i64,
    pub type_id: Option<i64>,
    pub entry_value: Option<String>,
    pub numeric_value: Option<i64>,
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

#[derive(Debug, Clone)]
pub struct DictEntryI18nRow {
    pub id: i64,
    pub entry_id: Option<i64>,
    pub language_code: Option<String>,
    pub entry_label: Option<String>,
    pub description: Option<String>,
    pub sort_order: Option<i64>,
    pub tenant_id: Option<i64>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

/// i18n 写入入参（handler 从请求 data.i18n 转换而来）
#[derive(Debug, Clone)]
pub struct I18nInput {
    pub entry_label: Option<String>,
    pub description: Option<String>,
}

/// SELECT 列（e.* + 租户显示名）
const SELECT_COLS: &str = "e.id, e.type_id, e.entry_value, e.numeric_value, e.is_enabled, e.sort_order, \
                           e.tenant_id, tn.name as tenant_name, \
                           e.created_by, e.updated_by, e.deleted_by, \
                           to_char(e.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(e.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(e.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from sys_dict_entries e \
                    left join sys_tenants tn on tn.id = e.tenant_id and tn.deleted_at is null";

const I18N_COLS: &str = "i.id, i.entry_id, i.language_code, i.entry_label, i.description, i.sort_order, \
                         i.tenant_id, i.created_by, i.updated_by, \
                         to_char(i.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                         to_char(i.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at";

fn map_row(row: &sqlx::any::AnyRow) -> Result<DictEntryRow, AppError> {
    Ok(DictEntryRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "dict entry row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        type_id: row.try_get::<Option<i64>, _>("type_id").ok().flatten(),
        entry_value: row.try_get::<Option<String>, _>("entry_value").ok().flatten(),
        numeric_value: row.try_get::<Option<i64>, _>("numeric_value").ok().flatten(),
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

fn map_i18n_row(row: &sqlx::any::AnyRow) -> Result<DictEntryI18nRow, AppError> {
    Ok(DictEntryI18nRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "dict entry i18n row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        entry_id: row.try_get::<Option<i64>, _>("entry_id").ok().flatten(),
        language_code: row.try_get::<Option<String>, _>("language_code").ok().flatten(),
        entry_label: row.try_get::<Option<String>, _>("entry_label").ok().flatten(),
        description: row.try_get::<Option<String>, _>("description").ok().flatten(),
        sort_order: row.try_get::<Option<i64>, _>("sort_order").ok().flatten(),
        tenant_id: row.try_get::<Option<i64>, _>("tenant_id").ok().flatten(),
        created_by: row.try_get::<Option<i64>, _>("created_by").ok().flatten(),
        updated_by: row.try_get::<Option<i64>, _>("updated_by").ok().flatten(),
        created_at: row.try_get::<Option<String>, _>("created_at").ok().flatten(),
        updated_at: row.try_get::<Option<String>, _>("updated_at").ok().flatten(),
    })
}

pub struct DictEntryRepo {
    pub db: AnyPool,
}

impl DictEntryRepo {
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
    ) -> Result<(Vec<DictEntryRow>, u64), AppError> {
        let total_sql = format!("select count(*) from sys_dict_entries e where e.deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "count dict entries failed".into(),
            source: Some(Box::new(e)),
        })?;

        let order = if order_by.is_empty() { "e.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where e.deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "list dict entries failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<DictEntryRow>, AppError> {
        let sql = format!("select {SELECT_COLS} {FROM} where e.id = $1 and e.deleted_at is null limit 1");
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get dict entry failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// 按字典类型编码查询启用条目（对齐 Go ListByTypeCode：is_enabled=true + type_code 匹配 + sort_order 升序）。
    pub async fn list_by_type_code(&self, type_code: &str) -> Result<Vec<DictEntryRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} {FROM} \
             join sys_dict_types dt on dt.id = e.type_id and dt.deleted_at is null \
             where e.is_enabled = true and dt.type_code = $1 and e.deleted_at is null \
             order by e.sort_order asc, e.id asc"
        );
        let rows = sqlx::query::<sqlx::Any>(&sql)
            .bind(type_code)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list dict entries by type code failed".into(),
                source: Some(Box::new(e)),
            })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok(items)
    }

    /// 创建字典条目（对齐 Go Create：entry_value 必填；type_id 仅在提供时落库）。
    pub async fn create(
        &self,
        entry_value: &str,
        numeric_value: Option<i64>,
        type_id: Option<i64>,
        is_enabled: Option<bool>,
        sort_order: Option<i64>,
        tenant_id: Option<i64>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into sys_dict_entries \
                   (entry_value, numeric_value, type_id, is_enabled, sort_order, tenant_id, created_by, created_at, updated_at) \
                   values ($1, $2::int4, $3, $4, $5, $6, $7, $8::timestamptz, $8::timestamptz) \
                   returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(entry_value)
            .bind(numeric_value)
            .bind(type_id)
            .bind(is_enabled)
            .bind(sort_order)
            .bind(tenant_id)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert dict entry failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新字典条目：仅非 None 字段。
    pub async fn update(
        &self,
        id: i64,
        entry_value: Option<&str>,
        numeric_value: Option<i64>,
        is_enabled: Option<bool>,
        sort_order: Option<i64>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        macro_rules! push {
            // $cast: "::int8" / "::int4" / "::bool" / ""
            ($col:expr, $val:expr, $cast:expr) => {{
                sets.push(format!("{} = ${}{}", $col, sets.len() + 1, $cast));
                params.push($val);
            }};
        }

        if let Some(v) = entry_value {
            push!("entry_value", v.to_string(), "");
        }
        if let Some(v) = numeric_value {
            push!("numeric_value", v.to_string(), "::int4");
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
            "update sys_dict_entries set {} where id = ${} and deleted_at is null",
            sets.join(", "),
            params.len() + 1
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        q = q.bind(id);
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "update dict entry failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 批量硬删除（对齐 Go BatchDelete：空 ids 由 handler 拦截；0 行命中不报错）。
    /// 依赖 DB 层 ON DELETE CASCADE 清理关联 i18n（对齐 Go，其 Delete 同样依赖 cascade）。
    pub async fn batch_delete(&self, ids: &[i64]) -> Result<u64, AppError> {
        if ids.is_empty() {
            return Ok(0);
        }
        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${i}")).collect();
        let sql = format!("delete from sys_dict_entries where id in ({})", placeholders.join(", "));
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for id in ids {
            q = q.bind(id);
        }
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "batch delete dict entries failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    // ==============================
    // i18n（sys_dict_entry_i18n）
    // ==============================

    pub async fn i18n_list_by_entry_id(&self, entry_id: i64) -> Result<Vec<DictEntryI18nRow>, AppError> {
        let sql = format!(
            "select {I18N_COLS} from sys_dict_entry_i18n i \
             where i.entry_id = $1 and i.deleted_at is null"
        );
        let rows = sqlx::query::<sqlx::Any>(&sql)
            .bind(entry_id)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list dict entry i18n failed".into(),
                source: Some(Box::new(e)),
            })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_i18n_row(row)?);
        }
        Ok(items)
    }

    pub async fn i18n_get_by_entry_id_and_lang(
        &self,
        entry_id: i64,
        lang_code: &str,
    ) -> Result<Option<DictEntryI18nRow>, AppError> {
        let sql = format!(
            "select {I18N_COLS} from sys_dict_entry_i18n i \
             where i.entry_id = $1 and i.language_code = $2 and i.deleted_at is null limit 1"
        );
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(entry_id)
            .bind(lang_code)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get dict entry i18n failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_i18n_row).transpose()
    }

    /// 替换某条目的 i18n：先清后插（对齐 Go ReplaceByEntryID，无事务简化版）。
    pub async fn i18n_replace_by_entry_id(
        &self,
        entry_id: i64,
        tenant_id: Option<i64>,
        operator_id: i64,
        items: &HashMap<String, I18nInput>,
    ) -> Result<(), AppError> {
        let del = sqlx::query::<sqlx::Any>("delete from sys_dict_entry_i18n where entry_id = $1")
            .bind(entry_id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "clean dict entry i18n failed".into(),
                source: Some(Box::new(e)),
            })?;
        let _ = del.rows_affected();

        if items.is_empty() {
            return Ok(());
        }

        let now = chrono::Utc::now().to_rfc3339();
        for (lang_code, item) in items {
            let sql = "insert into sys_dict_entry_i18n \
                       (language_code, entry_label, description, tenant_id, created_by, created_at, updated_at) \
                       values ($1, $2, $3, $4, $5, $6::timestamptz, $6::timestamptz)";
            sqlx::query::<sqlx::Any>(sql)
                .bind(lang_code)
                .bind(&item.entry_label)
                .bind(&item.description)
                .bind(tenant_id)
                .bind(operator_id)
                .bind(&now)
                .execute(&self.db)
                .await
                .map_err(|e| AppError::Internal {
                    context: "bulk insert dict entry i18n failed".into(),
                    source: Some(Box::new(e)),
                })?;
        }
        Ok(())
    }
}
