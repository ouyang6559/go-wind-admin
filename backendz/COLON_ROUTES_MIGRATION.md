# 冒号字面路由迁移说明（Resource:Action → go-zero）

本文说明 `backendz`（go-zero）为何用「手动补注册」而非 `.api` 声明的方式承载 Kratos openapi 契约中的 **`资源:动作`（colon 字面）路由**，列出全部差异接口与对应文件路径，供后续调整参考。

---

## 1. 问题背景

Kratos（`backend` 侧）的 openapi / gRPC-gateway 契约大量使用 **`资源:动作`** 风格 URL，例如：

```
POST /admin/v1/tenants:with-admin
GET  /admin/v1/tasks:type-names
POST /admin/v1/permissions/sync:perms
```

`backendz` 作为 go-zero 平替要实现相同 URL，于是产生一个框架差异。

## 2. 根本原因（为何 `.api` 无法声明，只能手动补）

go-zero 的**生成器**（goctl）与**运行时**（patRouter）对路径中 `:` 的处理不一致：

| 层面 | 对 `tasks:control` 的处理 | 结论 |
| --- | --- | --- |
| **goctl 生成器**（`goctl api go` 解析 `.api`） | 不允许路径段内出现字面 `:`，`sync:perms` 直接报 `expected 'IDENT', got ':'` | **无法在 desc/*.api 声明** |
| **运行时 patRouter**（[search/tree.go `match`](file:///Users/liu/Documents/go/gopath/pkg/mod/github.com/zeromicro/go-zero@v1.10.3/core/search/tree.go#L216-L229)） | 仅当段**以 `:` 开头**（如 `/:id`）才视为路径参数；`tasks:control` 冒号不居首 → **按字面精确匹配** | 运行时**支持**此字面路由 |

因此：**契约冒号 URL 只能绕过 goctl，在运行时手动 `AddRoutes` 补注册。**

### 附加约束：不能带 prefix
go-zero 注册时用 `path.Join(prefix, path)` 拼接前缀（[server.go `WithPrefix`](file:///Users/liu/Documents/go/gopath/pkg/mod/github.com/zeromicro/go-zero@v1.10.3/rest/server.go#L253-L265)），其结果**总会插入 `/`**。若把 `:control` 放进带前缀块，会被拼成 `tasks/:control`（变成参数），语义被改写。所以冒号路由必须以**无 prefix + 绝对路径**注册。

### 结论（方案 C 落地方式）
- desc/*.api 里**保留 goctl 可解析的斜杠写法**（同时保证前端现行斜杠 URL 仍可用），并附“代码转移”注释；
- 冒号字面路由**手动**补在独立手写文件 `colon_routes.go`，从 `admin.go`（非 goctl 生成）调用，确保 `goctl api go` 重新生成后**不会丢失**。

## 3. 差异接口对照表（9 个）

| 契约 URL（Kratos openapi，[来源](file:///Users/liu/Documents/go/gopath/src/hummingbot/project/go-wind-admin/backend/app/admin/service/cmd/server/assets/openapi.yaml)） | go-zero 斜杠路由（`.api` 声明，兼容用） | 冒号字面路由（手动补注册） | 对应 handler |
| --- | --- | --- | --- |
| `POST /admin/v1/permissions/sync:perms` | `POST /admin/v1/permissions/sync/perms` | `POST /admin/v1/permissions/sync:perms` | `PermissionSyncPermissionsHandler` |
| `POST /admin/v1/tasks:control` | `POST /admin/v1/tasks/control` | `POST /admin/v1/tasks:control` | `TaskControlTaskHandler` |
| `POST /admin/v1/tasks:restart` | `POST /admin/v1/tasks/restart` | `POST /admin/v1/tasks:restart` | `TaskRestartAllTaskHandler` |
| `POST /admin/v1/tasks:start` | `POST /admin/v1/tasks/start` | `POST /admin/v1/tasks:start` | `TaskStartAllTaskHandler` |
| `POST /admin/v1/tasks:stop` | `POST /admin/v1/tasks/stop` | `POST /admin/v1/tasks:stop` | `TaskStopAllTaskHandler` |
| `GET /admin/v1/tasks:type-names` | `GET /admin/v1/tasks/type-names` | `GET /admin/v1/tasks:type-names` | `TaskListTaskTypeNameHandler` |
| `GET /admin/v1/tenants:exists` | `GET /admin/v1/tenants/exists` | `GET /admin/v1/tenants:exists` | `TenantTenantExistsHandler` |
| `POST /admin/v1/tenants:with-admin` | `POST /admin/v1/tenants/with-admin` | `POST /admin/v1/tenants:with-admin` | `TenantCreateTenantWithAdminUserHandler` |
| `GET /admin/v1/users:exists` | `GET /admin/v1/users/exists` | `GET /admin/v1/users:exists` | `UserUserExistsHandler` |

## 4. 涉及文件

| 文件 | 作用 |
| --- | --- |
| [`internal/handler/colon_routes.go`](file:///Users/liu/Documents/go/gopath/src/hummingbot/project/go-wind-admin/backendz/internal/handler/colon_routes.go) | **手写（勿删）**：`RegisterColonRoutes` 补注册 9 条冒号字面路由，含约束注释 |
| [`admin.go`](file:///Users/liu/Documents/go/gopath/src/hummingbot/project/go-wind-admin/backendz/admin.go#L34-L37) | 入口，`handler.RegisterHandlers` 之后调用 `handler.RegisterColonRoutes`（此文件 goctl 不会覆盖） |
| [`internal/handler/routes.go`](file:///Users/liu/Documents/go/gopath/src/hummingbot/project/go-wind-admin/backendz/internal/handler/routes.go) | goctl **生成的**斜杠路由（含以上 9 条的斜杠版本），重新生成会整份重写 |
| [`desc/task.api`](file:///Users/liu/Documents/go/gopath/src/hummingbot/project/go-wind-admin/backendz/desc/task.api#L106-L123) | `tasks:control/restart/start/stop/type-names` 的斜杠声明 + “代码转移”注释 |
| [`desc/permission.api`](file:///Users/liu/Documents/go/gopath/src/hummingbot/project/go-wind-admin/backendz/desc/permission.api#L65-L69) | `permissions/sync/perms` 的斜杠声明 + 注释 |
| [`desc/tenant.api`](file:///Users/liu/Documents/go/gopath/src/hummingbot/project/go-wind-admin/backendz/desc/tenant.api#L126-L133) | `tenants/exists`、`tenants/with-admin` 的斜杠声明 + 注释 |
| [`desc/user.api`](file:///Users/liu/Documents/go/gopath/src/hummingbot/project/go-wind-admin/backendz/desc/user.api#L83-L87) | `users/exists` 的斜杠声明 + 注释 |

> 契约来源（Kratos）：`backend/api/protos/admin/service/v1/` 下 `i_tenant.proto`、`i_task.proto`、`i_permission.proto`，及对应领域 proto（`identity/…/tenant.proto`、`task/…/task.proto`）。

## 5. 后续调整注意事项 / 排查 Q&A

1. **跑 `goctl api go` 会不会丢冒号路由？**
   不会。冒号路由注册已从生成的 `routes.go` 移到手写 `colon_routes.go`，调用点在非生成的 `admin.go`。重新生成后 `routes.go` 干净、冒号路由照常生效。
2. **新增一个同类 `xxx:action` 路由怎么加？**
   在 `colon_routes.go` 的 `RegisterColonRoutes` 里追加一条 `{Method, Path: "/admin/v1/...:xxx", Handler: ...}`（绝对路径、**不要**带 `WithPrefix`）；若 `.api` 也要留斜杠兼容，就同步在对应 `desc/*.api` 里加斜杠路由并写“代码转移”注释。
3. **为何不用中间件拦截把斜杠转发成冒号？**
   路由意图藏在魔法层、难排障，且 go-zero 运行时本来就支持字面冒号段，无需转发（故不用方案 B 中间件）。
4. **字段绑定**：`body:"*"` 类请求依旧平铺在 JSON body；`:action` 段不是路径参数，不参与字段绑定，与斜杠路由的 handler 完全一致。
5. **接口同步**：若这些是已部署实例的新端点，记得在管理页「接口同步」登记进 Api 表，否则租户闸门 403。