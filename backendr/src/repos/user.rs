// user repo：sys_users + 关联表 CRUD SQL（对齐 Go user_repo.go）。
// 枚举按 proto 名存储：gender 存 SECRET/MALE/FEMALE，status 存 NORMAL/DISABLED/PENDING/LOCKED/EXPIRED/CLOSED。
// Create = 事务（sys_users + USERNAME 凭证 + sys_user_roles）；Update = 事务（字段 + 角色整体替换）；
// Delete 硬删除并清理 sys_user_roles/sys_user_org_units/sys_user_positions。
// 列表/详情聚合 roleIds/orgUnitIds/positionIds（Go ListUserRelationIDs 语义）。

use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: i64,
    pub tenant_id: Option<i64>,
    pub tenant_name: Option<String>,
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub realname: Option<String>,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub telephone: Option<String>,
    pub avatar: Option<String>,
    pub address: Option<String>,
    pub region: Option<String>,
    pub description: Option<String>,
    /// proto 枚举名（SECRET/MALE/FEMALE）
    pub gender: Option<String>,
    pub last_login_at: Option<String>,
    pub last_login_ip: Option<String>,
    pub locked_until: Option<String>,
    /// proto 枚举名（NORMAL/DISABLED/PENDING/LOCKED/EXPIRED/CLOSED）
    pub status: Option<String>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

/// SELECT 列（u.* + 租户显示名）
const SELECT_COLS: &str = "u.id, u.tenant_id, t.name as tenant_name, u.username, u.nickname, u.realname, \
                           u.email, u.mobile, u.telephone, u.avatar, u.address, u.region, u.description, \
                           u.gender, \
                           to_char(u.last_login_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as last_login_at, \
                           u.last_login_ip, \
                           to_char(u.locked_until, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as locked_until, \
                           u.status, u.remark, \
                           u.created_by, u.updated_by, u.deleted_by, \
                           to_char(u.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(u.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(u.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from sys_users u \
                    left join sys_tenants t on t.id = u.tenant_id and t.deleted_at is null";

fn map_row(row: &sqlx::any::AnyRow) -> Result<UserRow, AppError> {
    Ok(UserRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "user row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        tenant_id: row.try_get::<Option<i64>, _>("tenant_id").ok().flatten(),
        tenant_name: row.try_get::<Option<String>, _>("tenant_name").ok().flatten(),
        username: row.try_get::<Option<String>, _>("username").ok().flatten(),
        nickname: row.try_get::<Option<String>, _>("nickname").ok().flatten(),
        realname: row.try_get::<Option<String>, _>("realname").ok().flatten(),
        email: row.try_get::<Option<String>, _>("email").ok().flatten(),
        mobile: row.try_get::<Option<String>, _>("mobile").ok().flatten(),
        telephone: row.try_get::<Option<String>, _>("telephone").ok().flatten(),
        avatar: row.try_get::<Option<String>, _>("avatar").ok().flatten(),
        address: row.try_get::<Option<String>, _>("address").ok().flatten(),
        region: row.try_get::<Option<String>, _>("region").ok().flatten(),
        description: row.try_get::<Option<String>, _>("description").ok().flatten(),
        gender: row.try_get::<Option<String>, _>("gender").ok().flatten(),
        last_login_at: row.try_get::<Option<String>, _>("last_login_at").ok().flatten(),
        last_login_ip: row.try_get::<Option<String>, _>("last_login_ip").ok().flatten(),
        locked_until: row.try_get::<Option<String>, _>("locked_until").ok().flatten(),
        status: row.try_get::<Option<String>, _>("status").ok().flatten(),
        remark: row.try_get::<Option<String>, _>("remark").ok().flatten(),
        created_by: row.try_get::<Option<i64>, _>("created_by").ok().flatten(),
        updated_by: row.try_get::<Option<i64>, _>("updated_by").ok().flatten(),
        deleted_by: row.try_get::<Option<i64>, _>("deleted_by").ok().flatten(),
        created_at: row.try_get::<Option<String>, _>("created_at").ok().flatten(),
        updated_at: row.try_get::<Option<String>, _>("updated_at").ok().flatten(),
        deleted_at: row.try_get::<Option<String>, _>("deleted_at").ok().flatten(),
    })
}

pub struct UserRepo {
    pub db: AnyPool,
}

