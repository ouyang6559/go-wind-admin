// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package api

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/api"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type ApiListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewApiListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ApiListLogic {
	return &ApiListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ApiListLogic) ApiList(req *types.PageRequest) (resp *types.ListApiResponse, err error) {
	q := l.svcCtx.Ent.Api.Query().Where(api.DeletedAtIsNil())

	// 搜索：按 operation / path / method / module 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(api.Or(
			api.OperationContainsFold(k),
			api.PathContainsFold(k),
			api.MethodContainsFold(k),
			api.ModuleContainsFold(k),
			api.DescriptionContainsFold(k),
		))
	}

	// 排序
	q = q.Order(api.ByModule(sql.OrderAsc()), api.ByPath(sql.OrderAsc()), api.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count apis failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list apis failed: %v", err)
		return nil, err
	}

	items := make([]types.Api, 0, len(rows))
	for _, r := range rows {
		items = append(items, *toType(r))
	}

	return &types.ListApiResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}