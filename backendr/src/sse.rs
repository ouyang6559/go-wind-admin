//! SSE 网关（对齐 Go `kratos-transport/transport/sse` + `InternalMessageService` 的接入方式）。
//!
//! - 独立监听（默认 `0.0.0.0:7789`，`GW_ADMIN_SSE_ADDR` 配置，`off`/空 关闭），路径 `/events`
//! - 订阅：`GET /events?stream={userId}`；token 按 Go `DefaultTokenExtractor` 顺序取
//!   `Authorization: Bearer` → `X-Token` → `?token=`，验签 + 黑名单 + stream==uid 越权校验
//!   （对齐 Go `HandleAuthorize`；Go 端 Forbidden 经 `errors.Is(sse.ErrForbidden)` 判定恒为
//!   false，实际所有鉴权失败都落 401，这里保持一致）
//! - 事件帧对齐 Go `writeData` 顺序：`id:` → `data:` → `event:` → 空行；无 keepalive（Go 同）
//! - publish 为 try 语义（对齐 Go `TryPublish`）：流不存在（无订阅者）或缓冲满立即跳过，
//!   不阻塞调用方；离线用户重连后从收件箱补取，推送失败不影响落库

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::{Arc, Mutex};

use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{header, HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use serde::Serialize;
use tokio::sync::broadcast;

use crate::state::AppState;

/// 每条流的事件缓冲：慢客户端满时 try_publish 丢帧（对齐 Go「缓冲已满跳过」）
const STREAM_BUFFER: usize = 64;

/// CORS 放行来源（Go Server 默认 `*`）
const CORS_ALLOW_ORIGIN: &str = "*";

/// 站内信实时事件名（Go `publishNotification` 固定 `notification`）
const EVENT_NOTIFICATION: &str = "notification";

/// 按 streamID 的内存流注册表。断连后不主动清理，惰性在 try_publish 时回收
/// （无订阅者的流被移除），避免后台任务周期性触发的重建开销。
#[derive(Default)]
pub struct SseHub {
    streams: Mutex<HashMap<String, broadcast::Sender<String>>>,
}

impl SseHub {
    pub fn new() -> Self {
        Self::default()
    }

    /// 订阅（流不存在则创建，对齐 Go autoStream=true）
    fn subscribe(&self, stream_id: &str) -> broadcast::Receiver<String> {
        let mut map = self.streams.lock().expect("sse hub lock poisoned");
        map.entry(stream_id.to_string())
            .or_insert_with(|| broadcast::channel(STREAM_BUFFER).0)
            .subscribe()
    }

    /// try-publish：流不存在 / 无订阅者 / 缓冲满 → false（对齐 Go TryPublish）。
    /// 顺带回收无订阅者的死流。
    pub fn try_publish(&self, stream_id: &str, frame: &str) -> bool {
        let mut map = self.streams.lock().expect("sse hub lock poisoned");
        match map.get(stream_id) {
            Some(tx) if tx.receiver_count() > 0 => {
                let sent = tx.send(frame.to_string()).is_ok();
                if !sent && tx.receiver_count() == 0 {
                    map.remove(stream_id);
                }
                sent
            }
            _ => {
                map.remove(stream_id);
                false
            }
        }
    }

    pub fn subscriber_streams(&self) -> usize {
        let map = self.streams.lock().expect("sse hub lock poisoned");
        map.values().filter(|tx| tx.receiver_count() > 0).count()
    }
}

/// 站内信推送事件载荷（对齐 Go `publishNotification` 序列化的
/// `InternalMessageRecipient` protojson：camelCase、枚举为名字、可选字段缺省省略）。
/// 直发路径带 id（Go `recipient.Id = entity.Id`）；广播路径 Go 不带 → `Option`。
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationEvent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub message_id: i64,
    pub recipient_user_id: i64,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub received_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// 渲染一帧 SSE 事件（Go 写序：id → data → event → 空行）。
fn render_frame(event: &str, data: &str) -> String {
    let id = uuid::Uuid::new_v4().simple().to_string();
    format!("id: {id}\ndata: {data}\nevent: {event}\n\n")
}

/// 推送一条站内信通知（尽力而为）：stream=userId，事件名 `notification`。
/// 跳过时 debug 日志（对齐 Go `sse try publish skipped`）。
pub fn publish_notification(hub: &SseHub, ev: &NotificationEvent) -> bool {
    let stream_id = ev.recipient_user_id.to_string();
    let data = match serde_json::to_string(ev) {
        Ok(d) => d,
        Err(e) => {
            tracing::debug!(error = %e, "sse marshal notification failed, skip push");
            return false;
        }
    };
    let frame = render_frame(EVENT_NOTIFICATION, &data);
    if !hub.try_publish(&stream_id, &frame) {
        tracing::debug!(
            user = ev.recipient_user_id,
            stream = %stream_id,
            "sse try publish skipped (stream not exist or buffer full)"
        );
        return false;
    }
    true
}

/// SSE 网关路由（挂独立监听：`GET/OPTIONS /events`）
pub fn build_router() -> Router<AppState> {
    Router::new().route("/events", get(events).options(events_preflight))
}

/// CORS 预检（对齐 Go ServeHTTP 的 OPTIONS 分支：204 + 固定头）
async fn events_preflight() -> Response {
    Response::builder()
        .status(StatusCode::NO_CONTENT)
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, CORS_ALLOW_ORIGIN)
        .header(header::ACCESS_CONTROL_ALLOW_METHODS, "GET, OPTIONS")
        .header(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            "Content-Type, Authorization, X-Token, Last-Event-ID",
        )
        .header(header::ACCESS_CONTROL_MAX_AGE, "86400")
        .body(Body::empty())
        .expect("build preflight response")
}

