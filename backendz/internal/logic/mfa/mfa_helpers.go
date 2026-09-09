package mfa

import (
	"context"
	"strconv"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/usermfafactor"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/types"
)

// mfaIssuer otpauth URI 中的 issuer 标识。
const mfaIssuer = "GoWindAdmin"

// listFactorsByUser 列出某用户的全部 MFA 因子。
func listFactorsByUser(ctx context.Context, client *gen.Client, tenantID, userID uint32) ([]*gen.UserMfaFactor, error) {
	return client.UserMfaFactor.Query().
		Where(usermfafactor.TenantIDEQ(tenantID), usermfafactor.UserIDEQ(userID)).
		All(ctx)
}

// hasEnabledTotp 判断用户是否已绑定 ENABLED 的 TOTP 因子。
func hasEnabledTotp(ctx context.Context, client *gen.Client, tenantID, userID uint32) (bool, error) {
	n, err := client.UserMfaFactor.Query().
		Where(
			usermfafactor.TenantIDEQ(tenantID),
			usermfafactor.UserIDEQ(userID),
			usermfafactor.MethodEQ(usermfafactor.MethodTotp),
			usermfafactor.StatusEQ(usermfafactor.StatusEnabled),
		).Count(ctx)
	if err != nil {
		return false, err
	}
	return n > 0, nil
}

// toEnrolledMethod 将因子实体转为类型（不含 secret）。
func toEnrolledMethod(e *gen.UserMfaFactor) types.EnrolledMethod {
	return types.EnrolledMethod{
		Id:         strconv.FormatUint(uint64(e.ID), 10),
		Method:     methodString(e.Method),
		Display:    std.Str(e.DisplayName),
		Enabled:    e.Status != nil && *e.Status == usermfafactor.StatusEnabled,
		CreatedAt:  std.TimeStr(e.CreatedAt),
		LastUsedAt: std.TimeStr(e.LastUsedAt),
	}
}

func methodString(m *usermfafactor.Method) string {
	if m == nil {
		return ""
	}
	return string(*m)
}

// defaultDisplay 返回展示名（优先入参，其次 context 默认）。
func defaultDisplay(prefer, fallback string) string {
	if prefer != "" {
		return prefer
	}
	return fallback
}