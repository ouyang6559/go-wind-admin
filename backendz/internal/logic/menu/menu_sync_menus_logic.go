// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package menu

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/menu"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type MenuSyncMenusLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewMenuSyncMenusLogic(ctx context.Context, svcCtx *svc.ServiceContext) *MenuSyncMenusLogic {
	return &MenuSyncMenusLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// MenuSyncMenus 清空现有菜单并递归插入前端传入的树形菜单。
func (l *MenuSyncMenusLogic) MenuSyncMenus(req *types.SyncMenusRequest) error {
	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(l.ctx); ok {
		operatorID = c.UserID
	}

	// 硬删除既有菜单，随后重建（与 kratos 版 Truncate 语义一致）
	if _, derr := l.svcCtx.Ent.Menu.Delete().Exec(l.ctx); derr != nil {
		logx.WithContext(l.ctx).Errorf("truncate menus failed: %v", derr)
		return xerr.ServerErrorMsg("truncate menus failed")
	}

	if _, cerr := syncMenuTree(l.ctx, l.svcCtx.Ent, req.Items, nil, operatorID); cerr != nil {
		logx.WithContext(l.ctx).Errorf("sync menu tree failed: %v", cerr)
		return xerr.ServerErrorMsg("sync menus failed")
	}

	return nil
}

// syncMenuTree 先插父节点拿 ID，再递归设置子节点 parent_id。
func syncMenuTree(ctx context.Context, client *gen.Client, items []types.Menu, parentID *uint32, operatorID uint32) (int, error) {
	count := 0
	for _, m := range items {
		children := m.Children

		status := menu.StatusOn
		if m.Status != "" {
			status = menu.Status(m.Status)
		}

		create := client.Menu.Create().
			SetStatus(status).
			SetMeta(metaFromType(m.Meta)).
			SetNillableName(nilStr(m.Name)).
			SetNillablePath(nilStr(m.Path)).
			SetNillableComponent(nilStr(m.Component)).
			SetNillableRedirect(nilStr(m.Redirect)).
			SetNillableAlias(nilStr(m.Alias)).
			SetCreatedBy(operatorID).
			SetUpdatedBy(operatorID)
		if m.Type != "" {
			create = create.SetType(menu.Type(m.Type))
		}
		if m.Module != "" {
			create = create.SetModule(menu.Module(m.Module))
		}
		if parentID != nil {
			create = create.SetParentID(*parentID)
		}

		created, cerr := create.Save(ctx)
		if cerr != nil {
			return count, cerr
		}
		count++

		if len(children) > 0 {
			childID := created.ID
			childCount, cerr := syncMenuTree(ctx, client, children, &childID, operatorID)
			if cerr != nil {
				return count, cerr
			}
			count += childCount
		}
	}
	return count, nil
}