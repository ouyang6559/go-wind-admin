// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package plan

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/plan"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type PlanListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPlanListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PlanListLogic {
	return &PlanListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PlanListLogic) PlanList(req *types.PageRequest) (resp *types.ListPlanResponse, err error) {
	q := l.svcCtx.Ent.Plan.Query().Where(plan.DeletedAtIsNil())

	// 搜索：按 name 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(plan.NameContainsFold(k))
	}

	// 排序
	q = q.Order(plan.ByName(sql.OrderAsc()), plan.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count plans failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list plans failed: %v", err)
		return nil, err
	}

	items := make([]types.Plan, 0, len(rows))
	for _, r := range rows {
		items = append(items, *toType(r))
	}

	return &types.ListPlanResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}