// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package permission

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/permission"
	"go-wind-admin/backendz/internal/ent/gen/permissionapi"
	"go-wind-admin/backendz/internal/ent/gen/permissionmenu"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PermissionDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPermissionDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PermissionDeleteLogic {
	return &PermissionDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PermissionDeleteLogic) PermissionDelete(req *types.PermissionDeleteReq) error {
	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	existing, err := tx.Permission.Query().
		Where(permission.IDEQ(uint32(req.Id)), permission.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		_ = tx.Rollback()
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("permission not found")
		}
		logx.WithContext(l.ctx).Errorf("get permission for delete failed: %v", err)
		return xerr.ServerErrorMsg("get permission for delete failed")
	}

	// 清理菜单/API 关联
	if _, derr := tx.PermissionMenu.Delete().
		Where(permissionmenu.PermissionIDEQ(existing.ID)).
		Exec(l.ctx); derr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("clear permission menus failed: %v", derr)
		return xerr.ServerErrorMsg("clear permission menus failed")
	}
	if _, derr := tx.PermissionApi.Delete().
		Where(permissionapi.PermissionIDEQ(existing.ID)).
		Exec(l.ctx); derr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("clear permission apis failed: %v", derr)
		return xerr.ServerErrorMsg("clear permission apis failed")
	}

	if _, uerr := tx.Permission.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		Save(l.ctx); uerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("soft delete permission failed: %v", uerr)
		return xerr.ServerErrorMsg("soft delete permission failed")
	}

	if eerr := tx.Commit(); eerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", eerr)
		return xerr.ServerErrorMsg("commit tx failed")
	}

	return nil
}