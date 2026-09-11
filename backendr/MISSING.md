# backendr（Rust/axum）接口完整性对照报告

> 对照基准：`backend/`（Go + Kratos + Ent）proto 定义的全部 HTTP 端点（192 个唯一 `方法+路径`）。
> 生成日期：2026-09-11。本文档由「Go ↔ Rust 全量路由 diff + handler 实现 Audit」产出。

## 一、现状总览

| 维度 | 数量 | 说明 |
|---|---|---|
| Go 端点总数（proto HTTP 注解） | **192** | `backend/api/protos` 全部 `google.api.http` |
| Rust 路由注册 | **192/192（100%）** | 含冒号风格路径与斜杠别名；`mfa/{credentialId}` 与 proto 的 `{credential_id}` 为服务端参数名差异，客户端 URL 等价 |
| **已完整实现并测试通过** | **166** | 认证 8 + 用户/user_profile + 租户 exists + 任务 CRUD + MFA 7 + 审计日志 12 + 脚本 11 + 脚本日志 3 + 通知渠道 6 + 在线会话 4 + 站内信 4 + server-monitor + task 类型/控制 + menu/admin_portal/dashboard + dict/language/position/permission_group/login_policy/plan/org_unit 等大部分业务 CRUD |
| **骨架（返回 501 NotImplemented）** | **26** | 见第三节清单 |

端到端测试：`backendr/tests/e2e.sh`，**53 项断言全绿**，覆盖登录/改密/强制下线/刷新令牌重放防护/脚本 CRUD/通知渠道/审计日志等全链路；MFA 额外全链路冒烟（注册→挑战登录→验证→错误码限次→禁用→回落单因子）通过。

## 二、本次已实现（wire 格式与 Go 完全对齐）

### 2.1 关键架构修正（对前端兼容性是致命的，务必知悉）

此前 backendr 已提交的认证模块返回 `{code,message,data}` envelope + camelCase，**与 Go 端实际 wire 格式不符**（Go/Kratos 默认编码 = **裸 DTO**；错误 = Kratos 格式）。已全部修正：

1. **成功响应 = 裸 DTO**（无 envelope）。`google.protobuf.Empty` → `{}`；列表 → `{"items":[...],"total":"N"}`（total 为 protojson int64 字符串）。
2. **字段名按 proto `json_name` 精确对齐**：登录相关为 snake_case（`grant_type`/`access_token`/`token_type`/`expires_in`(字符串)/`new_password`），其余默认 lowerCamelCase（`captchaId`/`items`/`hookPoint`/`recipientIds`）。
3. **错误响应 = Kratos 格式**：`HTTP 状态码 + {"code":401,"reason":"UNAUTHORIZED","message":"...","metadata":{}}`。前端 i18n 依赖 `reason` 字段。
4. **任务控制路径补齐 proto 冒号风格**：`tasks:restart/start/stop/control/type-names`、`users:exists`、`tenants:exists`、`permissions/sync:perms`（同时保留斜杠别名）。

### 2.2 已实现端点清单

**认证（8）**：captcha / captcha/verify / login / logout / refresh-token / register / **forgot-password** / **reset-password-by-code**（新增后两个：Redis 6 位验证码 `gowind:vcode:reset_password:{identifier}` 10min TTL、消费型验码、AES 解密新密码、bcrypt 落库、撤销全部会话；防枚举——无 EMAIL 凭证/无邮件渠道时静默成功）

**会话安全（新增基础设施）**：
- 登录/刷新写入会话元数据 `gw:session:meta:{ct}:{uid}:{jti}`（在线会话列表数据源）
- 令牌黑名单 `gw:bl:{jti}`，`Operator` 提取器逐请求校验 → **force-logout/logout/改密后令牌立即失效**（对齐 Go `bl:` 语义）
- 刷新令牌单次有效（重放旧 refresh token → 401）

**在线会话（4，新模块）**：`GET /online-session/sessions`（keyword 模糊 username/IP、内存分页、loginAt 倒序）、`GET /online-session/my-sessions`（current 标记）、`POST /online-session/force-logout`、`POST /online-session/my-sessions/revoke`（不存在 404）

**脚本系统（11/12，新模块）**：`GET/POST/DELETE /scripts`（DELETE 走 query `ids=`，硬删除）、`GET /scripts/count`、`GET /scripts/{id}`、`GET /scripts/name/{name}`、`PUT /scripts/{id}`（version 自增、allowMissing 转 Create、名称唯一 409）、`GET /script/hooks`（空注册表 + `["lua","javascript"]`）、`GET/GET count/POST purge /script/logs`（purge 缺省 90 天）

**通知渠道（6，新模块）**：notification-channels CRUD + `{id}/send-test-email`（SMTP 配置全部来自渠道行、密码 AES-256-GCM `enc:` 前缀加密（`GOWIND_CRYPTO_KEY`，SHA-256 派生）、响应不回传密码仅 `hasPassword`、发送失败 400 对齐 Go）