/// 纯文本错误（Go `writeError`：body 为错误文本）
fn sse_error(status: StatusCode, msg: &str) -> Response {
    (status, msg.to_string()).into_response()
}

/// 提取 token（Go DefaultTokenExtractor 顺序：Bearer → X-Token → ?token=）
fn extract_token(headers: &HeaderMap, query_token: Option<&String>) -> Option<String> {
    if let Some(v) = headers.get(header::AUTHORIZATION).and_then(|v| v.to_str().ok()) {
        if let Some(tok) = crate::auth::bearer_token(Some(v)) {
            return Some(tok);
        }
    }
    if let Some(v) = headers
        .get("X-Token")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        return Some(v.to_string());
    }
    query_token
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

/// SSE 订阅端点（鉴权逻辑对齐 Go InternalMessageService.HandleAuthorize）。
async fn events(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    // 1. 鉴权：验签 + 黑名单（对齐 Go Authenticate：blocked/invalid → 401）
    let token = match extract_token(&headers, params.get("token")) {
        Some(t) => t,
        None => return sse_error(StatusCode::UNAUTHORIZED, "token is empty"),
    };
    let claims = match crate::auth::verify_access_token(&state.jwt_secret, &token) {
        Ok(c) => c,
        Err(_) => return sse_error(StatusCode::UNAUTHORIZED, "token is invalid"),
    };
    if crate::auth::is_jti_blacklisted(&state, &claims.jti).await {
        return sse_error(StatusCode::UNAUTHORIZED, "token is blocked");
    }

    // 2. stream 参数：缺失 500（Go 先鉴权后查 stream，对齐其顺序）
    let Some(stream) = params.get("stream").filter(|v| !v.is_empty()) else {
        return sse_error(StatusCode::INTERNAL_SERVER_ERROR, "Please specify a stream!");
    };

    // 3. 越权校验：stream 必须等于 token 的 userId（Go：mismatch → Forbidden → 实际 401）
    let Ok(stream_uid) = stream.parse::<i64>() else {
        return sse_error(StatusCode::UNAUTHORIZED, "stream user mismatch");
    };
    if stream_uid != claims.uid {
        return sse_error(StatusCode::UNAUTHORIZED, "stream user mismatch");
    }

    // 4. 订阅并转为字节流（首帧前响应头已随 200 flush，对齐 Go 行为）
    let rx = state.sse.subscribe(stream);
    let body = Body::from_stream(futures::stream::unfold(rx, |mut rx| async move {
        // 流关闭（hub 回收）→ 结束响应
        rx.recv().await.ok().map(|frame| {
            let chunk: Result<axum::body::Bytes, Infallible> = Ok(frame.into());
            (chunk, rx)
        })
    }));

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/event-stream")
        .header(header::CACHE_CONTROL, "no-cache")
        .header(header::CONNECTION, "keep-alive")
        .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, CORS_ALLOW_ORIGIN)
        .body(body)
        .expect("build sse response")
}
