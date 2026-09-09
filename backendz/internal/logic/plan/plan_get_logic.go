// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package plan

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/plan"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PlanGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPlanGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PlanGetLogic {
	return &PlanGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PlanGetLogic) PlanGet(req *types.PlanGetReq) (resp *types.Plan, err error) {
	if req.Id <= 0 {
		return nil, xerr.BadRequestMsg("id required")
	}

	e, err := l.svcCtx.Ent.Plan.Query().
		Where(plan.IDEQ(uint32(req.Id)), plan.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("plan not found")
		}
		logx.WithContext(l.ctx).Errorf("get plan failed: %v", err)
		return nil, err
	}

	return toType(e), nil
}