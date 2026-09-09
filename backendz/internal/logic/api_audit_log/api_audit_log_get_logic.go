// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package api_audit_log

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/apiauditlog"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type ApiAuditLogGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewApiAuditLogGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ApiAuditLogGetLogic {
	return &ApiAuditLogGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ApiAuditLogGetLogic) ApiAuditLogGet(req *types.ApiAuditLogGetReq) (resp *types.ApiAuditLog, err error) {
	e, err := l.svcCtx.Ent.ApiAuditLog.Query().
		Where(apiauditlog.IDEQ(uint32(req.Id))).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("api audit log not found")
		}
		logx.WithContext(l.ctx).Errorf("get api audit log failed: %v", err)
		return nil, err
	}

	item := toType(l.ctx, l.svcCtx.Ent, e)
	enrich(l.ctx, l.svcCtx.Ent, item)

	return item, nil
}