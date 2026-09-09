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

type DashboardGetOverviewLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDashboardGetOverviewLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DashboardGetOverviewLogic {
	return &DashboardGetOverviewLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DashboardGetOverviewLogic) DashboardGetOverview() (resp *types.DashboardOverviewResponse, err error) {
	userCount, err := countActiveUsers(l.ctx, l.svcCtx.Ent)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count active users failed: %v", err)
		return nil, xerr.ServerErrorMsg("count active users failed")
	}
	roleCount, err := countRoles(l.ctx, l.svcCtx.Ent)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count roles failed: %v", err)
		return nil, xerr.ServerErrorMsg("count roles failed")
	}
	todayLoginCount, err := countTodayLogins(l.ctx, l.svcCtx.Ent)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count today logins failed: %v", err)
		return nil, xerr.ServerErrorMsg("count today logins failed")
	}
	todayOperationCount, err := countTodayOperations(l.ctx, l.svcCtx.Ent)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count today operations failed: %v", err)
		return nil, xerr.ServerErrorMsg("count today operations failed")
	}

	return &types.DashboardOverviewResponse{
		UserCount:           int64(userCount),
		RoleCount:           int64(roleCount),
		TodayLoginCount:     int64(todayLoginCount),
		TodayOperationCount: int64(todayOperationCount),
	}, nil
}