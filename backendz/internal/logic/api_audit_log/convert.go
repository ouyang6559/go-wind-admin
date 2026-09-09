package api_audit_log

import (
	"context"
	"strconv"

	"go-wind-admin/backendz/internal/ent/gen"
	genapi "go-wind-admin/backendz/internal/ent/gen/api"
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

// toGeoLocation 将 schema.GeoLocation 转换为 types.GeoLocation。
func toGeoLocation(g *schema.GeoLocation) types.GeoLocation {
	if g == nil {
		return types.GeoLocation{}
	}
	var lat, lng string
	if g.Latitude != nil {
		lat = strconv.FormatInt(*g.Latitude, 10)
	}
	if g.Longitude != nil {
		lng = strconv.FormatInt(*g.Longitude, 10)
	}
	return types.GeoLocation{
		CountryCode:   g.CountryCode,
		Province:      g.Province,
		City:          g.City,
		Isp:           g.ISP,
		AddressRemark: g.AddressRemark,
		Latitude:      lat,
		Longitude:     lng,
	}
}

// toDeviceInfo 将 schema.DeviceInfo 转换为 types.DeviceInfo。
func toDeviceInfo(d *schema.DeviceInfo) types.DeviceInfo {
	if d == nil {
		return types.DeviceInfo{}
	}
	dt := ""
	if d.DeviceType != nil {
		dt = strconv.Itoa(int(*d.DeviceType))
	}
	sw, sh := int64(0), int64(0)
	if d.ScreenWidth != nil {
		sw = int64(*d.ScreenWidth)
	}
	if d.ScreenHeight != nil {
		sh = int64(*d.ScreenHeight)
	}
	return types.DeviceInfo{
		ClientId:          d.ClientID,
		ClientName:        d.ClientName,
		OsName:            d.OSName,
		OsVersion:         d.OSVersion,
		DeviceId:          d.DeviceID,
		DeviceType:        dt,
		Manufacturer:      d.Manufacturer,
		Model:             d.Model,
		Platform:          d.Platform,
		OsBuild:           d.OSBuild,
		AppName:           d.AppName,
		AppVersion:        d.AppVersion,
		ScreenWidth:       sw,
		ScreenHeight:      sh,
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

func signatureOf(s *[]byte) string {
	if s == nil {
		return ""
	}
	return string(*s)
}

// toType 将 ent 实体转换为 types.ApiAuditLog。
func toType(ctx context.Context, client *gen.Client, e *gen.ApiAuditLog) *types.ApiAuditLog {
	return &types.ApiAuditLog{
		Id:             int64(e.ID),
		TenantId:       std.Int64(e.TenantID),
		TenantName:     tenantNameOf(ctx, client, e.TenantID),
		UserId:         std.Int64(e.UserID),
		Username:       std.Str(e.Username),
		IpAddress:      std.Str(e.IPAddress),
		GeoLocation:    toGeoLocation(e.GeoLocation),
		DeviceInfo:     toDeviceInfo(e.DeviceInfo),
		Referer:        std.Str(e.Referer),
		AppVersion:     std.Str(e.AppVersion),
		HttpMethod:     std.Str(e.HTTPMethod),
		Path:           std.Str(e.Path),
		RequestUri:     std.Str(e.RequestURI),
		ApiModule:      std.Str(e.APIModule),
		ApiOperation:   std.Str(e.APIOperation),
		ApiDescription: std.Str(e.APIDescription),
		RequestId:      std.Str(e.RequestID),
		TraceId:        std.Str(e.TraceID),
		SpanId:         std.Str(e.SpanID),
		LatencyMs:      std.Int64(e.LatencyMs),
		Success:        std.Bool(e.Success),
		StatusCode:     std.Int64(e.StatusCode),
		Reason:         std.Str(e.Reason),
		RequestHeader:  std.Str(e.RequestHeader),
		RequestBody:    std.Str(e.RequestBody),
		Response:       std.Str(e.Response),
		LogHash:        std.Str(e.LogHash),
		Signature:      signatureOf(e.Signature),
		CreatedAt:      std.TimeStr(e.CreatedAt),
	}
}

// enrich 用 api 表回填 ApiModule/ApiDescription（找不到则忽略，不阻断）。
func enrich(ctx context.Context, client *gen.Client, item *types.ApiAuditLog) {
	if item == nil || item.Path == "" || item.HttpMethod == "" {
		return
	}
	a, err := client.Api.Query().
		Where(genapi.DeletedAtIsNil(), genapi.PathEQ(item.Path), genapi.MethodEQ(item.HttpMethod)).
		Only(ctx)
	if err != nil {
		return
	}
	item.ApiDescription = std.Str(a.Description)
	item.ApiModule = std.Str(a.ModuleDescription)
}