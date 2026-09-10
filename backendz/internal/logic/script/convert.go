package script

import (
	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/script"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// toType 将 ent 实体转换为 types.Script。
func toType(e *gen.Script) *types.Script {
	return &types.Script{
		Id:          int64(e.ID),
		Name:        std.Str(e.Name),
		Language:    stringVal(e.Language),
		HookPoint:   std.Str(e.HookPoint),
		Source:      std.Str(e.Source),
		Priority:    std.Int64(e.Priority),
		Description: std.Str(e.Description),
		Critical:    std.Bool(e.Critical),
		Version:     std.Int64(e.Version),
		IsEnabled:   std.Bool(e.IsEnabled),
		CreatedBy:   std.Int64(e.CreatedBy),
		UpdatedBy:   std.Int64(e.UpdatedBy),
		DeletedBy:   std.Int64(e.DeletedBy),
		CreatedAt:   std.TimeStr(e.CreatedAt),
		UpdatedAt:   std.TimeStr(e.UpdatedAt),
		DeletedAt:   std.TimeStr(e.DeletedAt),
	}
}

// stringVal 将 ent enum/string 指针转为字符串。
func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}

func strPtr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}

// langPtr 将语言字符串转为 ent enum 指针；空串返回 nil。
func langPtr(s string) *script.Language {
	if s == "" {
		return nil
	}
	v := script.Language(s)
	return &v
}
