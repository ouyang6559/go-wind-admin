# go-zero .api 文件编写规范

> 基于 go-zero 官方规范（goctl 1.10.1 / go-zero 1.10.3）+ go-wind-admin `backendz` 项目实战经验整理。
> 适用场景：新增/重构 go-zero 服务时，用 `.api` 文件定义路由、请求/响应类型，并一键生成 Go 代码。

---

## 1. 概述

`.api` 是 go-zero 自研的领域特性语言（DSL），用于描述 HTTP 服务。一份 `.api` 文件可生成完整的 API 服务骨架：

```
goctl api go --api desc/admin.api --dir . --type-group --style=go_zero
```

生成结构（`--type-group --style=go_zero`）：

```
internal/
├── config/config.go        # 配置（首次生成后不再覆盖）
├── handler/                # 每个 @server group 一个子目录，文件名为 snake_case
│   ├── routes.go           # 路由注册（每次重新生成）
│   └── <group>/xxx_handler.go
├── logic/                  # 业务逻辑骨架（已存在则不覆盖，Safe to edit）
│   └── <group>/xxx_logic.go
├── middleware/             # @server 中声明了 middleware 才生成
├── svc/service_context.go  # 服务上下文
└── types/                  # --type-group：按 service group 分文件生成
    ├── <group>.go
    └── types.go            # 共享类型（common.api 中的类型）
```

---

## 2. 文件组织规范（backendz 多文件实践）

路由多时**不要**把全部内容写进一个 `.api`，按业务域拆分 + 共享类型抽离：

| 文件 | 作用 |
|---|---|
| `admin.api` | 入口文件：`syntax` + `info` + `import` 全部分组文件；不含路由/类型 |
| `common.api` | 共享类型（`PageRequest`、`User`、`DeviceInfo`、`LoginResponse` 等），被各分组 import |
| `tenant.api` / `user.api` / … | 每个业务域一个文件：该域的类型 + 路由 |

入口文件示例：

```go
syntax = "v1"

info (
    title:   "GoWind Admin API"
    desc:    "由 openapi.yaml 一键生成的后端路由定义"
    author:  "go-wind-admin"
    version: "1.0"
)

//goctl api go --api desc/admin.api --dir . --type-group --style=go_zero
import (
    "common.api"
    "admin_portal.api"
    "authentication.api"
    // ... 其余分组文件
)
```

分组文件约定：

```go
import "common.api"          // 用到共享类型才 import

// 本分组类型定义
type TenantGetReq {
    Id   int64  `path:"id,optional"`
    Code string `form:"code,optional"`
}

@server (
    prefix: "/admin/v1/tenants"   // 该分组所有路由的公共前缀（尽量提取，见 §7）
    group:  tenant                // 对应 internal/handler/tenant 子目录
)
service admin-api {
    @handler TenantList
    get / (PageRequest) returns (ListTenantResponse)

    @handler TenantGet
    get /:id (TenantGetReq) returns (Tenant)
}
```

**关键约定：**

1. **service 名保持一致**：所有分组文件共用同一个 service 名（如 `admin-api`），go-zero 允许同一 service 分多组定义。
2. **`prefix` 提取公共路径**：把 `get /apis`、`get /apis/:id` 这类同前缀路由，提取为 `prefix: /admin/v1/apis` + `get /`、`get /:id`，避免长路径重复（backendz 用 `common_subpath()` 脚本自动计算最长公共前缀）。
3. **路径占位符写法**：goctl 路由用 `:id`（冒号形式），与类型字段的 `path:"id"` 对应；不是 `{id}`。

---

## 3. 语句顺序

`.api` 文件中语句按**从上到下**固定顺序：

```
1. syntax 语句        syntax = "v1"
2. info 语句          info ( title/desc/version ... )
3. import 语句        import "xxx.api"
4. type 结构体声明     type XxxReq { ... }
5. @server 语句块
6. @handler + 路由语句
7. service 声明块
8. 注释语句
```

---

## 4. 类型声明

与 Go 结构体语法几乎一致，区别：

- 必须以 `type` 开头，**不需要 `struct` 关键字**
- **不支持嵌套结构体声明**（即不能在结构体内 `struct { ... }` 内联定义，需要抽成独立类型）
- **不支持 type alias**（`type X = Y`）
- **不支持 `any` 类型**；需要任意类型时用 `map[string]interface{}`
- 单类型写法：`type Foo { ... }`；多类型也可用括号分组 `type ( Foo {...} Bar {...} )`