impl UserRepo {
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
    ) -> Result<(Vec<UserRow>, u64), AppError> {
        let total_sql = format!("select count(*) from sys_users u where u.deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "count users failed".into(),
            source: Some(Box::new(e)),
        })?;

        let order = if order_by.is_empty() { "u.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where u.deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "list users failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<UserRow>, AppError> {
        let sql = format!("select {SELECT_COLS} {FROM} where u.id = $1 and u.deleted_at is null limit 1");
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get user failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// 按用户名查询（须带租户过滤，防跨租户泄露）。
    pub async fn get_by_username(&self, tenant_id: i64, username: &str) -> Result<Option<UserRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} {FROM} where u.username = $1 and u.tenant_id = $2 and u.deleted_at is null limit 1"
        );
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(username)
            .bind(tenant_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get user by username failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// 用户名唯一性检查（租户内）。
    pub async fn username_exists(&self, tenant_id: i64, username: &str, exclude_id: i64) -> Result<bool, AppError> {
        let sql = "select 1 from sys_users \
                   where username = $1 and tenant_id = $2 and id <> $3 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(username)
            .bind(tenant_id)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check username existence failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    // ===================== 关联查询（对齐 ListUserRelationIDs / enrichRelations） =====================

    pub async fn list_role_ids(&self, user_id: i64) -> Result<Vec<i64>, AppError> {
        let sql = "select role_id from sys_user_roles \
                   where user_id = $1 and status = 'ACTIVE' and deleted_at is null order by id";
        let rows: Vec<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(user_id)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list user role ids failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub async fn list_org_unit_ids(&self, user_id: i64) -> Result<Vec<i64>, AppError> {
        let sql = "select org_unit_id from sys_user_org_units \
                   where user_id = $1 and status = 'ACTIVE' and deleted_at is null order by id";
        let rows: Vec<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(user_id)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list user org unit ids failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    pub async fn list_position_ids(&self, user_id: i64) -> Result<Vec<i64>, AppError> {
        let sql = "select position_id from sys_user_positions \
                   where user_id = $1 and status = 'ACTIVE' and deleted_at is null order by id";
        let rows: Vec<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(user_id)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list user position ids failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// 批量取角色信息：(id, code, name)，用于聚合 Roles/RoleNames。
    pub async fn list_roles_by_ids(&self, ids: &[i64]) -> Result<Vec<(i64, String, String)>, AppError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${i}")).collect();
        let sql = format!(
            "select id, coalesce(code,''), coalesce(name,'') from sys_roles \
             where id in ({}) and deleted_at is null",
            placeholders.join(", ")
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for id in ids {
            q = q.bind(id);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "query roles by ids failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut out = Vec::with_capacity(rows.len());
        for row in &rows {
            let id = row.try_get::<i64, _>("id").unwrap_or(0);
            let code = row.try_get::<Option<String>, _>("code").ok().flatten().unwrap_or_default();
            let name = row.try_get::<Option<String>, _>("name").ok().flatten().unwrap_or_default();
            out.push((id, code, name));
        }
        Ok(out)
    }

    /// 批量取组织名称：id → name。
    pub async fn list_org_unit_names(&self, ids: &[i64]) -> Result<Vec<(i64, String)>, AppError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${i}")).collect();
        let sql = format!(
            "select id, coalesce(name,'') from sys_org_units \
             where id in ({}) and deleted_at is null",
            placeholders.join(", ")
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for id in ids {
            q = q.bind(id);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "query org unit names failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut out = Vec::with_capacity(rows.len());
        for row in &rows {
            let id = row.try_get::<i64, _>("id").unwrap_or(0);
            let name = row.try_get::<Option<String>, _>("name").ok().flatten().unwrap_or_default();
            out.push((id, name));
        }
        Ok(out)
    }

    /// 批量取岗位名称：id → name。
    pub async fn list_position_names(&self, ids: &[i64]) -> Result<Vec<(i64, String)>, AppError> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${i}")).collect();
        let sql = format!(
            "select id, coalesce(name,'') from sys_positions \
             where id in ({}) and deleted_at is null",
            placeholders.join(", ")
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for id in ids {
            q = q.bind(id);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "query position names failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut out = Vec::with_capacity(rows.len());
        for row in &rows {
            let id = row.try_get::<i64, _>("id").unwrap_or(0);
            let name = row.try_get::<Option<String>, _>("name").ok().flatten().unwrap_or_default();
            out.push((id, name));
        }
        Ok(out)
    }

    /// 校验角色 ID 是否全部存在且类型匹配（租户操作者只能绑 TENANT 角色，平台绑 SYSTEM）。
    /// 返回存在的角色数；不匹配返回 None（调用方转 400 "some roles not found"）。
    pub async fn validate_roles(&self, ids: &[i64], tenant_id: i64) -> Result<usize, AppError> {
        if ids.is_empty() {
            return Ok(0);
        }
        let placeholders: Vec<String> = (1..=ids.len()).map(|i| format!("${i}")).collect();
        let type_filter = if tenant_id > 0 { "type = 'TENANT'" } else { "type = 'SYSTEM'" };
        let sql = format!(
            "select count(*) from sys_roles where id in ({}) and {type_filter} and deleted_at is null",
            placeholders.join(", ")
        );
        let mut q = sqlx::query_as::<sqlx::Any, (i64,)>(&sql);
        for id in ids {
            q = q.bind(id);
        }
        let row = q.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "validate roles failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(row.0 as usize)
    }

    // ===================== 写操作（事务） =====================

    /// 创建用户（事务）：sys_users + USERNAME 凭证 + sys_user_roles。
    /// password_hash 为空时不写凭证（对齐 Go：Create 后按需建凭证）。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        username: &str,
        tenant_id: i64,
        nickname: Option<&str>,
        realname: Option<&str>,
        email: Option<&str>,
        mobile: Option<&str>,
        telephone: Option<&str>,
        avatar: Option<&str>,
        address: Option<&str>,
        region: Option<&str>,
        description: Option<&str>,
        gender: Option<&str>,
        status: Option<&str>,
        remark: Option<&str>,
        role_ids: &[i64],
        password_hash: Option<&str>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let mut tx = self.db.begin().await.map_err(|e| AppError::Internal {
            context: "begin create user tx failed".into(),
            source: Some(Box::new(e)),
        })?;

        let now = chrono::Utc::now().to_rfc3339();
        let gender = gender.unwrap_or("SECRET");
        let status = status.unwrap_or("NORMAL");
        let sql = "insert into sys_users \
                   (tenant_id, username, nickname, realname, email, mobile, telephone, avatar, \
                    address, region, description, gender, status, remark, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, \
                           $16::timestamptz, $16::timestamptz) \
                   returning id";
        let row: (i64,) = match sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(tenant_id)
            .bind(username)
            .bind(nickname)
            .bind(realname)
            .bind(email)
            .bind(mobile)
            .bind(telephone)
            .bind(avatar)
            .bind(address)
            .bind(region)
            .bind(description)
            .bind(gender)
            .bind(status)
            .bind(remark)
            .bind(created_by)
            .bind(now.clone())
            .fetch_one(&mut *tx)
            .await
        {
            Ok(r) => r,
            Err(e) => {
                let _ = tx.rollback();
                return Err(AppError::Internal {
                    context: "insert user failed".into(),
                    source: Some(Box::new(e)),
                });
            }
        };
        let user_id = row.0;

        // USERNAME 凭证
        if let Some(hash) = password_hash {
            let cred_sql = "insert into sys_user_credentials \
                            (tenant_id, user_id, identity_type, identifier, credential_type, credential, \
                             is_primary, status, created_at, updated_at) \
                            values ($1, $2, 'USERNAME', $3, 'PASSWORD_HASH', $4, true, 'ENABLED', \
                                    $5::timestamptz, $5::timestamptz)";
            if let Err(e) = sqlx::query::<sqlx::Any>(cred_sql)
                .bind(tenant_id)
                .bind(user_id)
                .bind(username)
                .bind(hash)
                .bind(now.clone())
                .execute(&mut *tx)
                .await
            {
                let _ = tx.rollback();
                return Err(AppError::Internal {
                    context: "insert user credential failed".into(),
                    source: Some(Box::new(e)),
                });
            }
        }

        // 角色关联
        if !role_ids.is_empty() {
            for rid in role_ids {
                let ur_sql = "insert into sys_user_roles \
                              (tenant_id, user_id, role_id, status, is_primary, assigned_at, assigned_by, \
                               created_at, updated_at) \
                              values ($1, $2, $3, 'ACTIVE', true, $4::timestamptz, $5, $4::timestamptz, $4::timestamptz)";
                if let Err(e) = sqlx::query::<sqlx::Any>(ur_sql)
                    .bind(tenant_id)
                    .bind(user_id)
                    .bind(rid)
                    .bind(now.clone())
                    .bind(created_by)
                    .execute(&mut *tx)
                    .await
                {
                    let _ = tx.rollback();
                    return Err(AppError::Internal {
                        context: "assign role to user failed".into(),
                        source: Some(Box::new(e)),
                    });
                }
            }
        }

        tx.commit().await.map_err(|e| AppError::Internal {
            context: "commit create user tx failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(user_id)
    }

    /// 更新用户（事务）：sys_users 字段 + 角色整体替换（对齐 Go Update 的 delete-then-insert）。
    /// role_ids 为 Some 时整体替换（含清空）。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        nickname: Option<&str>,
        realname: Option<&str>,
        email: Option<&str>,
        mobile: Option<&str>,
        telephone: Option<&str>,
        avatar: Option<&str>,
        address: Option<&str>,
        region: Option<&str>,
        description: Option<&str>,
        gender: Option<&str>,
        status: Option<&str>,
        remark: Option<&str>,
        role_ids: Option<&[i64]>,
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

        if let Some(v) = nickname {
            push!("nickname", v.to_string(), "");
        }
        if let Some(v) = realname {
            push!("realname", v.to_string(), "");
        }
        if let Some(v) = email {
            push!("email", v.to_string(), "");
        }
        if let Some(v) = mobile {
            push!("mobile", v.to_string(), "");
        }
        if let Some(v) = telephone {
            push!("telephone", v.to_string(), "");
        }
        if let Some(v) = avatar {
            push!("avatar", v.to_string(), "");
        }
        if let Some(v) = address {
            push!("address", v.to_string(), "");
        }
        if let Some(v) = region {
            push!("region", v.to_string(), "");
        }
        if let Some(v) = description {
            push!("description", v.to_string(), "");
        }
        if let Some(v) = gender {
            push!("gender", v.to_string(), "");
        }
        if let Some(v) = status {
            push!("status", v.to_string(), "");
        }
        if let Some(v) = remark {
            push!("remark", v.to_string(), "");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() && role_ids.is_none() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let mut tx = self.db.begin().await.map_err(|e| AppError::Internal {
            context: "begin update user tx failed".into(),
            source: Some(Box::new(e)),
        })?;

        let mut affected = 0u64;
        if !sets.is_empty() {
            let sql = format!(
                "update sys_users set {} where id = ${} and deleted_at is null",
                sets.join(", "),
                params.len() + 1
            );
            let mut q = sqlx::query::<sqlx::Any>(&sql);
            for p in &params {
                q = q.bind(p);
            }
            q = q.bind(id);
            let res = q.execute(&mut *tx).await.map_err(|e| AppError::Internal {
                context: "update user failed".into(),
                source: Some(Box::new(e)),
            })?;
            affected = res.rows_affected();
        } else {
            // 仅改角色时仍确认用户存在
            let chk: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(
                "select 1 from sys_users where id = $1 and deleted_at is null limit 1",
            )
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(|e| AppError::Internal {
                context: "check user existence failed".into(),
                source: Some(Box::new(e)),
            })?;
            if chk.is_none() {
                let _ = tx.rollback();
                return Ok(0);
            }
            affected = 1;
        }

        // 角色整体替换
        if let Some(rids) = role_ids {
            sqlx::query::<sqlx::Any>("delete from sys_user_roles where user_id = $1")
                .bind(id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::Internal {
                    context: "delete old user roles failed".into(),
                    source: Some(Box::new(e)),
                })?;
            if !rids.is_empty() {
                let now = chrono::Utc::now().to_rfc3339();
                for rid in rids {
                    let ur_sql = "insert into sys_user_roles \
                                  (user_id, role_id, status, is_primary, assigned_at, assigned_by, \
                                   created_at, updated_at) \
                                  values ($1, $2, 'ACTIVE', true, $3::timestamptz, $4, $3::timestamptz, $3::timestamptz)";
                    if let Err(e) = sqlx::query::<sqlx::Any>(ur_sql)
                        .bind(id)
                        .bind(rid)
                        .bind(now.clone())
                        .bind(updated_by)
                        .execute(&mut *tx)
                        .await
                    {
                        let _ = tx.rollback();
                        return Err(AppError::Internal {
                            context: "assign role to user failed".into(),
                            source: Some(Box::new(e)),
                        });
                    }
                }
            }
        }

        tx.commit().await.map_err(|e| AppError::Internal {
            context: "commit update user tx failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(affected)
    }

    /// 硬删除用户并清理关联（sys_user_roles / sys_user_org_units / sys_user_positions）。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let mut tx = self.db.begin().await.map_err(|e| AppError::Internal {
            context: "begin delete user tx failed".into(),
            source: Some(Box::new(e)),
        })?;

        let res = sqlx::query::<sqlx::Any>("delete from sys_users where id = $1 and deleted_at is null")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete user failed".into(),
                source: Some(Box::new(e)),
            })?;
        let affected = res.rows_affected();

        if affected > 0 {
            for table in ["sys_user_roles", "sys_user_org_units", "sys_user_positions"] {
                let sql = format!("delete from {table} where user_id = $1");
                if let Err(e) = sqlx::query::<sqlx::Any>(&sql)
                    .bind(id)
                    .execute(&mut *tx)
                    .await
                {
                    let _ = tx.rollback();
                    return Err(AppError::Internal {
                        context: format!("clean {table} failed"),
                        source: Some(Box::new(e)),
                    });
                }
            }
        }

        tx.commit().await.map_err(|e| AppError::Internal {
            context: "commit delete user tx failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(affected)
    }
}
