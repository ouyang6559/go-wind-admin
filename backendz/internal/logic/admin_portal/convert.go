package admin_portal

import (
	"context"

	"entgo.io/ent/dialect/sql"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/menu"
	"go-wind-admin/backendz/internal/ent/gen/permission"
	"go-wind-admin/backendz/internal/ent/gen/permissionmenu"
	"go-wind-admin/backendz/internal/ent/gen/rolepermission"
	"go-wind-admin/backendz/internal/ent/gen/userrole"
	"go-wind-admin/backendz/internal/ent/schema"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// roleIDsOfUser 查询用户所拥有的角色 ID 列表（去重）。
func roleIDsOfUser(ctx context.Context, client *gen.Client, userID uint32) ([]uint32, error) {
	rows, err := client.UserRole.Query().
		Where(userrole.UserIDEQ(userID), userrole.DeletedAtIsNil()).
		Select(userrole.FieldRoleID).
		All(ctx)
	if err != nil {
		return nil, err
	}
	set := make(map[uint32]struct{}, len(rows))
	for _, r := range rows {
		if r.RoleID == nil {
			continue
		}
		set[*r.RoleID] = struct{}{}
	}
	ids := make([]uint32, 0, len(set))
	for id := range set {
		ids = append(ids, id)
	}
	return ids, nil
}

// permissionIDsOfRoles 查询多个角色关联的权限 ID 列表（去重）。
func permissionIDsOfRoles(ctx context.Context, client *gen.Client, roleIDs []uint32) ([]uint32, error) {
	if len(roleIDs) == 0 {
		return nil, nil
	}
	rows, err := client.RolePermission.Query().
		Where(rolepermission.RoleIDIn(roleIDs...), rolepermission.DeletedAtIsNil()).
		Select(rolepermission.FieldPermissionID).
		All(ctx)
	if err != nil {
		return nil, err
	}
	set := make(map[uint32]struct{}, len(rows))
	for _, r := range rows {
		if r.PermissionID == nil {
			continue
		}
		set[*r.PermissionID] = struct{}{}
	}
	ids := make([]uint32, 0, len(set))
	for id := range set {
		ids = append(ids, id)
	}
	return ids, nil
}

// menuIDsOfPermissions 查询多个权限关联的菜单 ID 列表（去重）。
func menuIDsOfPermissions(ctx context.Context, client *gen.Client, permissionIDs []uint32) ([]uint32, error) {
	if len(permissionIDs) == 0 {
		return nil, nil
	}
	rows, err := client.PermissionMenu.Query().
		Where(permissionmenu.PermissionIDIn(permissionIDs...), permissionmenu.DeletedAtIsNil()).
		Select(permissionmenu.FieldMenuID).
		All(ctx)
	if err != nil {
		return nil, err
	}
	set := make(map[uint32]struct{}, len(rows))
	for _, r := range rows {
		if r.MenuID == nil {
			continue
		}
		set[*r.MenuID] = struct{}{}
	}
	ids := make([]uint32, 0, len(set))
	for id := range set {
		ids = append(ids, id)
	}
	return ids, nil
}

// permissionCodesOf 查询多个权限的编码列表（启用且未删除）。
func permissionCodesOf(ctx context.Context, client *gen.Client, permissionIDs []uint32) ([]string, error) {
	if len(permissionIDs) == 0 {
		return nil, nil
	}
	rows, err := client.Permission.Query().
		Where(permission.IDIn(permissionIDs...), permission.DeletedAtIsNil()).
		Select(permission.FieldCode).
		All(ctx)
	if err != nil {
		return nil, err
	}
	codes := make([]string, 0, len(rows))
	for _, r := range rows {
		if c := std.Str(r.Code); c != "" {
			codes = append(codes, c)
		}
	}
	return codes, nil
}

// menuTree 查询指定菜单 ID 的启用菜单（非按钮）并组装为树。
func menuTree(ctx context.Context, client *gen.Client, menuUIDs []uint32) ([]types.Menu, error) {
	if len(menuUIDs) == 0 {
		return nil, nil
	}
	rows, err := client.Menu.Query().
		Where(menu.IDIn(menuUIDs...), menu.DeletedAtIsNil(), menu.StatusEQ(menu.StatusOn), menu.TypeNEQ(menu.TypeButton)).
		Order(menu.ByParentID(sql.OrderAsc()), menu.ByID(sql.OrderAsc())).
		All(ctx)
	if err != nil {
		return nil, err
	}
	flat := make([]*types.Menu, 0, len(rows))
	for _, r := range rows {
		flat = append(flat, menuToType(r))
	}
	return buildMenuTree(flat), nil
}

func menuToType(e *gen.Menu) *types.Menu {
	return &types.Menu{
		Id:        int64(e.ID),
		Status:    stringVal(e.Status),
		Type:      stringVal(e.Type),
		Path:      std.Str(e.Path),
		Redirect:  std.Str(e.Redirect),
		Alias:     std.Str(e.Alias),
		Name:      std.Str(e.Name),
		Component: std.Str(e.Component),
		Meta:      menuMetaToType(e.Meta),
		Module:    stringVal(e.Module),
		ParentId:  std.Int64(e.ParentID),
	}
}

func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}

