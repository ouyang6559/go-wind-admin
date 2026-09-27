package service

import (
	"context"
	"database/sql"
	"fmt"
	"regexp"
	"strings"
	"time"

	"github.com/sashabaranov/go-openai"
	"github.com/tx7do/go-utils/trans"
	"github.com/tx7do/kratos-bootstrap/bootstrap"
	bLogger "github.com/tx7do/kratos-bootstrap/logger"

	entCrud "github.com/tx7do/go-crud/entgo"

	adminV1 "go-wind-admin/api/gen/go/admin/service/v1"
	aiV1 "go-wind-admin/api/gen/go/ai/service/v1"
	"go-wind-admin/app/admin/service/internal/data"
	"go-wind-admin/app/admin/service/internal/data/ent"
	"go-wind-admin/pkg/middleware/auth"
)

// ── 智能问数安全边界 ────────────────────────────────────────────────

const (
	// aiQueryRowLimit 结果行硬上限（缺 LIMIT 自动补；超限的显式 LIMIT 也被改写）。
	aiQueryRowLimit = 100

	// aiQueryCallTimeout SQL 生成与结论生成的单次 LLM 调用兜底时长。
	aiQueryCallTimeout = 90 * time.Second
)

// aiQueryTableWhitelist 表白名单：NL→SQL 只允许触达这些表。
// 键为表名，值为给 LLM 的列说明（表名+列清单+备注一起拼进提示词当数据字典）。
var aiQueryTableWhitelist = map[string]string{
	"sys_users":                   "用户表：id, username(登录名), nickname(昵称), email, mobile, status(ON=启用/OFF=停用), created_at(注册时间)",
	"sys_roles":                   "角色表：id, name(角色名), code(角色编码), status, created_at",
	"sys_user_roles":              "用户-角色关联表：user_id, role_id",
	"sys_tenants":                 "租户表：id, code(租户编码), name(租户名), plan_id(套餐ID), status, created_at",
	"sys_plans":                   "套餐表：id, name(套餐名)",
	"sys_operation_audit_logs":    "操作审计日志：id, user_id, username, action(CREATE/UPDATE/DELETE/READ/ASSIGN/UNASSIGN/EXPORT/IMPORT/OTHER), resource_type(资源类型), resource_id, success(bool), failure_reason, created_at(操作时间)",
	"sys_login_audit_logs":        "登录审计日志：id, user_id, username, status(SUCCESS/FAILED/LOCKED), ip_address, created_at",
	"sys_ai_usage_logs":           "AI 用量流水：id, user_id, tenant_id, model_name, prompt_tokens, completion_tokens, total_tokens, duration_ms, created_at",
	"internal_messages":           "站内信消息：id, title, status(PUBLISHED/…), created_at",
	"internal_message_recipients": "站内信收件记录：id, message_id, recipient_user_id(收件人), status(RECEIVED=未读/READ=已读), created_at",
}

// aiQueryForbiddenPatterns 生成 SQL 的硬拒绝模式：写操作与危险语句。
var aiQueryForbiddenPatterns = []*regexp.Regexp{
	regexp.MustCompile(`(?i)\b(INSERT|UPDATE|DELETE|DROP|TRUNCATE|ALTER|CREATE|GRANT|REVOKE|COPY|VACUUM|REINDEX|CALL|DO|SET)\b`),
	regexp.MustCompile(`(?i)\b(pg_read_file|pg_ls_dir|pg_sleep|lo_import|lo_export|dblink)\b`),
	regexp.MustCompile(`\;`),     // 多语句
	regexp.MustCompile(`--|/\*`), // 注释（防注释截断绕过）
}

// aiQuerySelectPattern 必须以 SELECT 或 WITH 开头（含前导空白/括号容忍）。
var aiQuerySelectPattern = regexp.MustCompile(`(?is)^\s*\(?\s*(SELECT|WITH)\b`)

// ── 服务 ───────────────────────────────────────────────────────────

