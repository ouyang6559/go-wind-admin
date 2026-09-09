// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package dashboard

import (
	"context"

	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type DashboardGetOperationActionDistributionLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDashboardGetOperationActionDistributionLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DashboardGetOperationActionDistributionLogic {
	return &DashboardGetOperationActionDistributionLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DashboardGetOperationActionDistributionLogic) DashboardGetOperationActionDistribution() (resp *types.ActionDistributionResponse, err error) {
	rows, err := operationActionDistribution(l.ctx, l.svcCtx.Ent)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("operation action distribution query failed: %v", err)
		return nil, xerr.ServerErrorMsg("operation action distribution query failed")
	}

	items := make([]types.DistributionItem, 0, len(rows))
	for _, r := range rows {
		items = append(items, types.DistributionItem{Label: r.Action, Count: int64(r.Count)})
	}
	return &types.ActionDistributionResponse{Items: items}, nil
}