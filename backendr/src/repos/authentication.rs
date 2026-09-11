// authentication repository：用户/凭证/角色 SQL 访问。
// 共享 Kratos/backendz 的 `gwa` schema（sys_users / sys_user_credentials / sys_user_roles / sys_roles）。

use sqlx::AnyPool;

use crate::error::AppError;

/// 登录时命中的凭证行（仅查询姓名字段以外的最小集）
pub struct CredentialRow {
    pub user_id: i64,
    pub credential_type: String,
    pub credential: String,
    pub status: String,
}

/// 用户基础行
pub struct UserRow {
    pub id: i64,
    pub tenant_id: i64,
    pub username: String,
    pub email: Option<String>,
    pub mobile: Option<String>,
    pub status: String,
}

pub struct AuthenticationRepo {
    pub db: AnyPool,
}

impl AuthenticationRepo {
    pub fn new(db: AnyPool) -> Self {
        Self { db }
    }

    /// 在指定租户内反查 username：输入含 '@' 按 email、纯数字按 mobile 匹配。
    /// 未命中返回 None（调用方沿用原输入，走统一失败路径防枚举）。
    pub async fn find_username_by_identifier(
        &self,
        tenant_id: i64,
        identifier: &str,
    ) -> Result<Option<String>, AppError> {
        let is_email = identifier.contains('@');
        let is_mobile = !is_email && !identifier.is_empty() && identifier.bytes().all(|b| b.is_ascii_digit());

        if !is_email && !is_mobile {
            return Ok(None);
        }

        let sql = if is_email {
            "select username from sys_users \
             where tenant_id = $1 and email = $2 and deleted_at is null limit 1"
        } else {
            "select username from sys_users \
             where tenant_id = $1 and mobile = $2 and deleted_at is null limit 1"
        };

        let row: Option<(String,)> = sqlx::query_as::<sqlx::Any, (String,)>(sql)
            .bind(tenant_id)
            .bind(identifier)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "query username by identifier failed".into(),
                source: Some(Box::new(e)),
            })?;

        Ok(row.map(|r| r.0))
    }

    /// 在租户范围内按 USERNAME 凭证查询单条凭证。
    pub async fn get_credential(&self, tenant_id: i64, identifier: &str) -> Result<Option<CredentialRow>, AppError> {
        let sql = "select user_id, credential_type, credential, status \
                   from sys_user_credentials \
                   where tenant_id = $1 and identity_type = 'USERNAME' and identifier = $2 and deleted_at is null \
                   limit 1";
        let row: Option<(i64, String, String, String)> =
            sqlx::query_as::<sqlx::Any, (i64, String, String, String)>(sql)
                .bind(tenant_id)
                .bind(identifier)
                .fetch_optional(&self.db)
                .await
                .map_err(|e| AppError::Internal {
                    context: "query user credential failed".into(),
                    source: Some(Box::new(e)),
                })?;
        Ok(row.map(|r| CredentialRow {
            user_id: r.0,
            credential_type: r.1,
            credential: r.2,
            status: r.3,
        }))
    }

    /// 按 ID 精确查询用户。
    pub async fn get_user_by_id(&self, user_id: i64) -> Result<Option<UserRow>, AppError> {
        let sql = "select id, tenant_id, username, email, mobile, status \
                   from sys_users where id = $1 and deleted_at is null limit 1";
        let row: Option<(i64, i64, String, Option<String>, Option<String>, String)> =
            sqlx::query_as::<sqlx::Any, (i64, i64, String, Option<String>, Option<String>, String)>(
                sql,
            )
            .bind(user_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "query user by id failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.map(|r| UserRow {
            id: r.0,
            tenant_id: r.1,
            username: r.2,
            email: r.3,
            mobile: r.4,
            status: r.5,
        }))
    }

    /// 查询用户角色码列表（sys_user_roles → sys_roles）。
    pub async fn list_role_codes(&self, user_id: i64) -> Result<Vec<String>, AppError> {
        let sql = "select r.code from sys_roles r \
                   join sys_user_roles ur on ur.role_id = r.id and ur.deleted_at is null \
                   where ur.user_id = $1 and r.deleted_at is null and r.status = 'ON' \
                     and ur.status = 'ACTIVE'";
        let rows: Vec<(String,)> = sqlx::query_as::<sqlx::Any, (String,)>(sql)
            .bind(user_id)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "query role codes failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// 更新最近登录时间/IP（失败不阻断登录）。
    pub async fn update_last_login(&self, user_id: i64, ip: &str) -> Result<(), AppError> {
        let sql = "update sys_users \
                   set last_login_at = $1::timestamptz, last_login_ip = $2, updated_at = $1::timestamptz \
                   where id = $3";
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query::<sqlx::Any>(sql)
            .bind(now)
            .bind(ip)
            .bind(user_id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "update last login failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(())
    }

    /// 按租户编号查询租户 ID（仅限启用状态）。
    pub async fn get_tenant_id_by_code(&self, code: &str) -> Result<Option<i64>, AppError> {
        let sql = "select id from sys_tenants where code = $1 and status = 'ON' and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(code)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "query tenant by code failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.map(|r| r.0))
    }

    /// 在指定租户范围检查 username 是否已存在（注册防重）。
    pub async fn username_exists(&self, tenant_id: i64, username: &str) -> Result<bool, AppError> {
        let sql = "select 1 from sys_users where tenant_id = $1 and username = $2 and deleted_at is null limit 1";
        let row: Option<(bool,)> = sqlx::query_as::<sqlx::Any, (bool,)>(sql)
            .bind(tenant_id)
            .bind(username)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "query username existence failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 创建用户，返回新用户 ID。
    pub async fn create_user(
        &self,
        tenant_id: i64,
        username: &str,
        email: Option<&str>,
    ) -> Result<i64, AppError> {
        let sql = "insert into sys_users (tenant_id, username, email, gender, status, created_at, updated_at) \
                   values ($1, $2, $3, 'SECRET', 'NORMAL', $4::timestamptz, $4::timestamptz) returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(tenant_id)
            .bind(username)
            .bind(email)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "create user failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 创建 USERNAME/PASSWORD_HASH 凭证。
    #[allow(clippy::too_many_arguments)]
    pub async fn create_credential(
        &self,
        tenant_id: i64,
        user_id: i64,
        identifier: &str,
        password_hash: &str,
    ) -> Result<(), AppError> {
        let sql = "insert into sys_user_credentials \
                   (tenant_id, user_id, identity_type, identifier, credential_type, credential, is_primary, status, created_at, updated_at) \
                   values ($1, $2, 'USERNAME', $3, 'PASSWORD_HASH', $4, true, 'ENABLED', $5::timestamptz, $5::timestamptz)";
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query::<sqlx::Any>(sql)
            .bind(tenant_id)
            .bind(user_id)
            .bind(identifier)
            .bind(password_hash)
            .bind(now)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "create user credential failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(())
    }
// ===================== 找回/重置密码 =====================

/// 按 EMAIL 凭证查用户（不存在返回 None，调用方做防枚举静默处理）。
pub async fn find_user_id_by_email_credential(
    &self,
    identifier: &str,
) -> Result<Option<(i64, i64)>, AppError> {
    // 返回 (tenant_id, user_id)
    let sql = "select tenant_id, user_id from sys_user_credentials \
               where identity_type = 'EMAIL' and identifier = $1 and status = 'ENABLED' and deleted_at is null \
               limit 1";
    let row: Option<(i64, i64)> =
        sqlx::query_as::<sqlx::Any, (i64, i64)>(sql)
            .bind(identifier)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "query email credential failed".into(),
                source: Some(Box::new(e)),
            })?;
    Ok(row)
}

/// 查用户的 USERNAME 凭证标识（重置密码定位目标凭证行）。
pub async fn get_username_identifier(&self, user_id: i64) -> Result<Option<String>, AppError> {
    let sql = "select identifier from sys_user_credentials \
               where user_id = $1 and identity_type = 'USERNAME' and deleted_at is null \
               order by id limit 1";
    let row: Option<(String,)> = sqlx::query_as::<sqlx::Any, (String,)>(sql)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "query username credential failed".into(),
            source: Some(Box::new(e)),
        })?;
    Ok(row.map(|r| r.0))
}

