// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package login_audit_log

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/loginauditlog"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type LoginAuditLogGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewLoginAuditLogGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *LoginAuditLogGetLogic {
	return &LoginAuditLogGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *LoginAuditLogGetLogic) LoginAuditLogGet(req *types.LoginAuditLogGetReq) (resp *types.LoginAuditLog, err error) {
	if req.Id <= 0 {
		return nil, xerr.BadRequestMsg("id required")
	}

	e, err := l.svcCtx.Ent.LoginAuditLog.Query().
		Where(loginauditlog.IDEQ(uint32(req.Id))).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("login audit log not found")
		}
		logx.WithContext(l.ctx).Errorf("get login audit log failed: %v", err)
		return nil, err
	}

	t := toType(l.ctx, l.svcCtx.Ent, e)
	return &t, nil
}