// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package role

import (
	"context"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/role"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type RoleListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewRoleListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *RoleListLogic {
	return &RoleListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *RoleListLogic) RoleList(req *types.PageRequest) (resp *types.ListRoleResponse, err error) {
	q := l.svcCtx.Ent.Role.Query().Where(role.DeletedAtIsNil())

	// 搜索：按 code 或 name 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			role.Or(
				role.CodeContainsFold(k),
				role.NameContainsFold(k),
			),
		)
	}

	// 排序
	q = q.Order(role.BySortOrder(sql.OrderAsc()), role.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count roles failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list roles failed: %v", err)
		return nil, err
	}

	items := make([]types.Role, 0, len(rows))
	for _, r := range rows {
		item, cerr := toType(l.ctx, l.svcCtx.Ent, r)
		if cerr != nil {
			logx.WithContext(l.ctx).Errorf("convert role failed: %v", cerr)
			return nil, cerr
		}
		items = append(items, *item)
	}

	return &types.ListRoleResponse{
		Items: items,
		Total: int64(total),
	}, nil
}