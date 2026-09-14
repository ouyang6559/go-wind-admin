# backendr（Rust/axum）接口完整性测试报告

> 测试日期：2026-09-13（本会话）
> 部署方式：backendr 用 `docker compose`（`backendr/docker-compose.yaml`，镜像现场构建）；
> 前端 react 本地 dev 模式（`npm run dev`，:5888，`/admin` 代理 → 容器 :7666）。
> 数据库：compose 的 postgres/redis（bitnami），schema 由 Go 后端一次性迁移播种（backendr 自身无迁移）。
> 测试账号：`admin / Abcd@1234`（protojson 密码 AES-128-CBC 加密传输）。

## 一、测试结论总览

| 层级 | 结果 |
|---|---|
| API 全量 e2e（`tests/e2e.sh`，71 断言） | **71/71 全绿**（docker 化实例） |
| SSE 网关冒烟（`tests/sse_smoke.sh`，15 断言） | **15/15 全绿**（鉴权负例 + 实时收帧） |
| GUI 登录全链路（react → 代理 → backendr） | **通过**：验证码 → 登录 → /me → dashboard 渲染 |
| GUI SSE 实时推送（浏览器订阅 :7789 → API 发消息） | **通过**：顶栏未读数/最近消息立即刷新 |
| GUI 核心页面巡检（9 页） | **8/9 正常**；唯一失败 = access-keys（未实现，见缺口） |
| 路由静态 diff（Go proto vs Rust 路由） | 191/203 匹配；**缺 12 端点**（access-keys 5 + configs 6，远程 main 新增） |

## 二、本会话发现并修复的缺陷（已提交 `0041b2b1`）

以下 3 个 500/400 均由 **react 前端真实调用**触发（纯 API e2e 未覆盖到），已修复并回归全绿：

| # | 现象 | 触发方 | 根因 | 修复 |
|---|---|---|---|---|
| 1 | `GET /tenants?query={...}`、`GET /org-units?query={...}` 带**表别名前缀过滤列**时 500（`column "t.status" does not exist`） | 用户管理/组织管理页（下拉聚合 org/tenant 名） | `query.rs::compile_where` 对 `t.status` **整段加引号**，postgres 视作字面量单标识符 | 列引用按 `.` 分段：`t.status` → `"t"."status"` |
| 2 | `GET /internal-message/inbox` 组合过滤（`recipient_user_id` + `status__contains`）500（`count inbox failed`） | 顶栏通知组件（未读数查询） | inbox 的 count 语句**漏绑过滤参数**（只绑了主查询） | count 语句同样绑定 params |
| 3 | `GET /internal-message/inbox?orderBy=["-created_at"]` 400（`unknown query field: created_at`），顶栏反复弹「请求参数错误」 | 顶栏通知 + 收件箱页 | `INBOX_COLUMNS` 白名单缺 `recipient_user_id`/`created_at`/`updated_at` | 白名单补齐（行级仍强制 `r.recipient_user_id = operator`，无越权面） |

附带环境适配：`tests/e2e.sh`/`tests/sse_smoke.sh` 验证码读取支持 `REDIS_PASSWORD`（compose 的 bitnami redis 带密码）。

## 三、遗留缺口（无法由 backendr 当前代码满足，需后续开发）

### 3.1 远程 main 合入的 11 个新端点（react 已有对应页面/调用，backendr 未实现）

**access-keys（OpenAPI 凭证，5 端点）** —— react 页面 `/system/access-keys`（OpenAPI 凭证管理）已存在，实测打开即「获取数据失败」：

| 方法 | 路径 | 说明 |
|---|---|---|
| GET | `/admin/v1/access-keys` | 列表 |
| POST | `/admin/v1/access-keys` | 创建凭证 |
| GET | `/admin/v1/access-keys/{id}` | 详情 |
| PUT | `/admin/v1/access-keys/{id}` | 更新 |
| DELETE | `/admin/v1/access-keys/{id}` | 删除 |
| POST | `/admin/v1/access-keys/token` | 用 AccessKey/Secret 换机器令牌（**需单独对齐 Go 凭证签发/校验逻辑**，非模板 CRUD） |

