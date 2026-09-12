# 05 · 权限模型：认证、授权与前端可见性

> 前置：[04 章](./04-first-crud-module.md)（知道一次 CRUD 的样子）。读完你会理解"谁能看到什么、调到什么"的全部轴线，以及每条轴的**生效时延**——这是排查"改了权限怎么没生效/怎么还不生效"的钥匙。

## 1. 认证：从登录到令牌

```
登录页 → 图形验证码（Redis gowind:captcha:<captchaId> 校验）
       → 口令 AES 应用层加密传输（前端 VITE_AES_KEY）
       → 服务端 bcrypt 比对（登录失败按 IP+用户名双维度 Redis 限流）
       → 签发 JWT：access token（仅内存）+ refresh token（HttpOnly Cookie）
```

- **为什么这么分**：access 放内存抗 XSS 窃取，refresh 走 HttpOnly Cookie 抗脚本读取；
  服务端可随时吊销 refresh（在线用户管理 → 强制下线）。
- 每个请求由 transport 中间件验签，把 `UserTokenPayload`（用户/租户/角色/数据范围/字段黑名单等
  claim）注入上下文——后面所有轴都从这个上下文出发。
- 登录方式可配置（账号/邮箱/手机号 + 登录策略），TOTP MFA 可选绑定；这些是管理页里的配置，不是代码改动。

## 2. 授权的三个正交轴

本仓库的行级/接口级控制分属三个独立机制（**正交叠加，互不替代**）：

| 机制 | 管什么 | 层 |
|---|---|---|
| authz 引擎（casbin/opa/zanzibar） | 用户能否调用某个**接口/操作** | HTTP 中间件 + 策略引擎 |
| 租户隔离（TenantPrivacy） | 行属于哪个**租户**（tenant_id 列） | ent 隐私层 |
| 数据范围 | 租户内能看到哪些**业务行** | ent 查询隐私规则（逐表 opt-in） |

本章讲第一轴 + 前端可见性；行级两轴在[第 6 章](./06-multi-tenant-isolation.md)。
字段级（字段轴）见本章第 5 节。

## 3. authz 引擎（接口级）

- 配置在 `configs/auth.yaml` 的 `authz.type`：`noop`（默认，引擎全放行——**生产必须切
  casbin/opa/zanzibar 才有接口级拦截**，注意 noop 下租户闸门与行级隔离仍然生效，别误以为
  "没开引擎=没权限"）、`casbin`、`opa`、`zanzibar`（keto / open_fga）；
- 策略存数据库（角色—权限—接口映射），管理页变更后**热更新**，无需重启；
- 每次判定落 **PolicyEvaluationLog**（策略评估日志，含 trace_id 与评估上下文）——
  "为什么这个人能/不能调这个接口"在审计页可回溯。

## 4. 前端可见性：路由与按钮

**路由权限**有两种模式（`.env` 的 `VITE_ROUTER_ACCESS_MODE`）：

| 模式 | 原理 | 适用 |
|---|---|---|
| `frontend` | 路由与角色映射写死在前端，按登录角色过滤路由表 | 角色固定的小系统 |
| `backend` | 路由表由后端接口动态下发（`ListRoute`），前端 `router.addRoute` 挂载 | 权限复杂/动态的系统 |

**按钮权限**走**权限码**：登录后前端调 `ListPermissionCode` 拿到该用户的权限码数组
（如 `['AC_100100', ...]`），页面用 `AccessControl` 组件 / `hasAccessByCodes` /
`v-access:code` 指令包裹受控按钮。权限码与角色的配置在「权限管理」页，服务端接口级仍由
authz 引擎兜底——前端隐藏只是体验，不是安全边界。

这一节的全部代码样例（三端组件用法、路由 meta、字段写法）以
[frontend_authority.md](../frontend_authority.md) 为权威。

## 5. 字段级权限（V1，试点：用户表）

第三条可见性轴：角色可配置一组**黑名单字段**，受控用户在界面上看不到、接口响应里也拿不到。

- 服务端：黑名单在登录/刷新时聚进令牌 `hfs` claim；读路径按黑名单清值（字段从响应里
  **消失**），写路径清值并从 `field_mask` 剔除（越权字段被静默剥离，不会用零值覆盖库里真值）；
  `/me` 刻意不裁剪（自编辑需要全字段）；
- 前端：`ListPermissionCode` 的响应同时下发 `hiddenFields`（按资源分组的黑名单清单），三端
  access store 持久化，列表列/搜索项/详情/编辑抽屉按其过滤；
- **前端裁剪只是体验层，服务端才是权威**：绕过前端直接调接口，受控字段同样不可读不可写。

## 6. 生效时延（运维必背）

| 变更 | 生效时机 |
|---|---|
| 权限码/角色—权限映射（authz 策略） | 热更新，下次请求即生效 |
| 数据范围档位、字段黑名单（令牌 claim 承载） | **最迟随下次刷新令牌**（access token TTL 内）；要立即生效用「在线用户 → 强制下线」 |
| 路由/菜单可见性 | 重新登录后（路由表与权限码均在登录时拉取） |

排查"改了没生效"先对号入座：不是热更新的轴，等 TTL 或强制下线，别先去改代码。

## 深读

- [frontend_authority.md](../frontend_authority.md) —— 路由两种模式、按钮权限三端用法、字段级 V1 的权威说明
- [data_scope_design.md](../data_scope_design.md) —— 三轴关系表（第 1 节）与数据范围全部细节
- 根 [AGENTS.md](../../AGENTS.md) 与 [docs/frontend_authority](../frontend_authority.md) 的"项目代码"节 —— 前端权限的工程现状

下一步：[06 · 多租户与行级隔离](./06-multi-tenant-isolation.md)。