func menuMetaToType(m *schema.MenuMeta) types.MenuMeta {
	if m == nil {
		return types.MenuMeta{}
	}
	return types.MenuMeta{
		ActiveIcon:               m.ActiveIcon,
		ActivePath:               m.ActivePath,
		AffixTab:                 boolVal(m.AffixTab),
		AffixTabOrder:            int32toI64(m.AffixTabOrder),
		Authority:                m.Authority,
		Badge:                    m.Badge,
		BadgeType:                m.BadgeType,
		BadgeVariants:            m.BadgeVariants,
		HideChildrenInMenu:       boolVal(m.HideChildrenInMenu),
		HideInBreadcrumb:         boolVal(m.HideInBreadcrumb),
		HideInMenu:               boolVal(m.HideInMenu),
		HideInTab:                boolVal(m.HideInTab),
		Icon:                     m.Icon,
		IframeSrc:                m.IframeSrc,
		IgnoreAccess:             boolVal(m.IgnoreAccess),
		KeepAlive:                boolVal(m.KeepAlive),
		Link:                     m.Link,
		Loaded:                   boolVal(m.Loaded),
		MaxNumOfOpenTab:          int32toI64(m.MaxNumOfOpenTab),
		MenuVisibleWithForbidden: boolVal(m.MenuVisibleWithForbidden),
		OpenInNewWindow:          boolVal(m.OpenInNewWindow),
		Order:                    int32toI64(m.Order),
		Title:                    m.Title,
	}
}

func boolVal(b *bool) bool {
	if b == nil {
		return false
	}
	return *b
}

func int32toI64(v *int32) int64 {
	if v == nil {
		return 0
	}
	return int64(*v)
}

// buildMenuTree 将扁平菜单按 parent_id 组装为树。
func buildMenuTree(items []*types.Menu) []types.Menu {
	byID := make(map[int64]*types.Menu, len(items))
	var roots []types.Menu
	for i := range items {
		m := items[i]
		m.Children = nil
		byID[m.Id] = m
	}
	for i := range items {
		m := items[i]
		if m.ParentId > 0 {
			if p, ok := byID[m.ParentId]; ok {
				p.Children = append(p.Children, *m)
				continue
			}
		}
		roots = append(roots, *m)
	}
	return roots
}

// toRouteItems 将菜单树转换为前端路由项（过滤状态非 ON / 按钮类型）。
func toRouteItems(menus []types.Menu) []types.MenuRouteItem {
	var out []types.MenuRouteItem
	for _, v := range menus {
		if v.Status != "ON" || v.Type == "BUTTON" {
			continue
		}
		item := types.MenuRouteItem{
			Path:      v.Path,
			Component: v.Component,
			Name:      v.Name,
			Redirect:  v.Redirect,
			Alias:     v.Alias,
			Meta:      v.Meta,
		}
		if len(v.Children) > 0 {
			item.Children = toRouteItems(v.Children)
		}
		out = append(out, item)
	}
	return out
}