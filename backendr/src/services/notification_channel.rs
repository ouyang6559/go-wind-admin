// notification_channel service：通知渠道 CRUD 业务 + 测试邮件发送。
// 对齐 Go `notification_channel_service.go`：密码 AES-GCM 加密落库、响应永不回传
// 密码（仅 hasPassword 标识）、test email 仅 EMAIL 渠道且渠道启用时允许。

use serde::Serialize;

use crate::error::AppError;
use crate::repos::notification_channel::NotificationChannelRepo;
use crate::state::AppState;

/// DTO（对齐 proto NotificationChannel，camelCase json_name）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationChannelDto {
    pub id: i64,
    pub name: String,
    /// EMAIL | WEBHOOK
    pub r#type: String,
    pub smtp_host: String,
    pub smtp_port: Option<i64>,
    pub smtp_username: String,
    /// 密码不回传，仅标识是否已设置
    pub has_password: bool,
    pub smtp_from: String,
    /// NONE | START_TLS | SSL
    pub smtp_tls: String,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remark: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

pub struct NotificationChannelService {
    pub repo: NotificationChannelRepo,
    pub state: AppState,
}

impl NotificationChannelService {
    pub fn from_state(state: &AppState) -> Result<Self, AppError> {
        let db = state.db.clone().ok_or_else(|| AppError::Internal {
            context: "database not configured; notification channel unavailable".into(),
            source: None,
        })?;
        Ok(Self {
            repo: NotificationChannelRepo::new(db),
            state: state.clone(),
        })
    }
}

pub use crate::repos::notification_channel::ChannelRow;

/// 渠道行 → DTO（脱敏）
pub fn row_to_dto(row: &ChannelRow) -> NotificationChannelDto {
    NotificationChannelDto {
        id: row.id,
        name: row.name.clone(),
        r#type: row.r#type.clone(),
        smtp_host: row.smtp_host.clone().unwrap_or_default(),
        smtp_port: row.smtp_port,
        smtp_username: row.smtp_username.clone().unwrap_or_default(),
        has_password: row.smtp_password.is_some(),
        smtp_from: row.smtp_from.clone().unwrap_or_default(),
        smtp_tls: row.smtp_tls.clone().unwrap_or_else(|| "START_TLS".into()),
        enabled: row.status.as_deref() == Some("ON"),
        remark: row.remark.clone(),
        created_by: row.created_by,
        updated_by: row.updated_by,
        created_at: row.created_at.clone(),
        updated_at: row.updated_at.clone(),
    }
}

/// 校验渠道名（Create 用）
pub fn validate_name(name: &str) -> Result<(), AppError> {
    if name.trim().is_empty() {
        return Err(AppError::Validation("channel name is required".into()));
    }
    Ok(())
}
