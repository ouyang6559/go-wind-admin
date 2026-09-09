package org_unit

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

// f64Ptr float64 转指针。
func f64Ptr(v float64) *float64 {
	return &v
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

// toType 将 ent 实体转换为 types.OrgUnit。
func toType(ctx context.Context, client *gen.Client, e *gen.OrgUnit) *types.OrgUnit {
	var lat, lng float64
	if e.Latitude != nil {
		lat = *e.Latitude
	}
	if e.Longitude != nil {
		lng = *e.Longitude
	}
	return &types.OrgUnit{
		Id:                 int64(e.ID),
		Name:               std.Str(e.Name),
		Code:               std.Str(e.Code),
		Type:               stringVal(e.Type),
		Path:               std.Str(e.Path),
		Status:             stringVal(e.Status),
		SortOrder:          std.Int64(e.SortOrder),
		LeaderId:           std.Int64(e.LeaderID),
		TenantId:           std.Int64(e.TenantID),
		TenantName:         tenantNameOf(ctx, client, e.TenantID),
		Remark:             std.Str(e.Remark),
		Description:        std.Str(e.Description),
		BusinessScopes:     e.BusinessScopes,
		ExternalId:         std.Str(e.ExternalID),
		IsLegalEntity:      std.Bool(e.IsLegalEntity),
		RegistrationNumber: std.Str(e.RegistrationNumber),
		TaxId:              std.Str(e.TaxID),
		LegalEntityOrgId:   std.Int64(e.LegalEntityOrgID),
		Address:            std.Str(e.Address),
		Phone:              std.Str(e.Phone),
		Email:              std.Str(e.Email),
		Timezone:           std.Str(e.Timezone),
		Country:            std.Str(e.Country),
		Latitude:           lat,
		Longitude:          lng,
		StartAt:            std.TimeStr(e.StartAt),
		EndAt:              std.TimeStr(e.EndAt),
		PermissionTags:     e.PermissionTags,
		ContactUserId:      std.Int64(e.ContactUserID),
		ParentId:           std.Int64(e.ParentID),
		CreatedBy:          std.Int64(e.CreatedBy),
		UpdatedBy:          std.Int64(e.UpdatedBy),
		DeletedBy:          std.Int64(e.DeletedBy),
		CreatedAt:          std.TimeStr(e.CreatedAt),
		UpdatedAt:          std.TimeStr(e.UpdatedAt),
		DeletedAt:          std.TimeStr(e.DeletedAt),
	}
}