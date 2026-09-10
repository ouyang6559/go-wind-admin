// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package policy_evaluation_log

import (
	"context"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/policyevaluationlog"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type PolicyEvaluationLogListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPolicyEvaluationLogListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PolicyEvaluationLogListLogic {
	return &PolicyEvaluationLogListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PolicyEvaluationLogListLogic) PolicyEvaluationLogList(req *types.PageRequest) (resp *types.ListPolicyEvaluationLogResponse, err error) {
	q := l.svcCtx.Ent.PolicyEvaluationLog.Query()

	// 搜索：按请求路径模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(policyevaluationlog.RequestPathContainsFold(k))
	}

	q = q.Order(policyevaluationlog.ByCreatedAt(sql.OrderDesc()), policyevaluationlog.ByID(sql.OrderDesc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count policy evaluation logs failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list policy evaluation logs failed: %v", err)
		return nil, err
	}

	items := make([]types.PolicyEvaluationLog, 0, len(rows))
	for _, r := range rows {
		items = append(items, toType(l.ctx, r))
	}

	return &types.ListPolicyEvaluationLogResponse{
		Items: items,
		Total: int64(total),
	}, nil
}