### 4.1 支持的字段类型

| 类别 | 类型 | 示例 |
|---|---|---|
| 基本类型 | `int64` `string` `bool` `float64` `byte` `float32` 等 | `Age int64` |
| 数组 | `[]T` | `Ids []int64`、`Roles []string` |
| map | `map[K]V` | `I18n map[string]interface{}` |
| 引用类型 | 已定义的结构体 | `Data Tenant`、`Children []MenuRouteItem` |
| 指针 | `*T` | `Meta *MenuMeta` |

> 类型映射经验：OpenAPI/protobuf 的 `integer` → `int64`，`number` → `float64`，`boolean` → `bool`，`string` → `string`，`integer[]` → `[]int64`。

### 4.2 字段命名与 tag

- **Go 字段名**：PascalCase（`ClientId`）。
- **tag 名**：与外部接口保持一致的原始字段名，不做大小写转换。同一个语义字段在不同请求中可能不同（如登录用 `client_id`，设备信息用 `clientId`），以来源接口为准。
- **tag 类型三选一**：

| tag | 用途 | 示例 |
|---|---|---|
| `json` | JSON body 字段（POST/PUT 请求体、响应体） | `` `json:"client_id,optional"` `` |
| `path` | URL 路径参数，必须与路由 `:name` 对应 | `` `path:"id,optional"` `` |
| `form` | query 参数（GET 列表查询） | `` `form:"pageSize,optional"` `` |

```go
type UpdateTenantRequest {
    Id          int64  `path:"id,optional"`   // 与 put /:id 对应
    Data        Tenant `json:"data,optional"` // body
    UpdateMask  string `json:"updateMask,optional"`
    AllowMissing bool  `json:"allowMissing,optional"`
}
```

### 4.3 共享分页请求（建议）

GET 列表接口统一用共享的 `PageRequest`（`common.api`），不要每个 List 接口各自定义分页字段：

```go
type PageRequest {
    Page           int64  `form:"page,optional"`
    PageSize       int64  `form:"pageSize,optional"`
    Offset         int64  `form:"offset,optional"`
    Limit          int64  `form:"limit,optional"`
    Token          string `form:"token,optional"`
    NoPaging       bool   `form:"noPaging,optional"`
    Query          string `form:"query,optional"`
    Filter         string `form:"filter,optional"`
    FilterExprType string `form:"filterExpr.type,optional"`
    OrderBy        string `form:"orderBy,optional"`
    FieldMask      string `form:"fieldMask,optional"`
}
```

---

## 5. 字段 tag 修饰符（参数规则）

修饰符追加在 tag 名之后、用逗号分隔：`` `json:"age,optional"` ``。**仅对 request 类型生效**（response 字段上的 optional 无校验意义，生成器常统一补上）。

| 修饰符 | 说明 | 示例 |
|---|---|---|
| `optional` | 当前字段是可选参数，允许为零值（zero value） | `json:"foo,optional"` |
| `options` | 当前参数仅可接收的枚举值 | `json:"gender,options=male|female"` |
| `default` | 当前参数默认值（缺省时使用） | `json:"gender,default=male"` |
| `range` | 当前参数数值有效范围，仅对数值有效 | `json:"age,range=[0:120]"` |

> `options` 两种写法均支持：`options=foo|bar`（推荐）或 `options=[foo,bar]`。
> 若同时声明 `default` 与 `optional`，则请求缺省该字段时直接使用默认值（等价于该字段必填但有兜底）。

### 5.1 range 区间表达式规则（经源码确认，go-zero 1.10.3）

写法：`range=[min:max]`，括号决定开闭：

| 写法 | 含义 | 左括号 | 右括号 |
|---|---|---|---|
| `[min:max]` | `min <= v <= max`（闭区间） | 含 | 含 |
| `(min:max]` | `min < v <= max`（左开右闭） | 不含 | 含 |
| `[min:max)` | `min <= v < max`（左闭右开） | 含 | 不含 |
| `(min:max)` | `min < v < max`（开区间） | 不含 | 不含 |

边界规则（与网上流传说法不同，以源码 `core/mapping/utils.go#parseNumberRange` 为准）：

1. **min 缺省** → 表示 `-MaxFloat64`（负无穷大），如 `range=[:100]` 即 `v <= 100`。
2. **max 缺省** → 表示 `+MaxFloat64`（正无穷大），如 `range=[1:]` 即 `v >= 1`。
3. **min 与 max 不能同时缺省**（`range=[:]` 报错）。
4. `min > max` 报错。
5. `min == max` 时**仅** `[a:a]` 合法（同时包含两端），`(a:a]`、`[a:a)`、`(a:a)` 均报错。

