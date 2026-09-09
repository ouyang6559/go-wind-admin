// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package plan_quota

import (
	"context"
	"strconv"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/planquota"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PlanQuotaUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPlanQuotaUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PlanQuotaUpdateLogic {
	return &PlanQuotaUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PlanQuotaUpdateLogic) PlanQuotaUpdate(req *types.UpdatePlanQuotaRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.PlanQuota.Query().
		Where(planquota.IDEQ(uint32(req.Id)), planquota.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("plan quota not found")
		}
		logx.WithContext(l.ctx).Errorf("get plan quota for update failed: %v", err)
		return xerr.ServerErrorMsg("get plan quota for update failed")
	}

	// 操作人
	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.PlanQuota.UpdateOneID(existing.ID).
		SetUpdatedBy(operatorID).
		SetUpdatedAt(time.Now())
	if d.PlanId > 0 {
		upd.SetPlanID(uint32(d.PlanId))
	}
	if q := quotaTypePtr(d.QuotaType); q != nil {
		upd.SetNillableQuotaType(q)
	}
	if v, verr := strconv.ParseUint(d.QuotaValue, 10, 64); verr == nil {
		upd.SetNillableQuotaValue(&v)
	}

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("update plan quota failed: %v", uerr)
		return xerr.ServerErrorMsg("update plan quota failed")
	}

	return nil
}