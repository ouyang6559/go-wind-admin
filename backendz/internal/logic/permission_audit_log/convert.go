package permission_audit_log

import (
	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

func stringVal[T ~string](v *T) string {
	if v == nil {
		return ""
	}
	return string(*v)
}

func signatureOf(s *[]byte) string {
	if s == nil {
		return ""
	}
	return string(*s)
}

// toType 将 ent 实体转换为 types.PermissionAuditLog。
func toType(e *gen.PermissionAuditLog) *types.PermissionAuditLog {
	return &types.PermissionAuditLog{
		Id:           int64(e.ID),
		TenantId:     std.Int64(e.TenantID),
		OperatorId:   std.Int64(e.OperatorID),
		OperatorName: std.Str(e.OperatorName),
		TargetType:   std.Str(e.TargetType),
		TargetId:     std.Str(e.TargetID),
		TargetName:   std.Str(e.TargetName),
		Action:       stringVal(e.Action),
		OldValue:     std.Str(e.OldValue),
		NewValue:     std.Str(e.NewValue),
		IpAddress:    std.Str(e.IPAddress),
		RequestId:    std.Str(e.RequestID),
		Reason:       std.Str(e.Reason),
		LogHash:      std.Str(e.LogHash),
		Signature:    signatureOf(e.Signature),
		CreatedAt:    std.TimeStr(e.CreatedAt),
	}
}