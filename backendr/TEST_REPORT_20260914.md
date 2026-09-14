# backendr（Rust/axum）接口完整性测试报告 — 2026-09-14

> 上轮报告：`TEST_REPORT_20260913.md`（发现 3 缺陷已修 + 遗留 access-keys/configs 两模块缺口，
> 提交 `cd0cb185` 补齐）。本轮为补齐后**全量回归 + 新模块深测**。
> 环境：compose 栈（`backendr/docker-compose.yaml`，bitnami pg/redis + backendr 镜像现场构建）；
> react 本地 dev（:5888，`/admin` 代理 → :7666，SSE → :7789）。账号 `admin / Abcd@1234`。

## 一、结论总览：全部通过，重构完整性缺口清零

| 层级 | 测试 | 结果 |
|---|---|---|
| API 新模块冒烟（`tests/access_key_config_smoke.sh`，33 断言） | **跑两遍**（验证重跑幂等） | **33/33 × 2 全绿** |
| API 全量 e2e（`tests/e2e.sh`，71 断言） | 全回归 | **71/71 全绿** |
| SSE 网关冒烟（`tests/sse_smoke.sh`，15 断言） | 鉴权负例 + 实时收帧 | **15/15 全绿** |
| 路由静态 diff（`tests/route_diff.py`，Go proto vs Rust） | — | **203/203 匹配，0 缺失** |
| GUI 登录全链路（浏览器实测） | 验证码 → AES 密码 → token → 仪表盘 | **通过** |
| GUI access-keys 页（上轮唯一失败页） | 创建 → SK 一次明文 modal → 列表 → Popconfirm 删除 | **通过** |
| GUI configs 页（新模块） | 内置参数渲染 + 值类型 | **通过** |
| 容器运行 | 全程 0 ERROR / 0 panic | 健康 |

上轮 12 个缺失端点（access-keys 5 + configs 6 + token 1）经 `cd0cb185` 补齐后，
**Go→Rust 重构的接口覆盖已达 100%**（203 个 proto 端点全部实现并有路由）。

## 二、本轮验证细节

### 2.1 新模块冒烟（两遍全绿，幂等确认）
- **access-keys CRUD**：创建（AK `ak-` 8B hex / SK `sk-` 32B hex 形状校验）、列表、详情、更新（仅名称/状态/过期时间可改）、删除。
- **令牌交换 `POST /access-keys/token`**（免鉴权）：正例换 HS256 机器令牌（`uid=0, sub=ak:…, roles=[machine]`，可调 API）；负例齐全——禁用凭证 400、密钥错误 400（不区分 AK 不存在/密钥错，防枚举）、空参 400。
- **SK 轮换**：ResetSecret 后旧 SK 立即失效、新 SK 可用。
- **configs CRUD**：key 唯一（含软删行，对齐 `uidx_sys_configs_key`）、重复 400、非法值类型 400（FLOAT 拒绝）、内置禁删 400、删除幂等。
- 第二遍重跑通过，证明脚本清理逻辑 + key 唯一约束处理正确。

### 2.2 GUI 实测（浏览器，react dev）
- **登录全链路**：退出旧会话 → 刷新验证码（答案从 Redis 读）→ 表单提交 → 前端 AES-128-CBC 加密密码 POST /login → 200 + HS256 token → 仪表盘渲染（用户总数/角色总数/审计统计）。期间抓包确认请求体为 `{"username":"admin","password":"<base64-AES>","grant_type":"password"}`，与 Go 侧 protojson 协议一致。
- **access-keys 页**（`/system/access-keys`）：新建凭证弹窗 → 提交后 **SK 明文一次性展示 modal**（提示"仅存 SHA-256 摘要"）→ 列表出现新行（名称/AK/启用/操作列）→ 删除走 Popconfirm（警告"机器令牌交换立即失效"）→ 删除成功 toast → 空态恢复。上轮"打开即获取数据失败"的问题不复存在。
- **configs 页**（`/system/configs`）：3 条等保口令策略内置参数（minLen=8/maxAgeDays=90/historyCount=3）正确渲染，值类型「整数」、内置标记、操作列齐备。

### 2.3 数据基线核对（澄清一个中途疑点）
GUI configs 页看到 4 行（3 内置 + 1 `ci.builtin`），清理后库里剩 3 条一度疑似丢数据。
核对 Go 侧 `pkg/constants/default_data.go::DefaultConfigs`——**播种本来就只有 3 条**；
多出的 `ci.flag`（软删隐藏）与 `ci.builtin` 是冒烟脚本测试行，清理后即恢复基线，非缺陷。
清理语句已执行（configs 删 2、access_keys 删 2，其中含 GUI 创建的 `gui-e2e-凭证`）。

## 三、环境问题（非 backendr 缺陷，复现要点）

1. **具名卷被清空 → 空库重播**：本轮 compose 卷是新的，需按标准流程用 Go 后端播种一次：
   - 复制 `backend/app/admin/service/configs/` 至临时目录，`data.yaml` 的 host 改 `127.0.0.1`（compose 暴露 5432/6379），`server.yaml` 删掉 `sse:` 段（避免占用已被 backendr 占的宿主 :7789）；
   - `GOROOT=/opt/homebrew/opt/go/libexec go build ./app/admin/service/cmd/server/` 后 `--conf=<临时目录>`（注意 cobra flag 必须用 `--conf=` 长格式，`-conf` 会被解析成 `-c onf`）；
   - 启动 ~20s 完成 49 张表迁移 + 播种，之后停掉（asynq 连不上 redis 的 EOF 报错无碍，播种在报错前已完成）。
2. **IAB 浏览器点击问题（延续上轮已知）**：antd 弹层（Dropdown/Popconfirm）与图标按钮的 Playwright 高阶 click 超时，改用 `evaluate` 派发完整 pointer/mouse 事件序列完成——测试工具问题，前后端无缺陷。中途一次登录失败是页面 captchaId 已被服务端消费过期（刷图后重新取答案即过），非后端问题。

## 四、遗留（与上轮一致，无新增）

- JWT HS256（Go 为 RS256），token 不跨后端通用——前端透明，不影响使用。
- SQL 方言 postgres 优先；查询 DSL 为常用子集（regex/year 未实现，未知操作符按 exact 兜底）。
- server-monitor 指标为 Rust 进程口径（goroutine/GC 不适用）。

## 五、复现命令

```bash
cd backendr && docker compose up -d --build
# 空库时先按「三.1」用 Go 后端播种一次，然后：
E2E_BASE=http://127.0.0.1:7666 E2E_REDIS_CONTAINER=backendr-redis-1 \
  REDIS_PASSWORD='*Abcd123456' bash backendr/tests/access_key_config_smoke.sh   # 33 断言，可重跑
E2E_BASE=http://127.0.0.1:7666 E2E_REDIS_CONTAINER=backendr-redis-1 \
  REDIS_PASSWORD='*Abcd123456' GOWIND_CRYPTO_KEY='dev-secret-crypto-key-4e2a' \
  bash backendr/tests/e2e.sh                                                    # 71 断言
SSE_BASE=http://127.0.0.1:7789 E2E_REDIS_CONTAINER=backendr-redis-1 \
  REDIS_PASSWORD='*Abcd123456' E2E_BASE=http://127.0.0.1:7666 \
  GOWIND_CRYPTO_KEY='dev-secret-crypto-key-4e2a' bash backendr/tests/sse_smoke.sh  # 15 断言
python3 backendr/tests/route_diff.py                                             # 203/203
cd frontend/admin/react && npm run dev                                            # GUI :5888
```
