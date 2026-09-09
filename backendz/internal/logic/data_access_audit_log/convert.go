package data_access_audit_log

import (
	"context"
	"fmt"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/tenant"
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

// toType 将 ent 实体转换为 types.DataAccessAuditLog。
func toType(ctx context.Context, client *gen.Client, e *gen.DataAccessAuditLog) types.DataAccessAuditLog {
	return types.DataAccessAuditLog{
		Id:              int64(e.ID),
		TenantId:        std.Int64(e.TenantID),
		TenantName:      tenantNameOf(ctx, client, e.TenantID),
		UserId:          std.Int64(e.UserID),
		Username:        std.Str(e.Username),
		IpAddress:       std.Str(e.IPAddress),
		RequestId:       std.Str(e.RequestID),
		DataSource:      std.Str(e.DataSource),
		TableName:       std.Str(e.TableName),
		DataId:          std.Str(e.DataID),
		AccessType:      stringVal(e.AccessType),
		SqlDigest:       std.Str(e.SQLDigest),
		SqlText:         std.Str(e.SQLText),
		AffectedRows:    std.Int64(e.AffectedRows),
		LatencyMs:       std.Int64(e.LatencyMs),
		Success:         std.Bool(e.Success),
		SensitiveLevel:  stringVal(e.SensitiveLevel),
		DataMasked:      std.Bool(e.DataMasked),
		MaskingRules:    std.Str(e.MaskingRules),
		BusinessPurpose: std.Str(e.BusinessPurpose),
		DataCategory:    std.Str(e.DataCategory),
		DbUser:          std.Str(e.DbUser),
		LogHash:         std.Str(e.LogHash),
		Signature:       sigStr(e.Signature),
		CreatedAt:       std.TimeStr(e.CreatedAt),
	}
}

// sigStr 将二进制签名输出为十六进制字符串；nil 返回空串。
func sigStr(sig *[]byte) string {
	if sig == nil {
		return ""
	}
	return fmt.Sprintf("%x", *sig)
}

func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}