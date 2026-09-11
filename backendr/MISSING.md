# backendr（Rust/axum）接口完整性对照报告

> 对照基准：`backend/`（Go + Kratos + Ent）proto 定义的全部 HTTP 端点（192 个唯一 `方法+路径`）。
> 生成日期：2026-09-11。本文档由「Go ↔ Rust 全量路由 diff + handler 实现 Audit」产出。

## 一、现状总览

| 维度 | 数量 | 说明 |
|---|---|---|
| Go 端点总数（proto HTTP 注解） | **192** | `backend/api/protos` 全部 `google.api.http` |
| Rust 路由注册 | **192/192（100%）** | 含冒号风格路径与斜杠别名；`mfa/{credentialId}` 与 proto 的 `{credential_id}` 为服务端参数名差异，客户端 URL 等价 |
| **已完整实现并测试通过** | **55** | 认证 8 + 审计日志 12 + 脚本 11 + 脚本日志 3 + 通知渠道 6 + 在线会话 4 + 站内信收件箱 4 + server-monitor 1 + 任务类型/控制 5 + users/tenants exists 2 + 改密 1 + internal-message/status 1 等 |
| **骨架（返回 501 NotImplemented）** | **141** | 集中在各业务 CRUD，见第三节清单 |

端到端测试：`backendr/tests/e2e.sh`，**51 项断言全绿**（连续 3 轮），覆盖登录/改密/强制下线/刷新令牌重放防护/脚本 CRUD/通知渠道/审计日志等全链路。

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

## 三、遗留清单（141 个 501 骨架 + 专项功能，按建议开发顺序）

### A. 标准业务 CRUD（约 100 个端点，可复制现有模板批量推进）

> 这些模块的表已由 Go/Ent 迁移建好，直接参照 **已实现模块的模板** 开发：
> 简单列表+Get → `src/handlers/audit_logs.rs`（表驱动）；
> 全套 CRUD → `src/handlers/script.rs` + `src/repos/script.rs`（动态 update 的占位符编号已踩平）。
> 注意：postgres 方言（`to_char`/`::timestamptz`/`ilike`）；时间戳参数必须显式 cast，否则 sqlx Any 以 TEXT 传参会报类型错误。

| 模块 | 端点数 | 表 | 要点 |
|---|---|---|---|
| dict_type / dict_entry / language | 6/5/6 | `sys_dict_types` `sys_dict_entries`(+i18n) `sys_languages` | language 有 BatchCreate；dict_entry 关联 type 的唯一约束 |
| position / org_unit | 5/5 | `sys_positions` `sys_org_units` | org_unit 树形（parent_id）；查询常带 `parentId` 过滤 |
| plan / plan_module / plan_quota | 5/5/4 | `sys_plans` `sys_plan_modules` `sys_plan_quotas` | 配额与租户用量联动（GetUsage 在 tenant 模块） |
| login_policy / permission_group | 5/5 | `sys_login_policies` `sys_permission_groups` | login_policy 与认证链路闸门联动（当前 Rust 登录未实现策略闸门，见 D） |
| internal_message / internal_message_category | 6/5 | `internal_messages` `internal_message_categories` | 发送事务要批量写 recipients（当前 Rust 收件箱读侧已就绪） |
| api | 7 | `sys_apis` | 注意 hasPassword 类脱敏列；List 常按 module 分组 |
| role | 5 | `sys_roles`(+metadata/permissions) | 角色授权要写 `sys_role_permissions` |
| menu | 6 | `sys_menus`(+permission_menus) | 树形 + SyncMenus（见 B） |
| file / file_transfer | 5/3 | `files` | 依赖 MinIO/OSS（`oss.yaml`），Rust 需引入 S3 兼容客户端（如 `rust-s3`/`aws-sdk-s3`） |
| admin_portal / dashboard | 3/4 | 聚合查询 | 纯只读统计，实现成本低 |

### B. 需要专项设计的功能（无法简单照模板）

1. **user（7 个端点）**：`GET/POST /users`、`GET/PUT/DELETE /users/{id}`、`GET/DELETE /users/username/{username}`。
   难点：创建用户 = 事务（sys_users + sys_user_credentials + sys_user_roles/memberships）；查询响应聚合角色码/部门；`GET /users` 的 query 过滤字段较多。参照 Go `internal/data/user_repo.go` 与 `user_credential_repo.go`。

2. **tenant（5 个端点）**：`/tenants:with-admin`（创建租户+管理员用户的事务，涉及凭证播种、角色绑定、租户初始化）、`GET /tenants/{id}/usage`（跨表用量统计）、`POST /tenants/{id}/cleanup`（清理租户数据，多表事务）。
   `tenants:exists` 已实现。

