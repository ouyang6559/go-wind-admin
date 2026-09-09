// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package internal_message_category

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/internalmessagecategory"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type InternalMessageCategoryListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewInternalMessageCategoryListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *InternalMessageCategoryListLogic {
	return &InternalMessageCategoryListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *InternalMessageCategoryListLogic) InternalMessageCategoryList(req *types.PageRequest) (resp *types.ListInternalMessageCategoryResponse, err error) {
	q := l.svcCtx.Ent.InternalMessageCategory.Query().Where(internalmessagecategory.DeletedAtIsNil())

	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			internalmessagecategory.Or(
				internalmessagecategory.NameContainsFold(k),
				internalmessagecategory.CodeContainsFold(k),
			),
		)
	}

	q = q.Order(internalmessagecategory.BySortOrder(sql.OrderAsc()), internalmessagecategory.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count internal message categories failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list internal message categories failed: %v", err)
		return nil, err
	}

	items := make([]types.InternalMessageCategory, 0, len(rows))
	for _, r := range rows {
		items = append(items, *toType(l.ctx, l.svcCtx.Ent, r))
	}

	return &types.ListInternalMessageCategoryResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}