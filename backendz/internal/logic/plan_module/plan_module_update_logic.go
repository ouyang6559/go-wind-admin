// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package plan_module

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/planmodule"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PlanModuleUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPlanModuleUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PlanModuleUpdateLogic {
	return &PlanModuleUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PlanModuleUpdateLogic) PlanModuleUpdate(req *types.UpdatePlanModuleRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.PlanModule.Query().
		Where(planmodule.IDEQ(uint32(req.Id)), planmodule.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("plan module not found")
		}
		logx.WithContext(l.ctx).Errorf("get plan module for update failed: %v", err)
		return xerr.ServerErrorMsg("get plan module for update failed")
	}

	// 操作人
	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.PlanModule.UpdateOneID(existing.ID).
		SetUpdatedBy(operatorID).
		SetUpdatedAt(time.Now())
	if d.PlanId > 0 {
		upd.SetPlanID(uint32(d.PlanId))
	}
	// module 为空时跳过：ent schema 未声明零值，SetNillableModule 会触发校验失败
	if d.Module != "" {
		upd.SetNillableModule(modulePtr(d.Module))
	}

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("update plan module failed: %v", uerr)
		return xerr.ServerErrorMsg("update plan module failed")
	}

	return nil
}