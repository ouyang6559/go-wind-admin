# AI 模块（对话 / 提供商 / 用量配额 / 知识库 RAG）

> **定位**：AI 模块的参考层文档：子域划分、流式对话语义、密钥与配额机制、知识库 RAG 链路与部署要求。
> 改 AI 相关代码（`api/protos/ai/`、`pkg/ai/`、`internal/service/ai_*.go`、三端 `ai` 页面）前先读本文。

## 子域与数据模型

| 子域 | 表 | 说明 |
|---|---|---|
| 模型提供商 | `sys_ai_providers` | 一行 = 一个可调用端点（云端 OpenAI 兼容 / 本地 Ollama）；api_key AES-GCM 加密落库，读视图只有 `apiKeyHint` 脱敏 |
| 对话会话 | `sys_ai_conversations` | 归属用户（user_id），删除级联消息 |
| 对话消息 | `sys_ai_messages` | USER/ASSISTANT/SYSTEM 三角色；ASSISTANT 行带 tokens/耗时快照 |
| 用量流水 | `sys_ai_usage_logs` | 每次成功调用一行（只增不改），配额聚合的事实源 |
| 知识库 | `sys_ai_knowledge_bases` / `sys_ai_docs` / `sys_ai_chunks` | RAG：文档切片 → 向量化 → 余弦检索 |

## 流式对话语义

- `POST /admin/v1/ai/chat/completions` 是**普通 POST**（body 为扁平请求体，**不包 `data`**——与站内信 send 同型，多包一层会被 protojson 丢弃），同步返回完整回复（含会话与 tokens）。
- 生成过程中的增量 token 通过 SSE 网关（:7789）以 `ai_chat_chunk` 事件推送给发起用户（streamID=userId），事件 data 为 `ChatChunkEvent` 的 protojson（`conversationId`/`seq`/`delta`，camelCase）。**seq=0 时 protojson 省略零值字段**，前端不能依赖 seq 存在。
- chunk 是尽力而为（`TryPublish` 缓冲满即丢帧），以 POST 同步响应为最终事实。
- 流式时长由 context deadline（5 分钟）控制，不是 `http.Client.Timeout`——`pkg/ai/client.go` 显式注入无超时 client 覆盖上游默认 30s，否则长回复被腰斩。

## 密钥与配置

- api_key 经全局加密器（`pkg/crypto`，密钥来自环境变量 `GOWIND_CRYPTO_KEY`）加密落库；未设置该变量时明文落库（与全局加密既有语义一致）。
- Update 语义：apiKey 留空 = 不修改已存 Key（该字段从 updateMask 摘除，防 FieldMask 空值清写）。

## 配额与租户门禁

- `QuotaType.AI_TOKENS=4`（月度）：`chat` 前检查租户套餐配额，超限返回 400 "ai token quota exceeded for this month"；未配置该维度 = 不限量；平台用户（tenant_id=0）跳过检查。
- **套餐模块白名单**：AI 六服务已登记进 `pkg/constants/module_mapping.go`（Module 枚举 `AI=11`），租户访问 AI 端点要求租户套餐的白名单里有 `AI` 模块行（`sys_plan_modules`），否则 403 "module not allowed"。漏登记的后果是 fail-closed 拒绝，不是放行。
- 菜单归类：`ComponentToModule` 已加 `app/ai/` 前缀 → AI 模块。

## 知识库 RAG

链路：上传纯文本或文件 → 切片（500 字符/片，50 重叠）→ 调 provider 端点的 OpenAI 兼容 `/v1/embeddings` 批量向量化 → `sys_ai_chunks` 落库 → 检索（pgvector `<=>` 余弦距离 topK，SQL 层 tenant 过滤兜底）→ chat 携带 `knowledgeBaseId` 时把命中片段注入 system 消息（检索失败降级为普通对话，不阻断）。

**文件上传**：`POST /admin/v1/ai/knowledge-bases/{baseId}/docs/file`，body 为 JSON（`fileName` + `contentBase64`，protojson 的 bytes 即 base64——kratos 无 form-data codec 的既定形态）。抽取器在 `pkg/doctext`：纯文本族直读、docx 走标准库 zip+XML 剥标签、pdf 走 `ledongthuc/pdf` 文本层（扫描件无文本层会明确报错），其余扩展名拒绝；上限 10MB。

**重索引**：更换知识库 embedding 模型后，经「任务管理」创建 `ai_doc_reindex` 类型任务（payload `{"baseId":N}`，0=全部库）触发全量重算——切片文本不变只换向量，分批 32 条/次。

**审计日报**：系统常驻任务 `ai_audit_digest`（每日 08:00）聚合昨日操作审计（总数/失败/用户/动作分布），经默认模型生成 150 字中文摘要，站内信投递平台侧用户；LLM 失败自动降级为纯统计文本。

**部署要求（pgvector）**：

1. Postgres 必须带 pgvector 扩展（0.8.x 验证可用）。官方 `postgres` 镜像不含，需用 `pgvector/pgvector` 镜像或发行版包（Debian：`apt-get install postgresql-16-pgvector`，需 PGDG 源）。
2. 服务启动时自动执行（`AiKnowledgeRepo.MigrateVectorColumn`，幂等）：
   ```sql
   CREATE EXTENSION IF NOT EXISTS vector;
   ALTER TABLE sys_ai_chunks ADD COLUMN IF NOT EXISTS embedding vector;
   ```
   `CREATE EXTENSION` 需要超级用户权限；失败仅 RAG 降级（检索/入库报错），其余功能不受影响。
3. **embedding 向量列不进 ent schema**（ent 对 pgvector 自定义类型支持受限），切片读写走原生 SQL（`entClient.DB()`）。手改表结构时注意保持列名 `embedding`。
4. Docker 部署示例：把 `backend/scripts/deploy/` 下的 compose 里 postgres 镜像换成 `pgvector/pgvector:pg16` 即可，其余不变。

## 三端页面

| 端 | 路由 | 内容 |
|---|---|---|
| react | `/ai/chat` `/ai/providers` `/ai/knowledge` | 基准实现（流式 markdown 渲染、知识库选择器、文档上传） |
| vue-element | 同构 | ProPage + useDrawerForm |
| vue-vben | 同构（hash 路由） | VxeGrid + useVbenDrawer |

聊天页的知识库选择器：选中后每次发送携带 `knowledgeBaseId`，该会话的后续轮次都带检索注入。

## 已知边界

- e2e 用 `mock_llm.py`（OpenAI 兼容 echo + bigram 词袋 embeddings）验证；真实云端模型（DeepSeek/通义）与本地 Ollama 走同一 OpenAI 兼容协议，未逐家实测。
- 会话标题默认取首条消息前 30 字符；重命名接口已具备（Update conversation），react 端未出入口。
- 图像/多模态、Function Call、agent 编排未做（eino/langchaingo 插件能力在上游已备，按需接入）。
