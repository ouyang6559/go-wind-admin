package tenant

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// enumStr 解引用字符串枚举指针为字符串；nil 返回空串。
func enumStr[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}

// baseToType 将租户实体转换为 types.Tenant（不含关联信息）。
func baseToType(e *gen.Tenant) *types.Tenant {
	t := &types.Tenant{
		Id:               int64(e.ID),
		Name:             std.Str(e.Name),
		Code:             std.Str(e.Code),
		Domain:           std.Str(e.Domain),
		LogoUrl:          std.Str(e.LogoURL),
		Industry:         std.Str(e.Industry),
		Type:             enumStr(e.Type),
		Remark:           std.Str(e.Remark),
		AdminUserId:      std.Int64(e.AdminUserID),
		SubscriptionAt:   std.TimeStr(e.SubscriptionAt),
		UnsubscribeAt:    std.TimeStr(e.UnsubscribeAt),
		ExpiredAt:        std.TimeStr(e.ExpiredAt),
		SubscriptionPlan: std.Str(e.SubscriptionPlan),
		Status:           enumStr(e.Status),
		AuditStatus:      enumStr(e.AuditStatus),
		CreatedBy:        std.Int64(e.CreatedBy),
		UpdatedBy:        std.Int64(e.UpdatedBy),
		DeletedBy:        std.Int64(e.DeletedBy),
		CreatedAt:        std.TimeStr(e.CreatedAt),
		UpdatedAt:        std.TimeStr(e.UpdatedAt),
		DeletedAt:        std.TimeStr(e.DeletedAt),
	}
	if e.Edges.Plan != nil {
		t.PlanId = int64(e.Edges.Plan.ID)
	}
	return t
}

// fillAdminAndCount 回填管理员用户名与租户成员数（失败不回填，不阻断）。
func fillAdminAndCount(ctx context.Context, client *gen.Client, t *types.Tenant) {
	if t.AdminUserId > 0 {
		if u, err := client.User.Query().
			Where(user.IDEQ(uint32(t.AdminUserId)), user.DeletedAtIsNil()).
			Select(user.FieldUsername).
			Only(ctx); err == nil {
			t.AdminUserName = std.Str(u.Username)
		}
	}
	if t.Id > 0 {
		if cnt, err := client.User.Query().
			Where(user.TenantIDEQ(uint32(t.Id)), user.DeletedAtIsNil()).
			Count(ctx); err == nil {
			t.MemberCount = int64(cnt)
		}
	}
}