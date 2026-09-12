// permission repo：sys_permissions CRUD + 关联（分组名/菜单/API）+ sync:perms 支持。
// 对齐 Go permission_repo.go：
//  - List 带 LEFT JOIN sys_permission_groups 回填 groupName；menuIds/apiIds 按权限 ID 批量查询；
//  - Create/Update/Delete 为硬删除；Delete 时清理 join 表孤儿行；
//  - TruncateBizPermissions 仅清理 code 非 `sys:` 前缀的业务权限（保留系统权限已存在由默认播种）。
// status 按 proto 枚举名存储（ON/OFF）。

use std::collections::HashMap;

use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct PermissionRow {
    pub id: i64,
    pub name: Option<String>,
    pub code: Option<String>,
    pub group_id: Option<i64>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
    /// JOIN sys_permission_groups 回填
    pub group_name: Option<String>,
}

const SELECT_COLS: &str = "p.id, p.name, p.code, p.group_id, p.status, p.description, \
                           p.created_by, p.updated_by, p.deleted_by, \
                           to_char(p.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(p.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(p.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at, \
                           pg.name as group_name";

const FROM: &str = "from sys_permissions p \
                    left join sys_permission_groups pg on pg.id = p.group_id";

fn map_row(row: &sqlx::any::AnyRow) -> Result<PermissionRow, AppError> {
    Ok(PermissionRow {
        id: row.try_get::<i64, _>("id").map_err(|e| AppError::Internal {
            context: "permission row id decode failed".into(),
            source: Some(Box::new(e)),
        })?,
        name: row.try_get::<Option<String>, _>("name").ok().flatten(),
        code: row.try_get::<Option<String>, _>("code").ok().flatten(),
        group_id: row.try_get::<Option<i64>, _>("group_id").ok().flatten(),
        status: row.try_get::<Option<String>, _>("status").ok().flatten(),
        description: row.try_get::<Option<String>, _>("description").ok().flatten(),
        created_by: row.try_get::<Option<i64>, _>("created_by").ok().flatten(),
        updated_by: row.try_get::<Option<i64>, _>("updated_by").ok().flatten(),
        deleted_by: row.try_get::<Option<i64>, _>("deleted_by").ok().flatten(),
        created_at: row.try_get::<Option<String>, _>("created_at").ok().flatten(),
        updated_at: row.try_get::<Option<String>, _>("updated_at").ok().flatten(),
        deleted_at: row.try_get::<Option<String>, _>("deleted_at").ok().flatten(),
        group_name: row.try_get::<Option<String>, _>("group_name").ok().flatten(),
    })
}

pub struct PermissionRepo {
    pub db: AnyPool,
}

