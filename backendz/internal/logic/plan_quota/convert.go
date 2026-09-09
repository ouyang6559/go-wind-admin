package plan_quota

import (
	"strconv"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/planquota"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// toType 将 ent 实体转换为 types.PlanQuota（需预先加载 plan 边以取 PlanId）。
func toType(e *gen.PlanQuota) *types.PlanQuota {
	out := &types.PlanQuota{
		Id:        int64(e.ID),
		CreatedBy: std.Int64(e.CreatedBy),
		UpdatedBy: std.Int64(e.UpdatedBy),
		DeletedBy: std.Int64(e.DeletedBy),
		CreatedAt: std.TimeStr(e.CreatedAt),
		UpdatedAt: std.TimeStr(e.UpdatedAt),
		DeletedAt: std.TimeStr(e.DeletedAt),
	}
	if e.QuotaType != nil {
		out.QuotaType = string(*e.QuotaType)
	}
	if e.QuotaValue != nil {
		out.QuotaValue = strconv.FormatUint(*e.QuotaValue, 10)
	}
	if e.Edges.Plan != nil {
		out.PlanId = int64(e.Edges.Plan.ID)
	}
	return out
}

// quotaTypePtr 将字符串安全转换为 *planquota.QuotaType；空串返回 nil。
func quotaTypePtr(s string) *planquota.QuotaType {
	if s == "" {
		return nil
	}
	v := planquota.QuotaType(s)
	return &v
}