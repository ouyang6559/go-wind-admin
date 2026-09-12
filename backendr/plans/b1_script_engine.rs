//! B1 脚手架：脚本执行引擎（Lua/JS 沙箱）。
//!
//! 目标：补齐 `POST /scripts/test_run` 的真实执行能力、`GET /script/hooks` 的
//! hook 注册表、以及执行日志落库，对齐 Go `pkg/script`（gopher-lua + goja）。
//!
//! 当前接线点（替换实现即可，协议层已完整）：
//! - `src/handlers/script.rs` 的 `fn run_script_with_engine(
//!     language, name, source, input: &HashMap<String, serde_json::Value>
//!   ) -> Result<(), String>`（约 L414）：现返回 `Err("script engine is not integrated ...")`。
//! - `script_list_hook_points`（约 L427）：现返回空 `items`。
//!
//! 依赖选型（写进 `Cargo.toml`，已注释）：
//! ```toml
//! [dependencies]
//! mlua = { version = "0.9", features = ["luajit", "vendored"] }   # Lua：对齐 gopher-lua
//! rquickjs = { version = "0.9", features = ["tokio"] }            # JS：对齐 goja（可选）
//! ```
//!
//! 本文件为**独立编译**的参考实现骨架（不挂进 `src/`），供照抄/裁剪。

use std::collections::{HashMap, HashSet};
use std::time::Duration;

/// 出站 HTTP 白名单域（fail-closed）。来自环境变量 `SCRIPT_HTTP_ALLOWED_DOMAINS`。
fn allowed_outbound_hosts() -> HashSet<String> {
    std::env::var("SCRIPT_HTTP_ALLOWED_DOMAINS")
        .unwrap_or_default()
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect()
}

/// hook 点注册表：name → 允许的脚本语言。对齐 Go 运行时动态聚合语义。
const HOOK_POINTS: &[(&str, &[&str])] = &[
    ("user.after_login", &["lua", "javascript"]),
    ("user.after_create", &["lua"]),
    ("tenant.before_expiry_scan", &["lua", "javascript"]),
    ("order.after_created", &["lua"]),
];

/// 执行上下文：脚本可见的 keyword 参数 + 出站域白名单 + 结果回写区。
pub struct ExecContext<'a> {
    /// `input` 解析后的上下文键值（JSON 值）。
    pub kwargs: &'a HashMap<String, serde_json::Value>,
    /// 脚本内 HTTP 出站允许的主机（空 = 禁止出站）。
    pub allowed_hosts: HashSet<String>,
}

/// 执行结果：`success` + 可选 `error` + 回写 `context` 键值。
pub struct ExecOutput {
    pub success: bool,
    pub error: Option<String>,
    /// 脚本写入 `ctx` 的键值；编码为 JSON 后进入响应的 `context` 字段。
    pub ctx: HashMap<String, serde_json::Value>,
}

/// 主入口：按语言分发到沙箱。语言由 `script::language_engine` 归一为小写。
pub fn run(
    language: &str,
    name: &str,
    source: &str,
    hooks: &ExecContext,
    timeout: Duration,
) -> ExecOutput {
    match language {
        "lua" => run_lua(source, hooks, timeout),
        "javascript" => run_js(source, hooks, timeout),
        other => ExecOutput {
            success: false,
            error: Some(format!("unsupported language: {other}")),
            ctx: HashMap::new(),
        },
    }
}

#[allow(unused_variables)]
fn run_lua(
    source: &str,
    hooks: &ExecContext,
    timeout: Duration,
) -> ExecOutput {
    // mlua 骨架：
    //   let lua = mlua::Lua::new();
    //   // 注入 ctx/script 全局、注册沙箱函数（走 allowed_hosts 域名白名单）
    //   let globals = lua.globals();
    //   globals.set("ctx", mlua::Value::Table(...))?;
    //   // 执行：lua.load(source).set_name(name).exec();
    //   // 超时用 tokio::time::timeout 包一层；上下文隔离用 std::thread::scope + 每线程 Lua
    //   Ok(())
    ExecOutput {
        success: false,
        error: Some("B1 lua engine not wired yet (see plans/b1_script_engine.rs)".into()),
        ctx: HashMap::new(),
    }
}

#[allow(unused_variables)]
fn run_js(source: &str, hooks: &ExecContext, timeout: Duration) -> ExecOutput {
    // rquickjs 骨架：Runtime::new() + Context，注入 console 白名单 stdout，执行 source。
    ExecOutput {
        success: false,
        error: Some("B1 javascript engine not wired yet (see plans/b1_script_engine.rs)".into()),
        ctx: HashMap::new(),
    }
}

/// 校验 hook 名是否注册 + 该 hook 是否允许指定语言。
pub fn hook_supports_language(hook: &str, language: &str) -> bool {
    HOOK_POINTS
        .iter()
        .any(|(h, langs)| *h == hook && langs.contains(&language))
}

/// 供 `GET /script/hooks` 使用的注册表条目（对齐 Go 响应）。
pub fn hooks_registry() -> Vec<serde_json::Value> {
    HOOK_POINTS
        .iter()
        .map(|(name, langs)| {
            serde_json::json!({
                "name": name,
                "languages": langs,
            })
        })
        .collect()
}