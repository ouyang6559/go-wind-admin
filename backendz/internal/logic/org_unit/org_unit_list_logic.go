// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package org_unit

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/orgunit"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type OrgUnitListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewOrgUnitListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *OrgUnitListLogic {
	return &OrgUnitListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *OrgUnitListLogic) OrgUnitList(req *types.PageRequest) (resp *types.ListOrgUnitResponse, err error) {
	q := l.svcCtx.Ent.OrgUnit.Query().Where(orgunit.DeletedAtIsNil())

	// 搜索：按名称或编码模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			orgunit.Or(
				orgunit.NameContainsFold(k),
				orgunit.CodeContainsFold(k),
			),
		)
	}

	// 排序
	q = q.Order(orgunit.BySortOrder(sql.OrderAsc()), orgunit.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count org units failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list org units failed: %v", err)
		return nil, err
	}

	items := make([]types.OrgUnit, 0, len(rows))
	for _, e := range rows {
		items = append(items, *toType(l.ctx, l.svcCtx.Ent, e))
	}

	return &types.ListOrgUnitResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}
