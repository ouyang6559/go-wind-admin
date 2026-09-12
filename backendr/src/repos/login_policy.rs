// login_policy repo：sys_login_policies CRUD SQL（对齐 Go login_policy_repo.go）。
// type/method 枚举按 proto 枚举名存储（BLACKLIST/WHITELIST、IP/MAC/REGION/TIME/DEVICE），
// 与 Go 端 EnumTypeConverter 写库值一致；列表/查询过滤 deleted_at is null，
// Delete 为硬删除（Go 端 TimeAt mixin 无软删拦截器）。

use sqlx::AnyPool;
use sqlx::Row;

use crate::error::AppError;
use chrono::Timelike;

/// 登录闸门用的策略条目（对齐 Go `data.EffectivePolicy`）。
/// target_id 为 0 表示全局策略（不限定用户），否则仅约束该用户。
#[derive(Debug, Clone)]
pub struct EffectivePolicy {
    pub target_id: i64,
    /// BLACKLIST / WHITELIST
    pub r#type: String,
    /// IP / MAC / REGION / TIME / DEVICE
    pub method: String,
    pub value: String,
    pub reason: String,
}

/// 登录策略匹配器：纯函数，语义按维度独立判定（对齐 Go `data.MatchLoginPolicy`）。
///   - 黑名单：任一条目命中 → 拒绝
///   - 白名单：该维度存在白名单约束（全局或定向当前用户）且当前值未命中任何白名单 → 拒绝
///
/// 生效范围：条目 TargetID 为 0 表示全局（约束所有用户），否则仅约束该用户；
/// userId 传 0 表示只检查全局条目。维度支持 IP（精确 IP 或 CIDR）、TIME
/// （HH:MM-HH:MM 时间窗，支持跨午夜）、DEVICE（device_id 精确匹配）。
/// MAC / REGION 第一版不判定（与 Go 一致：HTTP 上下文拿不到 MAC，REGION 依赖地理库）。
/// 返回 (blocked, reason)。
#[allow(clippy::too_many_arguments)]
pub fn match_login_policy(
    policies: &[EffectivePolicy],
    user_id: i64,
    client_ip: &str,
    device_id: &str,
    now: chrono::NaiveTime,
) -> (bool, String) {
    for method in ["IP", "TIME", "DEVICE"] {
        let mut blacks: Vec<&EffectivePolicy> = Vec::new();
        let mut whites: Vec<&EffectivePolicy> = Vec::new();
        for p in policies {
            if p.method != method {
                continue;
            }
            if p.target_id != 0 && p.target_id != user_id {
                continue;
            }
            if p.r#type == "WHITELIST" {
                whites.push(p);
            } else {
                blacks.push(p);
            }
        }

        for p in &blacks {
            if policy_value_match(method, p.value.as_str(), client_ip, device_id, now) {
                let reason = if !p.reason.is_empty() {
                    p.reason.clone()
                } else {
                    format!("hit {} blacklist: {}", method.to_lowercase(), p.value)
                };
                return (true, reason);
            }
        }
        if !whites.is_empty() {
            let hit = whites.iter().any(|p| {
                policy_value_match(method, p.value.as_str(), client_ip, device_id, now)
            });
            if !hit {
                return (true, format!("not in {} whitelist", method.to_lowercase()));
            }
        }
    }
    (false, String::new())
}

/// 判定 clientIP / 当前时间 / device_id 是否命中某条策略值（按 method 分发）。
fn policy_value_match(
    method: &str,
    value: &str,
    client_ip: &str,
    device_id: &str,
    now: chrono::NaiveTime,
) -> bool {
    match method {
        "IP" => match_ip_value(client_ip, value),
        "TIME" => match_time_window(now, value),
        "DEVICE" => !device_id.is_empty() && device_id == value,
        _ => false,
    }
}

/// 判定 clientIP 是否命中策略值：支持精确 IP 或 CIDR 网段。
/// 解析失败的策略值视为不命中（配置错误不应阻断全部登录，由管理面保证值合法）。
fn match_ip_value(client_ip: &str, value: &str) -> bool {
    let value = value.trim();
    if client_ip.is_empty() || value.is_empty() {
        return false;
    }
    if value.contains('/') {
        let net: ipnet::IpNet = match value.parse() {
            Ok(v) => v,
            Err(_) => return false,
        };
        return match client_ip.parse::<std::net::IpAddr>() {
            Ok(ip) => net.contains(&ip),
            Err(_) => false,
        };
    }
    client_ip == value
}

/// 判定当前时间是否落在 "HH:MM-HH:MM" 时间窗内（支持跨午夜，如 "22:00-06:00"）。
/// 格式非法或 start==end 视为不命中。
fn match_time_window(now: chrono::NaiveTime, value: &str) -> bool {
    let parts: Vec<&str> = value.trim().split('-').collect();
    if parts.len() != 2 {
        return false;
    }
    let (Some(start), Some(end)) = (parse_hhmm(parts[0]), parse_hhmm(parts[1])) else {
        return false;
    };
    if start == end {
        return false;
    }
    let cur = now.hour() as i64 * 60 + now.minute() as i64;
    if start < end {
        cur >= start && cur < end
    } else {
        // 跨午夜：如 22:00-06:00 → [start,24:00) ∪ [00:00,end)
        cur >= start || cur < end
    }
}

