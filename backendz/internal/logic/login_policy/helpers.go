package login_policy

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/tenant"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// stringVal 解引用自定义 string 枚举到字符串；nil 返回空串。
func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}

// strPtr 字符串转指针，空串返回 nil。
func strPtr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
}

// uiPtr int64 转 *uint32，<=0 返回 nil。
func uiPtr(v int64) *uint32 {
	if v <= 0 {
		return nil
	}
	u := uint32(v)
	return &u
}

// tenantNameOf 查询租户名；找不到或 err 时返回空串（不阻断）。
func tenantNameOf(ctx context.Context, client *gen.Client, tid *uint32) string {
	if tid == nil {
		return ""
	}
	t, err := client.Tenant.Query().
		Where(tenant.IDEQ(*tid), tenant.DeletedAtIsNil()).
		Select(tenant.FieldName).
		Only(ctx)
	if err != nil {
		return ""
	}
	return std.Str(t.Name)
}

// toType 将 ent 实体转换为 types.LoginPolicy。
func toType(ctx context.Context, client *gen.Client, e *gen.LoginPolicy) *types.LoginPolicy {
	return &types.LoginPolicy{
		Id:         int64(e.ID),
		TargetId:   std.Int64(e.TargetID),
		Type:       stringVal(e.Type),
		Method:     stringVal(e.Method),
		Value:      std.Str(e.Value),
		Reason:     std.Str(e.Reason),
		TenantId:   std.Int64(e.TenantID),
		TenantName: tenantNameOf(ctx, client, e.TenantID),
		CreatedBy:  std.Int64(e.CreatedBy),
		UpdatedBy:  std.Int64(e.UpdatedBy),
		DeletedBy:  std.Int64(e.DeletedBy),
		CreatedAt:  std.TimeStr(e.CreatedAt),
		UpdatedAt:  std.TimeStr(e.UpdatedAt),
		DeletedAt:  std.TimeStr(e.DeletedAt),
	}
}