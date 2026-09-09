// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package dict_type

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/dicttype"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type DictTypeListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDictTypeListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DictTypeListLogic {
	return &DictTypeListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DictTypeListLogic) DictTypeList(req *types.PageRequest) (resp *types.ListDictTypeResponse, err error) {
	q := l.svcCtx.Ent.DictType.Query().Where(dicttype.DeletedAtIsNil())

	// 搜索：按 typeCode 或 typeName 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(dicttype.Or(
			dicttype.TypeCodeContainsFold(k),
			dicttype.TypeNameContainsFold(k),
		))
	}

	// 排序
	q = q.Order(dicttype.BySortOrder(sql.OrderAsc()), dicttype.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count dict types failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list dict types failed: %v", err)
		return nil, err
	}

	items := make([]types.DictType, 0, len(rows))
	for _, r := range rows {
		items = append(items, *toType(r))
	}

	return &types.ListDictTypeResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}