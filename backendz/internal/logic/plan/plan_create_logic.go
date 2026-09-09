// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package plan

import (
	"context"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PlanCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPlanCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PlanCreateLogic {
	return &PlanCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PlanCreateLogic) PlanCreate(req *types.CreatePlanRequest) error {
	d := req.Data
	if strings.TrimSpace(d.Name) == "" {
		return xerr.BadRequestMsg("plan name required")
	}

	// 操作人
	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	var days *uint32
	if d.DataRetentionDays > 0 {
		dv := uint32(d.DataRetentionDays)
		days = &dv
	}

	builder := l.svcCtx.Ent.Plan.Create().
		SetName(d.Name).
		SetNillableVersion(toVersion(d.Version)).
		SetNillableExpiryPolicy(toExpiryPolicy(d.ExpiryPolicy)).
		SetNillableDataRetentionDays(days).
		SetNillableDescription(strPtr(d.Description)).
		SetNillableRemark(strPtr(d.Remark)).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now())

	if _, err := builder.Save(l.ctx); err != nil {
		// sys_plans.name 有唯一索引：重名是用户可自行纠正的输入错误，返回 400
		if gen.IsConstraintError(err) {
			return xerr.BadRequestMsg("plan name already exists")
		}
		logx.WithContext(l.ctx).Errorf("create plan failed: %v", err)
		return xerr.ServerErrorMsg("create plan failed")
	}

	return nil
}

func strPtr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}