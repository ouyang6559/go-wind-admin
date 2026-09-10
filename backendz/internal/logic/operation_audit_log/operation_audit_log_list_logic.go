// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package operation_audit_log

import (
	"context"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/operationauditlog"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type OperationAuditLogListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewOperationAuditLogListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *OperationAuditLogListLogic {
	return &OperationAuditLogListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *OperationAuditLogListLogic) OperationAuditLogList(req *types.PageRequest) (resp *types.ListOperationAuditLogResponse, err error) {
	q := l.svcCtx.Ent.OperationAuditLog.Query()

	// 搜索：按账号名、资源类型或资源 ID 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			operationauditlog.Or(
				operationauditlog.UsernameContainsFold(k),
				operationauditlog.ResourceTypeContainsFold(k),
				operationauditlog.ResourceIDContainsFold(k),
			),
		)
	}

	q = q.Order(operationauditlog.ByCreatedAt(sql.OrderDesc()), operationauditlog.ByID(sql.OrderDesc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count operation audit logs failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list operation audit logs failed: %v", err)
		return nil, err
	}

	items := make([]types.OperationAuditLog, 0, len(rows))
	for _, r := range rows {
		items = append(items, toType(l.ctx, l.svcCtx.Ent, r))
	}

	return &types.ListOperationAuditLogResponse{
		Items: items,
		Total: int64(total),
	}, nil
}