// menu repo：sys_menus CRUD SQL（对齐 Go menu_repo.go）。
// meta 为 JSON 列：SELECT 用 ::text 转出后解析，写入用 $N::jsonb（对齐 org_unit jsonb 约定）。
// 枚举按 proto 名存储：type 存 CATALOG/MENU/BUTTON/EMBEDDED/LINK，status 存 ON/OFF，
// module 存 DASHBOARD/OPM/...（输出时 handler 映射回 proto 枚举名 MODULE_*）。
// 列表/查询过滤 deleted_at is null；Delete 硬删除并递归收集全部后代（对齐 QueryAllChildrenIds）；
// Truncate 清空全表（SyncMenus 前置，对齐 Go）。

use serde_json::Value;
use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct MenuRow {
    pub id: i64,
    /// proto 枚举名（CATALOG/MENU/BUTTON/EMBEDDED/LINK）
    pub r#type: Option<String>,
    pub path: Option<String>,
    pub redirect: Option<String>,
    pub alias: Option<String>,
    pub name: Option<String>,
    pub component: Option<String>,
    /// JSON 对象（键 camelCase，对齐 protojson），DB 中为 json/jsonb
    pub meta: Option<Value>,
    /// DB 值（DASHBOARD/OPM/...），handler 输出时映射 proto 枚举名 MODULE_*
    pub module: Option<String>,
    pub parent_id: Option<i64>,
    /// ON | OFF
    pub status: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

const SELECT_COLS: &str = "m.id, m.type, m.path, m.redirect, m.alias, m.name, m.component, \
                           m.meta::text as meta, m.module, m.parent_id, m.status, \
                           m.created_by, m.updated_by, m.deleted_by, \
                           to_char(m.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(m.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(m.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from sys_menus m";

fn map_row(row: &sqlx::any::AnyRow) -> Result<MenuRow, AppError> {
    Ok(MenuRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "menu row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        r#type: row.try_get::<Option<String>, _>("type").ok().flatten(),
        path: row.try_get::<Option<String>, _>("path").ok().flatten(),
        redirect: row.try_get::<Option<String>, _>("redirect").ok().flatten(),
        alias: row.try_get::<Option<String>, _>("alias").ok().flatten(),
        name: row.try_get::<Option<String>, _>("name").ok().flatten(),
        component: row.try_get::<Option<String>, _>("component").ok().flatten(),
        meta: row
            .try_get::<Option<String>, _>("meta")
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str::<Value>(&s).ok()),
        module: row.try_get::<Option<String>, _>("module").ok().flatten(),
        parent_id: row.try_get::<Option<i64>, _>("parent_id").ok().flatten(),
        status: row.try_get::<Option<String>, _>("status").ok().flatten(),
        created_by: row.try_get::<Option<i64>, _>("created_by").ok().flatten(),
        updated_by: row.try_get::<Option<i64>, _>("updated_by").ok().flatten(),
        deleted_by: row.try_get::<Option<i64>, _>("deleted_by").ok().flatten(),
        created_at: row.try_get::<Option<String>, _>("created_at").ok().flatten(),
        updated_at: row.try_get::<Option<String>, _>("updated_at").ok().flatten(),
        deleted_at: row.try_get::<Option<String>, _>("deleted_at").ok().flatten(),
    })
}

pub struct MenuRepo {
    pub db: AnyPool,
}

impl MenuRepo {
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
    ) -> Result<(Vec<MenuRow>, u64), AppError> {
        let total_sql = format!("select count(*) from sys_menus m where m.deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "count menus failed".into(),
            source: Some(Box::new(e)),
        })?;

        let order = if order_by.is_empty() { "m.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where m.deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "list menus failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<MenuRow>, AppError> {
        let sql = format!("select {SELECT_COLS} {FROM} where m.id = $1 and m.deleted_at is null limit 1");
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get menu failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// 创建菜单，返回数据库自增 ID。
    /// type/status 未提供时由调用方传入默认值（对齐 ent schema Default）；
    /// parent_id 传 None 表示挂根节点（对齐 Go parent_id=0 按无父级处理）。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        r#type: Option<&str>,
        path: Option<&str>,
        redirect: Option<&str>,
        alias: Option<&str>,
        name: Option<&str>,
        component: Option<&str>,
        meta: Option<&Value>,
        module: Option<&str>,
        parent_id: Option<i64>,
        status: Option<&str>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into sys_menus \
                   (type, path, redirect, alias, name, component, meta, module, parent_id, status, \
                    created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7::jsonb, $8, $9, $10, $11, \
                           $12::timestamptz, $12::timestamptz) \
                   returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let meta_json = meta.map(|m| m.to_string());
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(r#type)
            .bind(path)
            .bind(redirect)
            .bind(alias)
            .bind(name)
            .bind(component)
            .bind(meta_json)
            .bind(module)
            .bind(parent_id)
            .bind(status)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert menu failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新菜单：仅非 None 字段；meta 整体替换。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        r#type: Option<&str>,
        path: Option<&str>,
        redirect: Option<&str>,
        alias: Option<&str>,
        name: Option<&str>,
        component: Option<&str>,
        meta: Option<&Value>,
        module: Option<&str>,
        parent_id: Option<i64>,
        status: Option<&str>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        macro_rules! push {
            // $cast: "::int8" / "::jsonb" / "::timestamptz" / ""（文本列直接绑定）
            ($col:expr, $val:expr, $cast:expr) => {{
                sets.push(format!("{} = ${}{}", $col, sets.len() + 1, $cast));
                params.push($val);
            }};
        }

        if let Some(v) = r#type {
            push!("type", v.to_string(), "");
        }
        if let Some(v) = path {
            push!("path", v.to_string(), "");
        }
        if let Some(v) = redirect {
            push!("redirect", v.to_string(), "");
        }
        if let Some(v) = alias {
            push!("alias", v.to_string(), "");
        }
        if let Some(v) = name {
            push!("name", v.to_string(), "");
        }
        if let Some(v) = component {
            push!("component", v.to_string(), "");
        }
        if let Some(v) = meta {
            push!("meta", v.to_string(), "::jsonb");
        }
        if let Some(v) = module {
            push!("module", v.to_string(), "");
        }
        if let Some(v) = parent_id {
            push!("parent_id", v.to_string(), "::int8");
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
            "update sys_menus set {} where id = ${} and deleted_at is null",
            sets.join(", "),
            params.len() + 1
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        q = q.bind(id);
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "update menu failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 硬删除：递归收集该节点及全部后代后批量删除（对齐 Go QueryAllChildrenIds）。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let mut ids: Vec<i64> = vec![id];
        let mut stack: Vec<i64> = vec![id];
        while let Some(pid) = stack.pop() {
            let sql = "select id from sys_menus where parent_id = $1 and deleted_at is null";
            let rows: Vec<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
                .bind(pid)
                .fetch_all(&self.db)
                .await
                .map_err(|e| AppError::Internal {
                    context: "query menu children failed".into(),
                    source: Some(Box::new(e)),
                })?;
            for (cid,) in rows {
                ids.push(cid);
                stack.push(cid);
            }
        }
        let placeholders = (1..=ids.len())
            .map(|i| format!("${i}"))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("delete from sys_menus where id in ({placeholders})");
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for i in &ids {
            q = q.bind(i);
        }
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "delete menus failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 清空菜单表数据（SyncMenus 前置，对齐 Go Truncate）。
    pub async fn truncate(&self) -> Result<(), AppError> {
        sqlx::query::<sqlx::Any>("delete from sys_menus")
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "truncate menus failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(())
    }
}
