// notification_channel 模块 handlers（wire 对齐 Go：裸 DTO，Create 返回实体）。

use axum::extract::{Path, Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use std::collections::HashMap;

use crate::error::AppError;
use crate::state::AppState;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::response::{json_empty, json_ok, ListResponse};
use crate::services::notification_channel::{
    row_to_dto, NotificationChannelService,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateChannelBody {
    #[serde(default)]
    pub data: Option<ChannelData>,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelData {
    pub id: Option<i64>,
    pub name: Option<String>,
    #[serde(default, rename = "type")]
    pub r#type: Option<String>,
    #[serde(default)]
    pub smtp_host: Option<String>,
    #[serde(default)]
    pub smtp_port: Option<i64>,
    #[serde(default)]
    pub smtp_username: Option<String>,
    #[serde(default)]
    pub smtp_from: Option<String>,
    #[serde(default)]
    pub smtp_tls: Option<String>,
    #[serde(default)]
    pub enabled: Option<bool>,
    #[serde(default)]
    pub remark: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChannelBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub data: Option<ChannelData>,
    #[serde(default)]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendTestEmailBody {
    #[serde(default)]
    pub id: Option<i64>,
    #[serde(default)]
    pub recipient: Option<String>,
}

pub async fn notification_channel_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let service = NotificationChannelService::from_state(&state)?;
    let lq = ListQuery::parse(&params, &["name"])?;
    // 渠道列表过滤仅支持 name contains（对齐 Go 通用分页路径的主要用法）
    let _ = lq.filters;
    let (rows, total) = service.repo.list(lq.paging.offset(), lq.paging.limit()).await?;
    Ok(json_ok(ListResponse::new(
        rows.iter().map(row_to_dto).collect(),
        total,
    )))
}

pub async fn notification_channel_get(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    if id == 0 {
        return Err(AppError::Validation("id is required".into()));
    }
    let service = NotificationChannelService::from_state(&state)?;
    let row = service
        .repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("notification channel not found".into()))?;
    Ok(json_ok(row_to_dto(&row)))
}

pub async fn notification_channel_create(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<CreateChannelBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let name = data.name.clone().unwrap_or_default();
    crate::services::notification_channel::validate_name(&name)?;

    let service = NotificationChannelService::from_state(&state)?;
    let password_enc = match body.password.as_deref() {
        Some(p) if !p.is_empty() => Some(crate::crypto::encrypt_channel_secret(p)?),
        _ => None,
    };
    let channel_type = data.r#type.clone().unwrap_or_else(|| "EMAIL".into());
    let smtp_tls = data.smtp_tls.clone().unwrap_or_else(|| "START_TLS".into());

    let new_id = service
        .repo
        .create(
            name.trim(),
            &channel_type,
            data.smtp_host.as_deref(),
            data.smtp_port,
            data.smtp_username.as_deref(),
            password_enc.as_deref(),
            data.smtp_from.as_deref(),
            &smtp_tls,
            data.enabled.unwrap_or(true),
            data.remark.as_deref(),
            operator.user_id,
        )
        .await?;

    // 对齐 Go：Create 返回创建后的完整实体（Get 回查）
    let row = service.repo.get(new_id).await?.ok_or_else(|| AppError::Internal {
        context: "queryback created channel failed".into(),
        source: None,
    })?;
    Ok(json_ok(row_to_dto(&row)))
}

pub async fn notification_channel_update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    operator: Operator,
    Json(body): Json<UpdateChannelBody>,
) -> Result<impl IntoResponse, AppError> {
    let data = body.data.ok_or_else(|| AppError::Validation("data is required".into()))?;
    let id = body.id.unwrap_or(id);
    if id == 0 {
        return Err(AppError::Validation("id is required".into()));
    }
    // 对齐 Go：仅当提供了 name 才做非空校验（留空不更新）
    if let Some(n) = data.name.as_deref() {
        crate::services::notification_channel::validate_name(n)?;
    }
    let service = NotificationChannelService::from_state(&state)?;
    // password 非空才更新（留空=保留）
    let password_enc = match body.password.as_deref() {
        Some(p) if !p.is_empty() => Some(crate::crypto::encrypt_channel_secret(p)?),
        _ => None,
    };
    service
        .repo
        .update(
            id,
            data.name.as_deref(),
            data.smtp_host.as_deref(),
            data.smtp_port,
            data.smtp_username.as_deref(),
            password_enc.as_deref(),
            data.smtp_from.as_deref(),
            data.smtp_tls.as_deref(),
            data.enabled,
            data.remark.as_deref(),
            operator.user_id,
        )
        .await?;
    Ok(json_empty())
}

pub async fn notification_channel_delete(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    if id == 0 {
        return Err(AppError::Validation("id is required".into()));
    }
    let service = NotificationChannelService::from_state(&state)?;
    service.repo.delete(id).await?;
    Ok(json_empty())
}

pub async fn notification_channel_send_test_email(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    operator: Operator,
    Json(body): Json<SendTestEmailBody>,
) -> Result<impl IntoResponse, AppError> {
    let id = body.id.unwrap_or(id);
    if id == 0 {
        return Err(AppError::Validation("id is required".into()));
    }
    let recipient = body
        .recipient
        .clone()
        .filter(|r| !r.trim().is_empty())
        .ok_or_else(|| AppError::Validation("recipient is required".into()))?;

    let service = NotificationChannelService::from_state(&state)?;
    let row = service
        .repo
        .get(id)
        .await?
        .ok_or_else(|| AppError::NotFound("notification channel not found".into()))?;
    if row.r#type != "EMAIL" {
        return Err(AppError::Validation(
            "test email is only available for EMAIL channels".into(),
        ));
    }
    if row.status.as_deref() != Some("ON") {
        return Err(AppError::Validation("notification channel is disabled".into()));
    }

    // 解密 SMTP 密码（enc: 前缀的 AES-GCM；明文直存原样返回）
    let smtp_password = match row.smtp_password.as_deref() {
        Some(p) if !p.is_empty() => crate::crypto::decrypt_channel_secret(p)?,
        _ => String::new(),
    };
    let cfg = crate::mailer::SmtpConfig {
        host: row.smtp_host.clone().unwrap_or_default(),
        port: row.smtp_port.unwrap_or(0) as u16,
        username: row.smtp_username.clone().unwrap_or_default(),
        password: smtp_password,
        from: row.smtp_from.clone().unwrap_or_default(),
        tls: row.smtp_tls.clone().unwrap_or_else(|| "START_TLS".into()),
    };

    let subject = "GoWind Admin 通知渠道测试邮件";
    let body_text = format!(
        "这是一封来自 GoWind Admin 的测试邮件。\n如果您收到了它，说明渠道 [{}] 配置可用。\n操作人用户 ID: {}\n",
        row.id, operator.user_id
    );
    crate::mailer::send_mail(&cfg, recipient.trim(), subject, &body_text)
        .await
        .map_err(|e| match e {
            // 发送失败对齐 Go：400 + send test email failed
            AppError::Validation(msg) => {
                AppError::Validation(format!("send test email failed: {}", msg.strip_prefix("send mail failed: ").unwrap_or(&msg)))
            }
            other => other,
        })?;
    Ok(json_empty())
}
