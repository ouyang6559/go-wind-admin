package service

import (
	"fmt"
	"go-wind-admin/pkg/middleware/auth"
	"strings"
	"time"

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

// GetAiInsights AI 概览解读：把当日核心指标与近 7 天趋势喂给默认模型生成中文解读。
// 平台用户专属：dashboard 统计是全平台视角，且解读会把数据外发到模型端点，
// 租户用户一律拒绝（数据出域边界比普通统计接口严格）。
func (s *DashboardService) GetAiInsights(ctx context.Context, _ *emptypb.Empty) (*adminV1.AiInsightsResponse, error) {
	operator, err := auth.FromContext(ctx)
	if err != nil {
		return nil, err
	}
	if operator.GetTenantId() != 0 {
		return nil, adminV1.ErrorForbidden("ai insights is platform-only")
	}

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

	trendRows, err := s.dashboardRepo.LoginTrend(ctx, 7)
	if err != nil {
		return nil, err
	}
	actionRows, err := s.dashboardRepo.OperationActionDistribution(ctx)
	if err != nil {
		return nil, err
	}
	statusRows, err := s.dashboardRepo.LoginStatusDistribution(ctx)
	if err != nil {
		return nil, err
	}

	var facts strings.Builder
	facts.WriteString(fmt.Sprintf("截至 %s 的平台运营数据：\n", time.Now().Format("2006-01-02")))
	facts.WriteString(fmt.Sprintf("- 用户总数：%d\n- 角色总数：%d\n", userCount, roleCount))
	facts.WriteString(fmt.Sprintf("- 今日登录次数：%d\n- 今日操作次数：%d\n", todayLoginCount, todayOperationCount))
	facts.WriteString("- 近 7 天登录趋势：")
	for i, r := range trendRows {
		if i > 0 {
			facts.WriteString("，")
		}
		facts.WriteString(fmt.Sprintf("%s %d 次", r.Date, r.Count))
	}
	facts.WriteString("\n- 昨日以来操作动作分布：")
	for i, r := range actionRows {
		if i > 0 {
			facts.WriteString("，")
		}
		facts.WriteString(fmt.Sprintf("%s %d 次", r.Action, r.Count))
	}
	facts.WriteString("\n- 登录状态分布：")
	for i, r := range statusRows {
		if i > 0 {
			facts.WriteString("，")
		}
		facts.WriteString(fmt.Sprintf("%s %d 次", r.Status, r.Count))
	}
	facts.WriteString("\n")

	summary, err := s.scriptRuntime.ChatForScript(ctx, 0,
		"你是企业后台的运营数据分析助手。根据给出的平台运营统计数据，输出 3~5 条洞察要点："+
			"概括当日活跃度、点评登录趋势与失败信号、指出操作集中度，最后给一条可执行建议。"+
			"每条一行、以 \"- \" 开头，直接给结论并引用具体数字，不要输出标题或其他内容。",
		facts.String())
	if err != nil {
		s.log.Errorf(ctx, "dashboard ai insights failed: %v", err)
		return nil, adminV1.ErrorInternalServerError("ai insights failed: %v", err)
	}

	return &adminV1.AiInsightsResponse{
		Summary:  summary,
		Insights: splitInsightPoints(summary),
	}, nil
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
