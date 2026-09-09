// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package login_policy

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/loginpolicy"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type LoginPolicyGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewLoginPolicyGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *LoginPolicyGetLogic {
	return &LoginPolicyGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *LoginPolicyGetLogic) LoginPolicyGet(req *types.LoginPolicyGetReq) (resp *types.LoginPolicy, err error) {
	if req.Id <= 0 {
		return nil, xerr.BadRequestMsg("id required")
	}

	e, err := l.svcCtx.Ent.LoginPolicy.Query().
		Where(loginpolicy.IDEQ(uint32(req.Id)), loginpolicy.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("login policy not found")
		}
		logx.WithContext(l.ctx).Errorf("get login policy failed: %v", err)
		return nil, err
	}

	return toType(l.ctx, l.svcCtx.Ent, e), nil
}
