// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package tenant

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	tenantpkg "go-wind-admin/backendz/internal/ent/gen/tenant"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type TenantDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTenantDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TenantDeleteLogic {
	return &TenantDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TenantDeleteLogic) TenantDelete(req *types.TenantDeleteReq) error {
	existing, err := l.svcCtx.Ent.Tenant.Query().
		Where(tenantpkg.IDEQ(uint32(req.Id)), tenantpkg.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("tenant not found")
		}
		logx.WithContext(l.ctx).Errorf("get tenant for delete failed: %v", err)
		return xerr.ServerErrorMsg("get tenant for delete failed")
	}

	if err := l.svcCtx.Ent.Tenant.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete tenant failed: %v", err)
		return xerr.ServerErrorMsg("soft delete tenant failed")
	}

	return nil
}