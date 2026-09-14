//! 通用列表查询参数解析（对齐 `docs/list_query_rule.md` 与 go-crud PagingRequest）。
//!
//! 支持：page/pageSize/noPaging/orderBy，以及 `query`/`or` JSON 过滤
//! （`字段__操作符` 语法）。操作符集与 go-crud `operator_converter.go`
//! 的 operatorMap 逐字对齐（eq/equal/not/ne/nin/like/ilike/is_not_null/
//! between/regexp/contains/startswith/endswith/exact 等全部别名族）；
//! **未知操作符 fail-closed 报 400**（对齐 go-crud `unknown query operator`，
//! 绝不静默降级——exact 兜底曾把 `type__not` 反转成 `type = value`）。
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
    /// 时间戳列（handler 按 ColDef::Ts / TS_COLUMNS 标注）：比较/IN/BETWEEN
    /// 的占位符须 `::timestamptz` cast——sqlx Any 以 TEXT 绑参，不 cast 与
    /// timestamptz 列比较会报类型错误（MISSING.md 记录的已知坑）。
    pub ts: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterOp {
    Eq,
    /// go-crud `exact` 语义：LIKE 全值（值中的 %/_ 为通配符）
    LikeExact,
    Ne,
    Like,
    ILike,
    Gt,
    Gte,
    Lt,
    Lte,
    In,
    IsNull,
    Between,
    /// postgres `~` 正则（go-crud REGEXP）
    Regex,
    /// postgres `~*` 正则（go-crud IREGEXP）
    IRegex,
}

/// go-crud operatorMap（`filter/operator_converter.go`）的别名族权威全集：
/// 键 → (FilterOp, 是否 NOT 包装)。与三端前端 `hasOperatorSuffix` 守卫表
/// 同源，两侧任一侧变更须同步（键先小写归一，后端同）。
const OPERATORS: &[(&str, FilterOp, bool)] = &[
    // EQ 族
    ("eq", FilterOp::Eq, false),
    ("equal", FilterOp::Eq, false),
    ("equals", FilterOp::Eq, false),
    // NEQ 族（not 本身就是操作符：type__not → type <> value）
    ("ne", FilterOp::Ne, false),
    ("neq", FilterOp::Ne, false),
    ("not", FilterOp::Ne, false),
    ("not_equal", FilterOp::Ne, false),
    ("not_equals", FilterOp::Ne, false),
    ("not-equal", FilterOp::Ne, false),
    // 数值比较族
    ("gt", FilterOp::Gt, false),
    ("greater_than", FilterOp::Gt, false),
    ("greater-than", FilterOp::Gt, false),
    ("gte", FilterOp::Gte, false),
    ("greater_than_or_equal", FilterOp::Gte, false),
    ("greater_equals", FilterOp::Gte, false),
    ("greater_or_equal", FilterOp::Gte, false),
    ("greater-or-equal", FilterOp::Gte, false),
    ("lt", FilterOp::Lt, false),
    ("less_than", FilterOp::Lt, false),
    ("less-than", FilterOp::Lt, false),
    ("lte", FilterOp::Lte, false),
    ("less_than_or_equal", FilterOp::Lte, false),
    ("less_equals", FilterOp::Lte, false),
    ("less_or_equal", FilterOp::Lte, false),
    ("less-or-equal", FilterOp::Lte, false),
    // LIKE 族
    ("like", FilterOp::Like, false),
    ("ilike", FilterOp::ILike, false),
    ("i_like", FilterOp::ILike, false),
    ("not_like", FilterOp::Like, true),
    ("notlike", FilterOp::Like, true),
    // IN 族
    ("in", FilterOp::In, false),
    ("nin", FilterOp::In, true),
    ("not_in", FilterOp::In, true),
    ("notin", FilterOp::In, true),
    // NULL 族
    ("is_null", FilterOp::IsNull, false),
    ("isnull", FilterOp::IsNull, false),
    ("is_not_null", FilterOp::IsNull, true),
    ("isnot_null", FilterOp::IsNull, true),
    ("isnotnull", FilterOp::IsNull, true),
    ("not_isnull", FilterOp::IsNull, true),
    // BETWEEN 族
    ("between", FilterOp::Between, false),
    ("range", FilterOp::Between, false),
    // REGEXP 族（postgres `~`；mysql 需换 REGEXP，本仓 postgres 方言）
    ("regexp", FilterOp::Regex, false),
    ("regex", FilterOp::Regex, false),
    ("iregexp", FilterOp::IRegex, false),
    ("i_regexp", FilterOp::IRegex, false),
    ("iregex", FilterOp::IRegex, false),
    // contains 族
    ("contains", FilterOp::Like, false),
    ("icontains", FilterOp::ILike, false),
    ("i_contains", FilterOp::ILike, false),
    ("not_contains", FilterOp::Like, true),
    // startswith 族
    ("starts_with", FilterOp::Like, false),
    ("startswith", FilterOp::Like, false),
    ("istarts_with", FilterOp::ILike, false),
    ("i_starts_with", FilterOp::ILike, false),
    ("istartswith", FilterOp::ILike, false),
    // endswith 族
    ("ends_with", FilterOp::Like, false),
    ("endswith", FilterOp::Like, false),
    ("iends_with", FilterOp::ILike, false),
    ("i_ends_with", FilterOp::ILike, false),
    ("iendswith", FilterOp::ILike, false),
    // exact 族（go-crud exact = LIKE 全值，非 EQ）
    ("exact", FilterOp::LikeExact, false),
    ("iexact", FilterOp::ILike, false),
    ("i_exact", FilterOp::ILike, false),
    ("not_exact", FilterOp::LikeExact, true),
];

