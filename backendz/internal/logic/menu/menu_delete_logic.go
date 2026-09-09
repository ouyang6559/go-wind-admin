// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package menu

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/menu"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type MenuDeleteLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewMenuDeleteLogic(ctx context.Context, svcCtx *svc.ServiceContext) *MenuDeleteLogic {
	return &MenuDeleteLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *MenuDeleteLogic) MenuDelete(req *types.MenuDeleteReq) error {
	existing, err := l.svcCtx.Ent.Menu.Query().
		Where(menu.IDEQ(uint32(req.Id)), menu.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return xerr.NotFoundMsg("menu not found")
		}
		logx.WithContext(l.ctx).Errorf("get menu for delete failed: %v", err)
		return xerr.ServerErrorMsg("get menu for delete failed")
	}

	if err := l.svcCtx.Ent.Menu.UpdateOneID(existing.ID).
		SetDeletedAt(time.Now()).
		Exec(l.ctx); err != nil {
		logx.WithContext(l.ctx).Errorf("soft delete menu failed: %v", err)
		return xerr.ServerErrorMsg("soft delete menu failed")
	}

	return nil
}