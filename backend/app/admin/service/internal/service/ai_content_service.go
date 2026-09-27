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
	entClient    *entCrud.EntClient[*ent.Client]
}

func NewAiContentService(
	ctx *bootstrap.Context,
	providerRepo *data.AiProviderRepo,
	usageRepo *data.AiUsageLogRepo,
	entClient *entCrud.EntClient[*ent.Client],
) *AiContentService {
	return &AiContentService{
		log:          ctx.NewLoggerHelper("ai_content/service/admin-service"),
		providerRepo: providerRepo,
		usageRepo:    usageRepo,
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
