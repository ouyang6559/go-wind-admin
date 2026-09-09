// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package plan_quota

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/planquota"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type PlanQuotaListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPlanQuotaListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PlanQuotaListLogic {
	return &PlanQuotaListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PlanQuotaListLogic) PlanQuotaList(req *types.PageRequest) (resp *types.ListPlanQuotaResponse, err error) {
	q := l.svcCtx.Ent.PlanQuota.Query().
		Where(planquota.DeletedAtIsNil()).
		WithPlan()

	// 搜索：按 quotaType 枚举字符串匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(planquota.QuotaTypeEQ(planquota.QuotaType(k)))
	}

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count plan quotas failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).Order(planquota.ByID(sql.OrderAsc())).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list plan quotas failed: %v", err)
		return nil, err
	}

	items := make([]types.PlanQuota, 0, len(rows))
	for _, r := range rows {
		items = append(items, *toType(r))
	}

	return &types.ListPlanQuotaResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}