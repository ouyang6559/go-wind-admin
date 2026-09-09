// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package data_access_audit_log

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/dataaccessauditlog"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type DataAccessAuditLogGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDataAccessAuditLogGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DataAccessAuditLogGetLogic {
	return &DataAccessAuditLogGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DataAccessAuditLogGetLogic) DataAccessAuditLogGet(req *types.DataAccessAuditLogGetReq) (resp *types.DataAccessAuditLog, err error) {
	if req.Id <= 0 {
		return nil, xerr.BadRequestMsg("id required")
	}

	e, err := l.svcCtx.Ent.DataAccessAuditLog.Query().
		Where(dataaccessauditlog.IDEQ(uint32(req.Id))).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("data access audit log not found")
		}
		logx.WithContext(l.ctx).Errorf("get data access audit log failed: %v", err)
		return nil, err
	}

	t := toType(l.ctx, l.svcCtx.Ent, e)
	return &t, nil
}