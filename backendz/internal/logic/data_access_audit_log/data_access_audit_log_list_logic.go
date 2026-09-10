// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package data_access_audit_log

import (
	"context"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/dataaccessauditlog"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type DataAccessAuditLogListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewDataAccessAuditLogListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *DataAccessAuditLogListLogic {
	return &DataAccessAuditLogListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *DataAccessAuditLogListLogic) DataAccessAuditLogList(req *types.PageRequest) (resp *types.ListDataAccessAuditLogResponse, err error) {
	q := l.svcCtx.Ent.DataAccessAuditLog.Query()

	// 搜索：按账号名、数据源或表名模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			dataaccessauditlog.Or(
				dataaccessauditlog.UsernameContainsFold(k),
				dataaccessauditlog.DataSourceContainsFold(k),
				dataaccessauditlog.TableNameContainsFold(k),
			),
		)
	}

	q = q.Order(dataaccessauditlog.ByCreatedAt(sql.OrderDesc()), dataaccessauditlog.ByID(sql.OrderDesc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count data access audit logs failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list data access audit logs failed: %v", err)
		return nil, err
	}

	items := make([]types.DataAccessAuditLog, 0, len(rows))
	for _, r := range rows {
		items = append(items, toType(l.ctx, l.svcCtx.Ent, r))
	}

	return &types.ListDataAccessAuditLogResponse{
		Items: items,
		Total: int64(total),
	}, nil
}