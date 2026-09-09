package login_audit_log

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

// toType 将 ent 实体转换为 types.LoginAuditLog。
func toType(ctx context.Context, client *gen.Client, e *gen.LoginAuditLog) types.LoginAuditLog {
	return types.LoginAuditLog{
		Id:            int64(e.ID),
		TenantId:      std.Int64(e.TenantID),
		TenantName:    tenantNameOf(ctx, client, e.TenantID),
		UserId:        std.Int64(e.UserID),
		Username:      std.Str(e.Username),
		IpAddress:     std.Str(e.IPAddress),
		GeoLocation:   geoToType(e.GeoLocation),
		SessionId:     std.Str(e.SessionID),
		DeviceInfo:    deviceToType(e.DeviceInfo),
		RequestId:     std.Str(e.RequestID),
		TraceId:       std.Str(e.TraceID),
		ActionType:    stringVal(e.ActionType),
		Status:        stringVal(e.Status),
		FailureReason: std.Str(e.FailureReason),
		MfaStatus:     std.Str(e.MfaStatus),
		LoginMethod:   stringVal(e.LoginMethod),
		RiskScore:     std.Int64(e.RiskScore),
		RiskLevel:     stringVal(e.RiskLevel),
		RiskFactors:   e.RiskFactors,
		LogHash:       std.Str(e.LogHash),
		Signature:     sigStr(e.Signature),
		CreatedAt:     std.TimeStr(e.CreatedAt),
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

// deviceToType 将 schema.DeviceInfo 转换为 types.DeviceInfo。
func deviceToType(d *schema.DeviceInfo) types.DeviceInfo {
	if d == nil {
		return types.DeviceInfo{}
	}
	return types.DeviceInfo{
		ClientId:          d.ClientID,
		ClientName:        d.ClientName,
		OsName:            d.OSName,
		OsVersion:         d.OSVersion,
		DeviceId:          d.DeviceID,
		DeviceType:        deviceTypeStr(d.DeviceType),
		Manufacturer:      d.Manufacturer,
		Model:             d.Model,
		Platform:          d.Platform,
		OsBuild:           d.OSBuild,
		AppName:           d.AppName,
		AppVersion:        d.AppVersion,
		ScreenWidth:       uint32PtrInt64(d.ScreenWidth),
		ScreenHeight:      uint32PtrInt64(d.ScreenHeight),
		Locale:            d.Locale,
		TimeZone:          d.Timezone,
		NetworkType:       d.NetworkType,
		Carrier:           d.Carrier,
		DeviceFingerprint: d.DeviceFingerprint,
		UserAgent:         d.UserAgent,
		BrowserName:       d.BrowserName,
		BrowserVersion:    d.BrowserVersion,
		BrowserEngine:     d.BrowserEngine,
		EngineVersion:     d.EngineVersion,
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

// int32PtrStr 将 *int32 解引用为字符串；nil 返回空串。
func int32PtrStr(v *int32) string {
	if v == nil {
		return ""
	}
	return fmt.Sprintf("%d", *v)
}

// deviceTypeStr 将 *schema.DeviceType 解引用为字符串；nil 返回空串。
func deviceTypeStr(v *schema.DeviceType) string {
	if v == nil {
		return ""
	}
	return fmt.Sprintf("%d", int32(*v))
}

// uint32PtrInt64 解引用 *uint32 为 int64；nil 返回 0。
func uint32PtrInt64(v *uint32) int64 {
	if v == nil {
		return 0
	}
	return int64(*v)
}

func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}