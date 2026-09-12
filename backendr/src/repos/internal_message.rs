// internal_message repo：internal_messages CRUD SQL（对齐 Go internal_message_repo.go /
// internal_message_recipient_repo.go 的发送/撤销/删除语义）。
// status/type 枚举按 proto 枚举名存储（DRAFT/PUBLISHED/...、NOTIFICATION/PRIVATE/GROUP）；
// 列表/查询过滤 deleted_at is null；Delete/Revoke/SendMessage 为硬删/插入语义：
//   - send_message：事务内写消息本体 + 批量收件记录（status=RECEIVED、received_at=now）
//   - revoke：user_id>0 仅删该用户收件记录；user_id=0 同事务删消息本体 + 全部收件记录
//   - delete_with_recipients：同事务删消息本体 + 全部收件记录（消息管理后台删除入口）

use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct InternalMessageRow {
    pub id: i64,
    pub title: Option<String>,
    pub content: Option<String>,
    /// proto 枚举名（DRAFT/PUBLISHED/SCHEDULED/REVOKED/ARCHIVED/DELETED）
    pub status: Option<String>,
    /// proto 枚举名（NOTIFICATION/PRIVATE/GROUP）
    pub r#type: Option<String>,
    pub sender_id: Option<i64>,
    pub sender_name: Option<String>,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub tenant_id: Option<i64>,
    pub tenant_name: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

/// SELECT 列（m.* + 发送者/分类/租户显示名）
const SELECT_COLS: &str = "m.id, m.title, m.content, m.status, m.type, \
                           m.sender_id, u.username as sender_name, \
                           m.category_id, c.name as category_name, \
                           m.tenant_id, t.name as tenant_name, \
                           m.created_by, m.updated_by, m.deleted_by, \
                           to_char(m.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(m.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(m.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from internal_messages m \
                    left join sys_users u on u.id = m.sender_id and u.deleted_at is null \
                    left join internal_message_categories c on c.id = m.category_id and c.deleted_at is null \
                    left join sys_tenants t on t.id = m.tenant_id and t.deleted_at is null";

fn map_row(row: &sqlx::any::AnyRow) -> Result<InternalMessageRow, AppError> {
    Ok(InternalMessageRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "internal message row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        title: row.try_get::<Option<String>, _>("title").ok().flatten(),
        content: row.try_get::<Option<String>, _>("content").ok().flatten(),
        status: row.try_get::<Option<String>, _>("status").ok().flatten(),
        r#type: row.try_get::<Option<String>, _>("type").ok().flatten(),
        sender_id: row.try_get::<Option<i64>, _>("sender_id").ok().flatten(),
        sender_name: row.try_get::<Option<String>, _>("sender_name").ok().flatten(),
        category_id: row.try_get::<Option<i64>, _>("category_id").ok().flatten(),
        category_name: row.try_get::<Option<String>, _>("category_name").ok().flatten(),
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

pub struct InternalMessageRepo {
    pub db: AnyPool,
}

/// send_message 返回：消息 ID + 落库的收件记录（行 ID, 收件人 ID）+ 落库时间戳
/// （SSE 推送载荷用，避免二次回查）。
pub struct SentMessage {
    pub message_id: i64,
    pub recipients: Vec<(i64, i64)>,
    pub received_at: String,
}

impl InternalMessageRepo {
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
    ) -> Result<(Vec<InternalMessageRow>, u64), AppError> {
        let total_sql = format!(
            "select count(*) from internal_messages m where m.deleted_at is null{where_clause}"
        );
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "count internal messages failed".into(),
                source: Some(Box::new(e)),
            })?;

        let order = if order_by.is_empty() { "m.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where m.deleted_at is null{where_clause} \
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
                context: "list internal messages failed".into(),
                source: Some(Box::new(e)),
            })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<InternalMessageRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} {FROM} where m.id = $1 and m.deleted_at is null limit 1"
        );
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get internal message failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// 全员广播：拉取全部未删除用户 ID（由 send_message 在事务内写收件记录）。
    pub async fn all_user_ids(&self) -> Result<Vec<i64>, AppError> {
        let rows: Vec<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(
            "select id from sys_users where deleted_at is null",
        )
        .fetch_all(&self.db)
        .await
        .map_err(|e| AppError::Internal {
            context: "list all user ids failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    /// 创建消息（仅本体；SendMessage 请走 send_message 事务）。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        title: Option<&str>,
        content: Option<&str>,
        sender_id: i64,
        category_id: Option<i64>,
        status: &str,
        r#type: &str,
        tenant_id: Option<i64>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into internal_messages \
                   (title, content, sender_id, category_id, status, type, tenant_id, created_by, created_at, updated_at) \
                   values ($1, $2, $3::int8, $4::int8, $5, $6, $7::int8, $8::int8, $9::timestamptz, $9::timestamptz) \
                   returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(title)
            .bind(content)
            .bind(sender_id)
            .bind(category_id)
            .bind(status)
            .bind(r#type)
            .bind(tenant_id.unwrap_or(0))
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert internal message failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新消息：仅非 None 字段（tenant_id 不可变，对齐 Go TenantID mixin Immutable）。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        title: Option<&str>,
        content: Option<&str>,
        sender_id: Option<i64>,
        category_id: Option<i64>,
        status: Option<&str>,
        r#type: Option<&str>,
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

        if let Some(v) = title {
            push!("title", v.to_string(), "");
        }
        if let Some(v) = content {
            push!("content", v.to_string(), "");
        }
        if let Some(v) = sender_id {
            push!("sender_id", v.to_string(), "::int8");
        }
        if let Some(v) = category_id {
            push!("category_id", v.to_string(), "::int8");
        }
        if let Some(v) = status {
            push!("status", v.to_string(), "");
        }
        if let Some(v) = r#type {
            push!("type", v.to_string(), "");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let sql = format!(
            "update internal_messages set {} where id = ${} and deleted_at is null",
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
                context: "update internal message failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }

    /// send_message 返回：消息 ID + 落库的收件记录（行 ID, 收件人 ID）+ 落库时间戳
    /// （SSE 推送载荷用，避免二次回查）。
    /// 发送消息（事务）：写消息本体（status=PUBLISHED）+ 按 targets 批量写收件记录
    /// （status=RECEIVED、received_at=now，对齐 Go newMessageRecipient）。
    /// 返回消息 ID 与收件记录行（SSE 推送用）。
    #[allow(clippy::too_many_arguments)]
    pub async fn send_message(
        &self,
        title: Option<&str>,
        content: &str,
        category_id: Option<i64>,
        r#type: &str,
        sender_id: i64,
        created_by: i64,
        targets: &[i64],
    ) -> Result<SentMessage, AppError> {
        let mut txn = self.db.begin().await.map_err(|e| AppError::Internal {
            context: "begin send message txn failed".into(),
            source: Some(Box::new(e)),
        })?;

        let now = chrono::Utc::now().to_rfc3339();
        let sql = "insert into internal_messages \
                   (title, content, sender_id, category_id, status, type, tenant_id, created_by, created_at, updated_at) \
                   values ($1, $2, $3::int8, $4::int8, 'PUBLISHED', $5, 0, $6::int8, $7::timestamptz, $7::timestamptz) \
                   returning id";
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(title)
            .bind(content)
            .bind(sender_id)
            .bind(category_id)
            .bind(r#type)
            .bind(created_by)
            .bind(now.clone())
            .fetch_one(&mut *txn)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert send message failed".into(),
                source: Some(Box::new(e)),
            })?;
        let message_id = row.0;

        let recipient_sql = "insert into internal_message_recipients \
                             (message_id, recipient_user_id, status, received_at, tenant_id, created_at, updated_at) \
                             values ($1::int8, $2::int8, 'RECEIVED', $3::timestamptz, 0, $3::timestamptz, $3::timestamptz) \
                             returning id";
        let mut recipients = Vec::with_capacity(targets.len());
        for uid in targets {
            let rrow: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(recipient_sql)
                .bind(message_id)
                .bind(uid)
                .bind(now.clone())
                .fetch_one(&mut *txn)
                .await
                .map_err(|e| AppError::Internal {
                    context: "insert message recipient failed".into(),
                    source: Some(Box::new(e)),
                })?;
            recipients.push((rrow.0, *uid));
        }

        txn.commit().await.map_err(|e| AppError::Internal {
            context: "commit send message txn failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(SentMessage {
            message_id,
            recipients,
            received_at: now,
        })
    }

    /// 撤销消息（对齐 Go RevokeMessageWithMessage）：
    /// user_id > 0 仅删该用户收件记录；user_id == 0 同事务删消息本体 + 全部收件记录，
    /// 消息不存在返回 NotFound。
    pub async fn revoke(&self, message_id: i64, user_id: i64) -> Result<u64, AppError> {
        if user_id > 0 {
            let sql = "delete from internal_message_recipients where message_id = $1 and recipient_user_id = $2";
            let res = sqlx::query::<sqlx::Any>(sql)
                .bind(message_id)
                .bind(user_id)
                .execute(&self.db)
                .await
                .map_err(|e| AppError::Internal {
                    context: "revoke message recipients failed".into(),
                    source: Some(Box::new(e)),
                })?;
            return Ok(res.rows_affected());
        }

        let mut txn = self.db.begin().await.map_err(|e| AppError::Internal {
            context: "begin revoke message txn failed".into(),
            source: Some(Box::new(e)),
        })?;

        let res = sqlx::query::<sqlx::Any>("delete from internal_messages where id = $1")
            .bind(message_id)
            .execute(&mut *txn)
            .await
            .map_err(|e| AppError::Internal {
                context: "revoke message body failed".into(),
                source: Some(Box::new(e)),
            })?;
        if res.rows_affected() == 0 {
            txn.rollback().await.map_err(|e| AppError::Internal {
                context: "rollback revoke message txn failed".into(),
                source: Some(Box::new(e)),
            })?;
            return Err(AppError::NotFound("internal message not found".into()));
        }

        sqlx::query::<sqlx::Any>("delete from internal_message_recipients where message_id = $1")
            .bind(message_id)
            .execute(&mut *txn)
            .await
            .map_err(|e| AppError::Internal {
                context: "revoke message recipients failed".into(),
                source: Some(Box::new(e)),
            })?;

        txn.commit().await.map_err(|e| AppError::Internal {
            context: "commit revoke message txn failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(0)
    }

    /// 删除消息本体 + 全部收件记录（同一事务，对齐 Go DeleteMessageWithRecipients）。
    pub async fn delete_with_recipients(&self, message_id: i64) -> Result<u64, AppError> {
        let mut txn = self.db.begin().await.map_err(|e| AppError::Internal {
            context: "begin delete message txn failed".into(),
            source: Some(Box::new(e)),
        })?;

        let res = sqlx::query::<sqlx::Any>("delete from internal_messages where id = $1")
            .bind(message_id)
            .execute(&mut *txn)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete message body failed".into(),
                source: Some(Box::new(e)),
            })?;
        if res.rows_affected() == 0 {
            txn.rollback().await.map_err(|e| AppError::Internal {
                context: "rollback delete message txn failed".into(),
                source: Some(Box::new(e)),
            })?;
            return Err(AppError::NotFound("internal message not found".into()));
        }

        sqlx::query::<sqlx::Any>("delete from internal_message_recipients where message_id = $1")
            .bind(message_id)
            .execute(&mut *txn)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete message recipients failed".into(),
                source: Some(Box::new(e)),
            })?;

        txn.commit().await.map_err(|e| AppError::Internal {
            context: "commit delete message txn failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(0)
    }
}
