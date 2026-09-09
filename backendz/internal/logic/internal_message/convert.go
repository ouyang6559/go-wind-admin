package internal_message

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/internalmessagecategory"
	"go-wind-admin/backendz/internal/ent/gen/tenant"
	"go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// categoryNameOf 查询消息分类名；找不到或 err 时返回空串（不阻断）。
func categoryNameOf(ctx context.Context, client *gen.Client, cid *uint32) string {
	if cid == nil {
		return ""
	}
	c, err := client.InternalMessageCategory.Query().
		Where(internalmessagecategory.IDEQ(*cid)).
		Select(internalmessagecategory.FieldName).
		Only(ctx)
	if err != nil {
		return ""
	}
	return std.Str(c.Name)
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

// senderNameOf 查询发送者用户名；找不到或 err 时返回空串（不阻断）。
func senderNameOf(ctx context.Context, client *gen.Client, uid *uint32) string {
	if uid == nil {
		return ""
	}
	u, err := client.User.Query().
		Where(user.IDEQ(*uid)).
		Select(user.FieldUsername).
		Only(ctx)
	if err != nil {
		return ""
	}
	return std.Str(u.Username)
}

// toType 将 ent 实体转换为 types.InternalMessage（含分类名/发送者名/租户名）。
func toType(ctx context.Context, client *gen.Client, e *gen.InternalMessage) *types.InternalMessage {
	return &types.InternalMessage{
		Id:           int64(e.ID),
		Title:        std.Str(e.Title),
		Content:      std.Str(e.Content),
		Status:       stringVal(e.Status),
		Type:         stringVal(e.Type),
		SenderId:     std.Int64(e.SenderID),
		SenderName:   senderNameOf(ctx, client, e.SenderID),
		CategoryId:   std.Int64(e.CategoryID),
		CategoryName: categoryNameOf(ctx, client, e.CategoryID),
		TenantId:     std.Int64(e.TenantID),
		TenantName:   tenantNameOf(ctx, client, e.TenantID),
		CreatedBy:    std.Int64(e.CreatedBy),
		UpdatedBy:    std.Int64(e.UpdatedBy),
		DeletedBy:    std.Int64(e.DeletedBy),
		CreatedAt:    std.TimeStr(e.CreatedAt),
		UpdatedAt:    std.TimeStr(e.UpdatedAt),
		DeletedAt:    std.TimeStr(e.DeletedAt),
	}
}

func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}