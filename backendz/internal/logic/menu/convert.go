package menu

import (
	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/schema"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}

// metaToType 将 schema.MenuMeta 转换为 types.MenuMeta。
func metaToType(m *schema.MenuMeta) types.MenuMeta {
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

// metaFromType 将 types.MenuMeta 转换为 schema.MenuMeta（用于写入）。
func metaFromType(m types.MenuMeta) *schema.MenuMeta {
	return &schema.MenuMeta{
		ActiveIcon:                m.ActiveIcon,
		ActivePath:                m.ActivePath,
		AffixTab:                  boolPtr(m.AffixTab),
		AffixTabOrder:             i64toInt32(m.AffixTabOrder),
		Authority:                 m.Authority,
		Badge:                     m.Badge,
		BadgeType:                 m.BadgeType,
		BadgeVariants:             m.BadgeVariants,
		HideChildrenInMenu:        boolPtr(m.HideChildrenInMenu),
		HideInBreadcrumb:          boolPtr(m.HideInBreadcrumb),
		HideInMenu:                boolPtr(m.HideInMenu),
		HideInTab:                 boolPtr(m.HideInTab),
		Icon:                      m.Icon,
		IframeSrc:                 m.IframeSrc,
		IgnoreAccess:              boolPtr(m.IgnoreAccess),
		KeepAlive:                 boolPtr(m.KeepAlive),
		Link:                      m.Link,
		Loaded:                    boolPtr(m.Loaded),
		MaxNumOfOpenTab:           i64toInt32(m.MaxNumOfOpenTab),
		MenuVisibleWithForbidden:  boolPtr(m.MenuVisibleWithForbidden),
		OpenInNewWindow:           boolPtr(m.OpenInNewWindow),
		Order:                     i64toInt32(m.Order),
		Title:                     m.Title,
	}
}

func boolVal(b *bool) bool {
	if b == nil {
		return false
	}
	return *b
}

func boolPtr(b bool) *bool {
	return &b
}

func int32toI64(v *int32) int64 {
	if v == nil {
		return 0
	}
	return int64(*v)
}

func i64toInt32(v int64) *int32 {
	if v == 0 {
		return nil
	}
	i := int32(v)
	return &i
}

// toType 将 ent 实体转换为 types.Menu。
func toType(e *gen.Menu) *types.Menu {
	return &types.Menu{
		Id:        int64(e.ID),
		Status:    stringVal(e.Status),
		Type:      stringVal(e.Type),
		Path:      std.Str(e.Path),
		Redirect:  std.Str(e.Redirect),
		Alias:     std.Str(e.Alias),
		Name:      std.Str(e.Name),
		Component: std.Str(e.Component),
		Meta:      metaToType(e.Meta),
		Module:    stringVal(e.Module),
		ParentId:  std.Int64(e.ParentID),
		CreatedBy: std.Int64(e.CreatedBy),
		UpdatedBy: std.Int64(e.UpdatedBy),
		DeletedBy: std.Int64(e.DeletedBy),
		CreatedAt: std.TimeStr(e.CreatedAt),
		UpdatedAt: std.TimeStr(e.UpdatedAt),
		DeletedAt: std.TimeStr(e.DeletedAt),
	}
}

// buildTree 将扁平菜单列表按 parent_id 组装为树。
func buildTree(items []*types.Menu) []types.Menu {
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