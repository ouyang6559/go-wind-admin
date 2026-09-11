// tenant repo：sys_tenants CRUD + CreateTenantWithAdminUser 事务 + GetUsage 聚合 + CleanupData 清理事务。
// 枚举按 proto 名存储：status 存 ON/OFF/EXPIRED/FREEZE，type 存 TRIAL/PAID/INTERNAL/PARTNER/CUSTOM，
// audit_status 存 PENDING/APPROVED/REJECTED（对齐 Go converter 的 entity 名）。
// Delete 硬删除（对齐 Go DeleteOneID）；All 查询过滤 deleted_at is null。
// with-admin 对齐 tenant_service.go CreateTenantWithAdminUser：
//  - code/name 存在即 400（OR 语义）；
//  - 租户管理员角色从模板 `template:tenant:manager` 复制（TENANT 类型、is_protected=true、无权限）；
//  - 管理员用户（USERNAME 凭证=明文密码 bcrypt、绑定角色）写入同一事务；
//  - 提交后回填 sys_tenants.admin_user_id。

use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;

/// 租户管理员模板角色码（对齐 constants.TenantAdminTemplateRoleCode）
const TENANT_ADMIN_TEMPLATE_ROLE_CODE: &str = "template:tenant:manager";
/// 租户管理员角色码（对齐 constants.TenantAdminRoleCode）
const TENANT_ADMIN_ROLE_CODE: &str = "tenant:manager";
/// 租户管理员角色默认名称（对齐 constants.DefaultTenantManagerRoleName）
const TENANT_ADMIN_ROLE_NAME: &str = "租户管理员";

#[derive(Debug, Clone)]
pub struct TenantRow {
    pub id: i64,
    pub name: Option<String>,
    pub code: Option<String>,
    pub logo_url: Option<String>,
    pub domain: Option<String>,
    pub industry: Option<String>,
    pub admin_user_id: Option<i64>,
    pub admin_user_name: Option<String>,
    /// proto 枚举名（ON/OFF/EXPIRED/FREEZE）
    pub status: Option<String>,
    /// proto 枚举名（TRIAL/PAID/INTERNAL/PARTNER/CUSTOM）
    pub r#type: Option<String>,
    /// proto 枚举名（PENDING/APPROVED/REJECTED）
    pub audit_status: Option<String>,
    pub subscription_at: Option<String>,
    pub unsubscribe_at: Option<String>,
    pub subscription_plan: Option<String>,
    pub expired_at: Option<String>,
    pub plan_id: Option<i64>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

const SELECT_COLS: &str = "t.id, t.name, t.code, t.logo_url, t.domain, t.industry, t.admin_user_id, \
                           u.username as admin_user_name, \
                           t.status, t.type, t.audit_status, \
                           to_char(t.subscription_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as subscription_at, \
                           to_char(t.unsubscribe_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as unsubscribe_at, \
                           t.subscription_plan, \
                           to_char(t.expired_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as expired_at, \
                           t.plan_id, \
                           t.created_by, t.updated_by, t.deleted_by, \
                           to_char(t.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(t.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(t.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from sys_tenants t \
                    left join sys_users u on u.id = t.admin_user_id and u.deleted_at is null";

fn map_row(row: &sqlx::any::AnyRow) -> Result<TenantRow, AppError> {
    Ok(TenantRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "tenant row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        name: row.try_get::<Option<String>, _>("name").ok().flatten(),
        code: row.try_get::<Option<String>, _>("code").ok().flatten(),
        logo_url: row.try_get::<Option<String>, _>("logo_url").ok().flatten(),
        domain: row.try_get::<Option<String>, _>("domain").ok().flatten(),
        industry: row.try_get::<Option<String>, _>("industry").ok().flatten(),
        admin_user_id: row.try_get::<Option<i64>, _>("admin_user_id").ok().flatten(),
        admin_user_name: row.try_get::<Option<String>, _>("admin_user_name").ok().flatten(),
        status: row.try_get::<Option<String>, _>("status").ok().flatten(),
        r#type: row.try_get::<Option<String>, _>("type").ok().flatten(),
        audit_status: row.try_get::<Option<String>, _>("audit_status").ok().flatten(),
        subscription_at: row.try_get::<Option<String>, _>("subscription_at").ok().flatten(),
        unsubscribe_at: row.try_get::<Option<String>, _>("unsubscribe_at").ok().flatten(),
        subscription_plan: row.try_get::<Option<String>, _>("subscription_plan").ok().flatten(),
        expired_at: row.try_get::<Option<String>, _>("expired_at").ok().flatten(),
        plan_id: row.try_get::<Option<i64>, _>("plan_id").ok().flatten(),
        created_by: row.try_get::<Option<i64>, _>("created_by").ok().flatten(),
        updated_by: row.try_get::<Option<i64>, _>("updated_by").ok().flatten(),
        deleted_by: row.try_get::<Option<i64>, _>("deleted_by").ok().flatten(),
        created_at: row.try_get::<Option<String>, _>("created_at").ok().flatten(),
        updated_at: row.try_get::<Option<String>, _>("updated_at").ok().flatten(),
        deleted_at: row.try_get::<Option<String>, _>("deleted_at").ok().flatten(),
    })
}

