// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package dict_entry

import (
	"context"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/dictentry"
	"go-wind-admin/backendz/internal/ent/gen/dicttype"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type DictEntryListByTypeCodeLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDictEntryListByTypeCodeLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DictEntryListByTypeCodeLogic {
	return &DictEntryListByTypeCodeLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DictEntryListByTypeCodeLogic) DictEntryListByTypeCode(req *types.DictEntryListByTypeCodeReq) (resp *types.ListDictEntryByTypeCodeResponse, err error) {
	if req.TypeCode == "" {
		return nil, xerr.BadRequestMsg("typeCode required")
	}

	rows, err := l.svcCtx.Ent.DictEntry.Query().
		Where(
			dictentry.HasDictTypeWith(dicttype.TypeCodeEQ(req.TypeCode), dicttype.DeletedAtIsNil()),
			dictentry.IsEnabledEQ(true),
			dictentry.DeletedAtIsNil(),
		).
		Order(dictentry.BySortOrder(sql.OrderAsc()), dictentry.ByID(sql.OrderAsc())).
		All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("query dict entry by type code failed: %v", err)
		return nil, err
	}

	items := make([]types.DictEntry, 0, len(rows))
	for _, r := range rows {
		item := toType(r)
		i18n := i18nOf(l.ctx, l.svcCtx.Ent, r.ID)
		if req.Local != "" {
			// 仅返回指定语言的多语言数据
			if v, ok := i18n[req.Local]; ok {
				item.I18n = map[string]interface{}{req.Local: v}
			} else {
				item.I18n = map[string]interface{}{}
			}
		} else {
			item.I18n = i18n
		}
		items = append(items, *item)
	}

	return &types.ListDictEntryByTypeCodeResponse{Items: items}, nil
}