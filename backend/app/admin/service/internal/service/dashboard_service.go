package service

import (
	"fmt"
	"go-wind-admin/pkg/middleware/auth"
	"strings"

	"context"

	"github.com/tx7do/kratos-bootstrap/bootstrap"
	bLogger "github.com/tx7do/kratos-bootstrap/logger"
	"google.golang.org/protobuf/types/known/emptypb"

	"go-wind-admin/app/admin/service/internal/data"

	adminV1 "go-wind-admin/api/gen/go/admin/service/v1"
)

// DashboardService 为后台首页分析页提供只读聚合统计。
// 多租户隔离由 ent Policy + auth 中间件注入的 viewer 自动完成。
type DashboardService struct {
	adminV1.DashboardServiceHTTPServer

	log *bLogger.Helper

	dashboardRepo *data.DashboardRepo
	scriptRuntime *ScriptRuntime
}

func NewDashboardService(
	ctx *bootstrap.Context,
	dashboardRepo *data.DashboardRepo,
	scriptRuntime *ScriptRuntime,
) *DashboardService {
	return &DashboardService{
		log:           ctx.NewLoggerHelper("dashboard/service/admin-service"),
		scriptRuntime: scriptRuntime,
		dashboardRepo: dashboardRepo,
	}
}

// GetOverview 返回四张概览卡的计数。
func (s *DashboardService) GetOverview(ctx context.Context, _ *emptypb.Empty) (*adminV1.DashboardOverviewResponse, error) {
	userCount, err := s.dashboardRepo.CountActiveUsers(ctx)
	if err != nil {
		return nil, err
	}
	roleCount, err := s.dashboardRepo.CountRoles(ctx)
	if err != nil {
		return nil, err
	}
	todayLoginCount, err := s.dashboardRepo.CountTodayLogins(ctx)
	if err != nil {
		return nil, err
	}
	todayOperationCount, err := s.dashboardRepo.CountTodayOperations(ctx)
	if err != nil {
		return nil, err
	}

	return &adminV1.DashboardOverviewResponse{
		UserCount:           uint32(userCount),
		RoleCount:           uint32(roleCount),
		TodayLoginCount:     uint32(todayLoginCount),
		TodayOperationCount: uint32(todayOperationCount),
	}, nil
}

// GetLoginTrend 返回近 days 天每日登录次数趋势，按日期升序、缺日补零。
func (s *DashboardService) GetLoginTrend(ctx context.Context, req *adminV1.GetLoginTrendRequest) (*adminV1.LoginTrendResponse, error) {
	days := int(req.GetDays())
	rows, err := s.dashboardRepo.LoginTrend(ctx, days)
	if err != nil {
		return nil, err
	}

	points := make([]*adminV1.TrendPoint, 0, len(rows))
	for _, r := range rows {
		points = append(points, &adminV1.TrendPoint{
			Date:  r.Date,
			Count: uint32(r.Count),
		})
	}
	return &adminV1.LoginTrendResponse{Points: points}, nil
}

// GetOperationActionDistribution 返回操作审计按 action 的分布。
func (s *DashboardService) GetOperationActionDistribution(ctx context.Context, _ *emptypb.Empty) (*adminV1.ActionDistributionResponse, error) {
	rows, err := s.dashboardRepo.OperationActionDistribution(ctx)
	if err != nil {
		return nil, err
	}

	items := make([]*adminV1.DistributionItem, 0, len(rows))
	for _, r := range rows {
		items = append(items, &adminV1.DistributionItem{
			Label: r.Action,
			Count: uint32(r.Count),
		})
	}
	return &adminV1.ActionDistributionResponse{Items: items}, nil
}

// GetLoginStatusDistribution 返回登录审计按 status 的分布。
func (s *DashboardService) GetLoginStatusDistribution(ctx context.Context, _ *emptypb.Empty) (*adminV1.StatusDistributionResponse, error) {
	rows, err := s.dashboardRepo.LoginStatusDistribution(ctx)
	if err != nil {
		return nil, err
	}

	items := make([]*adminV1.DistributionItem, 0, len(rows))
	for _, r := range rows {
		items = append(items, &adminV1.DistributionItem{
			Label: r.Status,
			Count: uint32(r.Count),
		})
	}
	return &adminV1.StatusDistributionResponse{Items: items}, nil
}

// 异常信号阈值（24h 窗口）。
const (
	nightOpsAlertMin     = 1 // 深夜（本地 0-6 点）操作条数下限
	userFailedAlertMin   = 3 // 单用户失败操作条数下限
	loginFailAlertMin    = 3 // 单账号登录失败次数下限（疑似口令尝试）
	sensitiveOpsAlertMin = 1 // 敏感操作（DELETE/EXPORT/ASSIGN）条数下限
	sensitiveDetailMax   = 5 // 告警明细里最多列出的敏感操作条数
)