/// 重置 USERNAME 凭证的密码哈希。
pub async fn update_password_hash(
    &self,
    tenant_id: i64,
    identifier: &str,
    password_hash: &str,
) -> Result<u64, AppError> {
    let sql = "update sys_user_credentials \
               set credential = $1, updated_at = $2::timestamptz \
               where tenant_id = $3 and identity_type = 'USERNAME' and identifier = $4 and deleted_at is null";
    let now = chrono::Utc::now().to_rfc3339();
    let res = sqlx::query::<sqlx::Any>(sql)
        .bind(password_hash)
        .bind(now)
        .bind(tenant_id)
        .bind(identifier)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "update password hash failed".into(),
            source: Some(Box::new(e)),
        })?;
    Ok(res.rows_affected())
}

/// 第一个启用的 EMAIL 通知渠道（找回密码发信用）。
#[allow(clippy::type_complexity)]
pub async fn first_enabled_email_channel(
    &self,
) -> Result<Option<(i64, String, i64, String, Option<String>, String, String)>, AppError> {
    // (id, smtp_host, smtp_port, smtp_username, smtp_password, smtp_from, smtp_tls)
    let sql = "select id, coalesce(smtp_host,''), coalesce(smtp_port,0), coalesce(smtp_username,''), \
                      smtp_password, coalesce(smtp_from,''), coalesce(smtp_tls,'START_TLS') \
               from sys_notification_channels \
               where type = 'EMAIL' and status = 'ON' and deleted_at is null \
               order by id limit 1";
    let row: Option<(i64, String, i64, String, Option<String>, String, String)> =
        sqlx::query_as::<sqlx::Any, (i64, String, i64, String, Option<String>, String, String)>(sql)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "query email channel failed".into(),
                source: Some(Box::new(e)),
            })?;
    Ok(row)
}
}
