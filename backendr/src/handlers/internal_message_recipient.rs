// internal_message_recipient 模块 handlers（收件箱/已读/删除/状态回执）。

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::internal_message_recipient::{InternalMessageRecipientRepo, RecipientRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

const INBOX_COLUMNS: &[&str] = &["id", "message_id", "status", "received_at", "read_at"];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipientDto {
    pub id: i64,
    pub message_id: i64,
    pub recipient_user_id: i64,
    /// SENT | RECEIVED | READ | REVOKED | DELETED（对齐 proto 枚举名）
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub received_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub read_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<i64>,
}

fn to_dto(r: &RecipientRow) -> RecipientDto {
    RecipientDto {
        id: r.id,
        message_id: r.message_id,
        recipient_user_id: r.recipient_user_id,
        status: r.status.clone(),
        received_at: r.received_at.clone(),
        read_at: r.read_at.clone(),
        title: r.title.clone(),
        content: r.content.clone(),
        tenant_id: Some(r.tenant_id),
    }
}

/// GET /internal-message/inbox
pub async fn internal_message_recipient_list_user_inbox(
    State(state): State<AppState>,
    operator: Operator,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, INBOX_COLUMNS)?;
    let repo = InternalMessageRecipientRepo::new(crate::handlers::task::db_of(&state)?);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        where_clause = format!(" and {}", crate::query::compile_where(&lq.filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, INBOX_COLUMNS)?;
        order_parts.push(format!("r.{c} {}", if *desc { "desc" } else { "asc" }));
    }
    let (rows, total) = repo
        .list_inbox(
            operator.user_id,
            lq.paging.offset(),
            lq.paging.limit(),
            &where_clause,
            &bind,
            &order_parts.join(", "),
        )
        .await?;
    Ok(json_ok(ListResponse::new(
        rows.iter().map(to_dto).collect(),
        total,
    )))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxOpBody {
    #[serde(default)]
    pub user_id: Option<i64>,
    /// 空数组或缺省 = 全量（对齐 Go：read 空=全部未读、delete 空=全部）
    #[serde(default)]
    pub recipient_ids: Option<Vec<i64>>,
}

/// POST /internal-message/read
pub async fn internal_message_recipient_mark_notification_as_read(
    State(state): State<AppState>,
    _operator: Operator,
    Json(body): Json<InboxOpBody>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = body
        .user_id
        .filter(|v| *v != 0)
        .ok_or_else(|| AppError::Validation("userId is required".into()))?;
    let ids = body.recipient_ids.clone().unwrap_or_default();
    let repo = InternalMessageRecipientRepo::new(crate::handlers::task::db_of(&state)?);
    repo.mark_read(user_id, &ids).await?;
    Ok(json_empty())
}

/// POST /internal-message/inbox/delete
pub async fn internal_message_recipient_delete_notification_from_inbox(
    State(state): State<AppState>,
    _operator: Operator,
    Json(body): Json<InboxOpBody>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = body
        .user_id
        .filter(|v| *v != 0)
        .ok_or_else(|| AppError::Validation("userId is required".into()))?;
    let ids = body.recipient_ids.clone().unwrap_or_default();
    let repo = InternalMessageRecipientRepo::new(crate::handlers::task::db_of(&state)?);
    repo.delete_from_inbox(user_id, &ids).await?;
    Ok(json_empty())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarkStatusBody {
    #[serde(default)]
    pub user_id: Option<i64>,
    #[serde(default)]
    pub recipient_ids: Option<Vec<i64>>,
    /// SENT | RECEIVED | READ | REVOKED | DELETED
    #[serde(default)]
    pub new_status: Option<String>,
}

/// POST /internal-message/status —— recipientIds 必填非空（与 read/delete 的空=全量语义不同）。
pub async fn internal_message_recipient_mark_notifications_status(
    State(state): State<AppState>,
    _operator: Operator,
    Json(body): Json<MarkStatusBody>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = body
        .user_id
        .filter(|v| *v != 0)
        .ok_or_else(|| AppError::Validation("userId is required".into()))?;
    let ids = body.recipient_ids.clone().unwrap_or_default();
    let raw_status = body
        .new_status
        .clone()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| AppError::Validation("newStatus is required".into()))?;
    // 兼容枚举名与数字
    let status = normalize_status(&raw_status)?;

    let repo = InternalMessageRecipientRepo::new(crate::handlers::task::db_of(&state)?);
    repo.mark_status(user_id, &ids, status).await?;
    Ok(json_empty())
}

fn normalize_status(raw: &str) -> Result<&'static str, AppError> {
    Ok(match raw.to_ascii_uppercase().as_str() {
        "SENT" | "0" => "SENT",
        "RECEIVED" | "1" => "RECEIVED",
        "READ" | "2" => "READ",
        "REVOKED" | "3" => "REVOKED",
        "DELETED" | "4" => "DELETED",
        other => {
            return Err(AppError::Validation(format!("invalid newStatus: {other}")));
        }
    })
}

/// 兜底：防止 serde_json::Value 引入未用告警（占位，无运行时作用）
pub fn _unused(_: Option<Value>) {}
