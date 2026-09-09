// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package permission_group

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/permission"
	"go-wind-admin/backendz/internal/ent/gen/permissiongroup"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PermissionGroupDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPermissionGroupDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PermissionGroupDeleteLogic {
	return &PermissionGroupDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PermissionGroupDeleteLogic) PermissionGroupDelete(req *types.PermissionGroupDeleteReq) error {
	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	existing, err := tx.PermissionGroup.Query().
		Where(permissiongroup.IDEQ(uint32(req.Id)), permissiongroup.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		_ = tx.Rollback()
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("permission group not found")
		}
		logx.WithContext(l.ctx).Errorf("get permission group for delete failed: %v", err)
		return xerr.ServerErrorMsg("get permission group for delete failed")
	}

	// 先软删组内权限点，再软删分组本身
	if _, derr := tx.Permission.Update().
		Where(permission.GroupIDEQ(existing.ID), permission.DeletedAtIsNil()).
		SetDeletedAt(time.Now()).
		Save(l.ctx); derr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("soft delete group permissions failed: %v", derr)
		return xerr.ServerErrorMsg("soft delete group permissions failed")
	}

	if _, uerr := tx.PermissionGroup.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		Save(l.ctx); uerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("soft delete permission group failed: %v", uerr)
		return xerr.ServerErrorMsg("soft delete permission group failed")
	}

	if eerr := tx.Commit(); eerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("commit tx failed: %v", eerr)
		return xerr.ServerErrorMsg("commit tx failed")
	}

	return nil
}