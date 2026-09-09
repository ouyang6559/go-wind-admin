package internal_message_category

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/tenant"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

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

// toType 将 ent 实体转换为 types.InternalMessageCategory。
func toType(ctx context.Context, client *gen.Client, e *gen.InternalMessageCategory) *types.InternalMessageCategory {
	return &types.InternalMessageCategory{
		Id:         int64(e.ID),
		Name:       std.Str(e.Name),
		Code:       std.Str(e.Code),
		IconUrl:    std.Str(e.IconURL),
		SortOrder:  std.Int64(e.SortOrder),
		IsEnabled:  std.Bool(e.IsEnabled),
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