// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package plan_module

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/planmodule"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PlanModuleCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPlanModuleCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PlanModuleCreateLogic {
	return &PlanModuleCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PlanModuleCreateLogic) PlanModuleCreate(req *types.CreatePlanModuleRequest) error {
	d := req.Data
	if d.PlanId <= 0 {
		return xerr.BadRequestMsg("plan id required")
	}

	// 操作人
	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	builder := l.svcCtx.Ent.PlanModule.Create().
		SetPlanID(uint32(d.PlanId)).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now())

	// module 为空（未指定模块）时跳过：ent schema 未声明零值，SetNillableModule 会触发校验失败
	if d.Module != "" {
		builder.SetNillableModule(modulePtr(d.Module))
	}

	if _, err := builder.Save(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("create plan module failed: %v", err)
		return xerr.ServerErrorMsg("create plan module failed")
	}

	return nil
}

func modulePtr(s string) *planmodule.Module {
	m := planmodule.Module(s)
	return &m
}