```go
type AgeReq {
    Age int64 `json:"age,range=[0:120]"`  // 0 <= age <= 120
    Len int64 `json:"len,range=[1:)"`     // len >= 1，无上限
}
```

---

## 6. 路由与服务声明

### 6.1 路由写法

```go
service admin-api {
    @handler Ping
    get /ping                                       // 无请求无响应

    @handler Update
    put /update (UpdateReq)                          // 只有请求体

    @handler List
    get /list returns ([]ListItem)                   // 只有响应体

    @handler Login
    post /login (LoginReq) returns (LoginResp)       // 请求 + 响应

    @handler GetUser
    get /users/:id (GetUserReq) returns (User)       // path 参数
}
```

支持方法：`get` / `post` / `put` / `delete` / `patch` / `head` / `options`。

### 6.2 路由约束（重要）

- **路由 path 不能包含字面量冒号**。goctl 的 search tree 把 `:` 视为路径参数占位符，`/users:exists` 这类 protobuf 自定义方法路由**无法表达**。
  - 变通：统一改写为独立 path，如 `users:exists` → `get /exists`；`tenants:exists` → `get /exists`。保证功能等价即可（参数仍按 query 校验）。
- 路由以 `/` 开头；`prefix` 提取后相对路径写 `/`、`/:id` 等。
- `path:"id"` 字段必须与路由 `:id` 段对应，否则生成校验报错。

---

## 7. @server 配置项

```go
@server (
    prefix:     /v1              // 路由前缀，可区分版本
    group:      user             // 生成到 internal/handler/user 子目录，支持 a/b/c 多级
    jwt:        Auth             // 开启 JWT 认证（需要在 config 中配置）
    middleware: AuthInterceptor  // 为该组路由添加中间件
    timeout:    3s               // 路由超时
    maxBytes:   1024             // 请求体大小上限（byte），goctl >= 1.5.0
)
service admin-api {
    ...
}
```

- `prefix` 与 `group` 是分组的核心属性：**prefix 负责 URL，group 负责代码目录**。
- 同一个 service 名可以有多个 `@server` 块（不同 prefix/group），实现版本区分、认证分组。

---

## 8. 代码生成与校验

### 8.1 生成命令（务必带全参数）

```bash
# 生成 Go 代码（backendz 标准用法，--home 指定项目级模板目录）
goctl api go --api desc/admin.api --dir . --type-group --style=go_zero --home goctl-template

# 语法校验
goctl api validate --api desc/admin.api
```

**血泪教训：`--type-group --style=go_zero` 两个参数不能漏。**
若漏掉，goctl 会：
- 用默认命名（驼峰）生成**整套重复文件**（`types.go`、`servicecontext.go`、`dicttypecreatelogic.go` 等），与既有 snake_case 文件冲突，`go build` 报大量 `redeclared` 错误；
- 生成后清理方法：删除无下划线的驼峰文件（`internal/handler`、`internal/logic` 下 `! -name "*_*"` 且非 `routes.go` 的文件）、重复的 `types.go`、`servicecontext.go`。

### 8.2 生成行为

| 内容 | 行为 |
|---|---|
| `routes.go`、`types/` | 每次重新生成（覆盖） |
| `handler/` | **已存在则跳过，不覆盖**（需先删除 handler 目录再生成，才能套用新模板） |
| `logic/` | 已存在则忽略（保留手写业务逻辑） |
| `etc/*.yaml`、`internal/config`、入口 `main.go` | 已存在则忽略 |

> handler 不含路由信息（路由在 `routes.go`），改路由/响应格式时：先 `rm -rf internal/handler && mkdir internal/handler`，再重新生成，handler 才会用当前模板重写。

### 8.3 类型分组说明

- `--type-group`：每个 service group 的类型生成到独立文件 `internal/types/<group>.go`；无分组的共享类型（`common.api`）生成到 `internal/types/types.go`。
- 不用 `--type-group` 则所有类型合并进一个 `types.go`。

### 8.4 项目级 goctl 模板（code-data 统一响应）

goctl 模板默认在 `~/.goctl/<version>/<category>/`（影响所有项目）；**项目级模板**用 `--home` 指向仓库内目录，随项目走、可版本管理：

