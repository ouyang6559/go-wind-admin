package role

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/rolepermission"
	"go-wind-admin/backendz/internal/ent/gen/tenant"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// permissionsOf 查询指定角色关联的权限 ID 列表。
func permissionsOf(ctx context.Context, client *gen.Client, roleID uint32) ([]int64, error) {
	rows, err := client.RolePermission.Query().
		Where(rolepermission.RoleIDEQ(roleID)).
		Select(rolepermission.FieldPermissionID).
		All(ctx)
	if err != nil {
		return nil, err
	}
	ids := make([]int64, 0, len(rows))
	for _, r := range rows {
		ids = append(ids, std.Int64(r.PermissionID))
	}
	return ids, nil
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

// toType 将 ent 实体转换为 types.Role（含权限与租户名）。
func toType(ctx context.Context, client *gen.Client, e *gen.Role) (*types.Role, error) {
	perms, err := permissionsOf(ctx, client, e.ID)
	if err != nil {
		return nil, err
	}
	return &types.Role{
		Id:          int64(e.ID),
		Name:        std.Str(e.Name),
		Code:        std.Str(e.Code),
		SortOrder:   std.Int64(e.SortOrder),
		Status:      stringVal(e.Status),
		Description: std.Str(e.Description),
		IsProtected: std.Bool(e.IsProtected),
		Type:        stringVal(e.Type),
		Permissions: perms,
		TenantId:    std.Int64(e.TenantID),
		TenantName:  tenantNameOf(ctx, client, e.TenantID),
		CreatedBy:   std.Int64(e.CreatedBy),
		UpdatedBy:   std.Int64(e.UpdatedBy),
		DeletedBy:   std.Int64(e.DeletedBy),
		CreatedAt:   std.TimeStr(e.CreatedAt),
		UpdatedAt:   std.TimeStr(e.UpdatedAt),
		DeletedAt:   std.TimeStr(e.DeletedAt),
	}, nil
}

func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}