//! 通用列表查询参数解析（对齐 `docs/list_query_rule.md` 与 go-crud PagingRequest）。
//!
//! 支持：page/pageSize/noPaging/orderBy，以及 `query`/`or` JSON 过滤
//! （`字段__操作符` 语法：contains/icontains/in/gte/gt/lte/lt/isnull/not/exact/iexact/
//! startswith/istartswith/endswith/iendswith，未知操作符按 exact 处理）。
//! 字段名必须在本模块声明的列白名单内（防注入），值一律绑定参数。

use serde_json::Value;

use crate::error::AppError;

/// 分页参数（query string 解析结果）
#[derive(Debug, Clone, Default)]
pub struct Paging {
    pub page: u64,
    pub page_size: u64,
    pub no_paging: bool,
    /// `["-created_at","name"]` 解析结果：(列名, 是否降序)
    pub order_by: Vec<(String, bool)>,
}

impl Paging {
    pub fn parse(params: &std::collections::HashMap<String, String>) -> Self {
        let page = params
            .get("page")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(1)
            .max(1);
        let page_size = params
            .get("pageSize")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(10)
            .max(1);
        let no_paging = params
            .get("noPaging")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);
        let mut order_by = Vec::new();
        if let Some(raw) = params.get("orderBy") {
            if let Ok(Value::Array(arr)) = serde_json::from_str::<Value>(raw) {
                for item in arr {
                    if let Some(s) = item.as_str() {
                        if s.is_empty() {
                            continue;
                        }
                        let (col, desc) = match s.strip_prefix('-') {
                            Some(rest) => (rest.to_string(), true),
                            None => (s.to_string(), false),
                        };
                        order_by.push((col, desc));
                    }
                }
            }
        }
        Paging {
            page,
            page_size,
            no_paging,
            order_by,
        }
    }

    /// LIMIT 值（noPaging 时给一个安全上限）
    pub fn limit(&self) -> u64 {
        if self.no_paging {
            10000
        } else {
            self.page_size
        }
    }

    /// OFFSET 值
    pub fn offset(&self) -> u64 {
        if self.no_paging {
            0
        } else {
            (self.page - 1) * self.page_size
        }
    }
}

