// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package policy_evaluation_log

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/policyevaluationlog"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PolicyEvaluationLogGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPolicyEvaluationLogGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PolicyEvaluationLogGetLogic {
	return &PolicyEvaluationLogGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PolicyEvaluationLogGetLogic) PolicyEvaluationLogGet(req *types.PolicyEvaluationLogGetReq) (resp *types.PolicyEvaluationLog, err error) {
	if req.Id <= 0 {
		return nil, xerr.BadRequestMsg("id required")
	}

	e, err := l.svcCtx.Ent.PolicyEvaluationLog.Query().
		Where(policyevaluationlog.IDEQ(uint32(req.Id))).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("policy evaluation log not found")
		}
		logx.WithContext(l.ctx).Errorf("get policy evaluation log failed: %v", err)
		return nil, err
	}

	t := toType(l.ctx, e)
	return &t, nil
}