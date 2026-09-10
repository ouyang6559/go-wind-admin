package script_log

import (
	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// toType 将 ent 实体转换为 types.ScriptLog。
func toType(e *gen.ScriptLog) *types.ScriptLog {
	return &types.ScriptLog{
		Id:          int64(e.ID),
		ScriptId:    std.Int64(e.ScriptID),
		ScriptName:  std.Str(e.ScriptName),
		Language:    std.Str(e.Language),
		TriggerType: std.Str(e.TriggerType),
		HookPoint:   std.Str(e.HookPoint),
		Version:     std.Int64(e.Version),
		Success:     std.Bool(e.Success),
		DurationMs:  std.Int64(e.DurationMs),
		Error:       std.Str(e.Error),
		CreatedAt:   std.TimeStr(e.CreatedAt),
	}
}
