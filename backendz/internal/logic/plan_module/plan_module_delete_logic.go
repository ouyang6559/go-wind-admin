// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package plan_module

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/planmodule"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PlanModuleDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPlanModuleDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PlanModuleDeleteLogic {
	return &PlanModuleDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PlanModuleDeleteLogic) PlanModuleDelete(req *types.PlanModuleDeleteReq) error {
	if req.Id <= 0 {
		return xerr.BadRequestMsg("id required")
	}

	existing, err := l.svcCtx.Ent.PlanModule.Query().
		Where(planmodule.IDEQ(uint32(req.Id)), planmodule.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("plan module not found")
		}
		logx.WithContext(l.ctx).Errorf("get plan module for delete failed: %v", err)
		return xerr.ServerErrorMsg("get plan module for delete failed")
	}

	if err := l.svcCtx.Ent.PlanModule.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete plan module failed: %v", err)
		return xerr.ServerErrorMsg("soft delete plan module failed")
	}

	return nil
}