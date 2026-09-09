# backendr — GoWind Admin（Rust / axum 重写）

基于 Kratos 版后端 `openapi.yaml`（`backend/app/admin/service/cmd/server/assets/openapi.yaml`）
由代码生成脚本自动产出的 **完整项目骨架**。166 个接口、34 个业务模块的分层已铺好，
业务逻辑以 `NotImplemented` 占位，后续按模块逐个填充。

## 技术栈

- **Web**：axum 0.8 + tokio + tower / tower-http
- **数据**：sqlx（AnyPool，兼容 postgres / mysql / sqlite）+ bb8 / bb8-redis
- **序列化 / 配置**：serde / serde_json / serde_yaml + dotenvy
- **可观测**：tracing / tracing-subscriber
- **认证安全**：jsonwebtoken + argon2 + rand + base64 + captcha_rust
- **校验 / 邮件 / HTTP**：validator + lettre + reqwest
- **错误**：thiserror + anyhow

## 分层架构

```
src/
├── main.rs          入口：加载配置 -> tracing -> 状态 -> 路由 -> 监听
├── lib.rs           crate 根（模块声明 + 分层约定）
├── config.rs        环境变量配置（与 backendz etc/admin.yaml 字段对齐）
├── state.rs         AppState：配置 + db/redis 连接池 + JWT 密钥
├── error.rs         AppError 统一错误（IntoResponse -> 统一 JSON）
├── response.rs      ApiResponse 统一响应体 { code, message, data }
├── middleware/      中间件与扩展提取器（CORS/跟踪/Operator 占位）
├── routes/          按模块聚合 Router，挂载到 /admin/v1
├── handlers/        166 处 handler 骨架（只做 HTTP 解析与包装）
├── services/        业务逻辑层占位
├── repos/           数据访问层占位（sqlx AnyPool）
└── dto/             由 openapi schemas 生成的实体模型（serde camelCase）
```

## 启动

```bash
cp .env.example .env     # 按需改连接
cargo run                # 默认监听 0.0.0.0:7666
```

DB / Redis 不可用时服务照常启动（仅告警），路由返回 `NotImplemented (501)`，
便于无外部依赖先行跑通骨架。

## 代码生成

模块文件均由 `.tmp/parse_openapi/gen_rust.py` 从 `ops.json` / `schemas.json` 生成。
改接口后重跑生成脚本即可同步骨架。

## 与 backendz 的关系

三端并行的 Rust 版；后端权威实现以 `react 端 -> kratos(backend)` 为基准，
`backendz`（go-zero .api 定义）与 `backendr`（axum）均为其翻译/重写子项目。