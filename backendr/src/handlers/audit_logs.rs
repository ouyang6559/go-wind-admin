// 审计日志 6 模块通用 handlers（api/login/operation/permission/data_access/policy_evaluation）。
// 表驱动：每个模块声明列定义（列名 + 类型），共用 List/Get 泛型实现。
// wire 对齐 Go protojson：camelCase、NULL 字段省略、jsonb 透传、bytea → base64。

use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::{Map, Value};
use std::collections::HashMap;

use crate::error::AppError;
use crate::middleware::Operator;
use crate::query::{compile_where, ListQuery};
use crate::response::{json_ok, ListResponse};
use crate::state::AppState;

use sqlx::Row;

/// 列值类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VT {
    /// 整型（JSON number）
    Num,
    /// 布尔
    Bool,
    /// 文本
    Str,
    /// 时间戳（RFC3339 文本输出）
    Ts,
    /// jsonb（原样输出对象/数组）
    Json,
    /// bytea（base64 文本输出）
    B64,
}

#[derive(Debug, Clone, Copy)]
pub struct ColDef {
    pub name: &'static str,
    pub ty: VT,
}

const fn c(name: &'static str, ty: VT) -> ColDef {
    ColDef { name, ty }
}

pub const API_AUDIT: &[ColDef] = &[
    c("id", VT::Num),
    c("created_at", VT::Ts),
    c("tenant_id", VT::Num),
    c("user_id", VT::Num),
    c("username", VT::Str),
    c("ip_address", VT::Str),
    c("geo_location", VT::Json),
    c("device_info", VT::Json),
    c("referer", VT::Str),
    c("app_version", VT::Str),
    c("http_method", VT::Str),
    c("path", VT::Str),
    c("request_uri", VT::Str),
    c("api_module", VT::Str),
    c("api_operation", VT::Str),
    c("api_description", VT::Str),
    c("request_id", VT::Str),
    c("trace_id", VT::Str),
    c("span_id", VT::Str),
    c("latency_ms", VT::Num),
    c("success", VT::Bool),
    c("status_code", VT::Num),
    c("reason", VT::Str),
    c("request_header", VT::Str),
    c("request_body", VT::Str),
    c("response", VT::Str),
    c("log_hash", VT::Str),
    c("signature", VT::B64),
];

pub const LOGIN_AUDIT: &[ColDef] = &[
    c("id", VT::Num),
    c("created_at", VT::Ts),
    c("tenant_id", VT::Num),
    c("user_id", VT::Num),
    c("username", VT::Str),
    c("ip_address", VT::Str),
    c("geo_location", VT::Json),
    c("session_id", VT::Str),
    c("device_info", VT::Json),
    c("request_id", VT::Str),
    c("trace_id", VT::Str),
    c("action_type", VT::Str),
    c("status", VT::Str),
    c("login_method", VT::Str),
    c("failure_reason", VT::Str),
    c("mfa_status", VT::Str),
    c("risk_score", VT::Num),
    c("risk_level", VT::Str),
    c("risk_factors", VT::Json),
    c("log_hash", VT::Str),
    c("signature", VT::B64),
];

pub const OPERATION_AUDIT: &[ColDef] = &[
    c("id", VT::Num),
    c("created_at", VT::Ts),
    c("tenant_id", VT::Num),
    c("user_id", VT::Num),
    c("username", VT::Str),
    c("resource_type", VT::Str),
    c("resource_id", VT::Str),
    c("action", VT::Str),
    c("before_data", VT::Json),
    c("after_data", VT::Json),
    c("sensitive_level", VT::Str),
    c("request_id", VT::Str),
    c("trace_id", VT::Str),
    c("success", VT::Bool),
    c("failure_reason", VT::Str),
    c("ip_address", VT::Str),
    c("geo_location", VT::Json),
    c("device_info", VT::Json),
    c("log_hash", VT::Str),
    c("signature", VT::B64),
];

pub const PERMISSION_AUDIT: &[ColDef] = &[
    c("id", VT::Num),
    c("created_at", VT::Ts),
    c("tenant_id", VT::Num),
    c("operator_id", VT::Num),
    c("operator_name", VT::Str),
    c("target_type", VT::Str),
    c("target_id", VT::Str),
    c("target_name", VT::Str),
    c("action", VT::Str),
    c("old_value", VT::Json),
    c("new_value", VT::Json),
    c("ip_address", VT::Str),
    c("request_id", VT::Str),
    c("reason", VT::Str),
    c("log_hash", VT::Str),
    c("signature", VT::B64),
];

