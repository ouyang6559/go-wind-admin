package policy_evaluation_log

import (
	"context"
	"fmt"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// toType 将 ent 实体转换为 types.PolicyEvaluationLog。
func toType(_ context.Context, e *gen.PolicyEvaluationLog) types.PolicyEvaluationLog {
	return types.PolicyEvaluationLog{
		Id:                int64(e.ID),
		TenantId:          std.Int64(e.TenantID),
		UserId:            std.Int64(e.UserID),
		MembershipId:      std.Int64(e.MembershipID),
		PermissionId:      std.Int64(e.PermissionID),
		PolicyId:          std.Int64(e.PolicyID),
		RequestPath:       std.Str(e.RequestPath),
		RequestMethod:     std.Str(e.RequestMethod),
		Result:            std.Bool(e.Result),
		EffectDetails:     std.Str(e.EffectDetails),
		ScopeSql:          std.Str(e.ScopeSQL),
		IpAddress:         std.Str(e.IPAddress),
		TraceId:           std.Str(e.TraceID),
		EvaluationContext: std.Str(e.EvaluationContext),
		LogHash:           std.Str(e.LogHash),
		Signature:         sigStr(e.Signature),
		CreatedAt:         std.TimeStr(e.CreatedAt),
	}
}

// sigStr 将二进制签名输出为十六进制字符串；nil 返回空串。
func sigStr(sig *[]byte) string {
	if sig == nil {
		return ""
	}
	return fmt.Sprintf("%x", *sig)
}