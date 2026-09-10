// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package permission

import (
	"context"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/permission"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type PermissionListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPermissionListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PermissionListLogic {
	return &PermissionListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PermissionListLogic) PermissionList(req *types.PageRequest) (resp *types.ListPermissionResponse, err error) {
	q := l.svcCtx.Ent.Permission.Query().Where(permission.DeletedAtIsNil())

	// 搜索：按 code / name / description 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(permission.Or(
			permission.CodeContainsFold(k),
			permission.NameContainsFold(k),
			permission.DescriptionContainsFold(k),
		))
	}

	q = q.Order(permission.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count permissions failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list permissions failed: %v", err)
		return nil, err
	}

	items := make([]types.Permission, 0, len(rows))
	for _, r := range rows {
		item, cerr := toType(l.ctx, l.svcCtx.Ent, r)
		if cerr != nil {
			logx.WithContext(l.ctx).Errorf("convert permission failed: %v", cerr)
			return nil, cerr
		}
		items = append(items, *item)
	}

	return &types.ListPermissionResponse{
		Items: items,
		Total: int64(total),
	}, nil
}