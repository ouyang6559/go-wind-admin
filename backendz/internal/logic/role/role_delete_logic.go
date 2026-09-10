// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package role

import (
	"context"
	"strconv"
	"time"

	"go-wind-admin/backendz/internal/auditlog"
	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/operationauditlog"
	"go-wind-admin/backendz/internal/ent/gen/role"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type RoleDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewRoleDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *RoleDeleteLogic {
	return &RoleDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *RoleDeleteLogic) RoleDelete(req *types.RoleDeleteReq) error {
	existing, err := l.svcCtx.Ent.Role.Query().
		Where(role.IDEQ(uint32(req.Id)), role.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("role not found")
		}
		logx.WithContext(l.ctx).Errorf("get role for delete failed: %v", err)
		return xerr.ServerErrorMsg("get role for delete failed")
	}

	if existing.IsProtected != nil && *existing.IsProtected {
		return xerr.ForbiddenMsg("protected role cannot be deleted")
	}

	if err := l.svcCtx.Ent.Role.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete role failed: %v", err)
		return xerr.ServerErrorMsg("soft delete role failed")
	}

	// 操作审计：角色删除成功（best-effort，失败仅日志不阻断）。
	a := operationAuditContext(l.ctx)
	a.ResourceType = "role"
	a.ResourceID = strconv.FormatUint(uint64(existing.ID), 10)
	a.Action = operationauditlog.ActionDelete
	a.Success = true
	if existing.Name != nil {
		a.BeforeData = auditJSON(map[string]string{"name": *existing.Name})
	}
	auditlog.WriteOperation(l.ctx, l.svcCtx, a)

	return nil
}
