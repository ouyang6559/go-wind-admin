package service

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/sashabaranov/go-openai"
	"github.com/tx7do/go-utils/trans"
	"github.com/tx7do/kratos-bootstrap/bootstrap"
	bLogger "github.com/tx7do/kratos-bootstrap/logger"

	entCrud "github.com/tx7do/go-crud/entgo"

	paginationV1 "github.com/tx7do/go-crud/api/gen/go/pagination/v1"
	"google.golang.org/protobuf/types/known/emptypb"

	adminV1 "go-wind-admin/api/gen/go/admin/service/v1"
	aiV1 "go-wind-admin/api/gen/go/ai/service/v1"
	"go-wind-admin/app/admin/service/internal/data"
	"go-wind-admin/app/admin/service/internal/data/ent"
	"go-wind-admin/pkg/middleware/auth"
)

const aiContentCallTimeout = 60 * time.Second

// 场景 → 系统提示词模板（%s = langName(lang)）
var aiContentScenePrompts = map[aiV1.ContentScene]string{
	aiV1.ContentScene_DESCRIPTION:  "你是企业后台的内容撰写助手。根据用户给出的主题，为后台管理系统的实体（角色/菜单/字典项/文件等）撰写一段简明的描述文字。要求：专业、简洁、准确，不超过 100 字。用%s输出，只输出正文。",
	aiV1.ContentScene_ANNOUNCEMENT: "你是企业后台的公告撰写助手。根据用户给出的主题，撰写一则面向全体员工的通知公告。要求：包含标题行、正文（背景/内容/要求），语气正式但不生硬，总长度不超过 300 字。用%s输出，只输出正文。",
	aiV1.ContentScene_REPLY:        "你是企业后台的客服回复助手。根据用户给出的问题或场景，撰写一段专业的回复文案。要求：语气友好、有条理、针对性强，不超过 200 字。用%s输出，只输出正文。",
	aiV1.ContentScene_GENERAL:      "你是企业后台的智能写作助手。根据用户的指令生成内容。要求：直接输出结果，不要解释你的思考过程。用%s输出。",
}

type AiContentService struct {
	adminV1.AiContentServiceHTTPServer
	log *bLogger.Helper

	providerRepo *data.AiProviderRepo
	usageRepo    *data.AiUsageLogRepo
	menuRepo     *data.MenuRepo
	entClient    *entCrud.EntClient[*ent.Client]
}

func NewAiContentService(
	ctx *bootstrap.Context,
	providerRepo *data.AiProviderRepo,
	usageRepo *data.AiUsageLogRepo,
	menuRepo *data.MenuRepo,
	entClient *entCrud.EntClient[*ent.Client],
) *AiContentService {
	return &AiContentService{
		log:          ctx.NewLoggerHelper("ai_content/service/admin-service"),
		providerRepo: providerRepo,
		usageRepo:    usageRepo,
		menuRepo:     menuRepo,
		entClient:    entClient,
	}
}

