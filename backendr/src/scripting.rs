//! 脚本执行引擎（test_run）：Lua 用 `mlua`（vendored lua5.4），JavaScript 用
//! `boa_engine`（纯 Rust）。对齐 Go `pkg/scripting` 的一次性隔离语义：
//! - `input` 载入执行上下文 `ctx`（Lua `ctx:get/set`；JS `__get_ctx/__set_ctx`）；
//! - 执行后返回完整 `ctx` 数据（键 → serde_json::Value），由上层逐项编码为 JSON 字符串；
//! - 引擎与 VM 都留在超时线程内，仅送回 `Send` 的输出，死循环不会卡住请求。

use std::collections::HashMap;
use std::time::Duration;

use serde_json::Value;

/// 默认执行超时（防死循环阻塞请求）。
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// 在隔离 VM 中执行脚本。成功返回执行后完整 ctx 数据，失败返回错误字符串。
pub fn run(
    language: &str,
    _name: &str,
    source: &str,
    input: &HashMap<String, Value>,
) -> Result<HashMap<String, Value>, String> {
    if source.trim().is_empty() {
        return Err("script source is empty".into());
    }
    let lang = language.trim().to_ascii_lowercase();
    let source = source.to_string();
    let input = input.clone();

    // 引擎/VM 生命周期与宿主隔离：放线程里跑，超时则放弃本轮发送错误。
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = match lang.as_str() {
            "lua" => run_lua(&source, &input),
            "javascript" | "js" => run_js(&source, &input),
            other => Err(format!("unsupported script language: {other}")),
        };
        let _ = tx.send(result);
    });
    rx.recv_timeout(DEFAULT_TIMEOUT)
        .unwrap_or_else(|_| Err("script execution timed out".into()))
}

fn mlua_err(e: mlua::Error) -> String {
    e.to_string()
}

/// Lua 引擎：把 input 载入 `ctx` 表，metatable `__index` 暴露 `get/set/stop`，
/// 执行后遍历 `ctx` 自身键读出 JSON。
fn run_lua(source: &str, input: &HashMap<String, Value>) -> Result<HashMap<String, Value>, String> {
    use mlua::{Lua, LuaSerdeExt, Table, Value as LVal};

    let lua = Lua::new();
    let ctx: Table = lua.create_table().map_err(mlua_err)?;
    for (k, v) in input {
        ctx.set(k.as_str(), lua.to_value(v).map_err(mlua_err)?)
            .map_err(mlua_err)?;
    }

    // metatable __index 放方法，避免它们成为 ctx 数据键
    let methods = lua.create_table().map_err(mlua_err)?;
    methods
        .set(
            "get",
            lua.create_function(|_, (tab, k): (Table, mlua::String)| tab.raw_get::<LVal>(k))
                .map_err(mlua_err)?,
        )
        .map_err(mlua_err)?;
    methods
        .set(
            "set",
            lua.create_function(|_, (tab, k, v): (Table, mlua::String, LVal)| {
                tab.raw_set(k, v)?;
                Ok(())
            })
            .map_err(mlua_err)?,
        )
        .map_err(mlua_err)?;
    methods
        .set(
            "stop",
            lua.create_function(|_, reason: mlua::String| {
                Err(mlua::Error::RuntimeError(format!(
                    "script stopped: {}",
                    reason.to_str().unwrap_or("")
                )))
            })
            .map_err(mlua_err)?,
        )
        .map_err(mlua_err)?;

    let mt = lua.create_table().map_err(mlua_err)?;
    mt.set("__index", methods).map_err(mlua_err)?;
    ctx.set_metatable(Some(mt)).map_err(mlua_err)?;
    lua.globals().set("ctx", ctx.clone()).map_err(mlua_err)?;

    lua.load(source).exec().map_err(mlua_err)?;

    let mut out = HashMap::new();
    for pair in ctx.clone().pairs::<mlua::String, LVal>() {
        let (k, v) = pair.map_err(mlua_err)?;
        let jv: Value = lua.from_value(v).map_err(mlua_err)?;
        out.insert(k.to_str().map_err(mlua_err)?.to_string(), jv);
    }
    Ok(out)
}

