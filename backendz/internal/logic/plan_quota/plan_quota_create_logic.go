// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package plan_quota

import (
	"context"
	"strconv"
	"time"

	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PlanQuotaCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPlanQuotaCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PlanQuotaCreateLogic {
	return &PlanQuotaCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PlanQuotaCreateLogic) PlanQuotaCreate(req *types.CreatePlanQuotaRequest) error {
	d := req.Data
	if d.PlanId <= 0 {
		return xerr.BadRequestMsg("plan id required")
	}

	// 操作人
	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	builder := l.svcCtx.Ent.PlanQuota.Create().
		SetPlanID(uint32(d.PlanId)).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now())

	if q := quotaTypePtr(d.QuotaType); q != nil {
		builder.SetNillableQuotaType(q)
	}
	if v, verr := strconv.ParseUint(d.QuotaValue, 10, 64); verr == nil {
		builder.SetNillableQuotaValue(&v)
	}

	if _, err := builder.Save(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("create plan quota failed: %v", err)
		return xerr.ServerErrorMsg("create plan quota failed")
	}

	return nil
}