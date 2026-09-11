// permission_group repo：sys_permission_groups CRUD SQL（对齐 Go permission_group_repo.go）。
// status 按 proto 枚举名存储（ON/OFF）；创建时按 Go 语义计算树路径
// （ComputeTreePath：根节点为 "/"，非根节点为 parentPath + id + "/"）；
// 列表/查询过滤 deleted_at is null，Delete 为硬删除并顺带清理组内权限点。

use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct PermissionGroupRow {
    pub id: i64,
    pub name: String,
    pub path: Option<String>,
    pub module: Option<String>,
    pub sort_order: Option<i64>,
    /// proto 枚举名（ON/OFF）
    pub status: Option<String>,
    pub description: Option<String>,
    pub parent_id: Option<i64>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

/// SELECT 列（g.*）
const SELECT_COLS: &str = "g.id, g.name, g.path, g.module, g.sort_order, g.status, g.description, \
                           g.parent_id, g.created_by, g.updated_by, g.deleted_by, \
                           to_char(g.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(g.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(g.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from sys_permission_groups g";

fn map_row(row: &sqlx::any::AnyRow) -> Result<PermissionGroupRow, AppError> {
    Ok(PermissionGroupRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "permission group row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        name: row
            .try_get::<String, _>("name")
            .map_err(|e| AppError::Internal {
                context: "permission group name decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        path: row.try_get::<Option<String>, _>("path").ok().flatten(),
        module: row.try_get::<Option<String>, _>("module").ok().flatten(),
        sort_order: row.try_get::<Option<i64>, _>("sort_order").ok().flatten(),
        status: row.try_get::<Option<String>, _>("status").ok().flatten(),
        description: row.try_get::<Option<String>, _>("description").ok().flatten(),
        parent_id: row.try_get::<Option<i64>, _>("parent_id").ok().flatten(),
        created_by: row.try_get::<Option<i64>, _>("created_by").ok().flatten(),
        updated_by: row.try_get::<Option<i64>, _>("updated_by").ok().flatten(),
        deleted_by: row.try_get::<Option<i64>, _>("deleted_by").ok().flatten(),
        created_at: row.try_get::<Option<String>, _>("created_at").ok().flatten(),
        updated_at: row.try_get::<Option<String>, _>("updated_at").ok().flatten(),
        deleted_at: row.try_get::<Option<String>, _>("deleted_at").ok().flatten(),
    })
}

/// 对齐 Go entCrud.ComputeTreePath：父路径为空 → "/"；否则父路径（补尾斜杠）+ id + "/"。
fn compute_tree_path(parent_path: &str, node_id: i64) -> String {
    if parent_path.is_empty() {
        return "/".into();
    }
    let mut base = parent_path.to_string();
    if !base.ends_with('/') {
        base.push('/');
    }
    format!("{base}{node_id}/")
}

pub struct PermissionGroupRepo {
    pub db: AnyPool,
}

impl PermissionGroupRepo {
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
    ) -> Result<(Vec<PermissionGroupRow>, u64), AppError> {
        let total_sql = format!(
            "select count(*) from sys_permission_groups g where g.deleted_at is null{where_clause}"
        );
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "count permission groups failed".into(),
                source: Some(Box::new(e)),
            })?;

        let order = if order_by.is_empty() { "g.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where g.deleted_at is null{where_clause} \
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
                context: "list permission groups failed".into(),
                source: Some(Box::new(e)),
            })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<PermissionGroupRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} {FROM} where g.id = $1 and g.deleted_at is null limit 1"
        );
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get permission group failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// 创建权限组（含树路径计算，路径写入与插入同事务）。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        name: &str,
        module: Option<&str>,
        sort_order: Option<i64>,
        status: Option<&str>,
        description: Option<&str>,
        parent_id: Option<i64>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let mut txn = self.db.begin().await.map_err(|e| AppError::Internal {
            context: "begin permission group txn failed".into(),
            source: Some(Box::new(e)),
        })?;

        let sql = "insert into sys_permission_groups \
                   (name, module, sort_order, status, description, parent_id, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8::timestamptz, $8::timestamptz) returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(module)
            .bind(sort_order)
            .bind(status)
            .bind(description)
            .bind(parent_id)
            .bind(created_by)
            .bind(now.clone())
            .fetch_one(&mut *txn)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert permission group failed".into(),
                source: Some(Box::new(e)),
            })?;
        let id = row.0;

        // 树路径：取父节点 path 后按 ComputeTreePath 计算（根节点为 "/"）
        let parent_path: String = match parent_id {
            Some(pid) => {
                let prow: Option<(Option<String>,)> =
                    sqlx::query_as::<sqlx::Any, (Option<String>,)>(
                        "select path from sys_permission_groups where id = $1 and deleted_at is null limit 1",
                    )
                    .bind(pid)
                    .fetch_optional(&mut *txn)
                    .await
                    .map_err(|e| AppError::Internal {
                        context: "query permission group parent path failed".into(),
                        source: Some(Box::new(e)),
                    })?;
                prow.and_then(|r| r.0).unwrap_or_default()
            }
            None => String::new(),
        };
        let path = compute_tree_path(&parent_path, id);
        sqlx::query(
            "update sys_permission_groups set path = $1, updated_at = $2::timestamptz where id = $3",
        )
        .bind(path)
        .bind(now.clone())
        .bind(id)
        .execute(&mut *txn)
        .await
        .map_err(|e| AppError::Internal {
            context: "update permission group path failed".into(),
            source: Some(Box::new(e)),
        })?;

        txn.commit().await.map_err(|e| AppError::Internal {
            context: "commit permission group txn failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(id)
    }

    /// 更新权限组：仅非 None 字段（不重算 path，对齐 Go Update）。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        module: Option<&str>,
        sort_order: Option<i64>,
        status: Option<&str>,
        description: Option<&str>,
        parent_id: Option<i64>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        macro_rules! push {
            // $cast: "::int8" / "::timestamptz" / ""
            ($col:expr, $val:expr, $cast:expr) => {{
                sets.push(format!("{} = ${}{}", $col, sets.len() + 1, $cast));
                params.push($val);
            }};
        }

        if let Some(v) = name {
            push!("name", v.to_string(), "");
        }
        if let Some(v) = module {
            push!("module", v.to_string(), "");
        }
        if let Some(v) = sort_order {
            push!("sort_order", v.to_string(), "::int8");
        }
        if let Some(v) = status {
            push!("status", v.to_string(), "");
        }
        if let Some(v) = description {
            push!("description", v.to_string(), "");
        }
        if let Some(v) = parent_id {
            push!("parent_id", v.to_string(), "::int8");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let sql = format!(
            "update sys_permission_groups set {} where id = ${} and deleted_at is null",
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
                context: "update permission group failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// 硬删除：先清组内权限点（sys_permissions.group_id），再删分组本身（对齐 Go service Delete 顺序）。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        sqlx::query("delete from sys_permissions where group_id = $1")
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete permission group's permissions failed".into(),
                source: Some(Box::new(e)),
            })?;

        let sql = "delete from sys_permission_groups where id = $1 and deleted_at is null";
        let res = sqlx::query::<sqlx::Any>(sql)
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete permission group failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }
}
