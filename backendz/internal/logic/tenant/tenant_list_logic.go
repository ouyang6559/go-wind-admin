// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package tenant

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	tenantpkg "go-wind-admin/backendz/internal/ent/gen/tenant"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type TenantListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewTenantListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *TenantListLogic {
	return &TenantListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *TenantListLogic) TenantList(req *types.PageRequest) (resp *types.ListTenantResponse, err error) {
	q := l.svcCtx.Ent.Tenant.Query().Where(tenantpkg.DeletedAtIsNil())

	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			tenantpkg.Or(
				tenantpkg.NameContainsFold(k),
				tenantpkg.CodeContainsFold(k),
			),
		)
	}

	q = q.Order(tenantpkg.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count tenants failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list tenants failed: %v", err)
		return nil, err
	}

	items := make([]types.Tenant, 0, len(rows))
	for _, r := range rows {
		item := baseToType(r)
		fillAdminAndCount(l.ctx, l.svcCtx.Ent, item)
		items = append(items, *item)
	}

	return &types.ListTenantResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}