// AiQueryService 智能问数：NL → 只读 SQL → 结构化结果 →（可选）自然语言结论。
// 平台管理员专属——第一版查询的是全平台数据，且 SQL 直触原生库表。
type AiQueryService struct {
	adminV1.AiQueryServiceHTTPServer
	log *bLogger.Helper

	providerRepo *data.AiProviderRepo
	usageRepo    *data.AiUsageLogRepo
	entClient    *entCrud.EntClient[*ent.Client]
}

func NewAiQueryService(
	ctx *bootstrap.Context,
	providerRepo *data.AiProviderRepo,
	usageRepo *data.AiUsageLogRepo,
	entClient *entCrud.EntClient[*ent.Client],
) *AiQueryService {
	return &AiQueryService{
		log:          ctx.NewLoggerHelper("ai_query/service/admin-service"),
		providerRepo: providerRepo,
		usageRepo:    usageRepo,
		entClient:    entClient,
	}
}

// Ask 智能问数主链路：NL→SQL（第一次调用）→ 护栏校验 → 只读执行 → NL 结论（第二次调用）。
// 两次调用都复用脚本 ai 模块的 provider 解析与用量记账。
func (s *AiQueryService) Ask(ctx context.Context, req *aiV1.AskAiQueryRequest) (*aiV1.AskAiQueryResponse, error) {
	operator, err := auth.FromContext(ctx)
	if err != nil {
		return nil, err
	}
	if operator.GetTenantId() != 0 {
		// 第一版平台专属：SQL 直触全平台原生表，租户数据范围裁剪留待后续版本。
		return nil, adminV1.ErrorForbidden("ai query is platform-only")
	}
	question := strings.TrimSpace(req.GetQuestion())
	if question == "" {
		return nil, adminV1.ErrorBadRequest("question is required")
	}

	provider, err := s.providerRepo.GetEnabledDefault(ctx)
	if err != nil {
		return nil, err
	}
	client, err := newOpenAIClientForProvider(ctx, provider)
	if err != nil {
		return nil, err
	}

	lang := req.GetLang()
	if lang == "" {
		lang = "zh-CN"
	}

	// 1. NL → SQL
	sqlGenerated, genTokens, err := s.generateSQL(ctx, client, provider, question, lang)
	if err != nil {
		return nil, err
	}

	// 2. 四重护栏：只读语句 / 白名单表 / LIMIT 钳制 / 二次正则复核
	cleaned, err := sanitizeSQL(sqlGenerated)
	if err != nil {
		return nil, adminV1.ErrorBadRequest("generated sql rejected: %v", err)
	}

	// 3. 只读事务执行
	columns, rows, execErr := s.executeReadOnly(ctx, cleaned)

	resp := &aiV1.AskAiQueryResponse{
		Sql:     cleaned,
		Columns: columns,
		Rows:    rows,
	}
	if execErr != nil {
		// 执行失败把数据库错误回给前端（便于用户改问题重试），SQL 照样返回
		s.log.Errorf(ctx, "ai query execute failed: %v", execErr)
		return resp, nil
	}
	resp.RowCount = uint32(len(rows))

	// 4.（可选）结果 → 自然语言结论
	if !req.GetWithAnswer() || len(rows) == 0 {
		return resp, nil
	}
	answer, answerTokens, err := s.generateAnswer(ctx, client, provider, question, cleaned, columns, rows, lang)
	if err != nil {
		s.log.Errorf(ctx, "ai query answer failed, returning rows only: %v", err)
	} else {
		resp.Answer = trans.Ptr(answer)
	}

	resp.TotalTokens = genTokens + answerTokens
	return resp, nil
}

