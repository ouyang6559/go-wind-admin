// notification_channel repo：sys_notification_channels CRUD SQL。

use sqlx::AnyPool;

use crate::error::AppError;

/// 渠道行（含敏感列，出口前必须经 row_to_dto 脱敏）
#[derive(Debug, Clone)]
pub struct ChannelRow {
    pub id: i64,
    pub name: String,
    pub r#type: String,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<i64>,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from: Option<String>,
    pub smtp_tls: Option<String>,
    pub status: Option<String>,
    pub remark: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

const SELECT_COLS: &str = "id, name, type, smtp_host, smtp_port, smtp_username, smtp_password, \
                           smtp_from, smtp_tls, status, remark, created_by, updated_by, \
                           to_char(created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), \
                           to_char(updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"')";

pub struct NotificationChannelRepo {
    pub db: AnyPool,
}

impl NotificationChannelRepo {
    pub fn new(db: AnyPool) -> Self {
        Self { db }
    }

    #[allow(clippy::too_many_lines)]
    fn map_row(
        r: (
            i64,
            String,
            String,
            Option<String>,
            Option<i64>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<String>,
            Option<i64>,
            Option<i64>,
            Option<String>,
            Option<String>,
        ),
    ) -> ChannelRow {
        ChannelRow {
            id: r.0,
            name: r.1,
            r#type: r.2,
            smtp_host: r.3,
            smtp_port: r.4,
            smtp_username: r.5,
            smtp_password: r.6,
            smtp_from: r.7,
            smtp_tls: r.8,
            status: r.9,
            remark: r.10,
            created_by: r.11,
            updated_by: r.12,
            created_at: r.13,
            updated_at: r.14,
        }
    }

    /// 分页列表（无条件 + keyword contains on name）。
    pub async fn list(
        &self,
        offset: u64,
        limit: u64,
    ) -> Result<(Vec<ChannelRow>, u64), AppError> {
        let total: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(
            "select count(*) from sys_notification_channels where deleted_at is null",
        )
        .fetch_one(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "count notification channels failed".into(),
            source: Some(Box::new(e)),
        })?;

        let sql = format!(
            "select {SELECT_COLS} from sys_notification_channels \
             where deleted_at is null \
             order by id limit {limit} offset {offset}"
        );
        let rows = sqlx::query_as::<
            sqlx::Any,
            (
                i64,
                String,
                String,
                Option<String>,
                Option<i64>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<i64>,
                Option<i64>,
                Option<String>,
                Option<String>,
            ),
        >(&sql)
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "list notification channels failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok((rows.into_iter().map(Self::map_row).collect(), total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<ChannelRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} from sys_notification_channels where id = $1 and deleted_at is null limit 1"
        );
        let row = sqlx::query_as::<
            sqlx::Any,
            (
                i64,
                String,
                String,
                Option<String>,
                Option<i64>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<String>,
                Option<i64>,
                Option<i64>,
                Option<String>,
                Option<String>,
            ),
        >(&sql)
        .bind(id)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "get notification channel failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(row.map(Self::map_row))
    }

    /// 创建渠道；返回新 ID。password 已由 service 层加密。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        name: &str,
        r#type: &str,
        smtp_host: Option<&str>,
        smtp_port: Option<i64>,
        smtp_username: Option<&str>,
        smtp_password_enc: Option<&str>,
        smtp_from: Option<&str>,
        smtp_tls: &str,
        enabled: bool,
        remark: Option<&str>,
        operator_id: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into sys_notification_channels \
                   (name, type, smtp_host, smtp_port, smtp_username, smtp_password, smtp_from, smtp_tls, \
                    status, remark, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12::timestamptz, $12::timestamptz) returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(name)
            .bind(r#type)
            .bind(smtp_host)
            .bind(smtp_port)
            .bind(smtp_username)
            .bind(smtp_password_enc)
            .bind(smtp_from)
            .bind(smtp_tls)
            .bind(if enabled { "ON" } else { "OFF" })
            .bind(remark)
            .bind(operator_id)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert notification channel failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新渠道（仅非 None 字段）；password_enc 为 Some 时才更新密码。
    /// 全部参数按字符串绑定，整型列在 SQL 里显式 ::int8（postgres 强类型）。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        name: Option<&str>,
        smtp_host: Option<&str>,
        smtp_port: Option<i64>,
        smtp_username: Option<&str>,
        smtp_password_enc: Option<&str>,
        smtp_from: Option<&str>,
        smtp_tls: Option<&str>,
        enabled: Option<bool>,
        remark: Option<&str>,
        operator_id: i64,
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
        if let Some(v) = smtp_host {
            push!("smtp_host", v.to_string(), "");
        }
        if let Some(v) = smtp_port {
            push!("smtp_port", v.to_string(), "::int8");
        }
        if let Some(v) = smtp_username {
            push!("smtp_username", v.to_string(), "");
        }
        if let Some(v) = smtp_password_enc {
            push!("smtp_password", v.to_string(), "");
        }
        if let Some(v) = smtp_from {
            push!("smtp_from", v.to_string(), "");
        }
        if let Some(v) = smtp_tls {
            push!("smtp_tls", v.to_string(), "");
        }
        if let Some(v) = enabled {
            push!("status", if v { "ON" } else { "OFF" }.to_string(), "");
        }
        if let Some(v) = remark {
            push!("remark", v.to_string(), "");
        }
        push!("updated_by", operator_id.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() {
            return Ok(0);
        }
        let sql = format!(
            "update sys_notification_channels set {} where id = ${} and deleted_at is null",
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
                context: "update notification channel failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// 硬删除（对齐 Go DeleteOneID）。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let res = sqlx::query::<sqlx::Any>(
            "delete from sys_notification_channels where id = $1",
        )
        .bind(id)
        .execute(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "delete notification channel failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }
}