fn lookup_operator(op: &str) -> Option<(FilterOp, bool)> {
    let lower = op.to_ascii_lowercase();
    OPERATORS
        .iter()
        .find(|(k, _, _)| *k == lower)
        .map(|&(_, o, n)| (o, n))
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

/// handler 约定的时间戳列命名族（白名单列中的 timestamptz 列）：
/// created_at/updated_at/expires_at 等。用于查询过滤占位符的
/// `::timestamptz` cast 判定（注意不含 audit_status 这类字符串列）。
pub const TS_COLUMN_NAMES: &[&str] = &[
    "created_at",
    "updated_at",
    "deleted_at",
    "expires_at",
    "expire_at",
    "expired_at",
    "last_login_at",
    "sent_at",
    "received_at",
    "read_at",
    "archived_at",
    "start_time",
    "end_time",
    "entry_time",
    "login_at",
    "logout_at",
    "next_run_at",
    "last_run_at",
    "start_at",
    "end_at",
];

/// 白名单列集合中的时间戳列子集（lazy 判定，handler 侧一行接入）。
pub fn ts_columns_of(allowed: &[&str]) -> Vec<&'static str> {
    TS_COLUMN_NAMES
        .iter()
        .filter(|c| allowed.contains(c))
        .copied()
        .collect()
}

/// 解析 `query`/`or` 参数中的 JSON 过滤（object 或 object array，数组内为 AND 语义）。
/// `ts_columns`：该表的时间戳列名集合，用于占位符 cast。
pub fn parse_filters(
    raw_queries: &[String],
    allowed: &[&str],
    ts_columns: &[&str],
) -> Result<Vec<Filter>, AppError> {
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
                let (field, op) = split_lookup(k);
                let column = resolve_column(&field, allowed)?;
                let filter = build_filter(&column, &op, v, ts_columns)?;
                if let Some(f) = filter {
                    filters.push(f);
                }
            }
        }
    }
    Ok(filters)
}

/// 拆分 `field__op`。无 `__` 分隔的裸键为 EQ（对齐 go-crud 裸键语义，
/// 空操作符归一为 `eq`）。
fn split_lookup(key: &str) -> (String, String) {
    match key.split_once("__") {
        Some((f, op)) if !op.is_empty() => (f.to_string(), op.to_string()),
        _ => (key.to_string(), "eq".into()),
    }
}