// generateSQL 数据字典 + 问题 → 只读 SQL。
func (s *AiQueryService) generateSQL(ctx context.Context, client *openai.Client, provider *ent.AiProvider, question, lang string) (string, uint32, error) {
	var sb strings.Builder
	sb.WriteString("你是只读数据分析助手。根据下面的数据库表结构与用户问题，生成一条 Postgres 只读 SELECT 语句。\n")
	sb.WriteString("规则：\n")
	sb.WriteString("1. 只输出 SQL 本身，不要任何解释、不要 Markdown 代码块标记；\n")
	sb.WriteString("2. 只能使用下面列出的表；涉及多表时用 JOIN；\n")
	sb.WriteString("3. 必须带 LIMIT（最大 100）；时间比较用 created_at 与 NOW()；\n")
	sb.WriteString(fmt.Sprintf("4. 结论文字用%s。\n\n", langName(lang)))
	sb.WriteString("可用表：\n")
	for table, desc := range aiQueryTableWhitelist {
		sb.WriteString("- " + table + "：" + desc + "\n")
	}

	qCtx, cancel := context.WithTimeout(ctx, aiQueryCallTimeout)
	defer cancel()
	resp, err := client.CreateChatCompletion(qCtx, openai.ChatCompletionRequest{
		Model: ptrStrOr(provider.ModelName, "gpt-4o-mini"),
		Messages: []openai.ChatCompletionMessage{
			{Role: openai.ChatMessageRoleSystem, Content: sb.String()},
			{Role: openai.ChatMessageRoleUser, Content: question},
		},
	})
	if err != nil {
		return "", 0, adminV1.ErrorInternalServerError("ai query generate failed: %v", err)
	}
	tokens := uint32(resp.Usage.TotalTokens)
	if len(resp.Choices) == 0 {
		return "", tokens, adminV1.ErrorInternalServerError("ai query generate empty")
	}
	return stripSQLFence(resp.Choices[0].Message.Content), tokens, nil
}

// sanitizeSQL 四重护栏：只读语句校验 → 危险模式拒绝 → 白名单表校验 → LIMIT 钳制。
func sanitizeSQL(raw string) (string, error) {
	sqlText := stripSQLFence(raw)
	sqlText = strings.TrimRight(strings.TrimSpace(sqlText), ";")

	if !aiQuerySelectPattern.MatchString(sqlText) {
		return "", fmt.Errorf("only SELECT/WITH statements are allowed")
	}
	for _, pat := range aiQueryForbiddenPatterns {
		if pat.MatchString(sqlText) {
			return "", fmt.Errorf("forbidden pattern detected")
		}
	}
	// 白名单：提取 FROM/JOIN 后的表名逐一校验
	tablePattern := regexp.MustCompile(`(?i)\b(FROM|JOIN)\s+([a-zA-Z_][a-zA-Z0-9_]*)`)
	matches := tablePattern.FindAllStringSubmatch(sqlText, -1)
	for _, m := range matches {
		table := strings.ToLower(m[2])
		if _, ok := aiQueryTableWhitelist[table]; !ok {
			return "", fmt.Errorf("table %q is not in the whitelist", m[2])
		}
	}
	// LIMIT 钳制：无 LIMIT 补 aiQueryRowLimit；LIMIT N>N 上限改写
	lower := strings.ToLower(sqlText)
	if !regexp.MustCompile(`(?i)\blimit\s+\d+`).MatchString(lower) {
		sqlText = fmt.Sprintf("SELECT * FROM (%s) AS _q LIMIT %d", sqlText, aiQueryRowLimit)
	} else {
		reLimit := regexp.MustCompile(`(?i)\blimit\s+(\d+)`)
		sqlText = reLimit.ReplaceAllStringFunc(sqlText, func(m string) string {
			n := 0
			fmt.Sscanf(strings.TrimSpace(m[5:]), "%d", &n)
			if n > aiQueryRowLimit {
				return fmt.Sprintf("LIMIT %d", aiQueryRowLimit)
			}
			return m
		})
	}
	return sqlText, nil
}

