// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package login_policy

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/loginpolicy"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type LoginPolicyCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewLoginPolicyCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *LoginPolicyCreateLogic {
	return &LoginPolicyCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *LoginPolicyCreateLogic) LoginPolicyCreate(req *types.CreateLoginPolicyRequest) error {
	d := req.Data
	if d.Value == "" {
		return xerr.BadRequestMsg("login policy value required")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	lpType := loginpolicy.TypeBlacklist
	if d.Type != "" {
		lpType = loginpolicy.Type(d.Type)
	}
	lpMethod := loginpolicy.MethodIp
	if d.Method != "" {
		lpMethod = loginpolicy.Method(d.Method)
	}

	_, err := l.svcCtx.Ent.LoginPolicy.Create().
		SetNillableTargetID(uiPtr(d.TargetId)).
		SetValue(d.Value).
		SetType(lpType).
		SetMethod(lpMethod).
		SetNillableReason(strPtr(d.Reason)).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now()).
		Save(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("create login policy failed: %v", err)
		return xerr.ServerErrorMsg("create login policy failed")
	}

	return nil
}
