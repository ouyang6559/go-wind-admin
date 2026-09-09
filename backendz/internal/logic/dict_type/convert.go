package dict_type

import (
	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// toType 将 ent 实体转换为 types.DictType。
func toType(e *gen.DictType) *types.DictType {
	return &types.DictType{
		Id:        int64(e.ID),
		TypeCode:  std.Str(e.TypeCode),
		TypeName:  std.Str(e.TypeName),
		IsEnabled: std.Bool(e.IsEnabled),
		SortOrder: std.Int64(e.SortOrder),
		TenantId:  std.Int64(e.TenantID),
		CreatedBy: std.Int64(e.CreatedBy),
		UpdatedBy: std.Int64(e.UpdatedBy),
		DeletedBy: std.Int64(e.DeletedBy),
		CreatedAt: std.TimeStr(e.CreatedAt),
		UpdatedAt: std.TimeStr(e.UpdatedAt),
		DeletedAt: std.TimeStr(e.DeletedAt),
	}
}