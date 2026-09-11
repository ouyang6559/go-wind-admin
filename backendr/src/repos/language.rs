// language repo：sys_languages CRUD SQL（对齐 Go language_repo.go）。
// 硬删除（Go 端 TimeAt mixin 无软删拦截器）；列表/查询过滤 deleted_at is null；
// Delete 为单条删除（前端 DELETE /dict/langs?id=...），0 行命中不报错（对齐 Go repository.Delete）；
// language_code 全局唯一（uix_sys_languages_language_code），Create 时先查重返回 Conflict；
// Update 不更新 language_code（Go 端 Immutable，SetLanguageCode 被注释掉）。

use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct LanguageRow {
    pub id: i64,
    pub language_code: Option<String>,
    pub language_name: Option<String>,
    pub native_name: Option<String>,
    pub is_default: Option<bool>,
    pub is_enabled: Option<bool>,
    pub sort_order: Option<i64>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

/// SELECT 列（l.*）
const SELECT_COLS: &str = "l.id, l.language_code, l.language_name, l.native_name, \
                           l.is_default, l.is_enabled, l.sort_order, \
                           l.created_by, l.updated_by, l.deleted_by, \
                           to_char(l.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(l.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(l.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from sys_languages l";

fn map_row(row: &sqlx::any::AnyRow) -> Result<LanguageRow, AppError> {
    Ok(LanguageRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "language row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        language_code: row.try_get::<Option<String>, _>("language_code").ok().flatten(),
        language_name: row.try_get::<Option<String>, _>("language_name").ok().flatten(),
        native_name: row.try_get::<Option<String>, _>("native_name").ok().flatten(),
        is_default: row.try_get::<Option<bool>, _>("is_default").ok().flatten(),
        is_enabled: row.try_get::<Option<bool>, _>("is_enabled").ok().flatten(),
        sort_order: row.try_get::<Option<i64>, _>("sort_order").ok().flatten(),
        created_by: row.try_get::<Option<i64>, _>("created_by").ok().flatten(),
        updated_by: row.try_get::<Option<i64>, _>("updated_by").ok().flatten(),
        deleted_by: row.try_get::<Option<i64>, _>("deleted_by").ok().flatten(),
        created_at: row.try_get::<Option<String>, _>("created_at").ok().flatten(),
        updated_at: row.try_get::<Option<String>, _>("updated_at").ok().flatten(),
        deleted_at: row.try_get::<Option<String>, _>("deleted_at").ok().flatten(),
    })
}

pub struct LanguageRepo {
    pub db: AnyPool,
}

impl LanguageRepo {
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
    ) -> Result<(Vec<LanguageRow>, u64), AppError> {
        let total_sql = format!("select count(*) from sys_languages l where l.deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "count languages failed".into(),
            source: Some(Box::new(e)),
        })?;

        let order = if order_by.is_empty() { "l.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where l.deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "list languages failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<LanguageRow>, AppError> {
        let sql = format!("select {SELECT_COLS} {FROM} where l.id = $1 and l.deleted_at is null limit 1");
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get language failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    pub async fn get_by_code(&self, code: &str) -> Result<Option<LanguageRow>, AppError> {
        let sql = format!("select {SELECT_COLS} {FROM} where l.language_code = $1 and l.deleted_at is null limit 1");
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(code)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get language by code failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// language_code 唯一性检查（exclude_id 排除自身）
    pub async fn language_code_exists(&self, code: &str, exclude_id: i64) -> Result<bool, AppError> {
        let sql = "select 1 from sys_languages where language_code = $1 and id <> $2 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(code)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check language code failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 创建语言（对齐 Go Create）。
    pub async fn create(
        &self,
        language_code: &str,
        language_name: &str,
        native_name: &str,
        is_default: Option<bool>,
        is_enabled: Option<bool>,
        sort_order: Option<i64>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into sys_languages \
                   (language_code, language_name, native_name, is_default, is_enabled, sort_order, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8::timestamptz, $8::timestamptz) \
                   returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(language_code)
            .bind(language_name)
            .bind(native_name)
            .bind(is_default)
            .bind(is_enabled)
            .bind(sort_order)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert language failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新语言：仅非 None 字段；language_code 为 Immutable 不更新（对齐 Go）。
    pub async fn update(
        &self,
        id: i64,
        language_name: Option<&str>,
        native_name: Option<&str>,
        is_default: Option<bool>,
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

        if let Some(v) = language_name {
            push!("language_name", v.to_string(), "");
        }
        if let Some(v) = native_name {
            push!("native_name", v.to_string(), "");
        }
        if let Some(v) = is_default {
            push!("is_default", v.to_string(), "::bool");
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
            "update sys_languages set {} where id = ${} and deleted_at is null",
            sets.join(", "),
            params.len() + 1
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        q = q.bind(id);
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "update language failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 硬删除（对齐 Go Delete：0 行命中不报错）。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let sql = "delete from sys_languages where id = $1 and deleted_at is null";
        let res = sqlx::query::<sqlx::Any>(sql)
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete language failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }
}
