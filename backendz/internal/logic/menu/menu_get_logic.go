// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package menu

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/menu"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type MenuGetLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewMenuGetLogic(ctx context.Context, svcCtx *svc.ServiceContext) *MenuGetLogic {
	return &MenuGetLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *MenuGetLogic) MenuGet(req *types.MenuGetReq) (resp *types.Menu, err error) {
	e, err := l.svcCtx.Ent.Menu.Query().
		Where(menu.IDEQ(uint32(req.Id)), menu.DeletedAtIsNil()).
		Only(l.ctx)
	if err != nil {
		if gen.IsNotFound(err) {
			return nil, xerr.NotFoundMsg("menu not found")
		}
		logx.WithContext(l.ctx).Errorf("get menu failed: %v", err)
		return nil, err
	}

	return toType(e), nil
}