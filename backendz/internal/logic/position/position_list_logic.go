// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package position

import (
	"context"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/position"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type PositionListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPositionListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PositionListLogic {
	return &PositionListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PositionListLogic) PositionList(req *types.PageRequest) (resp *types.ListPositionResponse, err error) {
	q := l.svcCtx.Ent.Position.Query().Where(position.DeletedAtIsNil())

	// 搜索：按名称或编码模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			position.Or(
				position.NameContainsFold(k),
				position.CodeContainsFold(k),
			),
		)
	}

	// 排序
	q = q.Order(position.BySortOrder(sql.OrderAsc()), position.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count positions failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list positions failed: %v", err)
		return nil, err
	}

	items := make([]types.Position, 0, len(rows))
	for _, e := range rows {
		items = append(items, *toType(l.ctx, l.svcCtx.Ent, e))
	}

	return &types.ListPositionResponse{
		Items: items,
		Total: int64(total),
	}, nil
}
