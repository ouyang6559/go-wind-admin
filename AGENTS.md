# AGENTS.md — go-wind-admin Monorepo 开发指南（AI Agent 入口）

本文件是 monorepo 总入口：定位布局、指路各端规范、声明全仓铁律。**深入开发前必读对应端的 AGENTS.md**。

## 仓库布局

```
backend/                    Go + Kratos + Ent + Wire（HTTP :7788，SSE 网关 :7789）
frontend/admin/
├── react/                  React 18 + antd 6 + ProComponents + TanStack Query + zustand
├── vue-element/            Vue 3 + Element Plus + vxe-table + TanStack vue-query + Pinia
└── vue-vben/               Vben Admin 5.x monorepo（apps/admin + packages/*）+ Ant Design Vue
docs/                       后端部署/开发环境/前端权限等专题文档
```

## 三端门禁（必须保持全绿）

| 端 | 命令（在各自目录下） | dev 端口 |
|---|---|---|
| react | `npm run typecheck` | 5888 |
| vue-element | `npx vue-tsc --noEmit`（或 `npm run type-check`） | 5777 |
| vue-vben | `pnpm run check:type` | 5667 |

2026-09-07 起三端 typecheck 全部 0 错误。**门禁出现任何新报错，一律当作自己引入的 bug 修复**，不存在"可忽略的既有错误"。改完代码先跑门禁再声称完成。

## 全仓铁律

1. **不吞错**：任何 catch 至少二选一——`console.error/warn` 带出**原始错误对象**，或重新抛出。用户可见的通知/Message ≠ 日志（只有翻译文案）。合法裸 catch 仅限纯本地 best-effort 兜底且注释写明原因。历史教训：认证链路静默吞错曾让 bug 排查耗时数日。
2. **vue-vben 工具链版本已钉死**（catalog 精确版本 + packageManager 匹配本机 pnpm）：禁止改回 `^` 范围、禁止顺手升级 vue/typescript/vue-tsc/pnpm。原因与升级流程见 `frontend/admin/vue-vben/AGENTS.md`「工具链与已知坑」。
3. **搜索条件一律 contains 而非 EQ**、ID 类字段不进模糊搜索；CRUD 请求体必须包 `{ data: {...} }`——细节见 `.zcode/skills/add-crud-module/SKILL.md`。

## 开发策略：react 先行，其余移植

新功能/新模块以 **react 端为行为基准先实现**，验证通过后再移植到 vue-element / vue-vben。移植是"有参照的翻译"，远比三端并行首创便宜；vue-vben 框架变体语料薄，直接首创容易产出框架级错误（详见其 AGENTS.md）。

**CRUD 模块**：使用 `/add-crud-module` skill（后端 + 前端端到端流程）。

## 本地验证要点

- 后端起在 `:7788`（启动方式见 `docs/windows-startup-guide.md` / `docs/backend_deploy.md`）；前端 dev 端口见上表，代理已配置好 API 转发。
- 登录账号 `admin / admin`（dev 默认）。图形验证码的答案可在 Redis 中按 `gowind:captcha:<captchaId>` 直接读取，便于自动化验证。
- vue-element 在 dev 下若见 router-view 塌空/白屏：先重启 dev server 再下结论（vite 依赖优化竞态已做遏制与自愈，见其 AGENTS.md「dev 白屏处置」）。

## 文档索引

- 各端规范：`frontend/admin/{react,vue-element,vue-vben}/AGENTS.md`
- 后端：`docs/backend_project_struct.md`、`docs/backend_deploy.md`、`docs/audit-log-producer-design.md`
- 前端权限模型：`docs/frontend_authority.md`
- 查询/分页规则：`docs/list_query_rule.md`
- 脚本系统：`docs/script_system.md`（Lua/JS 脚本级插件：钩子点/定时任务/HTTP 出站/安全模型；改钩子点或模块先读它）
- 设计语言规范：`docs/design-language.md`（三端视觉唯一权威值表，改颜色/圆角/布局尺寸先改这里再同步三端）
