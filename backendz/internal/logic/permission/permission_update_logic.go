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
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type PermissionUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewPermissionUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *PermissionUpdateLogic {
	return &PermissionUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *PermissionUpdateLogic) PermissionUpdate(req *types.UpdatePermissionRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.Permission.Query().
		Where(permission.IDEQ(uint32(req.Id)), permission.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("permission not found")
		}
		logx.WithContext(l.ctx).Errorf("get permission for update failed: %v", err)
		return xerr.ServerErrorMsg("get permission for update failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	tx, terr := l.svcCtx.Ent.BeginTx(l.ctx, nil)
	if terr != nil {
		logx.WithContext(l.ctx).Errorf("begin tx failed: %v", terr)
		return xerr.ServerErrorMsg("begin tx failed")
	}

	upd := tx.Permission.UpdateOneID(existing.ID).SetUpdatedBy(operatorID).SetUpdatedAt(time.Now())
	if d.Name != "" {
		upd.SetName(d.Name)
	}
	if d.Code != "" {
		upd.SetCode(d.Code)
	}
	if d.Description != "" {
		upd.SetDescription(d.Description)
	}
	if d.Status != "" {
		upd.SetStatus(permission.Status(d.Status))
	}
	if d.GroupId > 0 {
		upd.SetGroupID(uint32(d.GroupId))
	}

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		_ = tx.Rollback()
		logx.WithContext(l.ctx).Errorf("update permission failed: %v", uerr)
		return xerr.ServerErrorMsg("update permission failed")
	}

	// 重绑菜单与 API 关联（仅在显式提供时）
	if d.MenuIds != nil {
		if _, derr := tx.PermissionMenu.Delete().
			Where(permissionmenu.PermissionIDEQ(existing.ID)).
			Exec(l.ctx); derr != nil {
			_ = tx.Rollback()
			logx.WithContext(l.ctx).Errorf("clear permission menus failed: %v", derr)
			return xerr.ServerErrorMsg("clear permission menus failed")
		}
		for _, mid := range d.MenuIds {
			if mid <= 0 {
				continue
			}
			if _, perr := tx.PermissionMenu.Create().
					SetPermissionID(existing.ID).
					SetMenuID(uint32(mid)).
					SetCreatedAt(time.Now()).
					SetUpdatedAt(time.Now()).
					Save(l.ctx); perr != nil {
				_ = tx.Rollback()
				logx.WithContext(l.ctx).Errorf("link permission menu failed: err=%v", perr)
				return xerr.ServerErrorMsg("link permission menu failed")
			}
		}
	}
	if d.ApiIds != nil {
		if _, derr := tx.PermissionApi.Delete().
			Where(permissionapi.PermissionIDEQ(existing.ID)).
			Exec(l.ctx); derr != nil {
			_ = tx.Rollback()
			logx.WithContext(l.ctx).Errorf("clear permission apis failed: %v", derr)
			return xerr.ServerErrorMsg("clear permission apis failed")
		}
		for _, aid := range d.ApiIds {
			if aid <= 0 {
				continue
			}
			if _, perr := tx.PermissionApi.Create().
					SetPermissionID(existing.ID).
					SetAPIID(uint32(aid)).
					SetCreatedAt(time.Now()).
					SetUpdatedAt(time.Now()).
					Save(l.ctx); perr != nil {
				_ = tx.Rollback()
				logx.WithContext(l.ctx).Errorf("link permission api failed: err=%v", perr)
				return xerr.ServerErrorMsg("link permission api failed")
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