#[allow(clippy::too_many_lines)]
fn build_filter(
    column: &str,
    op: &str,
    value: &Value,
    ts_columns: &[&str],
) -> Result<Option<Filter>, AppError> {
    // go-crud 对未知操作符 fail-closed（`unknown query operator %q` → 400），
    // 绝不静默降级为 exact/eq——那会把 not 族反转、把 between 变恒空。
    let (fop, op_not) = lookup_operator(op).ok_or_else(|| {
        AppError::Validation(format!("unknown query operator: {op}"))
    })?;
    let ts = ts_columns.contains(&column);
    let value_str = match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    };
    let f = match fop {
        // contains/startswith/endswith 系：自动包通配符，值中字面 %/_ 需转义
        FilterOp::Like if op_has_wildcard_pos(op) => Filter {
            column: column.into(),
            op: FilterOp::Like,
            values: vec![format!("%{}%", escape_like(&value_str))],
            negated: op_not,
            null_only: false,
            ts,
        },
        FilterOp::ILike if op_has_wildcard_pos(op) => Filter {
            column: column.into(),
            op: FilterOp::ILike,
            values: vec![format!("%{}%", escape_like(&value_str))],
            negated: op_not,
            null_only: false,
            ts,
        },
        FilterOp::Like if starts_variant(op) => Filter {
            column: column.into(),
            op: FilterOp::Like,
            values: vec![format!("{}%", escape_like(&value_str))],
            negated: op_not,
            null_only: false,
            ts,
        },
        FilterOp::ILike if starts_variant(op) => Filter {
            column: column.into(),
            op: FilterOp::ILike,
            values: vec![format!("{}%", escape_like(&value_str))],
            negated: op_not,
            null_only: false,
            ts,
        },
        // 落到这里的 Like/ILike = like/ilike/endswith 系（endswith 包后缀 %）。
        // like/ilike 显式拼写值原样（用户自传 %/_ 通配符，不转义）。
        FilterOp::Like => {
            let v = if is_bare_like(op) {
                value_str.clone()
            } else {
                format!("%{}", escape_like(&value_str))
            };
            Filter {
                column: column.into(),
                op: FilterOp::Like,
                values: vec![v],
                negated: op_not,
                null_only: false,
                ts,
            }
        }
        FilterOp::ILike => {
            let v = if is_bare_like(op) {
                value_str.clone()
            } else {
                format!("%{}", escape_like(&value_str))
            };
            Filter {
                column: column.into(),
                op: FilterOp::ILike,
                values: vec![v],
                negated: op_not,
                null_only: false,
                ts,
            }
        }
        FilterOp::Eq | FilterOp::LikeExact | FilterOp::Ne => Filter {
            column: column.into(),
            op: fop,
            values: vec![value_str],
            negated: op_not,
            null_only: false,
            ts,
        },
        FilterOp::Gt | FilterOp::Gte | FilterOp::Lt | FilterOp::Lte => Filter {
            column: column.into(),
            op: fop,
            values: vec![value_str],
            negated: op_not,
            null_only: false,
            ts,
        },
        FilterOp::In => {
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
                negated: op_not,
                null_only: false,
                ts,
            }
        }
        FilterOp::IsNull => Filter {
            column: column.into(),
            op: FilterOp::IsNull,
            values: vec![],
            negated: op_not,
            null_only: true,
            ts,
        },
        FilterOp::Between => {
            // between/range: `["a","b"]` → BETWEEN
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
                negated: op_not,
                null_only: false,
                ts,
            }));
        }
        FilterOp::Regex | FilterOp::IRegex => Filter {
            column: column.into(),
            op: fop,
            values: vec![value_str],
            negated: op_not,
            null_only: false,
            ts: false,
        },
    };
    Ok(Some(f))
}

/// 该操作符拼写族的 LIKE 模式是否为前后模糊（contains 系）
fn op_has_wildcard_pos(op: &str) -> bool {
    matches!(
        op.to_ascii_lowercase().as_str(),
        "contains" | "icontains" | "i_contains" | "not_contains"
    )
}

/// 该操作符拼写族是否为 starts_with 系（前缀模糊）
fn starts_variant(op: &str) -> bool {
    matches!(
        op.to_ascii_lowercase().as_str(),
        "starts_with"
            | "startswith"
            | "istarts_with"
            | "i_starts_with"
            | "istartswith"
            | "not_starts_with"
            | "not_startswith"
    )
}

/// 该操作符是否为裸 like/ilike（含 not_like）：值原样，用户自传通配符
fn is_bare_like(op: &str) -> bool {
    matches!(
        op.to_ascii_lowercase().as_str(),
        "like" | "ilike" | "i_like" | "not_like" | "notlike"
    )
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
    pub fn parse(
        params: &std::collections::HashMap<String, String>,
        allowed: &[&str],
        ts_columns: &[&str],
    ) -> Result<Self, AppError> {
        let paging = Paging::parse(params);
        let mut raws: Vec<String> = Vec::new();
        for key in ["query", "or"] {
            if let Some(v) = params.get(key) {
                if !v.trim().is_empty() {
                    raws.push(v.clone());
                }
            }
        }
        let filters = parse_filters(&raws, allowed, ts_columns)?;
        Ok(ListQuery { paging, filters })
    }
}

