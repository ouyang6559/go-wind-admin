// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package menu

import (
	"context"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/menu"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type MenuCreateLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewMenuCreateLogic(ctx context.Context, svcCtx *svc.ServiceContext) *MenuCreateLogic {
	return &MenuCreateLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *MenuCreateLogic) MenuCreate(req *types.CreateMenuRequest) error {
	d := req.Data
	if strings.TrimSpace(d.Name) == "" {
		return xerr.BadRequestMsg("menu name required")
	}

	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	status := menu.StatusOn
	if d.Status != "" {
		status = menu.Status(d.Status)
	}

	create := l.svcCtx.Ent.Menu.Create().
		SetStatus(status).
		SetMeta(metaFromType(d.Meta)).
		SetNillableName(nilStr(d.Name)).
		SetNillablePath(nilStr(d.Path)).
		SetNillableComponent(nilStr(d.Component)).
		SetNillableRedirect(nilStr(d.Redirect)).
		SetNillableAlias(nilStr(d.Alias)).
		SetNillableCreatedBy(&operatorID).
		SetCreatedAt(time.Now()).
		SetUpdatedAt(time.Now())
	if d.Type != "" {
		create = create.SetType(menu.Type(d.Type))
	}
	if d.ParentId > 0 {
		create = create.SetParentID(uint32(d.ParentId))
	}
	if d.Module != "" {
		create = create.SetModule(menu.Module(d.Module))
	}

	if _, cerr := create.Save(l.ctx); cerr != nil {
		logx.WithContext(l.ctx).Errorf("create menu failed: %v", cerr)
		return xerr.ServerErrorMsg("create menu failed")
	}

	return nil
}

func nilStr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}