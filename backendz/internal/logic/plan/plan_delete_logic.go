// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package plan

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/plan"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PlanDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPlanDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PlanDeleteLogic {
	return &PlanDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PlanDeleteLogic) PlanDelete(req *types.PlanDeleteReq) error {
	if req.Id <= 0 {
		return xerr.BadRequestMsg("id required")
	}

	existing, err := l.svcCtx.Ent.Plan.Query().
		Where(plan.IDEQ(uint32(req.Id)), plan.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("plan not found")
		}
		logx.WithContext(l.ctx).Errorf("get plan for delete failed: %v", err)
		return xerr.ServerErrorMsg("get plan for delete failed")
	}

	if err := l.svcCtx.Ent.Plan.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete plan failed: %v", err)
		return xerr.ServerErrorMsg("soft delete plan failed")
	}

	return nil
}