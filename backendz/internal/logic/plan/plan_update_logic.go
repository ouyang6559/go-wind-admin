// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package plan

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/plan"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PlanUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPlanUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PlanUpdateLogic {
	return &PlanUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PlanUpdateLogic) PlanUpdate(req *types.UpdatePlanRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.Plan.Query().
		Where(plan.IDEQ(uint32(req.Id)), plan.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("plan not found")
		}
		logx.WithContext(l.ctx).Errorf("get plan for update failed: %v", err)
		return xerr.ServerErrorMsg("get plan for update failed")
	}

	// 操作人
	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.Plan.UpdateOneID(existing.ID).
		SetUpdatedBy(operatorID).
		SetUpdatedAt(time.Now())
	if d.Name != "" {
		upd.SetName(d.Name)
	}
	if v := toVersion(d.Version); v != nil {
		upd.SetNillableVersion(v)
	}
	if v := toExpiryPolicy(d.ExpiryPolicy); v != nil {
		upd.SetNillableExpiryPolicy(v)
	}
	if d.DataRetentionDays > 0 {
		upd.SetDataRetentionDays(uint32(d.DataRetentionDays))
	}
	if d.Description != "" {
		upd.SetDescription(d.Description)
	}
	if d.Remark != "" {
		upd.SetRemark(d.Remark)
	}

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		if gen.IsConstraintError(uerr) {
			return xerr.BadRequestMsg("plan name already exists")
		}
		logx.WithContext(l.ctx).Errorf("update plan failed: %v", uerr)
		return xerr.ServerErrorMsg("update plan failed")
	}

	return nil
}