```bash
goctl template init --home goctl-template   # 一键物化全部默认模板（backendz 已初始化）
# 修改 goctl-template/api/handler.tpl 后，配合 §8.1 命令即可让生成代码套用新模板
```

模板解析规则：`--home` 下能找到对应模板就用项目的，否则回退 goctl 内置默认模板，因此只需维护改动的 `.tpl`。

**backendz 的 handler.tpl 已改为 `github.com/zeromicro/x/http` 的 code-data 统一响应**（`xhttp.JsonBaseResponseCtx`）：

```go
// 生成效果（有请求体）
func TenantListHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var req types.PageRequest
		if err := httpx.Parse(r, &req); err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
			return
		}
		l := tenant.NewTenantListLogic(r.Context(), svcCtx)
		resp, err := l.TenantList(&req)
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)   // {"code":-1,"msg":"..."} 或 CodeMsg 的 code/msg
		} else {
			xhttp.JsonBaseResponseCtx(r.Context(), w, resp)  // {"code":0,"msg":"ok","data":{...}}
		}
	}
}
```

要点：
- 响应包裹发生在 **handler 层**，`.api` 文件的类型/路由**不需要任何修改**（请求/响应类型仍是业务类型，外层 `{code,msg,data}` 由 xhttp 统一包裹）。
- 无响应体的路由生成 `xhttp.JsonBaseResponseCtx(r.Context(), w, nil)` → `{"code":0,"msg":"ok"}`。
- `xhttp` 的 `wrapBaseResponse` 规则：`*xerrors.CodeMsg`（带业务 code）、`error`（code=-1）、其余（code=0, data）。要返回 `{"code":1001,...}` 这类业务码，logic 中返回 `xerrors.New(1001, "用户名或密码错误")` 即可。
- 模板细节：`httpx` 导入用 `{{if .HasRequest}}` 条件渲染，避免无请求体 handler 的"imported and not used"。
- 依赖：`go get github.com/zeromicro/x@latest`。

---

## 9. 常见坑与规避（backendz 实战）

1. **重复生成文件**：见 §8.1，重跑 goctl 前确认参数；误生成后按命名规律清理。
2. **OpenAPI/protobuf 类型失真**：protobuf `int64` 字段在 OpenAPI JSON 里常显示为 `string`（JSON 序列化约定），`.api` 中应写 `int64`。判定依据是 **proto 源码与 Go 生成代码**，而非 OpenAPI 类型字段。典型：`rangeStart/rangeEnd`（openapi=string，proto=`*int64`）。
3. **Update 请求的 Id 绑定**：`UpdateXxxRequest.Id` 必须用 `path:"id,optional"`（对应 `put /:id`），不要写成 `json`。
4. **批量删除参数**：数组用 `[]int64`（`Ids []int64 \`form:"ids,optional"\``），不要写成 `int64`。
5. **`data` 包裹**：CRUD 的创建/更新请求体统一包一层 `Data` 字段（`Data Tenant \`json:"data,optional"\``），与后端 `{ data: {...} }` 约定一致。
6. **搜索条件**：查询参数一律 contains 语义，ID 类字段不进模糊搜索（后端约定，`.api` 侧体现为参数按需定义）。
7. **校验链路不吞错**：handler 中 `httpx.Parse` 失败直接 `httpx.ErrorCtx` 返回，不要静默忽略。

---

## 10. OpenAPI → .api 参数审计

重构/迁移接口时，可用脚本自动比对参数类型（backendz 位于 `.tmp/parse_openapi/audit_params.py`）：

- 解析 openapi.yaml 的 path/query/body 参数类型；
- 解析 desc 下所有 `.api` 的 route 与类型定义；
- 按 `(method, 归一化路径)` 匹配路由（`{name}`/`:name` 统一为占位符；`:exists` 归一为 `/exists`），逐参数比对类型并输出差异。

```bash
python3 .tmp/parse_openapi/audit_params.py
```

输出示例：

```
参数 rangeStart (query) openapi=string 但 .api=int64 @ FileTransferDownloadFileReq.RangeStart
```

此类差异按 §9.2 判定：以源码（proto / Go 生成代码）为准修正 `.api`。

---

## 11. 参考资料

- goctl api 官方文档：<https://go-zero.dev/docs/tutorials/api/parameter>
- go-zero api 语法参考：<https://go-zero.dev/docs/tutorials>
- 本项目实例：`backendz/desc/`（34 个业务域文件 + `common.api` + `admin.api`）
- 源码参考：`core/mapping/utils.go`（range/options 解析）、`core/search/tree.go`（路由冒号限制）
