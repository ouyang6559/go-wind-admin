// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package login_policy

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/loginpolicy"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type LoginPolicyListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewLoginPolicyListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *LoginPolicyListLogic {
	return &LoginPolicyListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *LoginPolicyListLogic) LoginPolicyList(req *types.PageRequest) (resp *types.ListLoginPolicyResponse, err error) {
	q := l.svcCtx.Ent.LoginPolicy.Query().Where(loginpolicy.DeletedAtIsNil())

	// 搜索：按限制值或原因模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			loginpolicy.Or(
				loginpolicy.ValueContainsFold(k),
				loginpolicy.ReasonContainsFold(k),
			),
		)
	}

	// 排序
	q = q.Order(loginpolicy.ByID(sql.OrderAsc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count login policies failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list login policies failed: %v", err)
		return nil, err
	}

	items := make([]types.LoginPolicy, 0, len(rows))
	for _, e := range rows {
		items = append(items, *toType(l.ctx, l.svcCtx.Ent, e))
	}

	return &types.ListLoginPolicyResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}
