package service

import (
	"context"
	"fmt"
	"strings"
	"time"

	"github.com/tx7do/go-utils/timeutil"
	"github.com/tx7do/go-utils/trans"
	"github.com/tx7do/kratos-bootstrap/bootstrap"
	bLogger "github.com/tx7do/kratos-bootstrap/logger"
	appViewer "go-wind-admin/pkg/entgo/viewer"

	entCrud "github.com/tx7do/go-crud/entgo"

	aiV1 "go-wind-admin/api/gen/go/ai/service/v1"
	internalMessageV1 "go-wind-admin/api/gen/go/internal_message/service/v1"
	"go-wind-admin/app/admin/service/internal/data"
	"go-wind-admin/app/admin/service/internal/data/ent"
	"go-wind-admin/app/admin/service/internal/data/ent/user"
	"go-wind-admin/pkg/task"
)

// AiDigestService 审计日报 AI 摘要：定时聚合昨日操作审计 → 默认模型生成中文摘要
// → 站内信投递平台侧用户。
type AiDigestService struct {
	log *bLogger.Helper

	auditRepo        *data.OperationAuditLogRepo
	internalMessages *InternalMessageService
	internalMsgRepo  *data.InternalMessageRepo
	scriptRuntime    *ScriptRuntime
	entClient        *entCrud.EntClient[*ent.Client]
}

func NewAiDigestService(
	ctx *bootstrap.Context,
	auditRepo *data.OperationAuditLogRepo,
	internalMessages *InternalMessageService,
	internalMsgRepo *data.InternalMessageRepo,
	scriptRuntime *ScriptRuntime,
	entClient *entCrud.EntClient[*ent.Client],
) *AiDigestService {
	return &AiDigestService{
		log:              ctx.NewLoggerHelper("ai_digest/service/admin-service"),
		auditRepo:        auditRepo,
		internalMessages: internalMessages,
		internalMsgRepo:  internalMsgRepo,
		scriptRuntime:    scriptRuntime,
		entClient:        entClient,
	}
}

// AsyncAiAuditDigest 日报任务 handler：昨日审计统计 → LLM 摘要 → 站内信投递平台用户。
func (s *AiDigestService) AsyncAiAuditDigest(taskType string, data *task.AiAuditDigestTaskData) error {
	// asynq ctx 不带 viewer；统计是平台视角 + 站内信落库都需要 viewer，用系统查看器。
	ctx := appViewer.NewSystemViewerContext(context.Background())

	now := time.Now()
	yesterday := time.Date(now.Year(), now.Month(), now.Day(), 0, 0, 0, 0, time.Local).AddDate(0, 0, -1)
	today := yesterday.AddDate(0, 0, 1)
	dayLabel := yesterday.Format("2006-01-02")

	stats, err := s.auditRepo.DigestStats(ctx, yesterday, today)
	if err != nil {
		return err
	}
	if stats.Total == 0 {
		s.log.Infof(ctx, "audit digest: no operations on %s, skip", dayLabel)
		return nil
	}

	summary, err := s.summarize(ctx, dayLabel, stats)
	if err != nil {
		// 摘要失败降级为纯统计文本，日报照发（AI 不可用不应吞掉日报）
		s.log.Errorf(ctx, "audit digest: llm summarize failed, fallback to plain stats: %v", err)
		summary = buildPlainDigest(dayLabel, stats)
	}

	title := fmt.Sprintf("AI 审计日报（%s）", dayLabel)
	if err = s.deliverToPlatformUsers(ctx, title, summary); err != nil {
		return err
	}
	s.log.Infof(ctx, "audit digest delivered for %s (%d ops)", dayLabel, stats.Total)
	return nil
}

// summarize 拼装事实清单并调默认模型生成摘要。
func (s *AiDigestService) summarize(ctx context.Context, dayLabel string, stats *data.AuditDigestStats) (string, error) {
	var facts strings.Builder
	facts.WriteString(fmt.Sprintf("日期：%s\n", dayLabel))
	facts.WriteString(fmt.Sprintf("操作总数：%d\n", stats.Total))
	facts.WriteString(fmt.Sprintf("失败操作数：%d\n", stats.Failed))
	facts.WriteString(fmt.Sprintf("涉及用户数：%d\n", stats.Users))
	facts.WriteString("动作分布：\n")
	for _, a := range stats.TopActions {
		facts.WriteString(fmt.Sprintf("  - %s: %d 次\n", a.Action, a.Count))
	}

	systemPrompt := "你是企业后台的运维审计分析助手。根据给出的昨日操作审计统计数据，" +
		"用中文写一段 150 字以内的日报摘要：先一句话概括总体活跃度，再指出值得关注的点" +
		"（如失败率偏高、DELETE/EXPORT 类敏感操作占比、异常活跃用户等，没有就说明整体平稳）。" +
		"只输出摘要正文，不要寒暄和标题。"
	return s.scriptRuntime.ChatForScript(ctx, 0, systemPrompt, facts.String())
}

// buildPlainDigest LLM 不可用时的纯统计兜底文本。
func buildPlainDigest(dayLabel string, stats *data.AuditDigestStats) string {
	var sb strings.Builder
	sb.WriteString(fmt.Sprintf("昨日（%s）操作审计：总数 %d，失败 %d，涉及用户 %d。\n动作分布：", dayLabel, stats.Total, stats.Failed, stats.Users))
	for i, a := range stats.TopActions {
		if i > 0 {
			sb.WriteString("，")
		}
		sb.WriteString(fmt.Sprintf("%s %d 次", a.Action, a.Count))
	}
	return sb.String()
}

// deliverToPlatformUsers 建站内信行并向全部平台侧（tenant_id=0）用户投递收件行。
func (s *AiDigestService) deliverToPlatformUsers(ctx context.Context, title, content string) error {
	users, err := s.entClient.Client().User.Query().
		Where(user.TenantIDEQ(0), user.DeletedAtIsNil()).
		All(ctx)
	if err != nil {
		s.log.Errorf(ctx, "audit digest: query platform users failed: %s", err.Error())
		return aiV1.ErrorInternalServerError("query platform users failed")
	}
	if len(users) == 0 {
		s.log.Infof(ctx, "audit digest: no platform users to deliver")
		return nil
	}

	now := time.Now()
	msg, err := s.internalMsgRepo.Create(ctx, &internalMessageV1.CreateInternalMessageRequest{
		Data: &internalMessageV1.InternalMessage{
			Title:     &title,
			Content:   trans.Ptr(content),
			Status:    trans.Ptr(internalMessageV1.InternalMessage_PUBLISHED),
			Type:      trans.Ptr(internalMessageV1.InternalMessage_NOTIFICATION),
			CreatedBy: trans.Ptr(uint32(0)),
			CreatedAt: timeutil.TimeToTimestamppb(&now),
		},
	})
	if err != nil {
		return err
	}

	for _, u := range users {
		if err = s.internalMessages.sendNotification(ctx, msg.GetId(), u.ID, 0, &now, title, content); err != nil {
			// 单人投递失败不阻断其余收件人
			s.log.Errorf(ctx, "audit digest: deliver to user %d failed: %v", u.ID, err)
		}
	}
	return nil
}