3. **permissions/sync:perms**：Go 端从 proto 注册表全量重建 `sys_apis`（租户闸门 fail-closed 依赖它）。Rust 没有 proto 注册表，建议：手工维护一份端点清单（可从 `routes/*.rs` 静态生成），或首次由 Go 实例同步后共享同一 DB。**已部署实例新增端点必须在管理页「接口同步」重建，否则租户闸门 403**。

4. **script TestRun（`POST /scripts/test_run`）**：需要嵌入式脚本引擎。选型建议：`mlua`（Lua 沙箱，对应 gopher-lua）+ `rquickjs`（JS）；需实现出站 HTTP 白名单（`SCRIPT_HTTP_ALLOWED_DOMAINS`，fail-closed）、hook 注册表（`GET /script/hooks` 目前返回空集合）、执行日志落 `sys_script_logs`（trigger_type="test_run"）。**报错仍返回 200 + `success:false`**。

5. **task CRUD + 内嵌调度器（6 个端点）**：`GET/POST/PUT/DELETE /tasks`、`GET /tasks/type-name/{type_name}`。
   Go 用 asynq（Redis 队列）。Rust 等价：`apalis`（redis-backed）或 tokio cron。控制端点已有「调度器未配置」降级；`ListTaskTypeName` 返回系统注册类型（broadcast_message/tenant_expiry_scan/backup/script_task/audit_log_archive）。注意 ControlTask 要求租户上下文（平台上下文 400）、创建时校验 typeName 已注册。

6. **mfa（7 个端点）**：TOTP 因子注册/验证（`sys_user_mfa_factors`）。Rust 需引入 `totp-rs`。登录链路当前未接 MFA 闸门（Go：绑定 ENABLED TOTP → 登录返回 `mfa_operation_id` 走二次验证）。前端 react 已有 MFA 挑战页，建议尽快补齐。

7. **user_profile（7 个端点）**：`/me` 资料读取、改联系方式、绑定邮箱/手机、头像删除（`DELETE /me/avatar` 依赖文件存储）。改密走 `users/{id}/password` 已实现，`POST /me/password` 待做（同一套解密+哈希逻辑）。

### C. 已实现但与 Go 存在行为差异（知悉即可）

| 差异点 | Go | Rust 现状 |
|---|---|---|
| server-monitor 运行时指标 | goroutine 数、GC 次数、Go 版本 | 无 goroutine 概念（0 占位）；rustc 版本（build.rs 未接入，现为常量）；进程内存 Linux /proc 可读，macOS 为 0 |
| 刷新令牌后的 loginAt | Lua 原子轮换，继承首次登录时间 | 刷新 = 新会话条目，loginAt 重新计时 |
| 会话元数据写入时机 | 刷新轮换原子迁移 | 刷新后 IP/UA 显示 `-`（刷新请求未带上下文） |
| 登录策略/限流闸门 | login_policy 表 + IP/用户名双维度限流器 | 未接入（Rust 登录只有验证码+凭证校验）；login_policy CRUD 待做后接线 |
| JWT 算法 | RS256（密钥对） | HS256（`GW_ADMIN_JWT_SECRET`）。token 不跨后端通用，对前端透明 |
| 审计日志写入 | 中间件逐请求落库（geo 解析、设备解析、哈希签名链） | Rust 无对应审计中间件——**Rust 自身流量不会产生审计日志**；DB 里现存的是 Go 实例写入的。需实现 tower 中间件 + GeoIP/UA 解析 + log_hash 签名链 |
| 查询过滤 DSL | go-crud 全量（regex/date/year…） | 已实现常用子集（contains/icontains/in/gt/gte/lt/lte/isnull/range/exact），未知操作符按 exact 兜底 |
| SQL 方言 | Ent 多方言 | 手写 SQL 为 postgres 方言（mysql 需替换 to_char/::cast/ilike） |

### D. 登录链路后续需补齐的闸门（Go 已有，Rust 登录目前没有）

1. 登录限流（IP+用户名双维度失败计数锁定）——Go `rateLimiter`
2. login_policy 闸门（全局 + 用户定向）——依赖 login_policy CRUD
3. MFA 闸门（绑定 ENABLED TOTP → 返回 `mfa_operation_id`）
4. 租户闸门（Kratos 的租户中间件按 `sys_apis` 校验端点可见性）——Rust 中间件目前只做 JWT+黑名单

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