// GenerateContent 按场景生成内容：登录用户均可使用（复用 ChatForScript 的全链路）。
// 前端在表单字段旁放 ✨ 按钮，点击后传入场景+主题，生成结果由前端自行填入字段。
func (s *AiContentService) GenerateContent(ctx context.Context, req *aiV1.GenerateContentRequest) (*aiV1.GenerateContentResponse, error) {
	operator, err := auth.FromContext(ctx)
	if err != nil {
		return nil, err
	}
	topic := strings.TrimSpace(req.GetTopic())
	if topic == "" {
		return nil, adminV1.ErrorBadRequest("topic is required")
	}

	lang := req.GetLang()
	if lang == "" {
		lang = "zh-CN"
	}
	langDisplay := langName(lang)

	scene := req.GetScene()
	systemPrompt, ok := aiContentScenePrompts[scene]
	if !ok {
		systemPrompt = aiContentScenePrompts[aiV1.ContentScene_GENERAL]
	}
	systemPrompt = fmt.Sprintf(systemPrompt, langDisplay)

	// 补充上下文作为追加指令
	if extra := strings.TrimSpace(req.GetContext()); extra != "" {
		systemPrompt += "\n补充要求：" + extra
	}
	if req.GetMaxLength() > 0 {
		systemPrompt += fmt.Sprintf("\n输出长度不超过 %d 字。", req.GetMaxLength())
	}

	provider, err := s.providerRepo.GetEnabledDefault(ctx)
	if err != nil {
		return nil, err
	}
	client, err := newOpenAIClientForProvider(ctx, provider)
	if err != nil {
		return nil, err
	}

	cCtx, cancel := context.WithTimeout(ctx, aiContentCallTimeout)
	defer cancel()
	resp, err := client.CreateChatCompletion(cCtx, openai.ChatCompletionRequest{
		Model: ptrStrOr(provider.ModelName, "gpt-4o-mini"),
		Messages: []openai.ChatCompletionMessage{
			{Role: openai.ChatMessageRoleSystem, Content: systemPrompt},
			{Role: openai.ChatMessageRoleUser, Content: topic},
		},
	})
	if err != nil {
		s.log.Errorf(ctx, "ai content generate failed: scene=%s: %v", scene.String(), err)
		return nil, adminV1.ErrorInternalServerError("ai content generate failed: %v", err)
	}

	content := strings.TrimSpace(resp.Choices[0].Message.Content)
	tokens := uint32(resp.Usage.TotalTokens)

	// 记用量（尽力而为，不阻断返回）
	if s.usageRepo != nil {
		modelName := ptrStrOr(provider.ModelName, "")
		promptTokens := uint32(resp.Usage.PromptTokens)
		completionTokens := uint32(resp.Usage.CompletionTokens)
		totalTokens := uint32(resp.Usage.TotalTokens)
		if uerr := s.usageRepo.Create(ctx, &aiV1.AiUsageLog{
			UserId:           trans.Ptr(operator.UserId),
			TenantId:         trans.Ptr(operator.GetTenantId()),
			ModelName:        &modelName,
			PromptTokens:     &promptTokens,
			CompletionTokens: &completionTokens,
			TotalTokens:      &totalTokens,
		}); uerr != nil {
			s.log.Errorf(ctx, "ai content usage log failed: %v", uerr)
		}
	}

	return &aiV1.GenerateContentResponse{
		Content:     content,
		TotalTokens: tokens,
	}, nil
}

// ── 语义搜索（pgvector 升级全局搜索） ────────────────────────────────

const buildSearchIndexSQL = "CREATE TABLE IF NOT EXISTS sys_ai_search_index (" +
	"id BIGSERIAL PRIMARY KEY, " +
	"created_at TIMESTAMPTZ DEFAULT NOW(), " +
	"item_type VARCHAR(20) NOT NULL DEFAULT 'menu', " +
	"item_id BIGINT NOT NULL, " +
	"title TEXT NOT NULL, " +
	"route TEXT NOT NULL DEFAULT '', " +
	"embedding vector)"

type searchResultRow struct {
	Title string  `sql:"title"`
	Route string  `sql:"route"`
	Score float64 `sql:"score"`
}

// BuildMenuSearchIndex 向量化全部菜单标题并写入搜索索引表（先清后写）。
func (s *AiContentService) BuildMenuSearchIndex(ctx context.Context) (uint32, error) {
	db := s.entClient.DB()
	if _, err := db.ExecContext(ctx, buildSearchIndexSQL); err != nil {
		return 0, fmt.Errorf("create search index table: %w", err)
	}
	menus, err := s.menuRepo.List(ctx, paginationV1PagingNoLimit(), true)
	if err != nil {
		return 0, err
	}
	if _, err = db.ExecContext(ctx, "DELETE FROM sys_ai_search_index WHERE item_type = 'menu'"); err != nil {
		return 0, fmt.Errorf("clear search index: %w", err)
	}

	provider, err := s.providerRepo.GetEnabledDefault(ctx)
	if err != nil {
		return 0, err
	}
	client, err := newOpenAIClientForProvider(ctx, provider)
	if err != nil {
		return 0, err
	}

	var texts []string
	var routes []string
	for _, m := range menus.GetItems() {
		if m.Path == nil {
			continue
		}
		title := ""
		if m.Meta != nil && m.Meta.Title != nil {
			title = *m.Meta.Title
		}
		if title == "" && m.Name != nil {
			title = *m.Name
		}
		if title == "" {
			continue
		}
		texts = append(texts, title)
		routes = append(routes, *m.Path)
	}
	if len(texts) == 0 {
		return 0, nil
	}

	embCtx, embCancel := context.WithTimeout(ctx, 60*time.Second)
	defer embCancel()
	var vectors []string
	for start := 0; start < len(texts); start += 32 {
		end := start + 32
		if end > len(texts) {
			end = len(texts)
		}
		resp, embErr := client.CreateEmbeddings(embCtx, openai.EmbeddingRequest{
			Model: openai.EmbeddingModel("text-embedding-3-small"),
			Input: texts[start:end],
		})
		if embErr != nil {
			return 0, fmt.Errorf("embed batch: %w", embErr)
		}
		for _, d := range resp.Data {
			vectors = append(vectors, vectorToLiteral(d.Embedding))
		}
	}

	inserted := uint32(0)
	for i := range texts {
		if _, execErr := db.ExecContext(ctx,
			"INSERT INTO sys_ai_search_index (item_type, item_id, title, route, embedding) VALUES ('menu', $1, $2, $3, $4::vector)",
			i, texts[i], routes[i], vectors[i],
		); execErr != nil {
			s.log.Errorf(ctx, "insert search index failed: %v", execErr)
			continue
		}
		inserted++
	}
	return inserted, nil
}