// executeReadOnly 只读事务执行（数据库层兜底：READ ONLY 事务里写操作必败）。
func (s *AiQueryService) executeReadOnly(ctx context.Context, sqlText string) ([]string, []*aiV1.AiQueryRow, error) {
	db := s.entClient.DB()
	txCtx, cancel := context.WithTimeout(ctx, 30*time.Second)
	defer cancel()

	tx, err := db.BeginTx(txCtx, &sql.TxOptions{ReadOnly: true})
	if err != nil {
		return nil, nil, fmt.Errorf("begin read-only tx: %w", err)
	}
	defer func() { _ = tx.Rollback() }()

	resultRows, err := tx.QueryContext(txCtx, sqlText)
	if err != nil {
		return nil, nil, err
	}
	defer func() { _ = resultRows.Close() }()

	columns, err := resultRows.Columns()
	if err != nil {
		return nil, nil, err
	}

	out := make([]*aiV1.AiQueryRow, 0, 16)
	for resultRows.Next() {
		values := make([]interface{}, len(columns))
		ptrs := make([]interface{}, len(columns))
		for i := range values {
			ptrs[i] = &values[i]
		}
		if err = resultRows.Scan(ptrs...); err != nil {
			return nil, nil, err
		}
		row := &aiV1.AiQueryRow{Values: make([]string, 0, len(columns))}
		for _, v := range values {
			row.Values = append(row.Values, stringifyCellValue(v))
		}
		out = append(out, row)
	}
	if err = resultRows.Err(); err != nil {
		return nil, nil, err
	}
	return columns, out, nil
}

// generateAnswer 结果集 → 自然语言结论（第二次调用）。
func (s *AiQueryService) generateAnswer(ctx context.Context, client *openai.Client, provider *ent.AiProvider, question, sqlText string, columns []string, rows []*aiV1.AiQueryRow, lang string) (string, uint32, error) {
	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("用户问题：%s\n\n执行的 SQL：%s\n\n查询结果（列：%s）：\n", question, sqlText, strings.Join(columns, ", ")))
	maxRows := len(rows)
	if maxRows > 20 {
		maxRows = 20 // 结论只看前 20 行，避免超长
	}
	for _, r := range rows[:maxRows] {
		sb.WriteString(strings.Join(r.Values, " | "))
		sb.WriteString("\n")
	}

	langLine := "用中文回答。"
	if strings.HasPrefix(lang, "en") {
		langLine = "Answer in English."
	}

	aCtx, cancel := context.WithTimeout(ctx, aiQueryCallTimeout)
	defer cancel()
	resp, err := client.CreateChatCompletion(aCtx, openai.ChatCompletionRequest{
		Model: ptrStrOr(provider.ModelName, "gpt-4o-mini"),
		Messages: []openai.ChatCompletionMessage{
			{
				Role: openai.ChatMessageRoleSystem,
				Content: "你是数据分析助手。根据用户问题、SQL 与查询结果，" +
					"给出简明的结论：直接回答问题本身，引用具体数字；" +
					"结果为空就明确说明没有匹配数据。" + langLine,
			},
			{Role: openai.ChatMessageRoleUser, Content: sb.String()},
		},
	})
	if err != nil {
		return "", 0, err
	}
	tokens := uint32(resp.Usage.TotalTokens)
	if len(resp.Choices) == 0 {
		return "", tokens, fmt.Errorf("empty answer")
	}
	return strings.TrimSpace(resp.Choices[0].Message.Content), tokens, nil
}

// stripSQLFence 剥掉模型可能输出的 Markdown 代码块围栏。
func stripSQLFence(s string) string {
	s = strings.TrimSpace(s)
	for _, fence := range []string{"```sql", "```SQL", "```"} {
		if strings.HasPrefix(s, fence) {
			s = strings.TrimPrefix(s, fence)
			if idx := strings.LastIndex(s, "```"); idx >= 0 {
				s = s[:idx]
			}
			break
		}
	}
	return strings.TrimSpace(s)
}

// stringifyCellValue 数据库原始值 → 展示字符串（time/[]byte/nil 特判）。
func stringifyCellValue(v interface{}) string {
	switch val := v.(type) {
	case nil:
		return ""
	case time.Time:
		return val.Format("2006-01-02 15:04:05")
	case []byte:
		return string(val)
	default:
		return fmt.Sprintf("%v", val)
	}
}

// langName BCP-47 → 提示词里的语言名。
func langName(lang string) string {
	if strings.HasPrefix(lang, "en") {
		return "English"
	}
	return "中文"
}
