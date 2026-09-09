// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package dict_entry

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/dictentry"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type DictEntryListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDictEntryListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DictEntryListLogic {
	return &DictEntryListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DictEntryListLogic) DictEntryList(req *types.PageRequest) (resp *types.ListDictEntryResponse, err error) {
	q := l.svcCtx.Ent.DictEntry.Query().
		Where(dictentry.DeletedAtIsNil()).
		WithDictType()

	// 搜索：按 entryValue 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(dictentry.EntryValueContainsFold(k))
	}

	// 排序
	q = q.Order(dictentry.BySortOrder(sql.OrderAsc()), dictentry.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count dict entries failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list dict entries failed: %v", err)
		return nil, err
	}

	items := make([]types.DictEntry, 0, len(rows))
	for _, r := range rows {
		item := toType(r)
		item.I18n = i18nOf(l.ctx, l.svcCtx.Ent, r.ID)
		items = append(items, *item)
	}

	return &types.ListDictEntryResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}