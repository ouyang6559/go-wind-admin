// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package tenant

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	tenantpkg "go-wind-admin/backendz/internal/ent/gen/tenant"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TenantGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTenantGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TenantGetLogic {
	return &TenantGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TenantGetLogic) TenantGet(req *types.TenantGetReq) (resp *types.Tenant, err error) {
	var q = l.svcCtx.Ent.Tenant.Query().Where(tenantpkg.DeletedAtIsNil())
	if req.Id > 0 {
		q = q.Where(tenantpkg.IDEQ(uint32(req.Id)))
	} else if req.Code != "" {
		q = q.Where(tenantpkg.CodeEQ(req.Code))
	} else if req.Name != "" {
		q = q.Where(tenantpkg.NameEQ(req.Name))
	} else {
		return nil, xerr.BadRequestMsg("id, code or name required")
	}

	e, err := q.Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("tenant not found")
		}
		logx.WithContext(l.ctx).Errorf("get tenant failed: %v", err)
		return nil, err
	}

	item := baseToType(e)
	fillAdminAndCount(l.ctx, l.svcCtx.Ent, item)
	return item, nil
}