// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package plan_module

import (
	"context"
	"strings"

	"go-wind-admin/backendz/internal/ent/gen/planmodule"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type PlanModuleListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPlanModuleListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PlanModuleListLogic {
	return &PlanModuleListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PlanModuleListLogic) PlanModuleList(req *types.PageRequest) (resp *types.ListPlanModuleResponse, err error) {
	q := l.svcCtx.Ent.PlanModule.Query().
		Where(planmodule.DeletedAtIsNil()).
		WithPlan()

	// 搜索：按 module 枚举字符串模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(planmodule.ModuleEQ(planmodule.Module(k)))
	}

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count plan modules failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list plan modules failed: %v", err)
		return nil, err
	}

	items := make([]types.PlanModule, 0, len(rows))
	for _, r := range rows {
		items = append(items, *toType(r))
	}

	return &types.ListPlanModuleResponse{
		Items: items,
		Total: int64(total),
	}, nil
}