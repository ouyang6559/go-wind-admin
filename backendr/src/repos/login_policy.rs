// login_policy repo：sys_login_policies CRUD SQL（对齐 Go login_policy_repo.go）。
// type/method 枚举按 proto 枚举名存储（BLACKLIST/WHITELIST、IP/MAC/REGION/TIME/DEVICE），
// 与 Go 端 EnumTypeConverter 写库值一致；列表/查询过滤 deleted_at is null，
// Delete 为硬删除（Go 端 TimeAt mixin 无软删拦截器）。

use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct LoginPolicyRow {
    pub id: i64,
    pub target_id: Option<i64>,
    pub value: Option<String>,
    pub reason: Option<String>,
    /// proto 枚举名（BLACKLIST/WHITELIST）
    pub r#type: Option<String>,
    /// proto 枚举名（IP/MAC/REGION/TIME/DEVICE）
    pub method: Option<String>,
    pub tenant_id: Option<i64>,
    pub tenant_name: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

/// SELECT 列（p.* + 租户显示名）
const SELECT_COLS: &str = "p.id, p.target_id, p.value, p.reason, p.type, p.method, \
                           p.tenant_id, t.name as tenant_name, \
                           p.created_by, p.updated_by, p.deleted_by, \
                           to_char(p.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(p.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(p.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from sys_login_policies p \
                    left join sys_tenants t on t.id = p.tenant_id and t.deleted_at is null";

fn map_row(row: &sqlx::any::AnyRow) -> Result<LoginPolicyRow, AppError> {
    Ok(LoginPolicyRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "login policy row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        target_id: row.try_get::<Option<i64>, _>("target_id").ok().flatten(),
        value: row.try_get::<Option<String>, _>("value").ok().flatten(),
        reason: row.try_get::<Option<String>, _>("reason").ok().flatten(),
        r#type: row.try_get::<Option<String>, _>("type").ok().flatten(),
        method: row.try_get::<Option<String>, _>("method").ok().flatten(),
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

pub struct LoginPolicyRepo {
    pub db: AnyPool,
}

impl LoginPolicyRepo {
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
    ) -> Result<(Vec<LoginPolicyRow>, u64), AppError> {
        let total_sql =
            format!("select count(*) from sys_login_policies p where p.deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "count login policies failed".into(),
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
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "list login policies failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<LoginPolicyRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} {FROM} where p.id = $1 and p.deleted_at is null limit 1"
        );
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get login policy failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// (tenant_id, target_id, type, method) 唯一性检查（exclude_id 排除自身）。
    /// target_id 可为 NULL，用 is not distinct from 匹配 NULL 相等语义。
    pub async fn conflict_exists(
        &self,
        tenant_id: Option<i64>,
        target_id: Option<i64>,
        r#type: &str,
        method: &str,
        exclude_id: i64,
    ) -> Result<bool, AppError> {
        let sql = "select 1 from sys_login_policies \
                   where tenant_id = $1 and target_id is not distinct from $2 \
                   and type = $3 and method = $4 and id <> $5 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(tenant_id.unwrap_or(0))
            .bind(target_id)
            .bind(r#type)
            .bind(method)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check login policy conflict failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 创建登录策略。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        tenant_id: Option<i64>,
        target_id: Option<i64>,
        r#type: &str,
        method: &str,
        value: &str,
        reason: Option<&str>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into sys_login_policies \
                   (tenant_id, target_id, type, method, value, reason, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8::timestamptz, $8::timestamptz) \
                   returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(tenant_id.unwrap_or(0))
            .bind(target_id)
            .bind(r#type)
            .bind(method)
            .bind(value)
            .bind(reason)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert login policy failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新登录策略：仅非 None 字段（tenant_id 不可变，对齐 Go TenantID mixin Immutable）。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        target_id: Option<i64>,
        r#type: Option<&str>,
        method: Option<&str>,
        value: Option<&str>,
        reason: Option<&str>,
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

        if let Some(v) = target_id {
            push!("target_id", v.to_string(), "::int8");
        }
        if let Some(v) = r#type {
            push!("type", v.to_string(), "");
        }
        if let Some(v) = method {
            push!("method", v.to_string(), "");
        }
        if let Some(v) = value {
            push!("value", v.to_string(), "");
        }
        if let Some(v) = reason {
            push!("reason", v.to_string(), "");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let sql = format!(
            "update sys_login_policies set {} where id = ${} and deleted_at is null",
            sets.join(", "),
            params.len() + 1
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        q = q.bind(id);
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "update login policy failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 硬删除。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let sql = "delete from sys_login_policies where id = $1 and deleted_at is null";
        let res = sqlx::query::<sqlx::Any>(sql)
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete login policy failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }
}
