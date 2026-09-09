// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package permission_audit_log

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/permissionauditlog"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PermissionAuditLogGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPermissionAuditLogGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PermissionAuditLogGetLogic {
	return &PermissionAuditLogGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PermissionAuditLogGetLogic) PermissionAuditLogGet(req *types.PermissionAuditLogGetReq) (resp *types.PermissionAuditLog, err error) {
	e, err := l.svcCtx.Ent.PermissionAuditLog.Query().
		Where(permissionauditlog.IDEQ(uint32(req.Id))).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("permission audit log not found")
		}
		logx.WithContext(l.ctx).Errorf("get permission audit log failed: %v", err)
		return nil, err
	}

	return toType(e), nil
}