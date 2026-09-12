// online_session 模块 handlers。
// 数据源：Redis 会话元数据（gw:session:meta:*，登录链路写入）。
// force-logout 语义对齐 Go：删会话 + 黑名单 jti（黑名单由 Operator 提取器强制执行）。

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::auth::{self, SessionMeta};
use crate::error::AppError;
use crate::middleware::Operator;
use crate::response::{json_empty, json_ok, ListResponse};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OnlineSessionDto {
    // Go 对齐：/sessions（管理员）不输出 current；/my-sessions 恒输出（含 false）。
    // 用 Option 区分——管理员视图传 None（省略），个人视图传 Some。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current: Option<bool>,
    pub jti: String,
    pub user_id: i64,
    pub username: String,
    pub tenant_id: i64,
    /// admin | app
    pub client_type: String,
    pub ip_address: String,
    pub user_agent: String,
    pub device_id: String,
    pub login_at: String,
}

fn to_dto(m: &SessionMeta, current_jti: Option<&str>) -> OnlineSessionDto {
    OnlineSessionDto {
        current: current_jti.map(|jti| !m.jti.is_empty() && m.jti == jti),
        jti: m.jti.clone(),
        user_id: m.uid,
        username: m.username.clone(),
        tenant_id: m.tenant_id,
        client_type: m.client_type.clone(),
        ip_address: m.ip.clone(),
        user_agent: m.ua.clone(),
        device_id: m.dev.clone(),
        login_at: m.login_at.clone(),
    }
}

/// GET /online-session/sessions?keyword=&page=&pageSize=（内存分页，loginAt 倒序）
pub async fn sessions_list(
    State(state): State<AppState>,
    _operator: Operator,
    Query(params): Query<HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let keyword = params.get("keyword").cloned().unwrap_or_default();
    let page = params.get("page").and_then(|v| v.parse::<u64>().ok()).unwrap_or(1).max(1);
    let page_size = params
        .get("pageSize")
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(20)
        .max(1);

    let metas = auth::scan_session_metas(&state).await?;
    let mut items: Vec<OnlineSessionDto> = metas
        .iter()
        .filter(|m| {
            if keyword.is_empty() {
                true
            } else {
                m.username.to_lowercase().contains(&keyword.to_lowercase())
                    || m.ip.to_lowercase().contains(&keyword.to_lowercase())
            }
        })
        .map(|m| to_dto(m, None))
        .collect();
    items.sort_by(|a, b| b.login_at.cmp(&a.login_at));

    let total = items.len() as u64;
    let start = (page - 1) * page_size;
    let items = if start >= total {
        Vec::new()
    } else {
        items[start as usize..((start + page_size).min(total) as usize)].to_vec()
    };
    Ok(json_ok(ListResponse::new(items, total)))
}

/// GET /online-session/my-sessions（不分页，current 标记当前 jti）
pub async fn my_sessions_list(
    State(state): State<AppState>,
    operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let metas = auth::scan_session_metas(&state).await?;
    let mut items: Vec<OnlineSessionDto> = metas
        .iter()
        .filter(|m| m.uid == operator.user_id)
        .map(|m| to_dto(m, Some(&operator.jti)))
        .collect();
    items.sort_by(|a, b| b.login_at.cmp(&a.login_at));
    let total = items.len() as u64;
    Ok(json_ok(ListResponse::new(items, total)))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForceLogoutBody {
    #[serde(default)]
    pub client_type: Option<String>,
    #[serde(default)]
    pub user_id: Option<i64>,
    #[serde(default)]
    pub jti: Option<String>,
}

/// POST /online-session/force-logout：删会话 + 黑名单 jti（必须同时给 userId + jti）
pub async fn force_logout_session(
    State(state): State<AppState>,
    _operator: Operator,
    Json(body): Json<ForceLogoutBody>,
) -> Result<impl IntoResponse, AppError> {
    let uid = body
        .user_id
        .ok_or_else(|| AppError::Validation("userId is required".into()))?;
    let jti = body
        .jti
        .clone()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| AppError::Validation("jti is required".into()))?;

    match body.client_type.as_deref() {
        Some(ct) if !ct.trim().is_empty() => {
            auth::revoke_session_by_jti(&state, ct, uid, jti.trim(), None).await?;
        }
        _ => {
            // 未指定 clientType：对 admin 与 app 各执行一次（对齐 Go）
            for ct in ["admin", "app"] {
                auth::revoke_session_by_jti(&state, ct, uid, jti.trim(), None).await?;
            }
        }
    }
    Ok(json_empty())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RevokeMyBody {
    #[serde(default)]
    pub client_type: Option<String>,
    #[serde(default)]
    pub jti: Option<String>,
}

/// POST /online-session/my-sessions/revoke：撤销自己的会话（元数据不存在 → 404）
pub async fn revoke_my_session(
    State(state): State<AppState>,
    operator: Operator,
    Json(body): Json<RevokeMyBody>,
) -> Result<impl IntoResponse, AppError> {
    let jti = body
        .jti
        .clone()
        .filter(|s| !s.trim().is_empty())
        .ok_or_else(|| AppError::Validation("jti is required".into()))?;

    // 元数据校验（对齐 Go：不存在 → 404 session not found or already offline）
    let metas = auth::scan_session_metas(&state).await?;
    let mine = metas
        .iter()
        .any(|m| m.uid == operator.user_id && m.jti == jti.trim());
    if !mine {
        return Err(AppError::NotFound(
            "session not found or already offline".into(),
        ));
    }

    let client_type = body
        .client_type
        .clone()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| operator.client_type.clone());
    auth::revoke_session_by_jti(&state, &client_type, operator.user_id, jti.trim(), None).await?;
    Ok(json_empty())
}