// GetAiInsights 安全与异常洞察：挖掘近 24h 审计明细里的行为模式
// （深夜操作 / 操作失败集中 / 疑似口令尝试 / 敏感操作），规则预筛出结构化告警，
// LLM 仅对告警清单生成总体评估措辞——告警本身是确定性事实，不依赖模型编造。
// 平台用户专属：明细日志为全平台数据，且会外发到模型端点。
func (s *DashboardService) GetAiInsights(ctx context.Context, _ *emptypb.Empty) (*adminV1.AiInsightsResponse, error) {
	operator, err := auth.FromContext(ctx)
	if err != nil {
		return nil, err
	}
	if operator.GetTenantId() != 0 {
		return nil, adminV1.ErrorForbidden("ai insights is platform-only")
	}

	var alerts []*adminV1.AiInsightAlert
	addAlert := func(severity, title, detail string) {
		alerts = append(alerts, &adminV1.AiInsightAlert{Severity: severity, Title: title, Detail: detail})
	}

	// 1. 疑似口令尝试：登录连续失败
	failAccounts, err := s.dashboardRepo.LoginFailAccounts(ctx, loginFailAlertMin)
	if err != nil {
		return nil, err
	}
	for _, acc := range failAccounts {
		addAlert("HIGH", fmt.Sprintf("疑似口令尝试：账号 %s 24h 内登录失败 %d 次", acc.Username, acc.Fails),
			fmt.Sprintf("最近一次失败来源 IP：%s。建议确认是否本人操作，必要时重置口令并临时封禁来源。", acc.LastIP))
	}

	// 2. 用户行为模式
	behaviors, err := s.dashboardRepo.UserBehavior24h(ctx)
	if err != nil {
		return nil, err
	}
	for _, b := range behaviors {
		if b.Night >= nightOpsAlertMin {
			addAlert("MEDIUM", fmt.Sprintf("非常规时段操作：账号 %s 在本地 0-6 点执行了 %d 次操作", b.Username, b.Night),
				"深夜时段的操作偏离常规作息，若非计划内变更/运维窗口，建议与该账号持有人核实。")
		}
		if b.Failed >= userFailedAlertMin {
			addAlert("MEDIUM", fmt.Sprintf("操作失败集中：账号 %s 24h 内失败操作 %d 次（共 %d 次）", b.Username, b.Failed, b.Total),
				"连续失败可能意味着权限不足或越权尝试，建议在操作日志中核对失败资源与原因。")
		}
	}

	// 3. 敏感操作（DELETE/EXPORT/ASSIGN）
	sensitiveOps, err := s.dashboardRepo.SensitiveOps24h(ctx, sensitiveDetailMax)
	if err != nil {
		return nil, err
	}
	if len(sensitiveOps) >= sensitiveOpsAlertMin {
		var sb strings.Builder
		sb.WriteString("最近 24 小时的敏感操作：")
		for i, op := range sensitiveOps {
			if i > 0 {
				sb.WriteString("；")
			}
			sb.WriteString(fmt.Sprintf("%s 于 %s 执行 %s（%s）",
				op.Username, op.CreatedAt.Format("01-02 15:04"), op.Action, op.ResourceType))
		}
		if len(sensitiveOps) >= sensitiveDetailMax {
			sb.WriteString("……")
		}
		addAlert("LOW", fmt.Sprintf("存在 %d 条敏感操作（删除/导出/授权变更）", len(sensitiveOps)), sb.String())
	}

	// 4. 总体评估：有告警时才调 LLM（无告警直接给确定性结论，不浪费调用也不编造）
	summary := "最近 24 小时未检测到异常行为模式。"
	if len(alerts) > 0 {
		var alertFacts strings.Builder
		for _, a := range alerts {
			alertFacts.WriteString("- [" + a.Severity + "] " + a.Title + "\n")
		}
		assessment, err := s.scriptRuntime.ChatForScript(ctx, 0,
			"你是企业后台的安全运营助手。以下是从审计日志规则筛出的异常信号清单，"+
				"用中文写一段 100 字以内的总体风险评估：概括风险等级与最需要优先处置的一项，"+
				"语气克制、只基于给定信号，不要编造未提及的信息。",
			alertFacts.String())
		if err != nil {
			// LLM 不可用不阻断告警：告警本身是确定性事实
			s.log.Errorf(ctx, "ai insights assessment failed, alerts still returned: %v", err)
		} else {
			summary = assessment
		}
	}

	return &adminV1.AiInsightsResponse{
		Summary:  summary,
		Insights: alertsToPoints(alerts),
		Alerts:   alerts,
	}, nil
}

// alertsToPoints 告警 → 前端要点列表（severity 前缀便于兼容旧渲染）。
func alertsToPoints(alerts []*adminV1.AiInsightAlert) []string {
	points := make([]string, 0, len(alerts))
	for _, a := range alerts {
		points = append(points, a.Title)
	}
	return points
}

// splitInsightPoints 把模型输出的要点文本拆成结构化条目：
// 识别 "- "/"* "/数字序号前缀；无多行时按句号拆分。最多 5 条。
func splitInsightPoints(summary string) []string {
	var points []string
	for _, line := range strings.Split(summary, "\n") {
		line = strings.TrimSpace(line)
		line = strings.TrimPrefix(line, "- ")
		line = strings.TrimPrefix(line, "* ")
		line = strings.TrimPrefix(line, "• ")
		// 去数字序号（"1. " / "1、"）
		if idx := strings.IndexAny(line, ".、"); idx > 0 && idx <= 2 {
			rest := strings.TrimSpace(line[idx+1:])
			if rest != "" {
				line = rest
			}
		}
		if line != "" {
			points = append(points, line)
		}
	}
	if len(points) <= 1 && summary != "" {
		for _, s := range strings.Split(summary, "。") {
			if s = strings.TrimSpace(s); s != "" {
				points = append(points, s+"。")
			}
		}
	}
	if len(points) > 5 {
		points = points[:5]
	}
	return points
}
