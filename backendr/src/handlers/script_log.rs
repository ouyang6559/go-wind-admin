// script_log 模块 handlers（List / Count / Purge）。

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::ListQuery;
use crate::repos::script_log::{ScriptLogRepo, ScriptLogRow};
use crate::response::{json_ok, ListResponse};
use crate::state::AppState;

const LOG_COLUMNS: &[&str] = &[
    "id", "script_id", "script_name", "language", "trigger_type", "hook_point",
    "version", "success", "duration_ms", "error", "created_at",
];

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScriptLogDto {
    pub id: i64,
    pub script_id: i64,
    pub script_name: String,
    pub language: String,
    pub trigger_type: String,
    pub hook_point: String,
    pub version: i64,
    pub success: bool,
    pub duration_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

fn to_dto(r: &ScriptLogRow) -> ScriptLogDto {
    ScriptLogDto {
        id: r.id,
        script_id: r.script_id,
        script_name: r.script_name.clone(),
        language: r.language.clone(),
        trigger_type: r.trigger_type.clone(),
        hook_point: r.hook_point.clone(),
        version: r.version,
        success: r.success,
        duration_ms: r.duration_ms,
        error: r.error.clone(),
        created_at: r.created_at.clone(),
    }
}

pub async fn script_log_list(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let lq = ListQuery::parse(&params, LOG_COLUMNS)?;
    let repo = ScriptLogRepo::new(crate::handlers::script::db_of(&state)?);
    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        where_clause = format!(" and {}", crate::query::compile_where(&lq.filters, &mut bind));
    }
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, LOG_COLUMNS)?;
        order_parts.push(format!("\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let (rows, total) = repo
        .list(
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

pub async fn script_log_count(
    State(state): State<AppState>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let repo = ScriptLogRepo::new(crate::handlers::script::db_of(&state)?);
    let count = repo.count().await?;
    Ok(json_ok(serde_json::json!({ "count": count.to_string() })))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PurgeBody {
    /// RFC3339 时间；缺省 = now - 90 天
    #[serde(default)]
    pub before: Option<String>,
}

pub async fn script_log_purge(
    State(state): State<AppState>,
    _operator: Operator,
    Json(body): Json<PurgeBody>,
) -> Result<impl IntoResponse, AppError> {
    let before = match body.before.as_deref() {
        Some(s) if !s.trim().is_empty() => Some(
            chrono::DateTime::parse_from_rfc3339(s.trim())
                .map_err(|e| AppError::Validation(format!("invalid before time: {e}")))?
                .with_timezone(&chrono::Utc),
        ),
        _ => None,
    };
    let repo = ScriptLogRepo::new(crate::handlers::script::db_of(&state)?);
    let deleted = repo.purge(before).await?;
    Ok(json_ok(serde_json::json!({ "deleted": deleted.to_string() })))
}