/// 一条过滤条件的已编译形态
#[derive(Debug, Clone)]
pub struct Filter {
    /// 数据库列名（已经白名单校验）
    pub column: String,
    /// SQL 操作片段模板：值占位符为 `{n}`（参数序号），由调用方编号
    pub op: FilterOp,
    /// 绑定值（in 操作为多值）
    pub values: Vec<String>,
    pub negated: bool,
    pub null_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterOp {
    Eq,
    Like,
    ILike,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
    IsNull,
    Between,
}

/// 列白名单校验：仅允许小写字母/数字/下划线，且必须在模块声明的集合内。
/// 先把 DTO camelCase 字段名转为 snake_case 再匹配。
pub fn resolve_column(field: &str, allowed: &[&str]) -> Result<String, AppError> {
    let col = camel_to_snake(field);
    let is_safe = !col.is_empty()
        && col.len() <= 64
        && col
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    if !is_safe || !allowed.contains(&col.as_str()) {
        return Err(AppError::Validation(format!(
            "unknown query field: {field}"
        )));
    }
    Ok(col)
}

/// camelCase → snake_case（ipAddress → ip_address）
pub fn camel_to_snake(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 4);
    for (i, ch) in s.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

/// 解析 `query`/`or` 参数中的 JSON 过滤（object 或 object array，数组内为 AND 语义）
pub fn parse_filters(raw_queries: &[String], allowed: &[&str]) -> Result<Vec<Filter>, AppError> {
    let mut filters = Vec::new();
    for raw in raw_queries {
        let parsed: Value = serde_json::from_str(raw).map_err(|e| {
            AppError::Validation(format!("invalid query json: {e}"))
        })?;
        let objects: Vec<Value> = match parsed {
            Value::Array(arr) => arr,
            obj @ Value::Object(_) => vec![obj],
            _ => continue,
        };
        for obj in objects {
            let map = match obj.as_object() {
                Some(m) => m,
                None => continue,
            };
            for (k, v) in map {
                let (field, op, negated) = split_lookup(k);
                let column = resolve_column(&field, allowed)?;
                let filter = build_filter(&column, op, v, negated)?;
                if let Some(f) = filter {
                    filters.push(f);
                }
            }
        }
    }
    Ok(filters)
}

/// 拆分 `field__op`；返回 (字段, 操作符, 是否 not)
fn split_lookup(key: &str) -> (String, String, bool) {
    let mut negated = false;
    let mut owned = key.to_string();
    if let Some(rest) = owned.strip_prefix("not_") {
        negated = true;
        owned = rest.to_string();
    }
    match owned.split_once("__") {
        Some((f, op)) => (f.to_string(), op.to_string(), negated),
        None => (owned, "exact".into(), negated),
    }
}

#[allow(clippy::too_many_lines)]
fn build_filter(
    column: &str,
    op: String,
    value: &Value,
    negated: bool,
) -> Result<Option<Filter>, AppError> {
    let value_str = match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    };
    let f = match op.as_str() {
        "contains" => Filter {
            column: column.into(),
            op: FilterOp::Like,
            values: vec![format!("%{}%", escape_like(&value_str))],
            negated,
            null_only: false,
        },
        "icontains" => Filter {
            column: column.into(),
            op: FilterOp::ILike,
            values: vec![format!("%{}%", escape_like(&value_str))],
            negated,
            null_only: false,
        },
        "startswith" => Filter {
            column: column.into(),
            op: FilterOp::Like,
            values: vec![format!("{}%", escape_like(&value_str))],
            negated,
            null_only: false,
        },
        "istartswith" => Filter {
            column: column.into(),
            op: FilterOp::ILike,
            values: vec![format!("{}%", escape_like(&value_str))],
            negated,
            null_only: false,
        },
        "endswith" => Filter {
            column: column.into(),
            op: FilterOp::Like,
            values: vec![format!("%{}", escape_like(&value_str))],
            negated,
            null_only: false,
        },
        "iendswith" => Filter {
            column: column.into(),
            op: FilterOp::ILike,
            values: vec![format!("%{}", escape_like(&value_str))],
            negated,
            null_only: false,
        },
        "exact" | "iexact" | "" => Filter {
            column: column.into(),
            op: if op == "iexact" { FilterOp::ILike } else { FilterOp::Eq },
            values: vec![if op == "iexact" { value_str.to_lowercase() } else { value_str }],
            negated,
            null_only: false,
        },
        "gt" | "gte" | "lt" | "lte" => Filter {
            column: column.into(),
            op: match op.as_str() {
                "gt" => FilterOp::Gt,
                "gte" => FilterOp::Gte,
                "lt" => FilterOp::Lt,
                _ => FilterOp::Lte,
            },
            values: vec![value_str],
            negated,
            null_only: false,
        },
        "in" | "not_in" => {
            let items: Vec<String> = serde_json::from_str(&value_str)
                .unwrap_or_else(|_| {
                    value_str
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                        .map(str::to_string)
                        .collect()
                });
            if items.is_empty() {
                return Ok(None);
            }
            Filter {
                column: column.into(),
                op: FilterOp::In,
                values: items,
                negated: negated || op == "not_in",
                null_only: false,
            }
        }
        "isnull" | "not_isnull" => Filter {
            column: column.into(),
            op: FilterOp::IsNull,
            values: vec![],
            negated: negated || op == "not_isnull",
            null_only: true,
        },
        // range: `["a","b"]` → BETWEEN
        "range" => {
            let items: Vec<String> = serde_json::from_str(&value_str).unwrap_or_default();
            if items.len() != 2 {
                return Err(AppError::Validation(
                    "range filter expects [start, end]".into(),
                ));
            }
            return Ok(Some(Filter {
                column: column.into(),
                op: FilterOp::Between,
                values: items,
                negated,
                null_only: false,
            }));
        }
        // 其余（date/year/regex 等）暂不支持：按 exact 处理，避免静默漏数据
        _ => Filter {
            column: column.into(),
            op: FilterOp::Eq,
            values: vec![value_str],
            negated,
            null_only: false,
        },
    };
    Ok(Some(f))
}

