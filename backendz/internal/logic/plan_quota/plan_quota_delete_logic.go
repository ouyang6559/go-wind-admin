// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package plan_quota

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/planquota"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PlanQuotaDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPlanQuotaDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PlanQuotaDeleteLogic {
	return &PlanQuotaDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PlanQuotaDeleteLogic) PlanQuotaDelete(req *types.PlanQuotaDeleteReq) error {
	if req.Id <= 0 {
		return xerr.BadRequestMsg("id required")
	}

	existing, err := l.svcCtx.Ent.PlanQuota.Query().
		Where(planquota.IDEQ(uint32(req.Id)), planquota.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("plan quota not found")
		}
		logx.WithContext(l.ctx).Errorf("get plan quota for delete failed: %v", err)
		return xerr.ServerErrorMsg("get plan quota for delete failed")
	}

	if err := l.svcCtx.Ent.PlanQuota.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete plan quota failed: %v", err)
		return xerr.ServerErrorMsg("soft delete plan quota failed")
	}

	return nil
}