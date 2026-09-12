# 04 · 第一个业务模块：端到端 CRUD 实战

> 前置：[03 章](./03-codegen-chain.md)。本章把"新增一个业务资源（后端 + 前端）"完整走一遍。
> **顺序是铁律：先后端、后前端**——前端的 `apiClient.<Entity>Service` 由后端 BFF proto 生成，后端没走完第 3 步，前端无从谈起。

本章是**路线图**，每步的逐条约定、样例文件索引、边界情形在仓库的 CRUD 手册里：
[`.zcode/skills/add-crud-module/references/backend.md`](../../.zcode/skills/add-crud-module/references/backend.md)
与三个框架各自的 `references/<framework>.md`。写代码前先读对应章节。

## 0. 动手前先回答五个问题

1. 实体名三形：PascalCase / snake_case / 中文标签；
2. 业务字段清单（审计字段 mixin 自动加，不列）；
3. 归属 domain：已有（`dict`/`permission`/`identity`/…）还是全新（全新要多动一处 buf 配置）；
4. 哪些前端：react / vue-element / vue-vben（后端必然包含）；
5. 模块形态：普通 CRUD / 树（parent_id）/ 主从（如 dict_type+dict_entry）/ 租户表。

## 后端十步

### 1–2. domain proto + BFF proto

- domain：`api/protos/<domain>/service/v1/<entity>.proto`，包名 `<domain>.service.v1`，**无 HTTP 注解**。
  标准 5 RPC（List/Count/Get/Create/Update/Delete）全是空体；Get 用 `oneof query_by` + `view_mask`；
  Update 携带 `update_mask` 与 `allow_missing`（upsert）。
- BFF：`api/protos/admin/service/v1/i_<entity>.proto`，**不定义任何 message**，只 import domain 并声明
  `google.api.http` 绑定，路由前缀一律 `/admin/v1/<entities>`。
- 全部字段 `optional`，`json_name` camelCase；审计字段编号固定
  `created_by/updated_by/deleted_by = 100/101/102`、`created_at/updated_at/deleted_at = 200/201/202`。
- 照抄样例：`dict_type.proto` + `i_dict_type.proto`（多维查询与批量删除见 `i_dict_type.proto` 的
  `additional_bindings`）。

### 3. 生成 Go 代码

```bash
cd backend && gow api && make openapi
```

产物含 `Register<Entity>ServiceHTTPServer`。前端在范围内则接着 `make ts` 重新生成三端 TS。

### 4–5. ent schema + 生成

`internal/data/ent/schema/<entity>.go`：`Annotations()`（表名 `sys_<entities>`、注释）、
`Fields()`（业务字段全部 `Optional().Nillable()`）、`Mixin()`（第 03 章第 4 节的清单）、
`Indexes()`。树形态加 `edge.To("children", ...).From("parent").Field("parent_id")`。

```bash
cd backend/app/admin/service && gow ent
```

### 6. Repository

`internal/data/<entity>_repo.go`，照抄 `api_repo.go`（含枚举转换器的完整版）或
`dict_type_repo.go`（纯标量简版）。骨架要点：

- 结构体 10 个泛型参数**顺序固定**（Query/Select/Create/CreateBulk/Update/UpdateOne/Delete/Predicate/DTO/Entity）；
- `init()` 里**必须**装时间转换器对（`NewTimeStringConvertPair` + `NewTimeTimestamppbConverterPair`），
  漏了就是审计字段静默零值；枚举字段加 `EnumTypeConverter`（两端名字必须逐字一致）；
- 五个方法的调用形态（`ListWithPaging(ctx, builder, builder.Clone(), req)`、`Get` 的
  `view_mask`、Create 的 `SetNillable*` 链、`UpdateX(ctx, builder, req.Data, req.GetUpdateMask(), ...)`、
  Delete 的 whereCallback）在手册 Step 6 有逐条样例。

