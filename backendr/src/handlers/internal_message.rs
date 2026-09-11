// internal_message 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略、枚举为 proto 名）。
// status 枚举 DRAFT/PUBLISHED/SCHEDULED/REVOKED/ARCHIVED/DELETED、type 枚举 NOTIFICATION/PRIVATE/GROUP。
// SendMessage 对齐 Go：先落消息本体（PUBLISHED）再写收件记录（事务内）；
// targetAll 全员广播（Rust 端同步拉全量用户，Go 端为异步 fan-out，落库效果一致）；
// RevokeMessage：userId>0 仅撤销该用户，userId=0 全局撤销（同事务删消息本体+全部收件记录）。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::internal_message::{InternalMessageRepo, InternalMessageRow};
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

/// 可过滤/排序的白名单列
const MESSAGE_COLUMNS: &[&str] = &[
    "id", "title", "content", "status", "type", "sender_id", "category_id", "tenant_id",
    "created_by", "updated_by", "created_at", "updated_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InternalMessageDto {
    pub id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// DRAFT | PUBLISHED | SCHEDULED | REVOKED | ARCHIVED | DELETED
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// NOTIFICATION | PRIVATE | GROUP
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at: Option<String>,
}

pub fn to_dto(row: &InternalMessageRow) -> InternalMessageDto {
    InternalMessageDto {
        id: row.id,
        title: row.title.clone(),
        content: row.content.clone(),
        status: row.status.clone(),
        r#type: row.r#type.clone(),
        sender_id: row.sender_id,
        sender_name: row.sender_name.clone(),
        category_id: row.category_id,
        category_name: row.category_name.clone(),
        tenant_id: row.tenant_id,
        tenant_name: row.tenant_name.clone(),
        created_by: row.created_by,
        updated_by: row.updated_by,
        deleted_by: row.deleted_by,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
        deleted_at: row.deleted_at.clone(),
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InternalMessageData {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub sender_id: Option<i64>,
    #[serde(default)]
    pub category_id: Option<i64>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub r#type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInternalMessageBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<InternalMessageData>,
    #[serde(default)]
    pub allow_missing: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageBody {
    /// NOTIFICATION | PRIVATE | GROUP（proto 非 optional，缺省按零值 NOTIFICATION）
    #[serde(default)]
    pub r#type: Option<String>,
    #[serde(default)]
    pub recipient_user_id: Option<i64>,
    #[serde(default)]
    pub conversation_id: Option<i64>,
    #[serde(default)]
    pub category_id: Option<i64>,
    #[serde(default)]
    pub target_user_ids: Option<Vec<i64>>,
    #[serde(default)]
    pub target_all: Option<bool>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevokeMessageBody {
    #[serde(default)]
    pub message_id: Option<i64>,
    /// 0 = 全局撤销（删消息本体+全部收件记录）；>0 = 仅撤销该用户收件记录
    #[serde(default)]
    pub user_id: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageResponse {
    pub message_id: i64,
}

/// status 枚举归一（InternalMessage.Status，proto 枚举名）
fn normalize_status(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "DRAFT" | "PUBLISHED" | "SCHEDULED" | "REVOKED" | "ARCHIVED" | "DELETED" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid message status: {v}"))),
    }
}

/// type 枚举归一（InternalMessage.Type，proto 枚举名）
fn normalize_type(v: &str) -> Result<String, AppError> {
    let up = v.to_ascii_uppercase();
    match up.as_str() {
        "NOTIFICATION" | "PRIVATE" | "GROUP" => Ok(up),
        _ => Err(AppError::Validation(format!("invalid message type: {v}"))),
    }
}

pub async fn internal_message_list_message(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, MESSAGE_COLUMNS)?;
    let repo = InternalMessageRepo::new(crate::handlers::script::db_of(&state)?);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        let mut filters = lq.filters.clone();
        for f in &mut filters {
            f.column = format!("m.{}", f.column);
        }
        where_clause = format!(" and {}", crate::query::compile_where(&filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, MESSAGE_COLUMNS)?;
        order_parts.push(format!("\"m\".\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let order_by = order_parts.join(", ");
    let (rows, total) = repo
        .list(lq.paging.offset(), lq.paging.limit(), &where_clause, &bind, &order_by)
        .await?;
    Ok(json_ok(ListResponse::new(
        rows.iter().map(to_dto).collect(),
        total,
    )))
}

pub async fn internal_message_get_message(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = InternalMessageRepo::new(crate::handlers::script::db_of(&state)?);
    let row = repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("internal message not found".into()))?;
    Ok(json_ok(to_dto(&row)))
}

pub async fn internal_message_update_message(
    State(state): State<AppState>,
    Path(path_id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateInternalMessageBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(path_id);
    let data = body
        .data
        .ok_or_else(|| AppError::Validation("data is required".into()))?;

    let repo = InternalMessageRepo::new(crate::handlers::script::db_of(&state)?);

    // allowMissing=true 且不存在 → 转为 Create（对齐 Go；status 缺省 DRAFT、type 缺省 NOTIFICATION）
    if body.allow_missing.unwrap_or(false) && repo.get(id).await?.is_none() {
        let status = data
            .status
            .as_deref()
            .map(normalize_status)
            .transpose()?
            .unwrap_or_else(|| "DRAFT".into());
        let r#type = data
            .r#type
            .as_deref()
            .map(normalize_type)
            .transpose()?
            .unwrap_or_else(|| "NOTIFICATION".into());
        repo.create(
            data.title.as_deref(),
            data.content.as_deref(),
            data.sender_id.unwrap_or(0),
            data.category_id,
            &status,
            &r#type,
            None,
            operator.user_id,
        )
        .await?;
        return Ok(json_empty());
    }

    // 提供 status/type 时做枚举归一
    let status = data.status.as_deref().map(normalize_status).transpose()?;
    let r#type = data.r#type.as_deref().map(normalize_type).transpose()?;

    let updated = repo
        .update(
            id,
            data.title.as_deref(),
            data.content.as_deref(),
            data.sender_id,
            data.category_id,
            status.as_deref(),
            r#type.as_deref(),
            operator.user_id,
        )
        .await?;
    if updated == 0 {
        return Err(AppError::NotFound("internal message not found".into()));
    }
    Ok(json_empty())
}

pub async fn internal_message_delete_message(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = InternalMessageRepo::new(crate::handlers::script::db_of(&state)?);
    repo.delete_with_recipients(id).await?;
    Ok(json_empty())
}

pub async fn internal_message_revoke_message(
    State(state): State<AppState>,
    _operator: Operator,
    Json(body): Json<RevokeMessageBody>,
) -> Result<impl IntoResponse, AppError> {
    let message_id = body
        .message_id
        .filter(|v| *v > 0)
        .ok_or_else(|| AppError::Validation("messageId is required".into()))?;
    let user_id = body.user_id.unwrap_or(0);
    let repo = InternalMessageRepo::new(crate::handlers::script::db_of(&state)?);
    repo.revoke(message_id, user_id).await?;
    Ok(json_empty())
}

pub async fn internal_message_send_message(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<SendMessageBody>,
) -> Result<impl IntoResponse, AppError> {
    let r#type = body
        .r#type
        .as_deref()
        .map(normalize_type)
        .transpose()?
        .unwrap_or_else(|| "NOTIFICATION".into());
    let content = body.content.clone().unwrap_or_default();

    let repo = InternalMessageRepo::new(crate::handlers::script::db_of(&state)?);

    // 目标收件人：targetAll 全员广播；否则 recipientUserId 优先，其次 targetUserIds
    let targets: Vec<i64> = if body.target_all.unwrap_or(false) {
        repo.all_user_ids().await?
    } else if let Some(uid) = body.recipient_user_id.filter(|v| *v > 0) {
        vec![uid]
    } else {
        let mut seen: Vec<i64> = Vec::new();
        for uid in body.target_user_ids.unwrap_or_default() {
            if uid > 0 && !seen.contains(&uid) {
                seen.push(uid);
            }
        }
        seen
    };

    let message_id = repo
        .send_message(
            body.title.as_deref(),
            &content,
            body.category_id,
            &r#type,
            operator.user_id,
            operator.user_id,
            &targets,
        )
        .await?;
    Ok(json_ok(SendMessageResponse { message_id }))
}
