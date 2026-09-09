package api

import (
	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// toType 将 ent 实体转换为 types.Api。
func toType(e *gen.Api) *types.Api {
	return &types.Api{
		Id:                int64(e.ID),
		Operation:         std.Str(e.Operation),
		Path:              std.Str(e.Path),
		Method:            std.Str(e.Method),
		Module:            std.Str(e.Module),
		ModuleDescription: std.Str(e.ModuleDescription),
		BusinessModule:    stringVal(e.BusinessModule),
		Description:       std.Str(e.Description),
		Scope:             stringVal(e.Scope),
		Status:            stringVal(e.Status),
		CreatedBy:         std.Int64(e.CreatedBy),
		UpdatedBy:         std.Int64(e.UpdatedBy),
		DeletedBy:         std.Int64(e.DeletedBy),
		CreatedAt:         std.TimeStr(e.CreatedAt),
		UpdatedAt:         std.TimeStr(e.UpdatedAt),
		DeletedAt:         std.TimeStr(e.DeletedAt),
	}
}

func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}