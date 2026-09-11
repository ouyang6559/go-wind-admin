// role repo：sys_roles + sys_role_permissions + sys_role_metadata CRUD SQL（对齐 Go role_repo.go）。
// 枚举按 proto 名存储：type 存 SYSTEM/TEMPLATE/TENANT，status 存 ON/OFF。
// Create/Update/Delete 均在一个事务内维护角色本体与权限关联（对齐 Go 端 tx 语义）；
// Create 额外写入 sys_role_metadata（is_template/scope/sync_policy，对齐 CreateWithTx）。
// 列表/查询过滤 deleted_at is null，Delete 为硬删除（Go 端 TimeAt mixin 无软删拦截器）。

use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct RoleRow {
    pub id: i64,
    pub name: Option<String>,
    pub code: Option<String>,
    pub sort_order: Option<i64>,
    /// proto 枚举名（SYSTEM/TEMPLATE/TENANT）
    pub r#type: Option<String>,
    /// proto 枚举名（ON/OFF）
    pub status: Option<String>,
    pub is_protected: Option<bool>,
    pub description: Option<String>,
    pub tenant_id: Option<i64>,
    pub tenant_name: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

/// SELECT 列（r.* + 租户显示名）
const SELECT_COLS: &str = "r.id, r.name, r.code, r.sort_order, r.type, r.status, r.is_protected, \
                           r.description, r.tenant_id, t.name as tenant_name, \
                           r.created_by, r.updated_by, r.deleted_by, \
                           to_char(r.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(r.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(r.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from sys_roles r \
                    left join sys_tenants t on t.id = r.tenant_id and t.deleted_at is null";

fn map_row(row: &sqlx::any::AnyRow) -> Result<RoleRow, AppError> {
    Ok(RoleRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "role row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        name: row.try_get::<Option<String>, _>("name").ok().flatten(),
        code: row.try_get::<Option<String>, _>("code").ok().flatten(),
        sort_order: row.try_get::<Option<i64>, _>("sort_order").ok().flatten(),
        r#type: row.try_get::<Option<String>, _>("type").ok().flatten(),
        status: row.try_get::<Option<String>, _>("status").ok().flatten(),
        is_protected: row
            .try_get::<Option<bool>, _>("is_protected")
            .ok()
            .flatten(),
        description: row.try_get::<Option<String>, _>("description").ok().flatten(),
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

pub struct RoleRepo {
    pub db: AnyPool,
}

impl RoleRepo {
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
    ) -> Result<(Vec<RoleRow>, u64), AppError> {
        let total_sql = format!("select count(*) from sys_roles r where r.deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "count roles failed".into(),
            source: Some(Box::new(e)),
        })?;

        let order = if order_by.is_empty() { "r.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where r.deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "list roles failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<RoleRow>, AppError> {
        let sql = format!("select {SELECT_COLS} {FROM} where r.id = $1 and r.deleted_at is null limit 1");
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get role failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// 编码唯一性检查（租户内 (tenant_id, code) 唯一，Rust 端以 code 全局近似）
    pub async fn code_exists(&self, code: &str, exclude_id: i64) -> Result<bool, AppError> {
        let sql = "select 1 from sys_roles where code = $1 and id <> $2 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(code)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check role code failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 列出角色的权限ID列表（sys_role_permissions）。
    pub async fn list_permission_ids(&self, role_id: i64) -> Result<Vec<i64>, AppError> {
        let sql = "select permission_id from sys_role_permissions where role_id = $1 order by id";
        let rows = sqlx::query::<sqlx::Any>(sql)
            .bind(role_id)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list role permission ids failed".into(),
                source: Some(Box::new(e)),
            })?;
        let mut ids = Vec::with_capacity(rows.len());
        for row in &rows {
            if let Ok(v) = row.try_get::<i64, _>("permission_id") {
                ids.push(v);
            }
        }
        Ok(ids)
    }

    /// 角色模板 code 去前缀（对齐 constants.ExtractRoleCodeFromTemplate）。
    fn extract_template_code(code: &str) -> String {
        code.strip_prefix("template:").unwrap_or(code).to_string()
    }

    /// 创建角色（事务）：sys_roles + sys_role_metadata + 授权 sys_role_permissions。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        name: &str,
        code: &str,
        sort_order: Option<i64>,
        is_protected: Option<bool>,
        r#type: Option<&str>,
        status: Option<&str>,
        description: Option<&str>,
        tenant_id: Option<i64>,
        created_by: i64,
        permissions: &[i64],
    ) -> Result<i64, AppError> {
        let mut tx = self.db.begin().await.map_err(|e| AppError::Internal {
            context: "begin create role tx failed".into(),
            source: Some(Box::new(e)),
        })?;

        let now = chrono::Utc::now().to_rfc3339();
        let r#type = r#type.unwrap_or("TENANT");
        let status = status.unwrap_or("ON");
        let is_protected = is_protected.unwrap_or(false);
        let sql = "insert into sys_roles \
                   (name, code, sort_order, type, status, is_protected, description, tenant_id, \
                    created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::timestamptz, $10::timestamptz) \
                   returning id";
        let row: (i64,) = match sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(code)
            .bind(sort_order)
            .bind(r#type)
            .bind(status)
            .bind(is_protected)
            .bind(description)
            .bind(tenant_id)
            .bind(created_by)
            .bind(now.clone())
            .fetch_one(&mut *tx)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.rollback();
                return Err(AppError::Internal {
                    context: "insert role failed".into(),
                    source: Some(Box::new(e)),
                });
            }
        };
        let role_id = row.0;

        // 角色元数据（对齐 CreateWithTx）：scope 由 type 推导，模板角色标记 is_template
        let is_template = r#type == "TEMPLATE";
        let scope = if r#type == "TENANT" { "TENANT" } else { "PLATFORM" };
        let template_for = if is_template {
            Self::extract_template_code(code)
        } else {
            String::new()
        };
        let meta_sql = "insert into sys_role_metadata \
                        (role_id, tenant_id, created_by, is_template, template_for, template_version, \
                         sync_policy, scope, custom_overrides, created_at, updated_at) \
                        values ($1, $2, $3, $4, $5, 1, 'AUTO', $6, '{}'::jsonb, $7::timestamptz, $7::timestamptz)";
        let meta_result = sqlx::query::<sqlx::Any>(meta_sql)
            .bind(role_id)
            .bind(tenant_id)
            .bind(created_by)
            .bind(is_template)
            .bind(template_for)
            .bind(scope)
            .bind(now.clone())
            .execute(&mut *tx)
            .await;
        if let Err(e) = meta_result {
            let _ = tx.rollback();
            return Err(AppError::Internal {
                context: "insert role metadata failed".into(),
                source: Some(Box::new(e)),
            });
        }

        // 分配权限（对齐 assignPermissionsToRole）
        if !permissions.is_empty() {
            for pid in permissions {
                let rp_sql = "insert into sys_role_permissions \
                              (role_id, permission_id, tenant_id, created_by, status, effect, priority, \
                               created_at, updated_at) \
                              values ($1, $2, $3, $4, 'ON', 'ALLOW', 0, $5::timestamptz, $5::timestamptz)";
                let rp_result = sqlx::query::<sqlx::Any>(rp_sql)
                    .bind(role_id)
                    .bind(pid)
                    .bind(tenant_id)
                    .bind(created_by)
                    .bind(now.clone())
                    .execute(&mut *tx)
                    .await;
                if let Err(e) = rp_result {
                    let _ = tx.rollback();
                    return Err(AppError::Internal {
                        context: "assign permission to role failed".into(),
                        source: Some(Box::new(e)),
                    });
                }
            }
        }

        tx.commit().await.map_err(|e| AppError::Internal {
            context: "commit create role tx failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(role_id)
    }

    /// 更新角色（事务）：sys_roles 字段 + permissions 整体替换 + 模板版本升级。
    /// permissions 为 Some 时表示用户提交了权限项（含清空场景），整体替换（对齐 ReplacePermissions）。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        code: Option<&str>,
        sort_order: Option<i64>,
        is_protected: Option<bool>,
        r#type: Option<&str>,
        status: Option<&str>,
        description: Option<&str>,
        updated_by: i64,
        permissions: Option<&[i64]>,
    ) -> Result<u64, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        macro_rules! push {
            // $cast: "::int8" / "::bool" / ""（文本列直接绑定）
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
        if let Some(v) = sort_order {
            push!("sort_order", v.to_string(), "::int8");
        }
        if let Some(v) = is_protected {
            push!("is_protected", v.to_string(), "::bool");
        }
        if let Some(v) = r#type {
            push!("type", v.to_string(), "");
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

        let mut tx = self.db.begin().await.map_err(|e| AppError::Internal {
            context: "begin update role tx failed".into(),
            source: Some(Box::new(e)),
        })?;

        let sql = format!(
            "update sys_roles set {} where id = ${} and deleted_at is null",
            sets.join(", "),
            params.len() + 1
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        q = q.bind(id);
        let res = q.execute(&mut *tx).await.map_err(|e| AppError::Internal {
            context: "update role failed".into(),
            source: Some(Box::new(e)),
        })?;
        let affected = res.rows_affected();

        // 权限整体替换（对齐 ReplacePermissions：先清空再插入）
        if let Some(pids) = permissions {
            let del_sql = "delete from sys_role_permissions where role_id = $1";
            sqlx::query::<sqlx::Any>(del_sql)
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Internal {
                    context: "delete old role permissions failed".into(),
                    source: Some(Box::new(e)),
                })?;
            if !pids.is_empty() {
                let now = chrono::Utc::now().to_rfc3339();
                for pid in pids {
                    let rp_sql = "insert into sys_role_permissions \
                                  (role_id, permission_id, created_by, status, effect, priority, \
                                   created_at, updated_at) \
                                  values ($1, $2, $3, 'ON', 'ALLOW', 0, $4::timestamptz, $4::timestamptz)";
                    sqlx::query::<sqlx::Any>(rp_sql)
                        .bind(id)
                        .bind(pid)
                        .bind(updated_by)
                        .bind(now.clone())
                        .execute(&mut *tx)
                        .await
                        .map_err(|e| AppError::Internal {
                            context: "assign permission to role failed".into(),
                            source: Some(Box::new(e)),
                        })?;
                }
            }
        }

        // 模板角色升级模板版本（对齐 UpgradeTemplateVersion）
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query::<sqlx::Any>(
            "update sys_role_metadata set template_version = template_version + 1, updated_at = $2::timestamptz \
             where role_id = $1 and is_template = true",
        )
        .bind(id)
        .bind(now)
        .execute(&mut *tx)
        .await
        .map_err(|e| AppError::Internal {
            context: "upgrade role metadata template version failed".into(),
            source: Some(Box::new(e)),
        })?;

        tx.commit().await.map_err(|e| AppError::Internal {
            context: "commit update role tx failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(affected)
    }

    /// 删除角色（事务）：sys_roles 硬删 + 清理 sys_role_permissions。
    /// 保护角色校验由 handler 在调用前完成（对齐 Go Delete 先查后删）。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let mut tx = self.db.begin().await.map_err(|e| AppError::Internal {
            context: "begin delete role tx failed".into(),
            source: Some(Box::new(e)),
        })?;

        let res = sqlx::query::<sqlx::Any>("delete from sys_roles where id = $1 and deleted_at is null")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete role failed".into(),
                source: Some(Box::new(e)),
            })?;
        let affected = res.rows_affected();

        if affected > 0 {
            sqlx::query::<sqlx::Any>("delete from sys_role_permissions where role_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Internal {
                    context: "clean role permissions failed".into(),
                    source: Some(Box::new(e)),
                })?;
        }

        tx.commit().await.map_err(|e| AppError::Internal {
            context: "commit delete role tx failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(affected)
    }
}