pub const DATA_ACCESS_AUDIT: &[ColDef] = &[
    c("id", VT::Num),
    c("created_at", VT::Ts),
    c("tenant_id", VT::Num),
    c("user_id", VT::Num),
    c("username", VT::Str),
    c("ip_address", VT::Str),
    c("geo_location", VT::Json),
    c("device_info", VT::Json),
    c("request_id", VT::Str),
    c("trace_id", VT::Str),
    c("data_source", VT::Str),
    c("table_name", VT::Str),
    c("data_id", VT::Str),
    c("access_type", VT::Str),
    c("sql_digest", VT::Str),
    c("sql_text", VT::Str),
    c("affected_rows", VT::Num),
    c("latency_ms", VT::Num),
    c("success", VT::Bool),
    c("sensitive_level", VT::Str),
    c("data_masked", VT::Bool),
    c("masking_rules", VT::Str),
    c("business_purpose", VT::Str),
    c("data_category", VT::Str),
    c("db_user", VT::Str),
    c("log_hash", VT::Str),
    c("signature", VT::B64),
];

pub const POLICY_EVALUATION: &[ColDef] = &[
    c("id", VT::Num),
    c("created_at", VT::Ts),
    c("tenant_id", VT::Num),
    c("user_id", VT::Num),
    c("membership_id", VT::Num),
    c("permission_id", VT::Num),
    c("policy_id", VT::Num),
    c("request_path", VT::Str),
    c("request_method", VT::Str),
    c("result", VT::Bool),
    c("effect_details", VT::Str),
    c("scope_sql", VT::Str),
    c("ip_address", VT::Str),
    c("trace_id", VT::Str),
    c("evaluation_context", VT::Str),
    c("log_hash", VT::Str),
    c("signature", VT::B64),
];

fn snake_to_camel(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut upper = false;
    for ch in s.chars() {
        if ch == '_' {
            upper = true;
        } else if upper {
            out.push(ch.to_ascii_uppercase());
            upper = false;
        } else {
            out.push(ch);
        }
    }
    out
}

