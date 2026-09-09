package permission

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/permissionapi"
	"go-wind-admin/backendz/internal/ent/gen/permissiongroup"
	"go-wind-admin/backendz/internal/ent/gen/permissionmenu"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}

// menuIDsOf 查询权限关联的菜单 ID 列表。
func menuIDsOf(ctx context.Context, client *gen.Client, permissionID uint32) ([]int64, error) {
	rows, err := client.PermissionMenu.Query().
		Where(permissionmenu.PermissionIDEQ(permissionID)).
		Select(permissionmenu.FieldMenuID).
		All(ctx)
	if err != nil {
		return nil, err
	}
	ids := make([]int64, 0, len(rows))
	for _, r := range rows {
		ids = append(ids, std.Int64(r.MenuID))
	}
	return ids, nil
}

// apiIDsOf 查询权限关联的 API 资源 ID 列表。
func apiIDsOf(ctx context.Context, client *gen.Client, permissionID uint32) ([]int64, error) {
	rows, err := client.PermissionApi.Query().
		Where(permissionapi.PermissionIDEQ(permissionID)).
		Select(permissionapi.FieldAPIID).
		All(ctx)
	if err != nil {
		return nil, err
	}
	ids := make([]int64, 0, len(rows))
	for _, r := range rows {
		ids = append(ids, std.Int64(r.APIID))
	}
	return ids, nil
}

// groupNameOf 查询权限分组名；找不到或 err 时返回空串（不阻断）。
func groupNameOf(ctx context.Context, client *gen.Client, gid *uint32) string {
	if gid == nil {
		return ""
	}
	g, err := client.PermissionGroup.Query().
		Where(permissiongroup.IDEQ(*gid), permissiongroup.DeletedAtIsNil()).
		Select(permissiongroup.FieldName).
		Only(ctx)
	if err != nil {
		return ""
	}
	return std.Str(g.Name)
}

// toType 将 ent 实体转换为 types.Permission（含菜单/API/分组关系）。
func toType(ctx context.Context, client *gen.Client, e *gen.Permission) (*types.Permission, error) {
	menus, err := menuIDsOf(ctx, client, e.ID)
	if err != nil {
		return nil, err
	}
	apis, err := apiIDsOf(ctx, client, e.ID)
	if err != nil {
		return nil, err
	}
	return &types.Permission{
		Id:          int64(e.ID),
		Name:        std.Str(e.Name),
		Code:        std.Str(e.Code),
		Description: std.Str(e.Description),
		Status:      stringVal(e.Status),
		GroupId:     std.Int64(e.GroupID),
		GroupName:   groupNameOf(ctx, client, e.GroupID),
		MenuIds:     menus,
		ApiIds:      apis,
		CreatedBy:   std.Int64(e.CreatedBy),
		UpdatedBy:   std.Int64(e.UpdatedBy),
		DeletedBy:   std.Int64(e.DeletedBy),
		CreatedAt:   std.TimeStr(e.CreatedAt),
		UpdatedAt:   std.TimeStr(e.UpdatedAt),
		DeletedAt:   std.TimeStr(e.DeletedAt),
	}, nil
}