/// 解析 "HH:MM" → 当日分钟数（0..=1439）。非法返回 None。
fn parse_hhmm(s: &str) -> Option<i64> {
    let s = s.trim();
    let (hh, mm) = s.split_once(':')?;
    if hh.is_empty() || mm.len() != 2 || !mm.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let h: i64 = hh.parse().ok()?;
    let m: i64 = mm.parse().ok()?;
    if h > 23 || m > 59 {
        return None;
    }
    Some(h * 60 + m)
}

#[derive(Debug, Clone)]
pub struct LoginPolicyRow {
    pub id: i64,
    pub target_id: Option<i64>,
    pub value: Option<String>,
    pub reason: Option<String>,
    /// proto 枚举名（BLACKLIST/WHITELIST）
    pub r#type: Option<String>,
    /// proto 枚举名（IP/MAC/REGION/TIME/DEVICE）
    pub method: Option<String>,
    pub tenant_id: Option<i64>,
    pub tenant_name: Option<String>,
    pub created_by: Option<i64>,
    pub updated_by: Option<i64>,
    pub deleted_by: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub deleted_at: Option<String>,
}

/// SELECT 列（p.* + 租户显示名）
const SELECT_COLS: &str = "p.id, p.target_id, p.value, p.reason, p.type, p.method, \
                           p.tenant_id, t.name as tenant_name, \
                           p.created_by, p.updated_by, p.deleted_by, \
                           to_char(p.created_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as created_at, \
                           to_char(p.updated_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as updated_at, \
                           to_char(p.deleted_at, 'YYYY-MM-DD\"T\"HH24:MI:SS\"Z\"') as deleted_at";

const FROM: &str = "from sys_login_policies p \
                    left join sys_tenants t on t.id = p.tenant_id and t.deleted_at is null";

