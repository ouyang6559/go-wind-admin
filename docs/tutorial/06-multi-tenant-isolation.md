# 06 · 多租户与行级隔离

> 前置：[05 章](./05-permission-model.md)（三轴正交的框架）。读完你会理解：租户上下文怎么进请求、行级隔离在数据层怎么强制、部署侧的 Api 表闸门为什么是 fail-closed、以及隔离的**覆盖边界**——出了边界要自己负责。

## 1. 租户模型

- 每个带租户维度的表有 `tenant_id` 列（ent `mixin.TenantID[uint32]{}` 装配，见第 03 章）；
- 请求进入后端后，令牌中的用户与租户信息随上下文传递；数据层据此构建 **ViewerContext**
  （租户 ID + 平台/系统上下文标志），作为隔离层判定依据。平台管理员上下文（tid=0）跨租户可见；
- 租户与其套餐的绑定由平台管理员在「租户管理 / 套餐管理」页维护；租户侧用户看到的数据范围由本章所述各层自动裁剪。

## 2. HTTP 层闸门：Api 表 `(path, method)`

每个租户请求在业务逻辑之前，按 `(path, method)` 查 **Api 表**（接口注册表，含模块归属）：

- 命中且该租户**套餐允许该模块** → 放行到业务；
- 未命中（表里没这行）或套餐不允许 → **fail-closed 403**。

关键运维事实：

- Api 表**仅在空表时**由启动期从内嵌 OpenAPI 文档自动同步；**已部署实例新增端点后必须在
  管理页「接口管理 → 接口同步」手动触发全量重建**——忘了这步，新端点对租户就是 403；
- 「平台管理员测通了」≠「租户测通了」：平台上下文不走这道闸门，验收要拿租户账号测。

## 3. 数据层：读与写各一条防线

**读隔离（ent 隐私层）**：`mixin.TenantID` 把库的 `TenantPrivacy` 规则**编译进每张租户表的
生成代码**——查询在 SQL 层自动注入 `tenant_id = <viewer tid>`，不是运行时 hook，无法绕过。

**写隔离（`schema/tenant_mutation_guard.go` 的 `TenantMutationGuardPolicy`）**：
Update / UpdateOne / Delete / DeleteOne 全部变更形态经 `WhereP` 注入租户谓词——跨租户的
Update/Delete 命中 0 行而非串数据；缺 ViewerContext 直接拒绝；平台/系统上下文放行。

组合策略：`TenantAndDataScopePolicy`（`schema/data_scope_guard.go`）把租户变更防护与数据范围
查询过滤链在一起——任一子策略拒绝即整体拒绝。

**两条防线的分工**：读隔离来自 go-crud 库（mixin 编译），写隔离在本仓 schema 层；接新表时
用 `mixin.TenantID` 就同时获得读隔离与写隔离的接入面（写侧策略随 `Policy()` 返回值挂载）。

## 4. 数据范围（行级第二维）

同一行隔离层里还有组织维度的过滤（角色五档：全部/本部门/本部门及以下/自定义/仅本人），
由令牌 `dss`/`dsu` claim 承载聚合结果，查询时注入 `created_by = uid` 或
`org_unit_id IN (targets)` 谓词。**V1 仅试点岗位表**，其余表逐表 opt-in 接入。
五档语义、聚合规则、新表接入步骤、fail-closed 退化（单元集超 256 / 空集整体拒绝）：
全部见 [data_scope_design.md](../data_scope_design.md)——这是唯一权威，接入前通读。

## 5. 覆盖边界（重要）

行级隔离覆盖的是 **ent 数据层**。以下路径**不自动注入租户谓词**，业务侧必须自己携带并校验租户维度：

- 应用层自有缓存（Redis key 构造）；
- 对象存储路径（OSS / MinIO 对象 key）；
- 异步任务载荷（asynq 任务体）；
- 任何绕过 ent 的直连 SQL / GORM 路径。

接入新存储或新队列时，先想清楚租户维度怎么进 key / 路径 / 载荷，再写代码。

## 6. 套餐联动（plan / billing）

租户绑定的套餐决定：可用模块白名单（与 Api 表闸门联动）、到期后的只读降级。套餐与配额的
管理页在「套餐管理 / 配额管理」。运维侧注意：租户报 403 或只读，先查套餐模块白名单与到期时间，
再查接口同步。

## 深读

- [data_scope_design.md](../data_scope_design.md) —— 数据范围唯一权威（五档/聚合/接入/运维/边界）
- [frontend_authority.md](../frontend_authority.md) —— 字段级权限（字段轴，与本章行级轴正交）
- `backend/app/admin/service/internal/data/ent/schema/tenant_mutation_guard.go`、
  `data_scope_guard.go` —— 写隔离与组合策略的源码（带注释）
- 根 [AGENTS.md](../../AGENTS.md)「仓库布局」节的 Api 表播种说明 —— 接口同步义务的出处

下一步：[07 · 审计与等保](./07-audit-compliance.md)。