**configs（系统参数，6 端点）** —— react 有 `/system/configs`（参数管理）页面与 `api/hooks/config.ts`：

| 方法 | 路径 |
|---|---|
| GET/POST | `/admin/v1/configs`、`/admin/v1/configs/{id}`（GET/PUT/DELETE） |

（两模块均为 Go 侧 `access_key`/`config` 服务，proto 在 `backend/api/protos/admin/service/v1/i_access_key.proto`、`i_config.proto`；表结构由 Go ent 迁移已建好，backendr 照 CRUD 模板 + api_manifest 登记即可，唯 `access-keys/token` 的机器令牌签发需按 Go `AuthenticationService` 对等实现。）

### 3.2 已知非阻塞差异（此前 MISSING.md 已记录，本轮未见新问题）

- JWT 算法 HS256（Go 为 RS256），token 不跨后端通用，前端透明
- SQL 方言 postgres 优先；查询 DSL 为常用子集（regex/year 等未实现，未知操作符按 exact 兜底）
- server-monitor 指标为 Rust 进程口径（goroutine/GC 概念不适用）

## 四、环境/测试过程问题（非 backendr 缺陷，知悉即可）

1. **`exceljs` 依赖缺失**：react `package.json` 声明了 `exceljs`（`src/utils/csv.ts` 引用）但 node_modules 未装全，dev server 报 `Failed to resolve import "exceljs"`。`pnpm add exceljs` 安装后恢复（package.json 版本范围未变）。
2. **宿主 `:7789` 被旧 Go dev 实例占用**：测试期间 compose SSE 映射临时改为 `7790:7789`，前端用 `.env.development.local`（gitignore）覆盖 `VITE_SSE_URL`。**测试后已恢复 `7789:7789`**；若本机 Go dev 实例常驻，二者只能占一个。
3. **Go 工具链版本错乱**（`GOROOT` 指向 1.25.12 而 brew go 为 1.27.1）：播种用的 Go server 需 `GOROOT=/opt/homebrew/opt/go/libexec` 显式修正才能编译。backendr 不受影响。
4. **IAB 浏览器 locator 兼容性**：Playwright 角色定位器点击登录按钮超时（页面用 XHR 提交），改用 `evaluate` 直接触发 click 完成 GUI 流程——测试工具问题，非前端缺陷。
5. GUI 巡检 9 页中「任务管理/通知渠道/脚本管理」显示暂无数据属全新播种库的正常空态，非错误。

## 五、复现命令

```bash
# 1. 起 compose 栈（pg/redis/backendr）
cd backendr && docker compose up -d --build

# 2. 首次需用 Go 后端建表播种（backendr 无迁移）：起一次 Go admin service 后停止
#    （conf 的 data 指向 compose 暴露的 5432/6379）

# 3. API 全量 + SSE 冒烟（bitnami redis 带密码）
E2E_BASE=http://127.0.0.1:7666 E2E_REDIS_CONTAINER=backendr-redis-1 \
  REDIS_PASSWORD='*Abcd123456' GOWIND_CRYPTO_KEY='dev-secret-crypto-key-4e2a' \
  bash backendr/tests/e2e.sh
SSE_BASE=http://127.0.0.1:7789 E2E_REDIS_CONTAINER=backendr-redis-1 REDIS_PASSWORD='*Abcd123456' \
  E2E_BASE=http://127.0.0.1:7666 bash backendr/tests/sse_smoke.sh

# 4. 前端 dev + 浏览器登录 admin / Abcd@1234
cd frontend/admin/react && npm run dev   # http://localhost:5888
```

> 容器日志 `docker logs backendr-backendr-1` 是排障第一现场：所有 500 均带数据库原始错误（column does not exist / ambiguous），本轮三处修复即据此定位。
