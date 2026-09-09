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

type DashboardGetLoginStatusDistributionLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDashboardGetLoginStatusDistributionLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DashboardGetLoginStatusDistributionLogic {
	return &DashboardGetLoginStatusDistributionLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DashboardGetLoginStatusDistributionLogic) DashboardGetLoginStatusDistribution() (resp *types.StatusDistributionResponse, err error) {
	rows, err := loginStatusDistribution(l.ctx, l.svcCtx.Ent)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("login status distribution query failed: %v", err)
		return nil, xerr.ServerErrorMsg("login status distribution query failed")
	}

	items := make([]types.DistributionItem, 0, len(rows))
	for _, r := range rows {
		items = append(items, types.DistributionItem{Label: r.Status, Count: int64(r.Count)})
	}
	return &types.StatusDistributionResponse{Items: items}, nil
}