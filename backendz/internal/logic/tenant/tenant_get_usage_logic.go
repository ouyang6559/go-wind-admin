// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package tenant

import (
	"context"
	"strconv"

	"go-wind-admin/backendz/internal/ent/gen"
	tenantpkg "go-wind-admin/backendz/internal/ent/gen/tenant"
	"go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TenantGetUsageLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTenantGetUsageLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TenantGetUsageLogic {
	return &TenantGetUsageLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TenantGetUsageLogic) TenantGetUsage(req *types.TenantGetUsageReq) (resp *types.TenantUsage, err error) {
	if req.Id <= 0 {
		return nil, xerr.BadRequestMsg("tenant id is required")
	}

	e, err := l.svcCtx.Ent.Tenant.Query().
		Where(tenantpkg.IDEQ(uint32(req.Id)), tenantpkg.DeletedAtIsNil()).
		WithPlan().
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("tenant not found")
		}
		logx.WithContext(l.ctx).Errorf("get tenant for usage failed: %v", err)
		return nil, xerr.ServerErrorMsg("get tenant failed")
	}

	// 用量汇总（简化为实体计数，storage/api 调用以 0 兜底）。
	userCount, cerr := l.svcCtx.Ent.User.Query().
		Where(user.TenantIDEQ(e.ID), user.DeletedAtIsNil()).
		Count(l.ctx)
	if cerr != nil {
		logx.WithContext(l.ctx).Errorf("count tenant users failed: %v", cerr)
		return nil, xerr.ServerErrorMsg("count tenant users failed")
	}

	var planID int64
	var planName string
	if p := e.Edges.Plan; p != nil {
		planID = int64(p.ID)
	}
	if planName == "" {
		planName = std.Str(e.SubscriptionPlan)
	}

	return &types.TenantUsage{
		TenantId:         int64(e.ID),
		UserCount:        strconv.FormatInt(int64(userCount), 10),
		StorageUsedBytes: "0",
		ApiCallCount:     "0",
		PlanId:           planID,
		PlanName:         planName,
		Quotas:           []types.QuotaUsage{},
	}, nil
}