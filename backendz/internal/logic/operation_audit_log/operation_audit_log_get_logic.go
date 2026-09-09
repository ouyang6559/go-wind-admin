// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package operation_audit_log

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/operationauditlog"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type OperationAuditLogGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewOperationAuditLogGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *OperationAuditLogGetLogic {
	return &OperationAuditLogGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *OperationAuditLogGetLogic) OperationAuditLogGet(req *types.OperationAuditLogGetReq) (resp *types.OperationAuditLog, err error) {
	if req.Id <= 0 {
		return nil, xerr.BadRequestMsg("id required")
	}

	e, err := l.svcCtx.Ent.OperationAuditLog.Query().
		Where(operationauditlog.IDEQ(uint32(req.Id))).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("operation audit log not found")
		}
		logx.WithContext(l.ctx).Errorf("get operation audit log failed: %v", err)
		return nil, err
	}

	t := toType(l.ctx, l.svcCtx.Ent, e)
	return &t, nil
}