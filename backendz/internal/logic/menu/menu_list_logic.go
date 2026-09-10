// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package menu

import (
	"context"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/menu"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type MenuListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewMenuListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *MenuListLogic {
	return &MenuListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *MenuListLogic) MenuList(req *types.PageRequest) (resp *types.ListMenuResponse, err error) {
	q := l.svcCtx.Ent.Menu.Query().Where(menu.DeletedAtIsNil())

	// 搜索：按 name / path / component 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(menu.Or(
			menu.NameContainsFold(k),
			menu.PathContainsFold(k),
			menu.ComponentContainsFold(k),
		))
	}

	q = q.Order(menu.ByParentID(sql.OrderAsc()), menu.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count menus failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list menus failed: %v", err)
		return nil, err
	}

	items := make([]*types.Menu, 0, len(rows))
	for _, r := range rows {
		items = append(items, toType(r))
	}

	return &types.ListMenuResponse{
		Items: buildTree(items),
		Total: int64(total),
	}, nil
}