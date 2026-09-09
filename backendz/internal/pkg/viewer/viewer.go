// Package viewer 提供从 JWT 载荷派生的 tx7do viewer.Context 实现，
// 用于驱动 ent 租户隐私策略（TenantPrivacy）与租户变更守卫。
package viewer

import (
	"github.com/tx7do/go-crud/viewer"

	"go-wind-admin/backendz/internal/pkg/token"
)

// Default 是不带身份信息时的兜底 Viewer：视为平台/系统视图，
// 使公开路径（登录/验证码等）能通过隐私策略放行。
var Default viewer.Context = &ctxViewer{}

// FromClaims 根据 JWT 载荷构造 Viewer。
//   - tid==0 → 平台管理视图（IsPlatformContext）
//   - tid>0  → 租户业务视图（IsTenantContext）
func FromClaims(c *token.Claim) viewer.Context {
	if c == nil {
		return Default
	}
	return &ctxViewer{
		uid: uint64(c.UserID),
		tid: uint64(c.TenantID),
		un:  c.Username,
	}
}

// ctxViewer 是 viewer.Context 的简单实现。
type ctxViewer struct {
	uid uint64
	tid uint64
	un  string
}

func (v *ctxViewer) UserID() uint64            { return v.uid }
func (v *ctxViewer) TenantID() uint64          { return v.tid }
func (v *ctxViewer) OrgUnitID() uint64         { return 0 }
func (v *ctxViewer) Permissions() []string     { return nil }
func (v *ctxViewer) Roles() []string           { return nil }
func (v *ctxViewer) TraceID() string           { return "" }
func (v *ctxViewer) IsSystemContext() bool     { return false }
func (v *ctxViewer) ShouldAudit() bool         { return false }
func (v *ctxViewer) IsPlatformContext() bool   { return v.tid == 0 }
func (v *ctxViewer) IsTenantContext() bool     { return v.tid > 0 }

// DataScope 全量放行（不为一般业务逻辑注入数据权限谓词）。
func (v *ctxViewer) DataScope() []viewer.DataScope {
	if v.tid == 0 {
		return nil
	}
	return []viewer.DataScope{{ScopeType: viewer.ScopeTypeAll}}
}

// HasPermission 权限点判断：暂全放行，后续接入 RBAC 鉴权后收紧。
func (v *ctxViewer) HasPermission(action, resource string) bool {
	return true
}