/// 依据列定义生成 SELECT 列表（postgres 方言：to_char / ::text / encode）
fn select_list(defs: &[ColDef]) -> String {
    defs.iter()
        .map(|d| match d.ty {
            VT::Ts => format!(
                "to_char({}, 'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') as {}",
                d.name, d.name
            ),
            VT::Json => format!("{}::text as {}", d.name, d.name),
            VT::B64 => format!("replace(encode({}, 'base64'), E'\\n', '') as {}", d.name, d.name),
            _ => d.name.to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// geo/device 结构在 Go 侧经 proto 序列化（camelCase 键）；DB jsonb 为 snake_case，
/// 出口递归转换键名以对齐。其余 jsonb（操作前后数据等）保持原样。
fn is_camelized_json(col: &str) -> bool {
    matches!(col, "geo_location" | "device_info")
}

fn camelize_keys(v: Value) -> Value {
    match v {
        Value::Object(m) => {
            let mut out = Map::new();
            for (k, val) in m {
                // deviceType / riskLevel 等枚举字段：DB 存数字，Go 经 proto 输出枚举名
                let key = snake_to_camel(&k);
                let val = match (&key as &str, val) {
                    ("deviceType", Value::Number(n)) => match n.as_i64() {
                        Some(1) => Value::from("DESKTOP"),
                        Some(2) => Value::from("MOBILE"),
                        Some(3) => Value::from("TABLET"),
                        Some(4) => Value::from("BOT"),
                        Some(5) => Value::from("OTHER"),
                        _ => Value::from("DEVICE_TYPE_UNSPECIFIED"),
                    },
                    (_, other) => camelize_keys(other),
                };
                out.insert(key, val);
            }
            Value::Object(out)
        }
        Value::Array(a) => Value::Array(a.into_iter().map(camelize_keys).collect()),
        other => other,
    }
}

/// 把一行 AnyRow 映射为 JSON（按列定义取值；NULL 省略，对齐 protojson 未设置字段）
fn row_to_json(defs: &[ColDef], row: &sqlx::any::AnyRow) -> Result<Value, AppError> {
    let mut map = Map::new();
    for d in defs {
        let camel = snake_to_camel(d.name);
        let v: Option<Value> = match d.ty {
            VT::Num => row
                .try_get::<Option<i64>, _>(d.name)
                .ok()
                .flatten()
                .map(|n| Value::from(n)),
            VT::Bool => row
                .try_get::<Option<bool>, _>(d.name)
                .ok()
                .flatten()
                .map(Value::from),
            VT::Str | VT::Ts | VT::B64 => row
                .try_get::<Option<String>, _>(d.name)
                .ok()
                .flatten()
                .map(Value::from),
            VT::Json => row
                .try_get::<Option<String>, _>(d.name)
                .ok()
                .flatten()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .map(|v| if is_camelized_json(d.name) { camelize_keys(v) } else { v }),
        };
        if let Some(v) = v {
            map.insert(camel, v);
        }
    }
    Ok(Value::Object(map))
}

/// 通用 List：白名单过滤列 = 本模块的全部 Str/Num/Bool 列（camel 输入转 snake 校验）
pub async fn audit_list(
    state: &AppState,
    table: &str,
    defs: &[ColDef],
    params: &HashMap<String, String>,
) -> Result<(Vec<Value>, u64), AppError> {
    // 审计日志表为追加型（无 deleted_at / 软删列）
    let _ = table;
    let filterable: Vec<&str> = defs
        .iter()
        .filter(|d| matches!(d.ty, VT::Num | VT::Str | VT::Bool))
        .map(|d| d.name)
        .collect();
    let lq = ListQuery::parse(params, &filterable)?;

    let db = state.db.clone().ok_or_else(|| AppError::Internal {
        context: "database not configured".into(),
        source: None,
    })?;

    let mut where_clause = String::new();
    let mut bind: Vec<String> = Vec::new();
    if !lq.filters.is_empty() {
        where_clause = format!(" and {}", compile_where(&lq.filters, &mut bind));
    }

    let total_sql =
        format!("select count(*) from {table} where 1=1{where_clause}");
    let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
    for p in &bind {
        tq = tq.bind(p);
    }
    let total = tq.fetch_one(&db).await.map_err(|e| AppError::Internal {
        context: format!("count {table} failed"),
        source: Some(Box::new(e)),
    })?;

    // orderBy 编译
    let mut order_parts = Vec::new();
    for (col, desc) in &lq.paging.order_by {
        let c = crate::query::resolve_column(col, &filterable)?;
        order_parts.push(format!("\"{c}\" {}", if *desc { "desc" } else { "asc" }));
    }
    let order = if order_parts.is_empty() {
        "\"id\"".to_string()
    } else {
        order_parts.join(", ")
    };

    let sql = format!(
        "select {} from {table} where 1=1{where_clause} \
         order by {order} limit {} offset {}",
        select_list(defs),
        lq.paging.limit(),
        lq.paging.offset(),
    );
    let mut q = sqlx::query::<sqlx::Any>(&sql);
    for p in &bind {
        q = q.bind(p);
    }
    let rows = q.fetch_all(&db).await.map_err(|e| AppError::Internal {
        context: format!("list {table} failed"),
        source: Some(Box::new(e)),
    })?;

    let mut items = Vec::with_capacity(rows.len());
    for row in &rows {
        items.push(row_to_json(defs, row)?);
    }
    Ok((items, total.0 as u64))
}

/// 通用 Get
pub async fn audit_get(
    state: &AppState,
    table: &str,
    defs: &[ColDef],
    id: i64,
) -> Result<Value, AppError> {
    let db = state.db.clone().ok_or_else(|| AppError::Internal {
        context: "database not configured".into(),
        source: None,
    })?;
    let sql = format!(
        "select {} from {table} where id = $1 limit 1",
        select_list(defs)
    );
    let row = sqlx::query::<sqlx::Any>(&sql)
        .bind(id)
        .fetch_optional(&db)
        .await
        .map_err(|e| AppError::Internal {
            context: format!("get {table} failed"),
            source: Some(Box::new(e)),
        })?
        .ok_or_else(|| AppError::NotFound("record not found".into()))?;
    row_to_json(defs, &row)
}

// ---------- 各模块 handler 包装 ----------

macro_rules! audit_handlers {
    ($list:ident, $get:ident, $table:literal, $defs:expr) => {
        pub async fn $list(
            State(state): State<AppState>,
            _operator: Operator,
            Query(params): Query<HashMap<String, String>>,
        ) -> Result<impl IntoResponse, AppError> {
            let (items, total) = audit_list(&state, $table, $defs, &params).await?;
            Ok(json_ok(ListResponse::new(items, total)))
        }

        pub async fn $get(
            State(state): State<AppState>,
            _operator: Operator,
            Path(id): Path<i64>,
        ) -> Result<impl IntoResponse, AppError> {
            Ok(json_ok(audit_get(&state, $table, $defs, id).await?))
        }
    };
}

audit_handlers!(api_audit_log_list, api_audit_log_get, "sys_api_audit_logs", API_AUDIT);
audit_handlers!(login_audit_log_list, login_audit_log_get, "sys_login_audit_logs", LOGIN_AUDIT);
audit_handlers!(operation_audit_log_list, operation_audit_log_get, "sys_operation_audit_logs", OPERATION_AUDIT);
audit_handlers!(permission_audit_log_list, permission_audit_log_get, "sys_permission_audit_logs", PERMISSION_AUDIT);
audit_handlers!(data_access_audit_log_list, data_access_audit_log_get, "sys_data_access_audit_logs", DATA_ACCESS_AUDIT);
audit_handlers!(policy_evaluation_log_list, policy_evaluation_log_get, "sys_policy_evaluation_logs", POLICY_EVALUATION);
