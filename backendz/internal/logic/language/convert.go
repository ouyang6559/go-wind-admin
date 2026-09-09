package language

import (
	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// toType 将 ent 实体转换为 types.Language。
func toType(e *gen.Language) *types.Language {
	return &types.Language{
		Id:           int64(e.ID),
		LanguageCode: std.Str(e.LanguageCode),
		LanguageName: std.Str(e.LanguageName),
		NativeName:   std.Str(e.NativeName),
		IsDefault:    std.Bool(e.IsDefault),
		IsEnabled:    std.Bool(e.IsEnabled),
		SortOrder:    std.Int64(e.SortOrder),
		CreatedBy:    std.Int64(e.CreatedBy),
		UpdatedBy:    std.Int64(e.UpdatedBy),
		DeletedBy:    std.Int64(e.DeletedBy),
		CreatedAt:    std.TimeStr(e.CreatedAt),
		UpdatedAt:    std.TimeStr(e.UpdatedAt),
		DeletedAt:    std.TimeStr(e.DeletedAt),
	}
}