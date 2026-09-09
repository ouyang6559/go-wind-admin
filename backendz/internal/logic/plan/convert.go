package plan

import (
	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/plan"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// toType 将 ent 实体转换为 types.Plan。
func toType(e *gen.Plan) *types.Plan {
	return &types.Plan{
		Id:                int64(e.ID),
		Name:              std.Str(e.Name),
		Version:           enumStr(e.Version),
		ExpiryPolicy:      enumStr(e.ExpiryPolicy),
		DataRetentionDays: std.Int64(e.DataRetentionDays),
		Description:       std.Str(e.Description),
		Remark:            std.Str(e.Remark),
		CreatedBy:         std.Int64(e.CreatedBy),
		UpdatedBy:         std.Int64(e.UpdatedBy),
		DeletedBy:         std.Int64(e.DeletedBy),
		CreatedAt:         std.TimeStr(e.CreatedAt),
		UpdatedAt:         std.TimeStr(e.UpdatedAt),
		DeletedAt:         std.TimeStr(e.DeletedAt),
	}
}

// enumStr 解引用 *plan.Version / *plan.ExpiryPolicy 等枚举指针为字符串。
func enumStr[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}

// toVersion 将字符串安全转换为 *plan.Version；空串返回 nil（避免校验失败）。
func toVersion(s string) *plan.Version {
	if s == "" {
		return nil
	}
	v := plan.Version(s)
	return &v
}

// toExpiryPolicy 将字符串安全转换为 *plan.ExpiryPolicy；空串返回 nil。
func toExpiryPolicy(s string) *plan.ExpiryPolicy {
	if s == "" {
		return nil
	}
	v := plan.ExpiryPolicy(s)
	return &v
}