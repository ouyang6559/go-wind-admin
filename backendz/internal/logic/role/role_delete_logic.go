// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package role

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
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

	return nil
}