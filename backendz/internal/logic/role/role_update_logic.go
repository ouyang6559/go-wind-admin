// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package role

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/role"
	"go-wind-admin/backendz/internal/ent/gen/rolepermission"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type RoleUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewRoleUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *RoleUpdateLogic {
	return &RoleUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *RoleUpdateLogic) RoleUpdate(req *types.UpdateRoleRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.Role.Query().
		Where(role.IDEQ(uint32(req.Id)), role.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("role not found")
		}
		logx.WithContext(l.ctx).Errorf("get role for update failed: %v", err)
		return xerr.ServerErrorMsg("get role for update failed")
	}

	// 保护角色：禁止修改关键字段
	protected := existing.IsProtected != nil && *existing.IsProtected

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	upd := tx.Role.UpdateOneID(existing.ID).SetUpdatedBy(operatorID).SetUpdatedAt(time.Now())
	if d.Name != "" {
		upd.SetName(d.Name)
	}
	if d.Code != "" && !protected {
		upd.SetCode(d.Code)
	}
	if d.Description != "" {
		upd.SetDescription(d.Description)
	}
	if d.SortOrder > 0 {
		upd.SetSortOrder(uint32(d.SortOrder))
	}
	if !protected {
		if d.Status != "" {
			upd.SetStatus(role.Status(d.Status))
		}
		if d.Type != "" {
			upd.SetType(role.Type(d.Type))
		}
	}

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("update role failed: %v", uerr)
		return xerr.ServerErrorMsg("update role failed")
	}

	// 重绑权限（仅在显式提供 permissions 时）
	if d.Permissions != nil {
		if _, derr := tx.RolePermission.Delete().
			Where(rolepermission.RoleIDEQ(existing.ID)).
			Exec(l.ctx); derr != nil {
			_ = tx.Rollback()
			logx.WithContext(l.ctx).Errorf("clear role permissions failed: %v", derr)
			return xerr.ServerErrorMsg("clear role permissions failed")
		}
		for _, pid := range d.Permissions {
			if pid <= 0 {
				continue
			}
			if _, perr := tx.RolePermission.Create().
					SetRoleID(existing.ID).
					SetPermissionID(uint32(pid)).
					SetCreatedAt(time.Now()).
					SetUpdatedAt(time.Now()).
					Save(l.ctx); perr != nil {
				_ = tx.Rollback()
				logx.WithContext(l.ctx).Errorf("link role permission failed: roleID=%d permissionID=%d err=%v", existing.ID, pid, perr)
				return xerr.ServerErrorMsg("link role permission failed")
			}
		}
	}

	if eerr := tx.Commit(); eerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", eerr)
		return xerr.ServerErrorMsg("commit tx failed")
	}

	return nil
}