/// LIKE 值转义（% _ \）
fn escape_like(s: &str) -> String {
    s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
}

/// GET 列表接口的完整查询参数（axum Query 提取结果）。
/// `query` 与 `or` 都参与 AND 过滤；`or` 的对象内条件以 OR 连接的语义
/// 目前以 AND 近似（go-crud 的 or 数组常见用法也是多条件并集，当前前端未依赖）。
#[derive(Debug, Clone, Default)]
pub struct ListQuery {
    pub paging: Paging,
    pub filters: Vec<Filter>,
}

impl ListQuery {
    pub fn parse(params: &std::collections::HashMap<String, String>, allowed: &[&str]) -> Result<Self, AppError> {
        let paging = Paging::parse(params);
        let mut raws: Vec<String> = Vec::new();
        for key in ["query", "or"] {
            if let Some(v) = params.get(key) {
                if !v.trim().is_empty() {
                    raws.push(v.clone());
                }
            }
        }
        let filters = parse_filters(&raws, allowed)?;
        Ok(ListQuery { paging, filters })
    }
}

/// 把 Filter 列表编译成 WHERE 片段（不含 WHERE 关键字），并把绑定值追加到 params。
/// 片段之间为 AND。
pub fn compile_where(filters: &[Filter], params: &mut Vec<String>) -> String {
    let mut parts: Vec<String> = Vec::new();
    for f in filters {
        let clause = match f.op {
            FilterOp::IsNull => {
                if f.negated {
                    format!("\"{}\" is not null", f.column)
                } else {
                    format!("\"{}\" is null", f.column)
                }
            }
            FilterOp::In => {
                let placeholders = f
                    .values
                    .iter()
                    .map(|v| {
                        params.push(v.clone());
                        format!("${}", params.len())
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                if f.negated {
                    format!("\"{}\" not in ({placeholders})", f.column)
                } else {
                    format!("\"{}\" in ({placeholders})", f.column)
                }
            }
            FilterOp::Between => {
                params.push(f.values[0].clone());
                let p1 = format!("${}", params.len());
                params.push(f.values[1].clone());
                let p2 = format!("${}", params.len());
                format!("\"{}\" between {p1} and {p2}", f.column)
            }
            FilterOp::Eq | FilterOp::Like | FilterOp::ILike | FilterOp::Gt | FilterOp::Gte
            | FilterOp::Lt | FilterOp::Lte => {
                params.push(f.values[0].clone());
                let p = format!("${}", params.len());
                let op_sql = match f.op {
                    FilterOp::Eq => "=",
                    FilterOp::Like => "like",
                    FilterOp::ILike => "ilike",
                    FilterOp::Gt => ">",
                    FilterOp::Gte => ">=",
                    FilterOp::Lt => "<",
                    _ => "<=",
                };
                let expr = if f.op == FilterOp::ILike {
                    // iexact 的值已 lowercase，列也需 lower 才能命中（postgres ilike 天然不区分大小写，
                    // 这里统一用 ilike 语义；mysql 下 like 不区分大小写亦兼容）
                    format!("\"{}\" ilike {p}", f.column)
                } else {
                    format!("\"{}\" {op_sql} {p}", f.column)
                };
                if f.negated {
                    format!("not ({expr})")
                } else {
                    expr
                }
            }
        };
        parts.push(clause);
    }
    parts.join(" and ")
}