/// 把 serde_json::Value 转换到 boa 的 JsValue（手工遍历，避免依赖可选 feature）。
fn json_to_js(v: &Value, context: &mut boa_engine::Context) -> Result<boa_engine::JsValue, String> {
    use boa_engine::object::builtins::JsArray;
    use boa_engine::object::ObjectInitializer;
    use boa_engine::{JsString, JsValue};

    Ok(match v {
        Value::Null => JsValue::null(),
        Value::Bool(b) => JsValue::from(*b),
        Value::Number(n) => JsValue::from(n.as_f64().unwrap_or_default()),
        Value::String(s) => JsValue::from(JsString::from(s.as_str())),
        Value::Array(a) => {
            let arr = JsArray::new(context);
            for x in a {
                arr.push(json_to_js(x, context)?, context)
                    .map_err(|e| e.to_string())?;
            }
            JsValue::from(arr)
        }
        Value::Object(map) => {
            let obj = ObjectInitializer::new(context).build();
            for (k, vet) in map {
                obj.set(
                    JsString::from(k.as_str()),
                    json_to_js(vet, context)?,
                    true,
                    context,
                )
                .map_err(|e| e.to_string())?;
            }
            JsValue::from(obj)
        }
    })
}

fn current_ctx(context: &mut boa_engine::Context) -> Result<boa_engine::JsObject, boa_engine::JsError> {
    context
        .global_object()
        .get("ctx", context)?
        .as_object()
        .cloned()
        .ok_or_else(|| boa_engine::JsNativeError::typ().with_message("ctx object missing").into())
}

fn get_ctx_fn(
    _this: boa_engine::JsValue,
    args: &[boa_engine::JsValue],
    context: &mut boa_engine::Context,
) -> Result<boa_engine::JsValue, boa_engine::JsError> {
    use boa_engine::JsArgs;
    let k: String = args
        .get_or_undefined(0)
        .clone()
        .try_js_into(context)
        .map_err(|_| boa_engine::JsNativeError::typ().with_message("ctx key must be a string"))?;
    let ctx = current_ctx(context)?;
    ctx.get(boa_engine::JsString::from(k), context)
}

fn set_ctx_fn(
    _this: boa_engine::JsValue,
    args: &[boa_engine::JsValue],
    context: &mut boa_engine::Context,
) -> Result<boa_engine::JsValue, boa_engine::JsError> {
    use boa_engine::JsArgs;
    let k: String = args
        .get_or_undefined(0)
        .clone()
        .try_js_into(context)
        .map_err(|_| boa_engine::JsNativeError::typ().with_message("ctx key must be a string"))?;
    let v = args.get_or_undefined(1).clone();
    let ctx = current_ctx(context)?;
    ctx.set(boa_engine::JsString::from(k), v, true, context)?;
    Ok(boa_engine::JsValue::undefined())
}

/// JS 引擎：input 展开进全局 `ctx` 对象，注册 `__get_ctx/__set_ctx` 读写，
/// 执行后经 `JSON.stringify(ctx)` 回读完整数据。
fn run_js(source: &str, input: &HashMap<String, Value>) -> Result<HashMap<String, Value>, String> {
    use boa_engine::native_function::NativeFunction;
    use boa_engine::object::ObjectInitializer;
    use boa_engine::property::Attribute;
    use boa_engine::{Context, JsValue, Source};

    let mut context = Context::default();

    let ctx = ObjectInitializer::new(&mut context).build();
    for (k, v) in input {
        ctx.set(
            boa_engine::JsString::from(k.as_str()),
            json_to_js(v, &mut context)?,
            true,
            &mut context,
        )
        .map_err(|e| e.to_string())?;
    }
    context
        .register_global_property("ctx", JsValue::from(ctx), Attribute::all())
        .map_err(|e| e.to_string())?;
    context
        .register_global_builtin_callable("__get_ctx", 1, NativeFunction::from_fn_ptr(get_ctx_fn))
        .map_err(|e| e.to_string())?;
    context
        .register_global_builtin_callable("__set_ctx", 2, NativeFunction::from_fn_ptr(set_ctx_fn))
        .map_err(|e| e.to_string())?;

    context
        .eval(Source::from_bytes(source.as_bytes()))
        .map_err(|e| e.to_string())?;

    let out_str: String = context
        .eval(Source::from_bytes(b"JSON.stringify(ctx)"))
        .map_err(|e| e.to_string())?
        .try_js_into(&mut context)
        .map_err(|e| e.to_string())?;

    let json: Value = serde_json::from_str(&out_str).map_err(|e| e.to_string())?;
    Ok(json
        .as_object()
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .collect())
}