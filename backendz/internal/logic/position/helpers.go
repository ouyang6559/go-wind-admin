package position

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/orgunit"
	"go-wind-admin/backendz/internal/ent/gen/position"
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

// i32Ptr int64 转 *int32，==0 返回 nil。
func i32Ptr(v int64) *int32 {
	if v == 0 {
		return nil
	}
	u := int32(v)
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

// orgUnitNameOf 查询组织单元名；找不到或 err 时返回空串（不阻断）。
func orgUnitNameOf(ctx context.Context, client *gen.Client, ouid *uint32) string {
	if ouid == nil {
		return ""
	}
	ou, err := client.OrgUnit.Query().
		Where(orgunit.IDEQ(*ouid), orgunit.DeletedAtIsNil()).
		Select(orgunit.FieldName).
		Only(ctx)
	if err != nil {
		return ""
	}
	return std.Str(ou.Name)
}

// reportsToNameOf 查询上报职位名；找不到或 err 时返回空串（不阻断）。
func reportsToNameOf(ctx context.Context, client *gen.Client, pid *uint32) string {
	if pid == nil {
		return ""
	}
	p, err := client.Position.Query().
		Where(position.IDEQ(*pid), position.DeletedAtIsNil()).
		Select(position.FieldName).
		Only(ctx)
	if err != nil {
		return ""
	}
	return std.Str(p.Name)
}

// toType 将 ent 实体转换为 types.Position。
func toType(ctx context.Context, client *gen.Client, e *gen.Position) *types.Position {
	return &types.Position{
		Id:                    int64(e.ID),
		Name:                  std.Str(e.Name),
		Code:                  std.Str(e.Code),
		Headcount:             std.Int64(e.Headcount),
		SortOrder:             std.Int64(e.SortOrder),
		Status:                stringVal(e.Status),
		Type:                  stringVal(&e.Type),
		Remark:                std.Str(e.Remark),
		Description:           std.Str(e.Description),
		JobFamily:             std.Str(e.JobFamily),
		JobGrade:              std.Str(e.JobGrade),
		Level:                 std.Int64(e.Level),
		IsKeyPosition:         std.Bool(e.IsKeyPosition),
		TenantId:              std.Int64(e.TenantID),
		TenantName:            tenantNameOf(ctx, client, e.TenantID),
		OrgUnitId:             std.Int64(e.OrgUnitID),
		OrgUnitName:           orgUnitNameOf(ctx, client, e.OrgUnitID),
		ReportsToPositionId:   std.Int64(e.ReportsToPositionID),
		ReportsToPositionName: reportsToNameOf(ctx, client, e.ReportsToPositionID),
		StartAt:               std.TimeStr(e.StartAt),
		EndAt:                 std.TimeStr(e.EndAt),
		CreatedBy:             std.Int64(e.CreatedBy),
		UpdatedBy:             std.Int64(e.UpdatedBy),
		DeletedBy:             std.Int64(e.DeletedBy),
		CreatedAt:             std.TimeStr(e.CreatedAt),
		UpdatedAt:             std.TimeStr(e.UpdatedAt),
		DeletedAt:             std.TimeStr(e.DeletedAt),
	}
}