fn map_row(row: &sqlx::any::AnyRow) -> Result<LoginPolicyRow, AppError> {
    Ok(LoginPolicyRow {
        id: row
            .try_get::<i64, _>("id")
            .map_err(|e| AppError::Internal {
                context: "login policy row id decode failed".into(),
                source: Some(Box::new(e)),
            })?,
        target_id: row.try_get::<Option<i64>, _>("target_id").ok().flatten(),
        value: row.try_get::<Option<String>, _>("value").ok().flatten(),
        reason: row.try_get::<Option<String>, _>("reason").ok().flatten(),
        r#type: row.try_get::<Option<String>, _>("type").ok().flatten(),
        method: row.try_get::<Option<String>, _>("method").ok().flatten(),
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

pub struct LoginPolicyRepo {
    pub db: AnyPool,
}

impl LoginPolicyRepo {
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
    ) -> Result<(Vec<LoginPolicyRow>, u64), AppError> {
        let total_sql =
            format!("select count(*) from sys_login_policies p where p.deleted_at is null{where_clause}");
        let mut tq = sqlx::query_as::<sqlx::Any, (i64,)>(&total_sql);
        for p in params {
            tq = tq.bind(p);
        }
        let total = tq.fetch_one(&self.db).await.map_err(|e| AppError::Internal {
            context: "count login policies failed".into(),
            source: Some(Box::new(e)),
        })?;

        let order = if order_by.is_empty() { "p.id" } else { order_by };
        let sql = format!(
            "select {SELECT_COLS} {FROM} where p.deleted_at is null{where_clause} \
             order by {order} limit {limit} offset {offset}"
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in params {
            q = q.bind(p);
        }
        let rows = q.fetch_all(&self.db).await.map_err(|e| AppError::Internal {
            context: "list login policies failed".into(),
            source: Some(Box::new(e)),
        })?;
        let mut items = Vec::with_capacity(rows.len());
        for row in &rows {
            items.push(map_row(row)?);
        }
        Ok((items, total.0 as u64))
    }

    pub async fn get(&self, id: i64) -> Result<Option<LoginPolicyRow>, AppError> {
        let sql = format!(
            "select {SELECT_COLS} {FROM} where p.id = $1 and p.deleted_at is null limit 1"
        );
        let row = sqlx::query::<sqlx::Any>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "get login policy failed".into(),
                source: Some(Box::new(e)),
            })?;
        row.as_ref().map(map_row).transpose()
    }

    /// 拉取租户内全部登录策略，供登录闸门在内存中按全局（target_id 为空）/ 用户定向
    /// （target_id = userId）两批匹配（对齐 Go `ListForLogin`）。策略量级小，全量拉取即可。
    pub async fn list_for_login(&self, tenant_id: i64) -> Result<Vec<EffectivePolicy>, AppError> {
        let sql = "select p.target_id, p.type, p.method, p.value, p.reason \
                   from sys_login_policies p \
                   where p.tenant_id = $1 and p.deleted_at is null";
        let rows = sqlx::query::<sqlx::Any>(sql)
            .bind(tenant_id)
            .fetch_all(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "list login policies for login failed".into(),
                source: Some(Box::new(e)),
            })?;
        let mut policies = Vec::with_capacity(rows.len());
        for row in &rows {
            let target_id: Option<i64> = row.try_get("target_id").ok().flatten();
            let r#type: Option<String> = row.try_get("type").ok().flatten();
            let method: Option<String> = row.try_get("method").ok().flatten();
            let value: Option<String> = row.try_get("value").ok().flatten();
            let reason: Option<String> = row.try_get("reason").ok().flatten();
            policies.push(EffectivePolicy {
                target_id: target_id.unwrap_or(0),
                r#type: r#type.unwrap_or_default(),
                method: method.unwrap_or_default(),
                value: value.unwrap_or_default(),
                reason: reason.unwrap_or_default(),
            });
        }
        Ok(policies)
    }

    /// (tenant_id, target_id, type, method) 唯一性检查（exclude_id 排除自身）。
    /// target_id 可为 NULL，用 is not distinct from 匹配 NULL 相等语义。
    pub async fn conflict_exists(
        &self,
        tenant_id: Option<i64>,
        target_id: Option<i64>,
        r#type: &str,
        method: &str,
        exclude_id: i64,
    ) -> Result<bool, AppError> {
        let sql = "select 1 from sys_login_policies \
                   where tenant_id = $1 and target_id is not distinct from $2 \
                   and type = $3 and method = $4 and id <> $5 and deleted_at is null limit 1";
        let row: Option<(i64,)> = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(tenant_id.unwrap_or(0))
            .bind(target_id)
            .bind(r#type)
            .bind(method)
            .bind(exclude_id)
            .fetch_optional(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "check login policy conflict failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.is_some())
    }

    /// 创建登录策略。
    #[allow(clippy::too_many_arguments)]
    pub async fn create(
        &self,
        tenant_id: Option<i64>,
        target_id: Option<i64>,
        r#type: &str,
        method: &str,
        value: &str,
        reason: Option<&str>,
        created_by: i64,
    ) -> Result<i64, AppError> {
        let sql = "insert into sys_login_policies \
                   (tenant_id, target_id, type, method, value, reason, created_by, created_at, updated_at) \
                   values ($1, $2, $3, $4, $5, $6, $7, $8::timestamptz, $8::timestamptz) \
                   returning id";
        let now = chrono::Utc::now().to_rfc3339();
        let row: (i64,) = sqlx::query_as::<sqlx::Any, (i64,)>(sql)
            .bind(tenant_id.unwrap_or(0))
            .bind(target_id)
            .bind(r#type)
            .bind(method)
            .bind(value)
            .bind(reason)
            .bind(created_by)
            .bind(now)
            .fetch_one(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "insert login policy failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(row.0)
    }

    /// 更新登录策略：仅非 None 字段（tenant_id 不可变，对齐 Go TenantID mixin Immutable）。
    #[allow(clippy::too_many_arguments)]
    pub async fn update(
        &self,
        id: i64,
        target_id: Option<i64>,
        r#type: Option<&str>,
        method: Option<&str>,
        value: Option<&str>,
        reason: Option<&str>,
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

        if let Some(v) = target_id {
            push!("target_id", v.to_string(), "::int8");
        }
        if let Some(v) = r#type {
            push!("type", v.to_string(), "");
        }
        if let Some(v) = method {
            push!("method", v.to_string(), "");
        }
        if let Some(v) = value {
            push!("value", v.to_string(), "");
        }
        if let Some(v) = reason {
            push!("reason", v.to_string(), "");
        }
        push!("updated_by", updated_by.to_string(), "::int8");
        push!("updated_at", chrono::Utc::now().to_rfc3339(), "::timestamptz");

        if sets.is_empty() {
            return Err(AppError::Validation("no fields to update".into()));
        }

        let sql = format!(
            "update sys_login_policies set {} where id = ${} and deleted_at is null",
            sets.join(", "),
            params.len() + 1
        );
        let mut q = sqlx::query::<sqlx::Any>(&sql);
        for p in &params {
            q = q.bind(p);
        }
        q = q.bind(id);
        let res = q.execute(&self.db).await.map_err(|e| AppError::Internal {
            context: "update login policy failed".into(),
            source: Some(Box::new(e)),
        })?;
        Ok(res.rows_affected())
    }

    /// 硬删除。
    pub async fn delete(&self, id: i64) -> Result<u64, AppError> {
        let sql = "delete from sys_login_policies where id = $1 and deleted_at is null";
        let res = sqlx::query::<sqlx::Any>(sql)
            .bind(id)
            .execute(&self.db)
            .await
            .map_err(|e| AppError::Internal {
                context: "delete login policy failed".into(),
                source: Some(Box::new(e)),
            })?;
        Ok(res.rows_affected())
    }
}
