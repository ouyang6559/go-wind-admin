// mfa repository：sys_user_mfa_factors CRUD + TOTP 因子查询。
// 对齐 Go 端 user_mfa_factor_repo.go；secret 落库为 AES-GCM 密文（encrypt_channel_secret），
// 读取时解密还原明文（decrypt_channel_secret）。软删除过滤 deleted_at is null。

use sqlx::AnyPool;

use crate::error::AppError;

/// 因子元信息（管理面展示，不含 secret）
pub struct EnrolledFactorInfo {
    pub id: i64,
    pub method: String,
    pub display_name: String,
    pub enabled: bool,
    pub created_at: Option<String>,
    pub last_used_at: Option<String>,
}

const SELECT_COLS: &str = "id, method, coalesce(display_name,''), status, \
                           to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), \
                           to_char(last_used_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"')";

pub struct MfaRepo {
    pub db: AnyPool,
}

impl MfaRepo {
    pub fn new(db: AnyPool) -> Self {
        Self { db }
    }

    /// 列出某用户在指定租户内的全部因子（按创建时间倒序）。
    pub async fn list_by_user(
        &self,
        tenant_id: i64,
        user_id: i64,
    ) -> Result<Vec<EnrolledFactorInfo>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} from sys_user_mfa_factors \
             where tenant_id = $1 and user_id = $2 and deleted_at is null order by id desc"
        );
        let rows: Vec<(i64, String, String, String, Option<String>, Option<String>)> =
            sqlx::query_as::<sqlx::Any, (i64, String, String, String, Option<String>, Option<String>)>(
                &sql,
            )
            .bind(tenant_id)
            .bind(user_id)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list mfa factors failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(rows
            .into_iter()
            .map(|r| EnrolledFactorInfo {
                id: r.0,
                method: r.1,
                display_name: r.2,
                enabled: r.3 == "ENABLED",
                created_at: r.4,
                last_used_at: r.5,
            })
            .collect())
    }

    /// 某用户在指定租户内是否绑定了 ENABLED 的 TOTP 因子（登录闸门用）。
    pub async fn has_enabled_totp(&self, tenant_id: i64, user_id: i64) -> Result<bool, AppError> {
        let sql = "select true from sys_user_mfa_factors \
                   where tenant_id = $1 and user_id = $2 and method = 'TOTP' \
                     and status = 'ENABLED' and deleted_at is null limit 1";
        let row: Option<(bool,)> = sqlx::query_as::<sqlx::Any, (bool,)>(sql)
            .bind(tenant_id)
            .bind(user_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "query has enabled totp failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 取某用户 ENABLED 的 TOTP 因子并解密 secret（校验登录挑战用）。
    /// 返回 (factor_id, plain_secret)。
    pub async fn find_enabled_totp(
        &self,
        tenant_id: i64,
        user_id: i64,
    ) -> Result<Option<(i64, String)>, AppError> {
        let sql = "select id, secret_hash from sys_user_mfa_factors \
                   where tenant_id = $1 and user_id = $2 and method = 'TOTP' \
                     and status = 'ENABLED' and deleted_at is null \
                   order by id limit 1";
        let row: Option<(i64, Option<String>)> =
            sqlx::query_as::<sqlx::Any, (i64, Option<String>)>(sql)
                .bind(tenant_id)
                .bind(user_id)
                .fetch_optional(&self.db)
                .await
                .map_err(|e| AppError::Internal {
                    context: "query enabled totp factor failed".into(),
                    source: Some(Box::new(e)),
                })?;
        let Some((factor_id, stored)) = row else {
            return Ok(None);
        };
        let Some(stored) = stored.filter(|s| !s.is_empty()) else {
            return Err(AppError::Internal {
                context: "mfa secret missing".into(),
                source: None,
            });
        };
        let plain = crate::crypto::decrypt_channel_secret(&stored)?;
        Ok(Some((factor_id, plain)))
    }

    /// 按 id 取因子归属（管理端定位目标用户用），不含 secret。返回 (tenant_id, user_id)。
    pub async fn get_factor_by_id(&self, factor_id: i64) -> Result<Option<(i64, i64)>, AppError> {
        let sql = "select tenant_id, user_id from sys_user_mfa_factors \
                   where id = $1 and deleted_at is null limit 1";
        let row: Option<(i64, i64)> = sqlx::query_as::<sqlx::Any, (i64, i64)>(sql)
            .bind(factor_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get mfa factor by id failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row)
    }

    /// 按用户+方法找首行因子归属（平台管理员救援重置定位租户用）。
    pub async fn find_first_by_user(
        &self,
        user_id: i64,
        method: &str,
    ) -> Result<Option<(i64, i64)>, AppError> {
        let sql = "select tenant_id, user_id from sys_user_mfa_factors \
                   where user_id = $1 and method = $2 and deleted_at is null \
                   order by id limit 1";
        let row: Option<(i64, i64)> = sqlx::query_as::<sqlx::Any, (i64, i64)>(sql)
            .bind(user_id)
            .bind(method)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "find mfa factor by user failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row)
    }

    /// 创建一条 ENABLED 的 TOTP 因子。secret 加密后落库。返回新因子 ID。
    pub async fn create_totp_factor(
        &self,
        tenant_id: i64,
        user_id: i64,
        plain_secret: &str,
        display_name: &str,
    ) -> Result<i64, AppError> {
        let enc = crate::crypto::encrypt_channel_secret(plain_secret)?;
        let sql = "insert into sys_user_mfa_factors \
                   (tenant_id, user_id, method, secret_hash, display_name, status, created_at, updated_at) \
                   values ($1, $2, 'TOTP', $3, $4, 'ENABLED', $5::timestamptz, $5::timestamptz) returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(tenant_id)
            .bind(user_id)
            .bind(enc)
            .bind(display_name)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "create mfa factor failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 按 factor_id 删除因子，强制校验 (tenant_id, user_id) 归属。返回是否删除了行。
    pub async fn delete_for_user(
        &self,
        tenant_id: i64,
        user_id: i64,
        factor_id: i64,
    ) -> Result<bool, AppError> {
        let sql = "delete from sys_user_mfa_factors \
                   where id = $1 and tenant_id = $2 and user_id = $3 and deleted_at is null";
        let res = sqlx::query::<sqlx::Any>(sql)
            .bind(factor_id)
            .bind(tenant_id)
            .bind(user_id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete mfa factor failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected() > 0)
    }

    /// 清空某用户指定方法的全部因子（管理端救援重置/本人按方法禁用）。返回删除行数。
    pub async fn delete_all_by_user_method(
        &self,
        tenant_id: i64,
        user_id: i64,
        method: &str,
    ) -> Result<u64, AppError> {
        let sql = "delete from sys_user_mfa_factors \
                   where tenant_id = $1 and user_id = $2 and method = $3 and deleted_at is null";
        let res = sqlx::query::<sqlx::Any>(sql)
            .bind(tenant_id)
            .bind(user_id)
            .bind(method)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete mfa factors by method failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// 更新因子最近使用时间（best-effort，失败不阻断登录）。
    pub async fn update_last_used(
        &self,
        tenant_id: i64,
        user_id: i64,
        factor_id: i64,
    ) -> Result<(), AppError> {
        let sql = "update sys_user_mfa_factors set last_used_at = $1::timestamptz, updated_at = $1::timestamptz \
                   where id = $2 and tenant_id = $3 and user_id = $4 and deleted_at is null";
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query::<sqlx::Any>(sql)
            .bind(now)
            .bind(factor_id)
            .bind(tenant_id)
            .bind(user_id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "update mfa last_used_at failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(())
    }
}