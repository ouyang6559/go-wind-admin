package permission_group

import (
	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}

// toType 将 ent 实体转换为 types.PermissionGroup。
func toType(e *gen.PermissionGroup) *types.PermissionGroup {
	return &types.PermissionGroup{
		Id:          int64(e.ID),
		Name:        std.Str(e.Name),
		Path:        std.Str(e.Path),
		Module:      std.Str(e.Module),
		SortOrder:   std.Int64(e.SortOrder),
		Status:      stringVal(e.Status),
		Description: std.Str(e.Description),
		ParentId:    std.Int64(e.ParentID),
		CreatedBy:   std.Int64(e.CreatedBy),
		UpdatedBy:   std.Int64(e.UpdatedBy),
		DeletedBy:   std.Int64(e.DeletedBy),
		CreatedAt:   std.TimeStr(e.CreatedAt),
		UpdatedAt:   std.TimeStr(e.UpdatedAt),
		DeletedAt:   std.TimeStr(e.DeletedAt),
	}
}

// buildTree 将扁平分组按 parent_id 组装为树。
func buildTree(items []*types.PermissionGroup) []types.PermissionGroup {
	byID := make(map[int64]*types.PermissionGroup, len(items))
	var roots []types.PermissionGroup
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