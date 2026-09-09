// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package login_audit_log

import (
	"context"
	"strconv"
	"strings"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen/loginauditlog"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"

	"github.com/zeromicro/go-zero/core/logx"
)

type LoginAuditLogListLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewLoginAuditLogListLogic(ctx context.Context, svcCtx *svc.ServiceContext) *LoginAuditLogListLogic {
	return &LoginAuditLogListLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *LoginAuditLogListLogic) LoginAuditLogList(req *types.PageRequest) (resp *types.ListLoginAuditLogResponse, err error) {
	q := l.svcCtx.Ent.LoginAuditLog.Query()

	// 搜索：按账号名或 IP 模糊匹配
	if k := strings.TrimSpace(req.Query); k != "" {
		q = q.Where(
			loginauditlog.Or(
				loginauditlog.UsernameContainsFold(k),
				loginauditlog.IPAddressContainsFold(k),
			),
		)
	}

	// 排序：创建时间倒序
	q = q.Order(loginauditlog.ByCreatedAt(sql.OrderDesc()), loginauditlog.ByID(sql.OrderDesc()))

	total, err := q.Count(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("count login audit logs failed: %v", err)
		return nil, err
	}

	offset, limit := std.Paginate(req.Page, req.PageSize)
	rows, err := q.Offset(offset).Limit(limit).All(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("list login audit logs failed: %v", err)
		return nil, err
	}

	items := make([]types.LoginAuditLog, 0, len(rows))
	for _, r := range rows {
		items = append(items, toType(l.ctx, l.svcCtx.Ent, r))
	}

	return &types.ListLoginAuditLogResponse{
		Items: items,
		Total: strconv.FormatInt(int64(total), 10),
	}, nil
}