impl PermissionRepo {
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
    ) -> Result<(Vec<PermissionRow>, u64), AppError> {
        let total_sql =
            format!("select count(*) from sys_permissions p where p.deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "count permissions failed".into(),
                source: Some(Box::new(e)),
            })?;

        let order = if order_by.is_empty() { "p.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where p.deleted_at is null{where_clause} \
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
                context: "list permissions failed".into(),
                source: Some(Box::new(e)),
            })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<PermissionRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} {FROM} where p.id = $1 and p.deleted_at is null limit 1"
        );
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get permission failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    pub async fn get_by_code(&self, code: &str) -> Result<Option<PermissionRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} {FROM} where p.code = $1 and p.deleted_at is null limit 1"
        );
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(code)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get permission by code failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    pub async fn exists_code(&self, code: &str, exclude_id: i64) -> Result<bool, AppError> {
        let sql = if exclude_id > 0 {
            "select 1 from sys_permissions where code = $1 and id <> $2 and deleted_at is null limit 1"
        } else {
            "select 1 from sys_permissions where code = $1 and deleted_at is null limit 1"
        };
        let mut q = sqlx::query::<sqlx::Any>(sql).bind(code);
        if exclude_id > 0 {
            q = q.bind(exclude_id);
        }
        let row = q
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check permission code exists failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 按权限 ID 批量查询 menuIds（permission_id -> Vec<menu_id>）
    pub async fn list_menu_ids(
        &self,
        permission_ids: &[i64],
    ) -> Result<HashMap<i64, Vec<i64>>, AppError> {
        if permission_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let ids = permission_ids
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!("select permission_id, menu_id from sys_permission_menus where permission_id in ({ids})");
        let rows = sqlx::query::<sqlx::Any>(&sql)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list permission menu ids failed".into(),
                source: Some(Box::new(e)),
            })?;
        let mut map: HashMap<i64, Vec<i64>> = HashMap::new();
        for row in rows {
            let pid = row.try_get::<i64, _>("permission_id").map_err(|e| {
                AppError::Internal {
                    context: "permission id decode failed".into(),
                    source: Some(Box::new(e)),
                }
            })?;
            let mid = row.try_get::<i64, _>("menu_id").map_err(|e| AppError::Internal {
                context: "menu id decode failed".into(),
                source: Some(Box::new(e)),
            })?;
            map.entry(pid).or_default().push(mid);
        }
        Ok(map)
    }

    /// 按权限 ID 批量查询 apiIds（permission_id -> Vec<api_id>）
    pub async fn list_api_ids(
        &self,
        permission_ids: &[i64],
    ) -> Result<HashMap<i64, Vec<i64>>, AppError> {
        if permission_ids.is_empty() {
            return Ok(HashMap::new());
        }
        let ids = permission_ids
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(",");
        let sql = format!("select permission_id, api_id from sys_permission_apis where permission_id in ({ids})");
        let rows = sqlx::query::<sqlx::Any>(&sql)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list permission api ids failed".into(),
                source: Some(Box::new(e)),
            })?;
        let mut map: HashMap<i64, Vec<i64>> = HashMap::new();
        for row in rows {
            let pid = row.try_get::<i64, _>("permission_id").map_err(|e| {
                AppError::Internal {
                    context: "permission id decode failed".into(),
                    source: Some(Box::new(e)),
                }
            })?;
            let aid = row.try_get::<i64, _>("api_id").map_err(|e| AppError::Internal {
                context: "api id decode failed".into(),
                source: Some(Box::new(e)),
            })?;
            map.entry(pid).or_default().push(aid);
        }
        Ok(map)
    }

    /// 创建权限点（含 join 表写入，非事务依赖唯一键约束兜底——与 Go BatchCreate 一致）。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        name: Option<&str>,
        code: Option<&str>,
        group_id: Option<i64>,
        status: Option<&str>,
        description: Option<&str>,
        menu_ids: &[i64],
        api_ids: &[i64],
        created_by: i64,
    ) -> Result<i64, AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        let sql = "insert into sys_permissions \
                   (name, code, group_id, status, description, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7::timestamptz, $7::timestamptz) returning id";
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(code)
            .bind(group_id)
            .bind(status)
            .bind(description)
            .bind(created_by)
            .bind(now.clone())
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert permission failed".into(),
                source: Some(Box::new(e)),
            })?;
        let id = row.0;
        self.assign_relations(id, menu_ids, api_ids, created_by).await?;
        Ok(id)
    }

    /// 覆盖式写入 join 表：先清空后重建（对齐 Go AssignApis/AssignMenus 语义）。
    pub async fn assign_relations(
        &self,
        permission_id: i64,
        menu_ids: &[i64],
        api_ids: &[i64],
        operator_id: i64,
    ) -> Result<(), AppError> {
        sqlx::query("delete from sys_permission_menus where permission_id = $1")
            .bind(permission_id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "clear permission menus failed".into(),
                source: Some(Box::new(e)),
            })?;
        sqlx::query("delete from sys_permission_apis where permission_id = $1")
            .bind(permission_id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "clear permission apis failed".into(),
                source: Some(Box::new(e)),
            })?;
        for mid in menu_ids {
            sqlx::query(
                "insert into sys_permission_menus (permission_id, menu_id, created_by, created_at, updated_at) \
                 values ($1, $2, $3, $4::timestamptz, $4::timestamptz)",
            )
            .bind(permission_id)
            .bind(mid)
            .bind(operator_id)
            .bind(chrono::Utc::now().to_rfc3339())
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert permission menu failed".into(),
                source: Some(Box::new(e)),
            })?;
        }
        for aid in api_ids {
            sqlx::query(
                "insert into sys_permission_apis (permission_id, api_id, created_by, created_at, updated_at) \
                 values ($1, $2, $3, $4::timestamptz, $4::timestamptz)",
            )
            .bind(permission_id)
            .bind(aid)
            .bind(operator_id)
            .bind(chrono::Utc::now().to_rfc3339())
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert permission api failed".into(),
                source: Some(Box::new(e)),
            })?;
        }
        Ok(())
    }

    /// 更新权限点：仅非 None 字段；menuIds/apiIds 提供时覆盖关联。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        code: Option<&str>,
        group_id: Option<i64>,
        status: Option<&str>,
        description: Option<&str>,
        menu_ids: Option<&[i64]>,
        api_ids: Option<&[i64]>,
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

        if let Some(v) = name {
            push!("name", v.to_string(), "");
        }
        if let Some(v) = code {
            push!("code", v.to_string(), "");
        }
        if let Some(v) = group_id {
            push!("group_id", v.to_string(), "::int8");
        }
        if let Some(v) = status {
            push!("status", v.to_string(), "");
        }
        if let Some(v) = description {
            push!("description", v.to_string(), "");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let sql = format!(
            "update sys_permissions set {} where id = ${} and deleted_at is null",
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
                context: "update permission failed".into(),
                source: Some(Box::new(e)),
            })?;

        // 关联覆盖仅在非空提供时执行（None 表示未提供，不触碰 join 表）
        if let Some(menus) = menu_ids {
            self.assign_relations(id, menus, api_ids.unwrap_or(&[]), updated_by).await?;
        }

        Ok(res.rows_affected())
    }

    /// 硬删除权限点 + 清理 join 表孤儿行。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        sqlx::query("delete from sys_permission_menus where permission_id = $1")
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete permission menus failed".into(),
                source: Some(Box::new(e)),
            })?;
        sqlx::query("delete from sys_permission_apis where permission_id = $1")
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete permission apis failed".into(),
                source: Some(Box::new(e)),
            })?;
        let res = sqlx::query::<sqlx::Any>("delete from sys_permissions where id = $1")
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete permission failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// 清理业务权限（code 非 `sys:` 前缀），连同 join 表孤儿一起清理（对齐 Go TruncateBizPermissions）。
    pub async fn truncate_biz_permissions(&self) -> Result<(), AppError> {
        let ids: Vec<i64> = {
            let sql = "select id from sys_permissions where code is null or code not like 'sys:%'";
            let rows = sqlx::query::<sqlx::Any>(sql)
                .fetch_all(&self.db)
                .await
                .map_err(|e| AppError::Internal {
                    context: "query biz permission ids failed".into(),
                    source: Some(Box::new(e)),
                })?;
            rows.iter()
                .filter_map(|r| r.try_get::<i64, _>("id").ok())
                .collect()
        };
        if !ids.is_empty() {
            let list = ids.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(",");
            sqlx::query::<sqlx::Any>(&format!(
                "delete from sys_permission_menus where permission_id in ({list})"
            ))
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "truncate biz permission menus failed".into(),
                source: Some(Box::new(e)),
            })?;
            sqlx::query::<sqlx::Any>(&format!(
                "delete from sys_permission_apis where permission_id in ({list})"
            ))
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "truncate biz permission apis failed".into(),
                source: Some(Box::new(e)),
            })?;
            sqlx::query::<sqlx::Any>(&format!("delete from sys_permissions where id in ({list})"))
                .execute(&self.db)
                .await
                .map_err(|e| AppError::Internal {
                    context: "truncate biz permissions failed".into(),
                    source: Some(Box::new(e)),
                })?;
        }
        Ok(())
    }

    /// 批量创建权限（sync:perms 用）。返回创建的 id 列表（顺序与输入一致）。
    pub async fn batch_create(
        &self,
        perms: &[PermissionNew],
        operator_id: i64,
    ) -> Result<Vec<i64>, AppError> {
        let mut ids = Vec::with_capacity(perms.len());
        for p in perms {
            let id = self
                .create(
                    p.name.as_deref(),
                    p.code.as_deref(),
                    p.group_id,
                    Some("ON"),
                    None,
                    &p.menu_ids,
                    &p.api_ids,
                    operator_id,
                )
                .await?;
            ids.push(id);
        }
        Ok(ids)
    }
}

/// sync:perms 用的批量写入载体
#[derive(Debug, Clone, Default)]
pub struct PermissionNew {
    pub name: Option<String>,
    pub code: Option<String>,
    pub group_id: Option<i64>,
    pub menu_ids: Vec<i64>,
    pub api_ids: Vec<i64>,
}