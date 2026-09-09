// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package tenant

import (
	"context"
	"strings"

	tenantpkg "go-wind-admin/backendz/internal/ent/gen/tenant"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type TenantTenantExistsLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTenantTenantExistsLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TenantTenantExistsLogic {
	return &TenantTenantExistsLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TenantTenantExistsLogic) TenantTenantExists(req *types.TenantTenantExistsReq) (resp *types.TenantExistsResponse, err error) {
	exist, err := tenantExists(l.ctx, l, strings.TrimSpace(req.Code), strings.TrimSpace(req.Name))
	if err != nil {
		logx.WithContext(l.ctx).Errorf("check tenant exists failed: %v", err)
		return nil, err
	}
	return &types.TenantExistsResponse{Exist: exist}, nil
}

// tenantExists 检查给定 code 或 name 的租户是否已存在。
func tenantExists(ctx context.Context, l *TenantTenantExistsLogic, code, name string) (bool, error) {
	if code == "" && name == "" {
		return false, nil
	}

	var q = l.svcCtx.Ent.Tenant.Query().Where(tenantpkg.DeletedAtIsNil())
	if code != "" && name != "" {
		q = q.Where(tenantpkg.Or(tenantpkg.CodeEQ(code), tenantpkg.NameEQ(name)))
	} else if code != "" {
		q = q.Where(tenantpkg.CodeEQ(code))
	} else {
		q = q.Where(tenantpkg.NameEQ(name))
	}

	count, err := q.Count(ctx)
	if err != nil {
		return false, err
	}
	return count > 0, nil
}