/// 把 Filter 列表编译成 WHERE 片段（不含 WHERE 关键字），并把绑定值追加到 params。
/// 片段之间为 AND。
pub fn compile_where(filters: &[Filter], params: &mut Vec<String>) -> String {
    // 列可能带表别名前缀（handler 侧拼 "t.status" 等）。整段加引号会变成
    // 字面量列名 "t.status"（postgres 视作一个标识符）→ column does not exist。
    // 须按 "." 分段引用：t.status -> "t"."status"。
    let col = |c: &str| {
        c.split('.')
            .map(|seg| format!("\"{}\"", seg))
            .collect::<Vec<_>>()
            .join(".")
    };
    let mut parts: Vec<String> = Vec::new();
    for f in filters {
        let clause = match f.op {
            FilterOp::IsNull => {
                if f.negated {
                    format!("{} is not null", col(&f.column))
                } else {
                    format!("{} is null", col(&f.column))
                }
            }
            FilterOp::In => {
                let placeholders = f
                    .values
                    .iter()
                    .map(|v| {
                        params.push(v.clone());
                        format!("${}{}", params.len(), cast_suffix(f.ts))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                if f.negated {
                    format!("{} not in ({placeholders})", col(&f.column))
                } else {
                    format!("{} in ({placeholders})", col(&f.column))
                }
            }
            FilterOp::Between => {
                params.push(f.values[0].clone());
                let p1 = format!("${}{}", params.len(), cast_suffix(f.ts));
                params.push(f.values[1].clone());
                let p2 = format!("${}{}", params.len(), cast_suffix(f.ts));
                format!("{} between {p1} and {p2}", col(&f.column))
            }
            FilterOp::Regex | FilterOp::IRegex => {
                params.push(f.values[0].clone());
                let p = format!("${}", params.len());
                let re_op = if f.op == FilterOp::IRegex { "~*" } else { "~" };
                let expr = format!("{} {re_op} {p}::text", col(&f.column));
                if f.negated {
                    format!("not ({expr})")
                } else {
                    expr
                }
            }
            FilterOp::Eq | FilterOp::LikeExact | FilterOp::Ne | FilterOp::Like
            | FilterOp::ILike | FilterOp::Gt | FilterOp::Gte | FilterOp::Lt | FilterOp::Lte => {
                params.push(f.values[0].clone());
                let p = format!("${}{}", params.len(), cast_suffix(f.ts));
                let op_sql = match f.op {
                    FilterOp::Eq => "=",
                    FilterOp::LikeExact => "like",
                    FilterOp::Ne => "<>",
                    FilterOp::Like => "like",
                    FilterOp::ILike => "ilike",
                    FilterOp::Gt => ">",
                    FilterOp::Gte => ">=",
                    FilterOp::Lt => "<",
                    _ => "<=",
                };
                let expr = format!("{} {op_sql} {p}", col(&f.column));
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

/// 时间戳列的占位符 cast 后缀：sqlx Any 以 TEXT 绑参，不 cast 与
/// timestamptz 列比较会报类型错误。
fn cast_suffix(ts: bool) -> &'static str {
    if ts { "::timestamptz" } else { "" }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filters(raw: &str, allowed: &[&str], ts: &[&str]) -> Result<Vec<Filter>, AppError> {
        parse_filters(&[raw.to_string()], allowed, ts)
    }

    #[test]
    fn bare_key_is_eq() {
        // 裸键（无 __）= EQ，query.rs 语义不变项
        let fs = filters(r#"{"status":"ON"}"#, &["status"], &[]).unwrap();
        assert_eq!(fs.len(), 1);
        assert_eq!(fs[0].op, FilterOp::Eq);
        assert_eq!(fs[0].values, ["ON"]);
    }

    #[test]
    fn not_operator_is_ne_not_eq() {
        // 回归：type__not 曾被 exact 兜底反转成 type = 'TEMPLATE'（vue 两端在用）
        let fs = filters(r#"{"type__not":"TEMPLATE"}"#, &["type"], &[]).unwrap();
        assert_eq!(fs.len(), 1);
        assert_eq!(fs[0].op, FilterOp::Ne);
        let mut p = Vec::new();
        let sql = compile_where(&fs, &mut p);
        assert_eq!(sql, r#""type" <> $1"#);
        assert_eq!(p, ["TEMPLATE"]);
    }

    #[test]
    fn ne_family_aliases() {
        for op in ["ne", "neq", "not_equal", "not_equals", "not-equal"] {
            let fs = filters(&format!(r#"{{"type__{op}":"X"}}"#), &["type"], &[]).unwrap();
            assert_eq!(fs[0].op, FilterOp::Ne, "op={op}");
        }
    }

    #[test]
    fn gt_family_aliases() {
        for (op, want) in [
            ("gt", FilterOp::Gt),
            ("greater_than", FilterOp::Gt),
            ("greater-than", FilterOp::Gt),
            ("gte", FilterOp::Gte),
            ("greater_than_or_equal", FilterOp::Gte),
            ("greater_equals", FilterOp::Gte),
            ("greater_or_equal", FilterOp::Gte),
            ("greater-or-equal", FilterOp::Gte),
            ("lt", FilterOp::Lt),
            ("less_than", FilterOp::Lt),
            ("less-than", FilterOp::Lt),
            ("lte", FilterOp::Lte),
            ("less_than_or_equal", FilterOp::Lte),
            ("less_equals", FilterOp::Lte),
            ("less_or_equal", FilterOp::Lte),
            ("less-or-equal", FilterOp::Lte),
        ] {
            let fs = filters(&format!(r#"{{"age__{op}":"18"}}"#), &["age"], &[]).unwrap();
            assert_eq!(fs[0].op, want, "op={op}");
        }
    }

    #[test]
    fn like_family() {
        let fs = filters(r#"{"title__like":"abc%"}"#, &["title"], &[]).unwrap();
        assert_eq!(fs[0].op, FilterOp::Like);
        assert_eq!(fs[0].values, ["abc%"]); // 值原样，不自动包 %
        let fs = filters(r#"{"title__not_like":"abc"}"#, &["title"], &[]).unwrap();
        assert!(fs[0].negated);
        let fs = filters(r#"{"title__ilike":"abc"}"#, &["title"], &[]).unwrap();
        assert_eq!(fs[0].op, FilterOp::ILike);
    }

    #[test]
    fn nin_and_not_in_negate() {
        for op in ["nin", "not_in", "notin"] {
            let fs = filters(&format!(r#"{{"id__{op}":["1","2"]}}"#), &["id"], &[]).unwrap();
            assert!(fs[0].negated, "op={op}");
            assert_eq!(fs[0].op, FilterOp::In);
        }
    }

    #[test]
    fn is_not_null_family() {
        for op in ["is_not_null", "isnot_null", "isnotnull", "not_isnull"] {
            let fs = filters(&format!(r#"{{"memo__{op}":""}}"#), &["memo"], &[]).unwrap();
            let mut p = Vec::new();
            let sql = compile_where(&fs, &mut p);
            assert_eq!(sql, r#""memo" is not null"#, "op={op}");
        }
        for op in ["is_null", "isnull"] {
            let fs = filters(&format!(r#"{{"memo__{op}":""}}"#), &["memo"], &[]).unwrap();
            let mut p = Vec::new();
            assert_eq!(compile_where(&fs, &mut p), r#""memo" is null"#, "op={op}");
        }
    }

    #[test]
    fn contains_and_startswith_wildcards() {
        let fs = filters(r#"{"name__contains":"ab"}"#, &["name"], &[]).unwrap();
        assert_eq!(fs[0].values, ["%ab%"]);
        let fs = filters(r#"{"name__startswith":"ab"}"#, &["name"], &[]).unwrap();
        assert_eq!(fs[0].values, ["ab%"]);
        let fs = filters(r#"{"name__i_contains":"ab"}"#, &["name"], &[]).unwrap();
        assert_eq!(fs[0].op, FilterOp::ILike);
    }

    #[test]
    fn endswith_produces_suffix_wildcard() {
        let fs = filters(r#"{"name__endswith":"ab"}"#, &["name"], &[]).unwrap();
        assert_eq!(fs[0].values, ["%ab"]); // endswith = %ab
    }

    #[test]
    fn exact_is_like_not_eq() {
        // go-crud exact = LIKE 全值（值含 %/_ 时为通配符）
        let fs = filters(r#"{"name__exact":"abc"}"#, &["name"], &[]).unwrap();
        assert_eq!(fs[0].op, FilterOp::LikeExact);
        let mut p = Vec::new();
        assert_eq!(compile_where(&fs, &mut p), r#""name" like $1"#);
    }

    #[test]
    fn unknown_operator_fail_closed() {
        // 未知操作符必须 400（对齐 go-crud），不可降级 exact
        for op in ["regex_x", "date", "year", "week_day", "bogus"] {
            let e = filters(&format!(r#"{{"f__{op}":"v"}}"#), &["f"], &[]).unwrap_err();
            assert!(
                matches!(e, AppError::Validation(ref m) if m.contains(&format!("unknown query operator: {op}"))),
                "op={op}"
            );
        }
    }

    #[test]
    fn unknown_field_fail_closed() {
        let e = filters(r#"{"evil":"1"}"#, &["id"], &[]).unwrap_err();
        assert!(matches!(e, AppError::Validation(_)));
    }

    #[test]
    fn ts_column_cast() {
        // created_at__gte 等时间过滤：占位符必须 ::timestamptz cast
        let fs = filters(
            r#"{"created_at__gte":"2026-01-01T00:00:00Z","created_at__lte":"2026-02-01T00:00:00Z"}"#,
            &["created_at"],
            &["created_at"],
        )
        .unwrap();
        let mut p = Vec::new();
        let sql = compile_where(&fs, &mut p);
        assert_eq!(
            sql,
            r#""created_at" >= $1::timestamptz and "created_at" <= $2::timestamptz"#
        );
        // 非时间列不加 cast
        let fs = filters(r#"{"age__gte":"18"}"#, &["age"], &[]).unwrap();
        let mut p = Vec::new();
        assert_eq!(compile_where(&fs, &mut p), r#""age" >= $1"#);
    }

    #[test]
    fn ts_between_and_in_cast() {
        let fs = filters(
            r#"{"created_at__between":["2026-01-01","2026-02-01"]}"#,
            &["created_at"],
            &["created_at"],
        )
        .unwrap();
        let mut p = Vec::new();
        let sql = compile_where(&fs, &mut p);
        assert_eq!(
            sql,
            r#""created_at" between $1::timestamptz and $2::timestamptz"#
        );
        let fs = filters(
            r#"{"created_at__in":["2026-01-01"]}"#,
            &["created_at"],
            &["created_at"],
        )
        .unwrap();
        let mut p = Vec::new();
        let sql = compile_where(&fs, &mut p);
        assert_eq!(sql, r#""created_at" in ($1::timestamptz)"#);
    }

    #[test]
    fn alias_prefixed_column_split_quoting() {
        // 列带表别名前缀时按 "." 分段引用（09-13 修复的回归项）
        let mut f = filters(r#"{"status":"ON"}"#, &["status"], &[]).unwrap();
        f[0].column = "u.status".into();
        let mut p = Vec::new();
        assert_eq!(compile_where(&f, &mut p), r#""u"."status" = $1"#);
    }

    #[test]
    fn regex_operators() {
        let fs = filters(r#"{"title__regex":"^A"}"#, &["title"], &[]).unwrap();
        let mut p = Vec::new();
        assert_eq!(compile_where(&fs, &mut p), r#""title" ~ $1::text"#);
        let fs = filters(r#"{"title__iregex":"^a"}"#, &["title"], &[]).unwrap();
        let mut p = Vec::new();
        assert_eq!(compile_where(&fs, &mut p), r#""title" ~* $1::text"#);
    }

    #[test]
    fn not_eq_wrapped() {
        let fs = filters(r#"{"type__ne":"X"}"#, &["type"], &[]).unwrap();
        let mut p = Vec::new();
        assert_eq!(compile_where(&fs, &mut p), r#""type" <> $1"#);
    }

    #[test]
    fn operator_case_insensitive() {
        // go-crud 转换前小写归一
        let fs = filters(r#"{"type__NOT":"X"}"#, &["type"], &[]).unwrap();
        assert_eq!(fs[0].op, FilterOp::Ne);
    }
}
