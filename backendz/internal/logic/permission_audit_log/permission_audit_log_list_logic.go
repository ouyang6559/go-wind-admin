// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package permission_audit_log

import (
	"context"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/permissionauditlog"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type PermissionAuditLogListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPermissionAuditLogListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PermissionAuditLogListLogic {
	return &PermissionAuditLogListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PermissionAuditLogListLogic) PermissionAuditLogList(req *types.PageRequest) (resp *types.ListPermissionAuditLogResponse, err error) {
	q := l.svcCtx.Ent.PermissionAuditLog.Query()

	// 搜索：按 operator_name / target_name / action 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(permissionauditlog.Or(
			permissionauditlog.OperatorNameContainsFold(k),
			permissionauditlog.TargetNameContainsFold(k),
			permissionauditlog.ActionEQ(permissionauditlog.Action(k)),
		))
	}

	q = q.Order(permissionauditlog.ByID(sql.OrderDesc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count permission audit logs failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list permission audit logs failed: %v", err)
		return nil, err
	}

	items := make([]types.PermissionAuditLog, 0, len(rows))
	for _, r := range rows {
		items = append(items, *toType(r))
	}

	return &types.ListPermissionAuditLogResponse{
		Items: items,
		Total: int64(total),
	}, nil
}