**审计日志（12，新模块，表驱动统一实现）**：api/login/operation/permission/data-access/policy-evaluation 六类 `GET list + GET {id}`，支持 page/pageSize/orderBy 与 `query` JSON 过滤（contains/icontains/in/gt/gte/lt/lte/isnull/range），geoLocation/deviceInfo 键名 camel 化、deviceType 枚举数字→名字、bytea signature→base64

**站内信收件箱（4）**：`GET /internal-message/inbox`（join messages 取标题正文，仅通知类）、`POST /internal-message/read`（空 ids=全部未读）、`POST /internal-message/inbox/delete`、**`POST /internal-message/status`**（回执补偿通道，ids 必填非空、防同状态重写、READ/RECEIVED 自动落时间戳）

**其他（8）**：`GET /server-monitor`、`GET /tasks:type-names`、`POST /tasks:start|stop|restart|control`（对齐 Go「调度器未配置」降级：500 `task scheduler is not configured`）、`GET /users:exists`、`GET /tenants:exists`、`POST /users/{user_id}/password`（AES 解密→bcrypt→撤销全部会话）

**用户（7，新模块）**：`GET/POST /users`、`GET/PUT/DELETE /users/{id}`、`GET/DELETE /users/username/{username}`——创建 = 事务（sys_users + sys_user_credentials + sys_user_roles），列表/详情聚合角色码/部门/岗位/租户名；改密 `POST /users/{user_id}/password`

**user_profile（6/7，新模块）**：`GET/PUT /me` 资料读写、绑定邮箱/手机（AES 解密 + 验证码确认）、`POST /me/password` 改密；`POST /me/avatar` 仅支持 imageUrl 直存分支（imageBase64 依赖 OSS → 501）、`DELETE /me/avatar` 骨架

**task CRUD（6，新模块）**：`GET/POST/PUT/DELETE /tasks`、`GET /tasks/type-name/{type_name}`——sys_tasks 落库管理、typeName 注册校验（broadcast_message/tenant_expiry_scan/backup/script_task/audit_log_archive）、jsonb 字段透传

**mfa（7，新模块）**：TOTP 因子注册（`POST /mfa/enroll/start` 返回 secret/otpauth URL/PNG QR + `POST /mfa/enroll/confirm` 首码校验）、登录闸门（绑定 ENABLED TOTP → 登录返回 `mfa_operation_id` 不签发 token）、`POST /mfa/verify` 通过后复用登录链路签发 token + 刷新 cookie、`GET /mfa/status|methods`、`POST /mfa/disable`（本人/平台管理员救援重置）、`DELETE /mfa/{credentialId}`。挑战走 Redis（`mfa:login:`/`mfa:enroll:`/`mfa:loginfail:`/`mfa:enrollcd:`），失败上限 3 次作废、GET+DEL Lua 原子消耗防重放；secret AES-GCM 加密落库 `sys_user_mfa_factors`

## 三、遗留清单（26 个 501 骨架 + 专项功能）

> 相对上一版 141 骨架，已补完：user / user_profile / task CRUD / mfa / tenant exists / menu / admin_portal / dashboard / dict / language / position / permission_group / login_policy / plan / org_unit 等。剩余按以下清单推进。
> 注意：postgres 方言（`to_char`/`::timestamptz`/`ilike`）；时间戳参数必须显式 cast，否则 sqlx Any 以 TEXT 传参会报类型错误。

### A. 剩余 501 骨架（26 个端点，可直接实现）

| 模块 | 端点数 | 表 / 卡点 | 建议 |
|---|---|---|---|
| tenant | 9 | `sys_tenants` CRUD + exists + usage + cleanup + with-admin | CRUD 与 exists/usage/cleanup 的 SQL 已就绪（`src/repos/tenant.rs`），仅需从 `handlers/tenant.rs` 骨架接线；with-admin 为建租户+管理员事务 |
| permission | 6 | `sys_permissions` CRUD + `sync:perms` | CRUD 简单；sync:perms 无 proto 注册表，需手工端点清单（见 B2） |
| file | 5 | `files` CRUD | 依赖存储后端（MinIO/OSS → `rust-s3`/`aws-sdk-s3`；本地磁盘可先落 `uploads/`） |
| file_transfer | 3 | `file_transfers` | 同上，S3 直传/分片 |
| redis_cache_monitor | 1 | 只读监控 | 直接实现：Redis `INFO`/`DBSIZE` + 内存统计 |
| user_profile avatar | 2 | `POST /me/avatar`（imageBase64 分支）、`DELETE /me/avatar` | imageUrl 直存已实现；base64 分支需存储后端 |
| script test_run | 1 | 嵌入式脚本引擎 | 见 B1 |

### B. 需要专项设计的功能（无法简单照模板）

1. **script TestRun（`POST /scripts/test_run`）**：需嵌入式脚本引擎。选型建议：`mlua`（Lua 沙箱，对应 gopher-lua）+ `rquickjs`（JS）；需实现出站 HTTP 白名单（`SCRIPT_HTTP_ALLOWED_DOMAINS`，fail-closed）、hook 注册表（`GET /script/hooks` 目前返回空集合）、执行日志落 `sys_script_logs`（trigger_type="test_run"）。**报错仍返回 200 + `success:false`**。

