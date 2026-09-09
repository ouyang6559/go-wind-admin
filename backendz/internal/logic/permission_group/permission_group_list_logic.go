// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package permission_group

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/permissiongroup"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type PermissionGroupListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPermissionGroupListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PermissionGroupListLogic {
	return &PermissionGroupListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PermissionGroupListLogic) PermissionGroupList(req *types.PageRequest) (resp *types.ListPermissionGroupResponse, err error) {
	q := l.svcCtx.Ent.PermissionGroup.Query().Where(permissiongroup.DeletedAtIsNil())

	// 搜索：按 name / module 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(permissiongroup.Or(
			permissiongroup.NameContainsFold(k),
			permissiongroup.ModuleContainsFold(k),
		))
	}

	q = q.Order(permissiongroup.BySortOrder(sql.OrderAsc()), permissiongroup.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count permission groups failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list permission groups failed: %v", err)
		return nil, err
	}

	items := make([]*types.PermissionGroup, 0, len(rows))
	for _, r := range rows {
		items = append(items, toType(r))
	}

	return &types.ListPermissionGroupResponse{
		Items: buildTree(items),
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}