pub struct TenantRepo {
    pub db: AnyPool,
}

impl TenantRepo {
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
    ) -> Result<(Vec<TenantRow>, u64), AppError> {
        let total_sql = format!("select count(*) from sys_tenants t where t.deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "count tenants failed".into(),
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
            context: "list tenants failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<TenantRow>, AppError> {
        let sql = format!("select {SELECT_COLS} {FROM} where t.id = $1 and t.deleted_at is null limit 1");
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get tenant failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    pub async fn get_by_code(&self, code: &str) -> Result<Option<TenantRow>, AppError> {
        let sql = format!("select {SELECT_COLS} {FROM} where t.code = $1 and t.deleted_at is null limit 1");
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(code)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get tenant by code failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    pub async fn get_by_name(&self, name: &str) -> Result<Option<TenantRow>, AppError> {
        let sql = format!("select {SELECT_COLS} {FROM} where t.name = $1 and t.deleted_at is null limit 1");
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(name)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get tenant by name failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// code 或 name 已存在（OR 语义，对齐 Go TenantExists）。空值不参与匹配。
    pub async fn exists(&self, code: Option<&str>, name: Option<&str>) -> Result<bool, AppError> {
        let code = code.map(|c| c.trim()).filter(|c| !c.is_empty());
        let name = name.map(|n| n.trim()).filter(|n| !n.is_empty());
        if code.is_none() && name.is_none() {
            return Ok(false);
        }
        let mut conds: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();
        if let Some(c) = code {
            conds.push(format!("code = ${}", params.len() + 1));
            params.push(c.to_string());
        }
        if let Some(n) = name {
            conds.push(format!("name = ${}", params.len() + 1));
            params.push(n.to_string());
        }
        let sql = format!(
            "select 1 from sys_tenants where ({}) and deleted_at is null limit 1",
            conds.join(" or ")
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        let row: Option<(i64,)> = q.fetch_optional(&self.db).await.map_err(|e| {
            AppError::Internal {
                context: "check tenant existence failed".into(),
                source: Some(Box::new(e)),
            }
        })?;
        Ok(row.is_some())
    }

    /// 创建租户（单表）。返回值 = 新租户 ID。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        name: Option<&str>,
        code: Option<&str>,
        logo_url: Option<&str>,
        domain: Option<&str>,
        industry: Option<&str>,
        admin_user_id: Option<i64>,
        status: Option<&str>,
        r#type: Option<&str>,
        audit_status: Option<&str>,
        subscription_plan: Option<&str>,
        expired_at: Option<&str>,
        subscription_at: Option<&str>,
        unsubscribe_at: Option<&str>,
        plan_id: Option<i64>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        let status = status.unwrap_or("ON");
        let r#type = r#type.unwrap_or("PAID");
        let audit_status = audit_status.unwrap_or("APPROVED");
        let sql = "insert into sys_tenants \
                   (name, code, logo_url, domain, industry, admin_user_id, status, type, audit_status, \
                    subscription_plan, expired_at, subscription_at, unsubscribe_at, plan_id, \
                    created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, \
                           $11::timestamptz, $12::timestamptz, $13::timestamptz, $14, \
                           $15, $16::timestamptz, $16::timestamptz) \
                   returning id";
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(code)
            .bind(logo_url)
            .bind(domain)
            .bind(industry)
            .bind(admin_user_id)
            .bind(status)
            .bind(r#type)
            .bind(audit_status)
            .bind(subscription_plan)
            .bind(expired_at)
            .bind(subscription_at)
            .bind(unsubscribe_at)
            .bind(plan_id)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert tenant failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新租户字段（动态 SET，仅更新 Some 字段）。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        code: Option<&str>,
        logo_url: Option<&str>,
        domain: Option<&str>,
        industry: Option<&str>,
        admin_user_id: Option<i64>,
        status: Option<&str>,
        r#type: Option<&str>,
        audit_status: Option<&str>,
        subscription_plan: Option<&str>,
        expired_at: Option<&str>,
        subscription_at: Option<&str>,
        unsubscribe_at: Option<&str>,
        plan_id: Option<i64>,
        updated_by: i64,
    ) -> Result<u64, AppError> {
        let mut sets: Vec<String> = Vec::new();
        let mut params: Vec<String> = Vec::new();

        macro_rules! push_text {
            ($col:expr, $val:expr) => {{
                sets.push(format!("{} = ${}", $col, sets.len() + 1));
                params.push($val.to_string());
            }};
        }
        if let Some(v) = name {
            push_text!("name", v);
        }
        if let Some(v) = code {
            push_text!("code", v);
        }
        if let Some(v) = logo_url {
            push_text!("logo_url", v);
        }
        if let Some(v) = domain {
            push_text!("domain", v);
        }
        if let Some(v) = industry {
            push_text!("industry", v);
        }
        if let Some(v) = admin_user_id {
            sets.push(format!("admin_user_id = ${}", sets.len() + 1));
            params.push(v.to_string());
        }
        if let Some(v) = status {
            push_text!("status", v);
        }
        if let Some(v) = r#type {
            push_text!("type", v);
        }
        if let Some(v) = audit_status {
            push_text!("audit_status", v);
        }
        if let Some(v) = subscription_plan {
            push_text!("subscription_plan", v);
        }
        if let Some(v) = expired_at {
            sets.push(format!("expired_at = ${}::timestamptz", sets.len() + 1));
            params.push(v.to_string());
        }
        if let Some(v) = subscription_at {
            sets.push(format!("subscription_at = ${}::timestamptz", sets.len() + 1));
            params.push(v.to_string());
        }
        if let Some(v) = unsubscribe_at {
            sets.push(format!("unsubscribe_at = ${}::timestamptz", sets.len() + 1));
            params.push(v.to_string());
        }
        if let Some(v) = plan_id {
            sets.push(format!("plan_id = ${}", sets.len() + 1));
            params.push(v.to_string());
        }
        sets.push(format!("updated_by = ${}::int8", sets.len() + 1));
        params.push(updated_by.to_string());
        sets.push(format!("updated_at = ${}::timestamptz", sets.len() + 1));
        params.push(chrono::Utc::now().to_rfc3339());

        if sets.is_empty() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let sql = format!(
            "update sys_tenants set {} where id = ${} and deleted_at is null",
            sets.join(", "),
            params.len() + 1
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        q = q.bind(id);
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "update tenant failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 硬删除租户（对齐 Go DeleteOneID；软删列仅用于过滤）。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let res = sqlx::query::<sqlx::Any>("delete from sys_tenants where id = $1 and deleted_at is null")
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete tenant failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// 批量统计各租户用户数（member_count 回填）。
    pub async fn count_users_by_tenant_ids(&self, ids: &[i64]) -> Result<std::collections::HashMap<i64, i64>, AppError> {
        let mut out = std::collections::HashMap::new();
        if ids.is_empty() {
            return Ok(out);
        }
        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${i}")).collect();
        let sql = format!(
            "select tenant_id, count(*) from sys_users \
             where tenant_id in ({}) and deleted_at is null group by tenant_id",
            placeholders.join(", ")
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for id in ids {
            q = q.bind(id);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "count users by tenant ids failed".into(),
            source: Some(Box::new(e)),
        })?;
        for row in &rows {
            let tid = row.try_get::<i64, _>("tenant_id").unwrap_or(0);
            let cnt = row.try_get::<i64, _>("count").unwrap_or(0);
            out.insert(tid, cnt);
        }
        Ok(out)
    }

    /// 查询租户用量与配额（对齐 Go TenantUsageRepo.GetUsage）。
    pub async fn get_usage(&self, tenant_id: i64) -> Result<serde_json::Value, AppError> {
        // 1. 租户 + 套餐名 + 配额
        let tenant: Option<(Option<String>, Option<i64>, Option<String>)> = sqlx::query_as::<sqlx::Any, (Option<String>, Option<i64>, Option<String>)>(
            "select t.name, t.plan_id, p.name \
             from sys_tenants t left join sys_plans p on p.id = t.plan_id and p.deleted_at is null \
             where t.id = $1 and t.deleted_at is null limit 1",
        )
        .bind(tenant_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "query tenant for usage failed".into(),
            source: Some(Box::new(e)),
        })?;
        let (_tenant_name, plan_id, plan_name) = match tenant {
            Some(t) => t,
            None => return Err(AppError::Validation("tenant not found".into())),
        };

        let now = chrono::Utc::now().to_rfc3339();
        let _ = now;

        // 2. 配额列表（proto 枚举名即 DB 存储名：USER_LIMIT/STORAGE/API_CALL）
        let mut quotas: Vec<serde_json::Value> = Vec::new();
        if let Some(pid) = plan_id {
            let rows: Vec<(String, i64)> = sqlx::query_as::<sqlx::Any, (String, i64)>(
                "select coalesce(quota_type,''), coalesce(quota_value,0) \
                 from sys_plan_quotas where plan_id = $1 and deleted_at is null",
            )
            .bind(pid)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "query plan quotas failed".into(),
                source: Some(Box::new(e)),
            })?;
            for (qt, qv) in rows {
                quotas.push(serde_json::json!({
                    "quotaType": qt,
                    "quotaValue": qv.to_string(),
                }));
            }
        }

        // 3. 用户数
        let user_count: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(
            "select count(*) from sys_users where tenant_id = $1 and deleted_at is null",
        )
        .bind(tenant_id)
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "count tenant users failed".into(),
            source: Some(Box::new(e)),
        })?;

        // 4. 存储占用（file.size 求和；该表可能不存在时为 0）
        let storage_used: i64 = sqlx::query_as::<sqlx::Any, (i64,)>(
            "select coalesce(sum(size),0) from files where tenant_id = $1",
        )
        .bind(tenant_id)
        .fetch_one(&self.db)
        .await
        .map(|r| r.0)
        .unwrap_or(0);

        // 5. API 调用量（审计日志）
        let api_call: i64 = sqlx::query_as::<sqlx::Any, (i64,)>(
            "select count(*) from sys_api_audit_logs where tenant_id = $1",
        )
        .bind(tenant_id)
        .fetch_one(&self.db)
        .await
        .map(|r| r.0)
        .unwrap_or(0);

        Ok(serde_json::json!({
            "tenantId": tenant_id,
            "userCount": user_count.0.to_string(),
            "storageUsedBytes": storage_used.to_string(),
            "apiCallCount": api_call.to_string(),
            "planId": plan_id.map(|v| v.to_string()),
            "planName": plan_name,
            "quotas": quotas,
        }))
    }

    /// 清理租户数据（事务）：硬删该租户全部业务表数据 + 保留租户记录（status→OFF）+ 吊销令牌。
    pub async fn cleanup_data(&self, tenant_id: i64, state: &crate::state::AppState) -> Result<(), AppError> {
        let mut tx = self.db.begin().await.map_err(|e| AppError::Internal {
            context: "begin cleanup tenant tx failed".into(),
            source: Some(Box::new(e)),
        })?;

        // 提交前收集租户用户 ID（删除用户后无法反查）
        let user_id_rows: Vec<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(
            "select id from sys_users where tenant_id = $1",
        )
        .bind(tenant_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| AppError::Internal {
            context: "query tenant user ids failed".into(),
            source: Some(Box::new(e)),
        })?;
        let user_ids: Vec<i64> = user_id_rows.into_iter().map(|r| r.0).collect();

        // 对齐 Go CleanupTenantData 的表清单（29 张带 tenant_id 的业务表）
        let tables = [
            "sys_api_audit_logs",
            "sys_data_access_audit_logs",
            "sys_dict_entries",
            "sys_dict_entry_i18n",
            "sys_dict_types",
            "files",
            "internal_messages",
            "internal_message_categories",
            "internal_message_recipients",
            "sys_login_audit_logs",
            "sys_login_policies",
            "sys_memberships",
            "sys_membership_org_units",
            "sys_membership_positions",
            "sys_membership_roles",
            "sys_operation_audit_logs",
            "sys_org_units",
            "sys_permission_audit_logs",
            "sys_policy_evaluation_logs",
            "sys_positions",
            "sys_roles",
            "sys_role_metadata",
            "sys_role_permissions",
            "sys_tasks",
            "sys_users",
            "sys_user_credentials",
            "sys_user_org_units",
            "sys_user_positions",
            "sys_user_roles",
        ];
        for t in tables {
            let sql = format!("delete from {t} where tenant_id = $1");
            if let Err(e) = sqlx::query::<sqlx::Any>(&sql)
                .bind(tenant_id)
                .execute(&mut *tx)
                .await
            {
                // 表可能不存在或列名不同：记录告警继续（对齐 Go 逐表容错思想，但保留可见日志）
                tracing::warn!(table = t, tenant_id, error = %e, "cleanup tenant table skipped");
            }
        }

        // 保留租户记录，状态改 OFF 阻断后续访问（对齐 TenantAccessChecker status!=ON→403）
        let now = chrono::Utc::now().to_rfc3339();
        if let Err(e) = sqlx::query::<sqlx::Any>(
            "update sys_tenants set status = 'OFF', updated_at = $2::timestamptz where id = $1",
        )
        .bind(tenant_id)
        .bind(now)
        .execute(&mut *tx)
        .await
        {
            let _ = tx.rollback();
            return Err(AppError::Internal {
                context: "set tenant status OFF failed".into(),
                source: Some(Box::new(e)),
            });
        }

        tx.commit().await.map_err(|e| AppError::Internal {
            context: "commit cleanup tenant tx failed".into(),
            source: Some(Box::new(e)),
        })?;

        // 提交后吊销该租户全部用户令牌（admin+app 双 ClientType；失败仅告警）
        for uid in user_ids {
            for ct in ["admin", "app"] {
                let _ = crate::auth::revoke_all_sessions(state, ct, uid, "").await;
            }
        }
        Ok(())
    }
}

