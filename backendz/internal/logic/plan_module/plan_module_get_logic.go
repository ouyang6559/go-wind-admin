// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package plan_module

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/planmodule"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PlanModuleGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPlanModuleGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PlanModuleGetLogic {
	return &PlanModuleGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PlanModuleGetLogic) PlanModuleGet(req *types.PlanModuleGetReq) (resp *types.PlanModule, err error) {
	if req.Id <= 0 {
		return nil, xerr.BadRequestMsg("id required")
	}

	e, err := l.svcCtx.Ent.PlanModule.Query().
		Where(planmodule.IDEQ(uint32(req.Id)), planmodule.DeletedAtIsNil()).
		WithPlan().
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("plan module not found")
		}
		logx.WithContext(l.ctx).Errorf("get plan module failed: %v", err)
		return nil, err
	}

	return toType(e), nil
}