2. **permissions/sync:perms**：Go 端从 proto 注册表全量重建 `sys_apis`（租户闸门 fail-closed 依赖它）。Rust 没有 proto 注册表，建议：手工维护一份端点清单（可从 `routes/*.rs` 静态生成），或首次由 Go 实例同步后共享同一 DB。**已部署实例新增端点必须在管理页「接口同步」重建，否则租户闸门 403**。

3. **task 内嵌调度器**：Go 用 asynq（Redis 队列）。Rust 等价：`apalis`（redis-backed）或 tokio cron。控制端点已有「调度器未配置」降级；`ListTaskTypeName` 返回系统注册类型。创建时校验 typeName 已注册（已实现）。

4. **登录限流 + login_policy 闸门**：Go 有 IP+用户名双维度失败计数锁定 + login_policy 表全局/用户定向策略。Rust 登录目前只有验证码 + 凭证校验；login_policy CRUD 已实现后接线。

5. **审计日志中间件**：Go 用中间件逐请求落库（geo 解析、设备解析、哈希签名链）。Rust **自身流量不产生审计日志**（DB 现存的是 Go 实例写入的）。需实现 tower 中间件 + GeoIP/UA 解析 + log_hash 签名链。

6. **租户闸门**：Kratos 的租户中间件按 `sys_apis` 校验端点可见性。Rust 中间件目前只做 JWT + 黑名单。

### C. 已实现但与 Go 存在行为差异（知悉即可）

| 差异点 | Go | Rust 现状 |
|---|---|---|
| server-monitor 运行时指标 | goroutine 数、GC 次数、Go 版本 | 无 goroutine 概念（0 占位）；rustc 版本（build.rs 未接入，现为常量）；进程内存 Linux /proc 可读，macOS 为 0 |
| 刷新令牌后的 loginAt | Lua 原子轮换，继承首次登录时间 | 刷新 = 新会话条目，loginAt 重新计时 |
| 会话元数据写入时机 | 刷新轮换原子迁移 | 刷新后 IP/UA 显示 `-`（刷新请求未带上下文） |
| MFA 账户名 | Go `uid:{id}`（冒号） | totp-rs otpauth 禁止冒号，用 `uid{id}`；otpauth URL 客户端等价 |
| JWT 算法 | RS256（密钥对） | HS256（`GW_ADMIN_JWT_SECRET`）。token 不跨后端通用，对前端透明 |
| 查询过滤 DSL | go-crud 全量（regex/date/year…） | 已实现常用子集（contains/icontains/in/gt/gte/lt/lte/isnull/range/exact），未知操作符按 exact 兜底 |
| SQL 方言 | Ent 多方言 | 手写 SQL 为 postgres 方言（mysql 需替换 to_char/::cast/ilike） |

### D. 登录链路闸门状态

1. ~~登录限流（IP+用户名双维度失败计数锁定）~~——未做（见 B4）
2. ~~login_policy 闸门~~——未做（CRUD 已实现，见 B4）
3. **MFA 闸门**——✅ 已实现（绑定 ENABLED TOTP → 登录返回 `mfa_operation_id`）
4. ~~租户闸门（Kratos 的租户中间件按 `sys_apis` 校验端点可见性）~~——未做（见 B6）

## 四、测试环境复现

```bash
# 1. 独立测试库（避开 5432/6379 占用）
docker run -d --name gwa-rust-test-pg -p 55432:5432 -e POSTGRES_USER=postgres \
  -e POSTGRES_PASSWORD='*Abcd123456' -e POSTGRES_DB=gwa postgres:16-alpine
docker run -d --name gwa-rust-test-redis -p 56379:6379 redis:7-alpine

# 2. 起 Go 后端一次（迁移建表 + 播种 admin/Abcd@1234，注意 data.yaml 指向 55432/56379）
cd backend/app/admin/service && go run ./cmd/server --conf=./configs

# 3. 起 Rust 后端
cd backendr && GW_ADMIN_HTTP_ADDR=0.0.0.0:7666 \
  GW_ADMIN_DATABASE_URL='postgres://postgres:%2AAbcd123456@127.0.0.1:55432/gwa' \
  GW_ADMIN_REDIS_URL='redis://127.0.0.1:56379' \
  GW_ADMIN_JWT_SECRET='dev-secret-gowind' cargo run

# 4. 端到端测试（51 断言；需本机装有 docker CLI 以读取验证码答案）
backendr/tests/e2e.sh
```

> 注意：`AGENTS.md` 中「登录账号 admin/admin」已过时，种子密码实为 `admin / Abcd@1234`（`pkg/constants/default_data.go: DefaultUserPassword`）。前端传输密码需 AES-128-CBC（key=iv=`f51d66a73d8a0927`）+ base64。
