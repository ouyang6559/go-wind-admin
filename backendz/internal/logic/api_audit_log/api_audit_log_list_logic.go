// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package api_audit_log

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/apiauditlog"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type ApiAuditLogListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewApiAuditLogListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *ApiAuditLogListLogic {
	return &ApiAuditLogListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *ApiAuditLogListLogic) ApiAuditLogList(req *types.PageRequest) (resp *types.ListApiAuditLogResponse, err error) {
	q := l.svcCtx.Ent.ApiAuditLog.Query()

	// 搜索：按 path / username / http_method 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(apiauditlog.Or(
			apiauditlog.PathContainsFold(k),
			apiauditlog.UsernameContainsFold(k),
			apiauditlog.HTTPMethodContainsFold(k),
		))
	}

	q = q.Order(apiauditlog.ByID(sql.OrderDesc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count api audit logs failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list api audit logs failed: %v", err)
		return nil, err
	}

	items := make([]types.ApiAuditLog, 0, len(rows))
	for _, r := range rows {
		item := toType(l.ctx, l.svcCtx.Ent, r)
		enrich(l.ctx, l.svcCtx.Ent, item)
		items = append(items, *item)
	}

	return &types.ListApiAuditLogResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}