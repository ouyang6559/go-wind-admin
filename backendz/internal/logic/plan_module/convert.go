package plan_module

import (
	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// toType 将 ent 实体转换为 types.PlanModule（需预先加载 plan 边以取 PlanId）。
func toType(e *gen.PlanModule) *types.PlanModule {
	out := &types.PlanModule{
		Id:        int64(e.ID),
		CreatedBy: std.Int64(e.CreatedBy),
		UpdatedBy: std.Int64(e.UpdatedBy),
		DeletedBy: std.Int64(e.DeletedBy),
		CreatedAt: std.TimeStr(e.CreatedAt),
		UpdatedAt: std.TimeStr(e.UpdatedAt),
		DeletedAt: std.TimeStr(e.DeletedAt),
	}
	if e.Module != nil {
		out.Module = string(*e.Module)
	}
	if e.Edges.Plan != nil {
		out.PlanId = int64(e.Edges.Plan.ID)
	}
	return out
}