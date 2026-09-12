# backendr 功能级缺口补全：脚手架（plans/）

路由层面 Rust 重写已 **192/192 对齐**（见 `MISSING.md` 一），e2e 70 项全绿；
响应 wire 形状与 Go 对齐（`tests/compare_wire.py`，26 OK + 1 预期 Go 500）。
剩下的缺口是**需要专项设计的功能级能力**（无法简单照模板抄），逐条给出可编译的脚手架文件。

> 这些 `.rs` 是**参考脚手架，未写入 `src/`、不进 crate 构建**，因此不参与三端口门禁。
> 补齐时按各文件顶部的「接线点」复制/合并进对应 `src/` 文件，并在 `MISSING.md` 勾除清单后即可。

## 缺口清单（对应 MISSING.md 三.B）——2026-09-12 状态

| 编号 | 能力 | 脚手架文件 | 状态 |
|---|---|---|---|
| B1 | 脚本执行引擎（Lua/JS 沙箱 + 出站 HTTP 白名单 + hook 注册表 + 日志） | ~~`b1_script_engine.rs`~~ | ✅ 已接线（mlua + boa 0.20，commit e84180e6/a1490d2f） |
| B2 | permissions/sync:perms 协议层 | 已实现（`handlers/permission.rs`） | ✅ 持续维护项：新增端点时同步维护清单 |
| B3 | task 内嵌调度器（tokio-cron / apalis） | `b3_task_scheduler.rs` | ⬜ **唯一遗留**，待手工开发 |
| B4 | 登录限流 + login_policy 闸门 | ~~`b4_login_rate_limit.rs`~~ | ✅ 已接线（commit e135b6e3） |
| B5 | 审计日志中间件（tower） | ~~`b5_audit_middleware.rs`~~ | ✅ 已接线（commit 5b1f7176） |
| B6 | 租户闸门（按 `sys_apis` fail-closed） | ~~`b6_tenant_gate_middleware.rs`~~ | ✅ 已接线（commit 93f5e572） |

## 补全后验收

1. `cargo build` 0 错误；`cargo clippy` 0 新告警。
2. 复用 `tests/compare_wire.py` 复核影响接口 wire 无回归。
3. `GOWIND_CRYPTO_KEY='test-crypto-key-4e2a' backendr/tests/e2e.sh` 70 项全绿。
4. 回到 `MISSING.md` 三.B 对应项打勾、附表更新。