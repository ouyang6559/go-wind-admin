// dashboard 模块 handlers（wire 对齐 Go：裸 DTO、camelCase、NULL 省略）。
// 统计口径对齐 Go dashboard_service.go + dashboard_repo.go：
//   - userCount/roleCount：有效（未软删）用户/角色总数
//   - todayLoginCount：action_type=LOGIN 且 created_at >= 今日零点（服务器本地时区）
//   - todayOperationCount：created_at >= 今日零点
//   - login-trend：近 days 天每日登录次数，按日期升序、缺日补零（默认 7）
//   - 两个分布：按 action/status 分组计数，label 为枚举名（前端 i18n 映射）

use axum::extract::{Query, State};
use axum::response::IntoResponse;
use serde::Serialize;
use std::collections::HashMap;

use chrono::{DateTime, Local, NaiveTime};
use sqlx::Row;

use crate::error::AppError;
use crate::handlers::script::db_of;
use crate::middleware::Operator;
use crate::response::json_ok;
use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardOverviewDto {
    pub user_count: u64,
    pub role_count: u64,
    pub today_login_count: u64,
    pub today_operation_count: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendPointDto {
    pub date: String,
    pub count: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendResponseDto {
    pub points: Vec<TrendPointDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionItemDto {
    pub label: String,
    pub count: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DistributionResponseDto {
    pub items: Vec<DistributionItemDto>,
}

/// 今日零点（服务器本地时区，对齐 Go startOfToday）。
fn start_of_today() -> DateTime<Local> {
    let now = Local::now();
    let midnight = NaiveTime::from_hms_opt(0, 0, 0).expect("midnight is a valid time");
    now.date_naive()
        .and_time(midnight)
        .and_local_timezone(Local)
        .single()
        .unwrap_or(now)
}

/// 单值计数查询（binds 为 $N::timestamptz 等参数值）。
async fn count_rows(db: &sqlx::AnyPool, sql: &str, binds: &[String]) -> Result<u64, AppError> {
    let mut q = sqlx::query_as::<sqlx::Any, (i64,)>(sql);
    for b in binds {
        q = q.bind(b.as_str());
    }
    let (n,): (i64,) = q.fetch_one(db).await.map_err(|e| AppError::Internal {
        context: format!("dashboard count query failed: {sql}"),
        source: Some(Box::new(e)),
    })?;
    Ok(n as u64)
}

pub async fn dashboard_get_overview(
    State(state): State<AppState>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let db = db_of(&state)?;

    let user_count = count_rows(&db, "select count(*) from sys_users where deleted_at is null", &[]).await?;
    let role_count = count_rows(&db, "select count(*) from sys_roles where deleted_at is null", &[]).await?;

    let today = start_of_today().to_rfc3339();
    let today_login_count = count_rows(
        &db,
        "select count(*) from sys_login_audit_logs \
         where action_type = 'LOGIN' and created_at >= $1::timestamptz",
        &[today.clone()],
    )
    .await?;
    let today_operation_count = count_rows(
        &db,
        "select count(*) from sys_operation_audit_logs \
         where created_at >= $1::timestamptz",
        &[today],
    )
    .await?;

    Ok(json_ok(DashboardOverviewDto {
        user_count,
        role_count,
        today_login_count,
        today_operation_count,
    }))
}

pub async fn dashboard_get_login_trend(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let db = db_of(&state)?;

    // days <= 0 或缺失 → 默认 7（对齐 Go LoginTrend）
    let days = params
        .get("days")
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|&d| d > 0)
        .unwrap_or(7);

    let today = start_of_today();
    let start = today - chrono::Duration::days(days - 1);

    // 只取 created_at 单列，在应用层按本地时区分桶补零（对齐 Go：ent GroupBy 不支持 DATE() 表达式）
    let sql = "select to_char(created_at at time zone 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at \
               from sys_login_audit_logs \
               where action_type = 'LOGIN' and created_at >= $1::timestamptz";
    let rows = sqlx::query::<sqlx::Any>(sql)
        .bind(start.to_rfc3339())
        .fetch_all(&db)
        .await
        .map_err(|e| AppError::Internal {
            context: "login trend query failed".into(),
            source: Some(Box::new(e)),
        })?;

    // 按日期初始化桶，保证无记录的日期也有零值，结果按日期升序
    let mut buckets: Vec<(String, u64)> = Vec::with_capacity(days as usize);
    let mut idx: HashMap<String, usize> = HashMap::with_capacity(days as usize);
    for i in 0..days {
        let d = (start.date_naive() + chrono::Duration::days(i))
            .format("%Y-%m-%d")
            .to_string();
        idx.insert(d.clone(), buckets.len());
        buckets.push((d, 0));
    }
    for row in &rows {
        let s: Option<String> = row.try_get("created_at").ok().flatten();
        if let Some(s) = s {
            if let Ok(dt) = DateTime::parse_from_rfc3339(&s) {
                let d = dt.with_timezone(&Local).format("%Y-%m-%d").to_string();
                if let Some(&i) = idx.get(&d) {
                    buckets[i].1 += 1;
                }
            }
        }
    }

    Ok(json_ok(TrendResponseDto {
        points: buckets
            .into_iter()
            .map(|(date, count)| TrendPointDto { date, count })
            .collect(),
    }))
}

/// 分组计数查询：label 为分组列（枚举名），cnt 为 count(*)。
async fn distribution_items(db: &sqlx::AnyPool, sql: &str) -> Result<Vec<DistributionItemDto>, AppError> {
    let rows = sqlx::query::<sqlx::Any>(sql)
        .fetch_all(db)
        .await
        .map_err(|e| AppError::Internal {
            context: format!("dashboard distribution query failed: {sql}"),
            source: Some(Box::new(e)),
        })?;
    let mut items = Vec::with_capacity(rows.len());
    for row in &rows {
        items.push(DistributionItemDto {
            // 分组列可空（audit 枚举列 Optional），对齐 Go 零值输出空 label
            label: row
                .try_get::<Option<String>, _>("label")
                .ok()
                .flatten()
                .unwrap_or_default(),
            count: row
                .try_get::<Option<i64>, _>("cnt")
                .ok()
                .flatten()
                .unwrap_or(0) as u64,
        });
    }
    Ok(items)
}

pub async fn dashboard_get_operation_action_distribution(
    State(state): State<AppState>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let db = db_of(&state)?;
    let items = distribution_items(
        &db,
        "select action as label, count(*) as cnt from sys_operation_audit_logs group by action",
    )
    .await?;
    Ok(json_ok(DistributionResponseDto { items }))
}

pub async fn dashboard_get_login_status_distribution(
    State(state): State<AppState>,
    _operator: Operator,
) -> Result<impl IntoResponse, AppError> {
    let db = db_of(&state)?;
    let items = distribution_items(
        &db,
        "select status as label, count(*) as cnt from sys_login_audit_logs group by status",
    )
    .await?;
    Ok(json_ok(DistributionResponseDto { items }))
}
