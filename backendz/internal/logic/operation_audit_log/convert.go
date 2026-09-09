package operation_audit_log

import (
	"context"
	"fmt"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/tenant"
	"go-wind-admin/backendz/internal/ent/schema"
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

// toType 将 ent 实体转换为 types.OperationAuditLog。
func toType(ctx context.Context, client *gen.Client, e *gen.OperationAuditLog) types.OperationAuditLog {
	return types.OperationAuditLog{
		Id:             int64(e.ID),
		TenantId:       std.Int64(e.TenantID),
		TenantName:     tenantNameOf(ctx, client, e.TenantID),
		UserId:         std.Int64(e.UserID),
		Username:       std.Str(e.Username),
		ResourceType:   std.Str(e.ResourceType),
		ResourceId:     std.Str(e.ResourceID),
		Action:         stringVal(e.Action),
		BeforeData:     std.Str(e.BeforeData),
		AfterData:      std.Str(e.AfterData),
		SensitiveLevel: stringVal(e.SensitiveLevel),
		RequestId:      std.Str(e.RequestID),
		TraceId:        std.Str(e.TraceID),
		Success:        std.Bool(e.Success),
		FailureReason:  std.Str(e.FailureReason),
		IpAddress:      std.Str(e.IPAddress),
		GeoLocation:    geoToType(e.GeoLocation),
		LogHash:        std.Str(e.LogHash),
		Signature:      sigStr(e.Signature),
		CreatedAt:      std.TimeStr(e.CreatedAt),
	}
}

// geoToType 将 schema.GeoLocation 转换为 types.GeoLocation。
func geoToType(g *schema.GeoLocation) types.GeoLocation {
	if g == nil {
		return types.GeoLocation{}
	}
	return types.GeoLocation{
		CountryCode:   g.CountryCode,
		Province:      g.Province,
		City:          g.City,
		Isp:           g.ISP,
		AddressRemark: g.AddressRemark,
		Latitude:      int64PtrStr(g.Latitude),
		Longitude:     int64PtrStr(g.Longitude),
	}
}

// sigStr 将二进制签名输出为十六进制字符串；nil 返回空串。
func sigStr(sig *[]byte) string {
	if sig == nil {
		return ""
	}
	return fmt.Sprintf("%x", *sig)
}

// int64PtrStr 将 *int64 解引用为字符串；nil 返回空串。
func int64PtrStr(v *int64) string {
	if v == nil {
		return ""
	}
	return fmt.Sprintf("%d", *v)
}

func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}