// ===================== CreateTenantWithAdminUser（事务） =====================

/// 创建租户 + 从模板复制管理员角色 + 创建管理员用户（USERNAME 凭证）+ 绑定角色 + 回填 admin_user_id。
/// password 为**明文**（前端 with-admin 不加密，直接 bcrypt 落库，对齐 Go CreateWithTx 无 NeedDecrypt）。
#[allow(clippy::too_many_arguments)]
pub async fn create_tenant_with_admin(
    db: &AnyPool,
    tenant: &TenantNew,
    username: &str,
    password_hash: &str,
    operator_id: i64,
) -> Result<i64, AppError> {
    let mut tx = db.begin().await.map_err(|e| AppError::Internal {
        context: "begin create tenant with admin tx failed".into(),
        source: Some(Box::new(e)),
    })?;

    let now = chrono::Utc::now().to_rfc3339();

    // 1. 创建租户
    let ten_status = tenant.status.unwrap_or("ON");
    let ten_type = tenant.r#type.unwrap_or("PAID");
    let ten_audit = tenant.audit_status.unwrap_or("APPROVED");
    let ten_sql = "insert into sys_tenants \
                   (name, code, domain, logo_url, industry, status, type, audit_status, \
                    subscription_plan, expired_at, plan_id, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10::timestamptz, $11, \
                           $12, $13::timestamptz, $13::timestamptz) \
                   returning id";
    let tenant_id: i64 = match sqlx::query_as::<sqlx::Any, (i64,)>(ten_sql)
        .bind(tenant.name.as_deref())
        .bind(tenant.code.as_deref())
        .bind(tenant.domain.as_deref())
        .bind(tenant.logo_url.as_deref())
        .bind(tenant.industry.as_deref())
        .bind(ten_status)
        .bind(ten_type)
        .bind(ten_audit)
        .bind(tenant.subscription_plan.as_deref())
        .bind(tenant.expired_at.as_deref())
        .bind(tenant.plan_id)
        .bind(operator_id)
        .bind(now.clone())
        .fetch_one(&mut *tx)
        .await
    {
        Ok(r) => r.0,
        Err(e) => {
            let _ = tx.rollback();
            return Err(AppError::Internal {
                context: "insert tenant in with-admin tx failed".into(),
                source: Some(Box::new(e)),
            });
        }
    };

    // 2. 从模板复制租户管理员角色（无权限，对齐 CreateTenantRoleFromTemplate → CreateWithTx）
    let tmpl: Option<(String, i64, bool)> = sqlx::query_as::<sqlx::Any, (String, i64, bool)>(
        "select coalesce(description,''), coalesce(sort_order,0), coalesce(is_protected,false) \
         from sys_roles where code = $1 and tenant_id is null and deleted_at is null limit 1",
    )
    .bind(TENANT_ADMIN_TEMPLATE_ROLE_CODE)
    .fetch_optional(&mut *tx)
    .await
    .map_err(|e| AppError::Internal {
        context: "query tenant admin role template failed".into(),
        source: Some(Box::new(e)),
    })?;

    let role_sql = "insert into sys_roles \
                    (name, code, sort_order, type, status, is_protected, description, tenant_id, \
                     created_by, created_at, updated_at) \
                    values ($1, $2, $3, 'TENANT', 'ON', $4, $5, $6, $7, $8::timestamptz, $8::timestamptz) \
                    returning id";
    let role_id: i64 = match sqlx::query_as::<sqlx::Any, (i64,)>(role_sql)
        .bind(TENANT_ADMIN_ROLE_NAME)
        .bind(TENANT_ADMIN_ROLE_CODE)
        .bind(tmpl.as_ref().map(|t| t.1))
        .bind(tmpl.as_ref().map_or(true, |t| t.2))
        .bind(tmpl.as_ref().map(|t| t.0.clone()))
        .bind(tenant_id)
        .bind(operator_id)
        .bind(now.clone())
        .fetch_one(&mut *tx)
        .await
    {
        Ok(r) => r.0,
        Err(e) => {
            let _ = tx.rollback();
            return Err(AppError::Internal {
                context: "insert tenant admin role failed".into(),
                source: Some(Box::new(e)),
            });
        }
    };

    // 角色元数据（对齐 RoleRepo.CreateWithTx）
    let meta_sql = "insert into sys_role_metadata \
                    (role_id, tenant_id, created_by, is_template, template_for, template_version, \
                     sync_policy, scope, custom_overrides, created_at, updated_at) \
                    values ($1, $2, $3, false, '', 1, 'AUTO', 'TENANT', '{}'::jsonb, $4::timestamptz, $4::timestamptz)";
    if let Err(e) = sqlx::query::<sqlx::Any>(meta_sql)
        .bind(role_id)
        .bind(tenant_id)
        .bind(operator_id)
        .bind(now.clone())
        .execute(&mut *tx)
        .await
    {
        let _ = tx.rollback();
        return Err(AppError::Internal {
            context: "insert tenant admin role metadata failed".into(),
            source: Some(Box::new(e)),
        });
    }

    // 3. 创建管理员用户
    let user_sql = "insert into sys_users \
                    (tenant_id, username, nickname, realname, email, mobile, gender, status, \
                     created_by, created_at, updated_at) \
                    values ($1, $2, $3, $4, $5, $6, 'SECRET', 'NORMAL', \
                            $7, $8::timestamptz, $8::timestamptz) \
                    returning id";
    let user_id: i64 = match sqlx::query_as::<sqlx::Any, (i64,)>(user_sql)
        .bind(tenant_id)
        .bind(username)
        .bind(tenant.admin_name.as_deref().unwrap_or(username))
        .bind(tenant.admin_realname.as_deref())
        .bind(tenant.admin_email.as_deref())
        .bind(tenant.admin_mobile.as_deref())
        .bind(operator_id)
        .bind(now.clone())
        .fetch_one(&mut *tx)
        .await
    {
        Ok(r) => r.0,
        Err(e) => {
            let _ = tx.rollback();
            return Err(AppError::Internal {
                context: "insert tenant admin user failed".into(),
                source: Some(Box::new(e)),
            });
        }
    };

    // 4. USERNAME 凭证
    let cred_sql = "insert into sys_user_credentials \
                    (tenant_id, user_id, identity_type, identifier, credential_type, credential, \
                     is_primary, status, created_at, updated_at) \
                    values ($1, $2, 'USERNAME', $3, 'PASSWORD_HASH', $4, true, 'ENABLED', $5::timestamptz, $5::timestamptz)";
    if let Err(e) = sqlx::query::<sqlx::Any>(cred_sql)
        .bind(tenant_id)
        .bind(user_id)
        .bind(username)
        .bind(password_hash)
        .bind(now.clone())
        .execute(&mut *tx)
        .await
    {
        let _ = tx.rollback();
        return Err(AppError::Internal {
            context: "insert tenant admin credential failed".into(),
            source: Some(Box::new(e)),
        });
    }

    // 5. 绑定角色
    let ur_sql = "insert into sys_user_roles \
                  (tenant_id, user_id, role_id, status, is_primary, assigned_at, assigned_by, \
                   created_at, updated_at) \
                  values ($1, $2, $3, 'ACTIVE', true, $4::timestamptz, $5, $4::timestamptz, $4::timestamptz)";
    if let Err(e) = sqlx::query::<sqlx::Any>(ur_sql)
        .bind(tenant_id)
        .bind(user_id)
        .bind(role_id)
        .bind(now.clone())
        .bind(operator_id)
        .execute(&mut *tx)
        .await
    {
        let _ = tx.rollback();
        return Err(AppError::Internal {
            context: "assign tenant admin role failed".into(),
            source: Some(Box::new(e)),
        });
    }

    // 6. 回填 admin_user_id
    if let Err(e) = sqlx::query::<sqlx::Any>(
        "update sys_tenants set admin_user_id = $2, updated_at = $3::timestamptz where id = $1",
    )
    .bind(tenant_id)
    .bind(user_id)
    .bind(now.clone())
    .execute(&mut *tx)
    .await
    {
        let _ = tx.rollback();
        return Err(AppError::Internal {
            context: "assign tenant admin id failed".into(),
            source: Some(Box::new(e)),
        });
    }

    tx.commit().await.map_err(|e| AppError::Internal {
        context: "commit create tenant with admin tx failed".into(),
        source: Some(Box::new(e)),
    })?;
    Ok(tenant_id)
}

/// 创建租户及管理员用户的入参（handler 解析后传入）。
#[derive(Debug, Default)]
pub struct TenantNew {
    pub name: Option<String>,
    pub code: Option<String>,
    pub domain: Option<String>,
    pub logo_url: Option<String>,
    pub industry: Option<String>,
    pub status: Option<String>,
    pub r#type: Option<String>,
    pub audit_status: Option<String>,
    pub subscription_plan: Option<String>,
    pub expired_at: Option<String>,
    pub plan_id: Option<i64>,
    /// 管理员显示名（无则回落 username）
    pub admin_name: Option<String>,
    pub admin_realname: Option<String>,
    pub admin_email: Option<String>,
    pub admin_mobile: Option<String>,
}