// SemanticSearch 语义搜索：向量化查询 → pgvector 余弦检索菜单索引。
func (s *AiContentService) SemanticSearch(ctx context.Context, req *aiV1.SemanticSearchRequest) (*aiV1.SemanticSearchResponse, error) {
	query := strings.TrimSpace(req.GetQuery())
	if query == "" {
		return &aiV1.SemanticSearchResponse{}, nil
	}
	limit := int(req.GetLimit())
	if limit <= 0 {
		limit = 8
	}

	provider, err := s.providerRepo.GetEnabledDefault(ctx)
	if err != nil {
		return nil, err
	}
	client, err := newOpenAIClientForProvider(ctx, provider)
	if err != nil {
		return nil, err
	}

	embCtx, embCancel := context.WithTimeout(ctx, 30*time.Second)
	defer embCancel()
	embResp, embErr := client.CreateEmbeddings(embCtx, openai.EmbeddingRequest{
		Model: openai.EmbeddingModel("text-embedding-3-small"),
		Input: []string{query},
	})
	if embErr != nil {
		return nil, adminV1.ErrorInternalServerError("embed query failed: %v", embErr)
	}
	queryVec := vectorToLiteral(embResp.Data[0].Embedding)

	db := s.entClient.DB()
	rows, dbErr := db.QueryContext(ctx,
		"SELECT title, route, 1 - (embedding <=> $1::vector) AS score FROM sys_ai_search_index WHERE item_type = 'menu' ORDER BY embedding <=> $1::vector LIMIT $2",
		queryVec, limit,
	)
	if dbErr != nil {
		s.log.Errorf(ctx, "semantic search query failed: %s", dbErr.Error())
		return nil, adminV1.ErrorInternalServerError("semantic search failed")
	}
	defer func() { _ = rows.Close() }()

	resp := &aiV1.SemanticSearchResponse{Items: make([]*aiV1.SemanticSearchItem, 0, limit)}
	for rows.Next() {
		var title, route string
		var score float64
		if err = rows.Scan(&title, &route, &score); err != nil {
			continue
		}
		resp.Items = append(resp.Items, &aiV1.SemanticSearchItem{
			Title: title, Route: route, ItemType: "menu", Score: float32(score),
		})
	}
	return resp, rows.Err()
}

// RebuildSearchIndex 重建搜索索引 RPC。
func (s *AiContentService) RebuildSearchIndex(ctx context.Context, _ *emptypb.Empty) (*aiV1.RebuildSearchIndexResponse, error) {
	count, err := s.BuildMenuSearchIndex(ctx)
	if err != nil {
		return nil, adminV1.ErrorInternalServerError("rebuild search index failed: %v", err)
	}
	return &aiV1.RebuildSearchIndexResponse{IndexedCount: count}, nil
}

// paginationV1PagingNoLimit 构造不分页的 PagingRequest。
func paginationV1PagingNoLimit() *paginationV1.PagingRequest {
	return &paginationV1.PagingRequest{NoPaging: trans.Ptr(true)}
}
