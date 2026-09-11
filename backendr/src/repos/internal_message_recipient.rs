// internal_message_recipient repo：收件箱查询与状态回写。

use sqlx::AnyPool;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct RecipientRow {
    pub id: i64,
    pub message_id: i64,
    pub recipient_user_id: i64,
    /// SENT | RECEIVED | READ | REVOKED | DELETED
    pub status: String,
    pub received_at: Option<String>,
    pub read_at: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub tenant_id: i64,
}

type RecTuple = (
    i64,
    i64,
    i64,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
    i64,
);

const SELECT_COLS: &str = "r.id, r.message_id, r.recipient_user_id, r.status, \
                           to_char(r.received_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), \
                           to_char(r.read_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"'), \
                           m.title, m.content, m.tenant_id";

pub struct InternalMessageRecipientRepo {
    pub db: AnyPool,
}

impl InternalMessageRecipientRepo {
    pub fn new(db: AnyPool) -> Self {
        Self { db }
    }

    /// 用户收件箱（通知类消息，含标题/正文 join）。
    pub async fn list_inbox(
        &self,
        user_id: i64,
        offset: u64,
        limit: u64,
        where_clause: &str,
        params: &[String],
        order_by: &str,
    ) -> Result<(Vec<RecipientRow>, u64), AppError> {
        let base = "from internal_message_recipients r \
                    join internal_messages m on m.id = r.message_id and m.deleted_at is null \
                    where r.recipient_user_id = $1 and r.deleted_at is null and m.type = 'NOTIFICATION'";
        let total_sql = format!("select count(*) {base}{where_clause}");
        let total = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql)
            .bind(user_id)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "count inbox failed".into(),
                source: Some(Box::new(e)),
            })?;

        let order = if order_by.is_empty() { "r.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {base}{where_clause} order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query_as::<sqlx::Any, RecTuple>(&sql);
        q = q.bind(user_id);
        for p in params {
            q = q.bind(p);
        }
        let rows = q
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list inbox failed".into(),
                source: Some(Box::new(e)),
            })?;
        let items = rows
            .into_iter()
            .map(|r| RecipientRow {
                id: r.0,
                message_id: r.1,
                recipient_user_id: r.2,
                status: r.3.unwrap_or_else(|| "SENT".into()),
                received_at: r.4,
                read_at: r.5,
                title: r.6,
                content: r.7,
                tenant_id: r.8,
            })
            .collect();
        Ok((items, total.0 as u64))
    }

    /// 标记已读：status→READ、read_at=now；ids 为空 = 全部未读。
    pub async fn mark_read(&self, user_id: i64, ids: &[i64]) -> Result<u64, AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        let (sql, params): (String, Vec<String>) = if ids.is_empty() {
            (
                "update internal_message_recipients \
                 set status = 'READ', read_at = $1::timestamptz, updated_at = $1::timestamptz \
                 where recipient_user_id = $2::int8 and status <> 'READ' and deleted_at is null"
                    .into(),
                vec![now.clone(), user_id.to_string()],
            )
        } else {
            let placeholders = (0..ids.len())
                .map(|i| format!("${}::int8", i + 3))
                .collect::<Vec<_>>()
                .join(", ");
            (
                format!(
                    "update internal_message_recipients \
                     set status = 'READ', read_at = $1::timestamptz, updated_at = $1::timestamptz \
                     where recipient_user_id = $2::int8 and id in ({placeholders}) and status <> 'READ' and deleted_at is null"
                ),
                {
                    let mut p = vec![now.clone(), user_id.to_string()];
                    p.extend(ids.iter().map(|i| i.to_string()));
                    p
                },
            )
        };
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        let res = q
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "mark notifications read failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// 收件箱删除：status→DELETED；ids 为空 = 全部。
    pub async fn delete_from_inbox(&self, user_id: i64, ids: &[i64]) -> Result<u64, AppError> {
        let now = chrono::Utc::now().to_rfc3339();
        let (sql, params): (String, Vec<String>) = if ids.is_empty() {
            (
                "update internal_message_recipients \
                 set status = 'DELETED', updated_at = $1::timestamptz \
                 where recipient_user_id = $2::int8 and status <> 'DELETED' and deleted_at is null"
                    .into(),
                vec![now.clone(), user_id.to_string()],
            )
        } else {
            let placeholders = (0..ids.len())
                .map(|i| format!("${}::int8", i + 3))
                .collect::<Vec<_>>()
                .join(", ");
            (
                format!(
                    "update internal_message_recipients \
                     set status = 'DELETED', updated_at = $1::timestamptz \
                     where recipient_user_id = $2::int8 and id in ({placeholders}) and status <> 'DELETED' and deleted_at is null"
                ),
                {
                    let mut p = vec![now.clone(), user_id.to_string()];
                    p.extend(ids.iter().map(|i| i.to_string()));
                    p
                },
            )
        };
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        let res = q
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete notifications from inbox failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// 状态回写（回执补偿通道）：ids 必填非空，防同状态重写。
    pub async fn mark_status(
        &self,
        user_id: i64,
        ids: &[i64],
        new_status: &str,
    ) -> Result<u64, AppError> {
        if ids.is_empty() {
            return Err(AppError::Validation("recipientIds is required".into()));
        }
        let now = chrono::Utc::now().to_rfc3339();
        let mut sets = "status = $1, updated_at = $2::timestamptz".to_string();
        if new_status == "READ" {
            sets.push_str(", read_at = $2::timestamptz");
        }
        if new_status == "RECEIVED" {
            sets.push_str(", received_at = $2::timestamptz");
        }
        let placeholders = (0..ids.len())
            .map(|i| format!("${}::int8", i + 4))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "update internal_message_recipients set {sets} \
             where recipient_user_id = $3::int8 and id in ({placeholders}) and status <> $1 and deleted_at is null"
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        q = q.bind(new_status);
        q = q.bind(now);
        q = q.bind(user_id);
        for id in ids {
            q = q.bind(id);
        }
        let res = q
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "mark notifications status failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }
}
