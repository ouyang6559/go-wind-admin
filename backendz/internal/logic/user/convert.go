// Package user 提供用户管理相关逻辑。
package user

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/role"
	"go-wind-admin/backendz/internal/ent/gen/tenant"
	genuser "go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/ent/gen/userrole"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}

func strPtr(s string) *string {
	if s == "" {
		return nil
	}
	return &s
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

// rolesOf 查询用户关联的有效角色 ID、角色 code 与角色名；出错时返回空（不阻断）。
func rolesOf(ctx context.Context, client *gen.Client, userID uint32) (ids []int64, codes []string, names []string) {
	rows, err := client.UserRole.Query().
		Where(userrole.UserIDEQ(userID), userrole.StatusEQ(userrole.StatusActive), userrole.DeletedAtIsNil()).
		Select(userrole.FieldRoleID).
		All(ctx)
	if err != nil {
		return
	}
	roleIDs := make([]uint32, 0, len(rows))
	for _, r := range rows {
		if r.RoleID != nil {
			roleIDs = append(roleIDs, *r.RoleID)
		}
	}
	if len(roleIDs) == 0 {
		return
	}
	roles, rerr := client.Role.Query().
		Where(role.IDIn(roleIDs...), role.DeletedAtIsNil()).
		All(ctx)
	if rerr != nil {
		return
	}
	ids = make([]int64, 0, len(roles))
	codes = make([]string, 0, len(roles))
	names = make([]string, 0, len(roles))
	for _, r := range roles {
		ids = append(ids, int64(r.ID))
		codes = append(codes, std.Str(r.Code))
		names = append(names, std.Str(r.Name))
	}
	return
}

// UserToType 将 ent 用户实体转换为 types.User，并填充租户名与角色信息。
// 导出供 user_profile 模块复用。
func UserToType(ctx context.Context, client *gen.Client, e *gen.User) (*types.User, error) {
	roleIDs, roleCodes, roleNames := rolesOf(ctx, client, e.ID)
	return &types.User{
		Id:          int64(e.ID),
		TenantId:    std.Int64(e.TenantID),
		TenantName:  tenantNameOf(ctx, client, e.TenantID),
		Username:    std.Str(e.Username),
		Nickname:    std.Str(e.Nickname),
		Realname:    std.Str(e.Realname),
		Email:       std.Str(e.Email),
		Mobile:      std.Str(e.Mobile),
		Telephone:   std.Str(e.Telephone),
		Avatar:      std.Str(e.Avatar),
		Address:     std.Str(e.Address),
		Region:      std.Str(e.Region),
		Description: std.Str(e.Description),
		Gender:      stringVal(e.Gender),
		LastLoginAt: std.TimeStr(e.LastLoginAt),
		LastLoginIp: std.Str(e.LastLoginIP),
		Status:      stringVal(e.Status),
		LockedUntil: std.TimeStr(e.LockedUntil),
		RoleIds:     roleIDs,
		Roles:       roleCodes,
		RoleNames:   roleNames,
		CreatedBy:   std.Int64(e.CreatedBy),
		UpdatedBy:   std.Int64(e.UpdatedBy),
		DeletedBy:   std.Int64(e.DeletedBy),
		CreatedAt:   std.TimeStr(e.CreatedAt),
		UpdatedAt:   std.TimeStr(e.UpdatedAt),
		DeletedAt:   std.TimeStr(e.DeletedAt),
	}, nil
}

// ApplyUserPatch 将可设置的非空字段批量应用到用户更新上（供 user 与 user_profile 复用）。
func ApplyUserPatch(upd *gen.UserUpdateOne, d types.User) {
	if d.Remark != "" {
		upd.SetRemark(d.Remark)
	}
	if d.Nickname != "" {
		upd.SetNickname(d.Nickname)
	}
	if d.Realname != "" {
		upd.SetRealname(d.Realname)
	}
	if d.Email != "" {
		upd.SetEmail(d.Email)
	}
	if d.Mobile != "" {
		upd.SetMobile(d.Mobile)
	}
	if d.Telephone != "" {
		upd.SetTelephone(d.Telephone)
	}
	if d.Avatar != "" {
		upd.SetAvatar(d.Avatar)
	}
	if d.Address != "" {
		upd.SetAddress(d.Address)
	}
	if d.Region != "" {
		upd.SetRegion(d.Region)
	}
	if d.Description != "" {
		upd.SetDescription(d.Description)
	}
	if d.Gender != "" {
		upd.SetGender(genuser.Gender(d.Gender))
	}
	if d.Status != "" {
		upd.SetStatus(genuser.Status(d.Status))
	}
}
