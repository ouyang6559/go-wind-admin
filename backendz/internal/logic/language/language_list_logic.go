// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package language

import (
	"context"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/language"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type LanguageListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewLanguageListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *LanguageListLogic {
	return &LanguageListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *LanguageListLogic) LanguageList(req *types.PageRequest) (resp *types.ListLanguageResponse, err error) {
	q := l.svcCtx.Ent.Language.Query().Where(language.DeletedAtIsNil())

	// 搜索：按 languageCode 或 languageName 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(language.Or(
			language.LanguageCodeContainsFold(k),
			language.LanguageNameContainsFold(k),
		))
	}

	// 排序
	q = q.Order(language.BySortOrder(sql.OrderAsc()), language.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count languages failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list languages failed: %v", err)
		return nil, err
	}

	items := make([]types.Language, 0, len(rows))
	for _, r := range rows {
		items = append(items, *toType(r))
	}

	return &types.ListLanguageResponse{
		Items: items,
		Total: int64(total),
	}, nil
}