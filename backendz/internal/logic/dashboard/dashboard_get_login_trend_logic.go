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

type DashboardGetLoginTrendLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDashboardGetLoginTrendLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DashboardGetLoginTrendLogic {
	return &DashboardGetLoginTrendLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DashboardGetLoginTrendLogic) DashboardGetLoginTrend(req *types.DashboardGetLoginTrendReq) (resp *types.LoginTrendResponse, err error) {
	rows, err := loginTrend(l.ctx, l.svcCtx.Ent, int(req.Days))
	if err != nil {
		logx.WithContext(l.ctx).Errorf("login trend query failed: %v", err)
		return nil, xerr.ServerErrorMsg("login trend query failed")
	}

	points := make([]types.TrendPoint, 0, len(rows))
	for _, r := range rows {
		points = append(points, types.TrendPoint{Date: r.Date, Count: int64(r.Count)})
	}
	return &types.LoginTrendResponse{Points: points}, nil
}