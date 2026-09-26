package service

import (
	"context"

	"github.com/tx7do/kratos-bootstrap/bootstrap"
	bLogger "github.com/tx7do/kratos-bootstrap/logger"

	paginationV1 "github.com/tx7do/go-crud/api/gen/go/pagination/v1"

	adminV1 "go-wind-admin/api/gen/go/admin/service/v1"
	aiV1 "go-wind-admin/api/gen/go/ai/service/v1"
	"go-wind-admin/app/admin/service/internal/data"
)

// AiUsageLogService 用量流水查询（配额记账事实源的只读视图）。
// 租户隔离由 mixin 谓词自动生效：租户管理员只见本租户流水，平台管理员见全量。
type AiUsageLogService struct {
	adminV1.AiUsageLogServiceHTTPServer
	log  *bLogger.Helper
	repo *data.AiUsageLogRepo
}

func NewAiUsageLogService(ctx *bootstrap.Context, repo *data.AiUsageLogRepo) *AiUsageLogService {
	return &AiUsageLogService{
		log:  ctx.NewLoggerHelper("ai_usage_log/service/admin-service"),
		repo: repo,
	}
}

func (s *AiUsageLogService) List(ctx context.Context, req *paginationV1.PagingRequest) (*aiV1.ListAiUsageLogResponse, error) {
	return s.repo.List(ctx, req)
}

func (s *AiUsageLogService) Count(ctx context.Context, req *paginationV1.PagingRequest) (*aiV1.CountAiUsageLogResponse, error) {
	return s.repo.Count(ctx, req)
}