**带关联（M2M/O2M 边）的 Update 有强制模式**：快照载荷 → 黑名单出 mask → `UpdateX` 只写列字段 →
同事务内 Replace 关联。跳过任何一步 = SQL 报错或静默清空关联表。照抄 `role_repo.go` / `user_repo.go`。

### 7. Service

`internal/service/<entity>_service.go`，照抄 `dict_type_service.go`。要点：

- List/Get 直通 repo；Create/Update 注入 operator（`auth.FromContext`）写 `created_by`/`updated_by`，
  Update 强制把 `updated_by` 追加进 mask；
- 入口校验 `req.Data == nil` 返回 `adminV1.ErrorBadRequest`；错误一律 `adminV1.ErrorXxx`，
  不用 `errors.New`。

### 8–9. 注册与装配

```bash
cd backend && make register ENTITY=<entity>
```

一条命令在 `register:*` 标记处注入五处（`wiring_ent.go` 的 repo/service/`NewRestServer` 实参、
`rest_server.go` 的形参与 `Register<Entity>ServiceHTTPServer` 调用）。幂等可重跑；
构造函数有额外依赖时手工补。

### 10. 构建与验证

```bash
cd backend/app/admin/service && make build
```

起来后 <http://localhost:7788/docs> 应出现新资源与路由；用前端或 curl 打一轮 List/Create/Update/Delete。

## 前端（每端读各自的框架手册）

三端机制差异很大，**不可互相照抄**：

| | 列表 | 表单 | 刷新 |
|---|---|---|---|
| react | ProTable | DrawerForm + formRef | `queryClient.invalidateQueries({ queryKey: ['listXxx'] })` |
| vue-element | ProPage（配置驱动） | ProModal + ElForm `:rules` | `pageRef.value?.refresh()`（invalidate 在此端无效） |
| vue-vben | VxeGrid + proxyConfig | useVbenDrawer + useVbenForm | `gridApi.reload()`（挂在 drawer 的 onOpenChange） |

两条跨端铁律：

- **Update 必须带 updateMask，且用生成的 `useUpdateXxx`**（内部 `makeUpdateMask(Object.keys(values))`）。
  后端 `FilterByFieldMask` 对漏 mask 的 DTO 是**全字段直写**——手拼 mask 或不带 mask 的旧表单
  round-trip 会静默污染未触碰的列；
- **`PaginationQuery` 必须 `new` 实例化**（`.toRawParams()` 依赖实例 getter），对象字面量直接崩。

i18n：页面文案全部走 `$t`/`t` + locale JSON（`import.meta.glob` 自动注册，别手动注册）；
菜单标题在单独的 `routes.json`（react/vue-element）/ `menu.json`（vben），路由 `meta.title`
指过去。三端的已知坑（搜索表单布局、窄栏定式、EP Splitter、IAB 截图假回归等）见各端
`frontend/admin/*/AGENTS.md`——那是各端工程约定的权威文档。

## 部署侧收尾（新手最常漏的一步）

- **全新库**：首启自动播种一切（含 Api 表行、菜单），无需动作；
- **已部署实例**：新端点**不会**自动进 Api 表——必须在管理页「接口管理 → 接口同步」全量重建，
  否则租户请求被 `(path, method)` 闸门 fail-closed 403（第 6 章讲为什么）；新菜单走「菜单管理」
  页面加（改 `pkg/constants/default_data.go` 只影响全新库）；
- 把整个链路跑通后，对照手册的 completion checklist 逐项勾。

## 深读

- [`.zcode/skills/add-crud-module/references/backend.md`](../../.zcode/skills/add-crud-module/references/backend.md)
  —— 后端十步完整手册（样例索引、关联表模式、陷阱清单）
- 同目录 `references/react.md` / `vue-element.md` / `vue-vben.md` —— 三端各自的页面实现手册
- [list_query_rule.md](../list_query_rule.md) —— List 请求的过滤/排序/字段掩码协议
- [backend_project_struct.md](../backend_project_struct.md) —— 目录职责

下一步：[05 · 权限模型](./05-permission-model.md)。
