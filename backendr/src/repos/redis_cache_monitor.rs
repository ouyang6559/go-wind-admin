// redis_cache_monitor repo：Redis 运行时只读监控（INFO / DBSIZE / SLOWLOG GET 聚合视图）。
// 对齐 Go `redis_cache_monitor_repo.go` 设计：全部命令只读，任一命令失败仅告警并将对应
// 字段置空，不阻断整体视图返回（fail-soft）；Redis 未配置时返回空视图而非错误。
//
// 输出 DTO 字段按 proto json_name 精确对齐：
// - `sections`：`[{name, entries:[{key,value}]}]`
// - `dbSize`：uint64 → protojson 字符串
// - `slowlog`：`[{id, createdAt, durationUsec, args, clientAddr, clientName}]`（int64 → 字符串）

use crate::dto::types::{InfoEntry, InfoSection, RedisCacheMonitorInfo, SlowLogEntry};

/// redis 0.30 的 `Value` 提取辅助（`as_i64`/`as_string` 已从 API 移除，手动枚举匹配）。
fn value_i64(v: &redis::Value) -> Option<i64> {
    match v {
        redis::Value::Int(i) => Some(*i),
        _ => None,
    }
}

fn value_string(v: &redis::Value) -> Option<String> {
    match v {
        redis::Value::BulkString(b) => Some(String::from_utf8_lossy(b).into_owned()),
        redis::Value::SimpleString(s) => Some(s.clone()),
        redis::Value::VerbatimString { text, .. } => Some(text.clone()),
        _ => None,
    }
}

/// SLOWLOG GET 抓取的最近条目数上限（对齐 Go `slowLogFetchLimit`）。
const SLOWLOG_FETCH_LIMIT: i64 = 10;

pub struct RedisCacheMonitorRepo;

impl RedisCacheMonitorRepo {
    pub fn new() -> Self {
        Self
    }

    /// 聚合返回 Redis 的 INFO / DBSIZE / SLOWLOG 三类只读运维指标。
    /// Redis 客户端未配置或连接失败时返回空视图而非错误（对齐 Go fail-soft 风格）。
    pub async fn get_info(&self, client: Option<&redis::Client>) -> RedisCacheMonitorInfo {
        let empty = RedisCacheMonitorInfo {
            sections: Vec::new(),
            dbSize: String::new(),
            slowlog: Vec::new(),
        };

        let Some(client) = client else {
            return empty;
        };

        let mut con = match client.get_multiplexed_tokio_connection().await {
            Ok(con) => con,
            Err(e) => {
                tracing::warn!(error = %e, "redis connect failed (cache monitor view degraded)");
                return empty;
            }
        };

        // INFO：原始串按 section/entry 拆分后落入泛型结构
        let sections = match redis::cmd("INFO")
            .query_async::<String>(&mut con)
            .await
        {
            Ok(raw) => parse_info_sections(&raw),
            Err(e) => {
                tracing::warn!(error = %e, "redis INFO failed");
                Vec::new()
            }
        };

        // DBSIZE：当前库 key 总数（uint64 → protojson 字符串）
        let db_size = match redis::cmd("DBSIZE").query_async::<i64>(&mut con).await {
            Ok(n) if n >= 0 => n.to_string(),
            Ok(_) => String::new(),
            Err(e) => {
                tracing::warn!(error = %e, "redis DBSIZE failed");
                String::new()
            }
        };

        // SLOWLOG GET：最近慢日志条目
        let slowlog = match redis::cmd("SLOWLOG")
            .arg("GET")
            .arg(SLOWLOG_FETCH_LIMIT)
            .query_async::<Vec<Vec<redis::Value>>>(&mut con)
            .await
        {
            Ok(entries) => map_slowlog_entries(&entries),
            Err(e) => {
                tracing::warn!(error = %e, "redis SLOWLOG GET failed");
                Vec::new()
            }
        };

        RedisCacheMonitorInfo {
            sections,
            dbSize: db_size,
            slowlog,
        }
    }
}

/// 解析 Redis INFO 原始串为 section/entry 树（对齐 Go `parseInfoSections`）。
/// INFO 输出格式：以 `# <SectionName>` 行标记新 section，其后 `key:value` 行挂入该 section；
/// 空行与无法识别、无归属 section 的行被忽略。Keyspace 的 `db0:keys=...` 同样落入 key/value。
fn parse_info_sections(raw: &str) -> Vec<InfoSection> {
    let mut sections: Vec<InfoSection> = Vec::new();
    let mut current_idx: Option<usize> = None;

    for raw_line in raw.lines() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if line.is_empty() {
            continue;
        }

        if let Some(name) = line.strip_prefix("# ") {
            let name = name.trim();
            if name.is_empty() {
                // 畸形 section 头：停止向其挂条目，避免误归属。
                current_idx = None;
                continue;
            }
            sections.push(InfoSection {
                name: name.to_string(),
                entries: Vec::new(),
            });
            current_idx = Some(sections.len() - 1);
            continue;
        }

        let Some(idx) = line.find(':') else {
            continue;
        };
        let Some(section) = current_idx.map(|i| &mut sections[i]) else {
            // 无归属 section 的孤立条目，忽略。
            continue;
        };
        section.entries.push(InfoEntry {
            key: line[..idx].to_string(),
            value: line[idx + 1..].to_string(),
        });
    }

    sections
}

/// 将 SLOWLOG GET 原始条目映射为 DTO。
/// Redis slowlog 条目为 6 元数组：id / unix秒 / 耗时(微秒) / 命令参数 / 客户端地址 / 客户端名；
/// 字段与 Go `mapSlowLogEntries` 一一对应（id→id、时间→createdAt、耗时→durationUsec）。
fn map_slowlog_entries(entries: &[Vec<redis::Value>]) -> Vec<SlowLogEntry> {
    let mut out = Vec::with_capacity(entries.len());
    for ent in entries {
        let id = ent.first().and_then(value_i64).unwrap_or(0);
        let time_secs = ent.get(1).and_then(value_i64).unwrap_or(0);
        let duration_usec = ent.get(2).and_then(value_i64).unwrap_or(0);
        let args = ent
            .get(3)
            .and_then(|v| v.as_sequence())
            .map(|seq| seq.iter().map(|a| value_string(a).unwrap_or_default()).collect())
            .unwrap_or_default();
        let client_addr = ent.get(4).and_then(value_string).unwrap_or_default();
        let client_name = ent.get(5).and_then(value_string).unwrap_or_default();

        out.push(SlowLogEntry {
            id: id.to_string(),
            createdAt: format_unix_ts(time_secs),
            durationUsec: duration_usec.to_string(),
            args,
            clientAddr: client_addr.to_string(),
            clientName: client_name.to_string(),
        });
    }
    out
}

/// 秒级 unix 时间戳 → protojson 风格 RFC3339（毫秒精度，对齐 timestamppb 序列化）。
fn format_unix_ts(secs: i64) -> String {
    chrono::DateTime::<chrono::Utc>::from_timestamp(secs, 0)
        .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true))
        .unwrap_or_default()
}