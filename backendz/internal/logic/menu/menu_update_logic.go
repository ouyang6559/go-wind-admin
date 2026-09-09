// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package menu

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/menu"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type MenuUpdateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewMenuUpdateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *MenuUpdateLogic {
	return &MenuUpdateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *MenuUpdateLogic) MenuUpdate(req *types.UpdateMenuRequest) error {
	d := req.Data
	existing, err := l.svcCtx.Ent.Menu.Query().
		Where(menu.IDEQ(uint32(req.Id)), menu.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("menu not found")
		}
		logx.WithContext(l.ctx).Errorf("get menu for update failed: %v", err)
		return xerr.ServerErrorMsg("get menu for update failed")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	upd := l.svcCtx.Ent.Menu.UpdateOneID(existing.ID).SetUpdatedBy(operatorID).SetUpdatedAt(time.Now())
	if d.Name != "" {
		upd.SetName(d.Name)
	}
	if d.Path != "" {
		upd.SetPath(d.Path)
	}
	if d.Component != "" {
		upd.SetComponent(d.Component)
	}
	if d.Redirect != "" {
		upd.SetRedirect(d.Redirect)
	}
	if d.Alias != "" {
		upd.SetAlias(d.Alias)
	}
	if d.Status != "" {
		upd.SetStatus(menu.Status(d.Status))
	}
	if d.Type != "" {
		upd.SetType(menu.Type(d.Type))
	}
	if d.Module != "" {
		upd.SetModule(menu.Module(d.Module))
	}
	if d.ParentId > 0 {
		upd.SetParentID(uint32(d.ParentId))
	}
	// meta 结构特殊：请求体总是传入完整对象，故整体覆写。
	upd.SetMeta(metaFromType(d.Meta))

	if _, uerr := upd.Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("update menu failed: %v", uerr)
		return xerr.ServerErrorMsg("update menu failed")
	}

	return nil
}