package viewer

import (
	"testing"

	"go-wind-admin/backendz/internal/pkg/token"
)

func TestDefaultIsPlatformView(t *testing.T) {
	v := Default
	if v == nil {
		t.Fatal("Default must be non-nil")
	}
	if !v.IsPlatformContext() {
		t.Fatal("Default should be platform context (tid=0)")
	}
	if v.IsTenantContext() {
		t.Fatal("Default should not be tenant context")
	}
	if !v.HasPermission("any", "any") {
		t.Fatal("Default HasPermission currently all-open")
	}
}

func TestFromClaimsPlatform(t *testing.T) {
	c := &token.Claim{UserID: 1, TenantID: 0, Username: "admin"}
	v := FromClaims(c)
	if v.UserID() != 1 || v.TenantID() != 0 {
		t.Fatalf("claim viewer wrong: uid=%d tid=%d", v.UserID(), v.TenantID())
	}
	if !v.IsPlatformContext() || v.IsTenantContext() {
		t.Fatal("tid=0 must be platform, not tenant")
	}
}

func TestFromClaimsTenant(t *testing.T) {
	c := &token.Claim{UserID: 5, TenantID: 88, Username: "u"}
	v := FromClaims(c)
	if !v.IsTenantContext() || v.IsPlatformContext() {
		t.Fatal("tid=88 must be tenant, not platform")
	}
	if v.TenantID() != 88 {
		t.Fatalf("tid=%d", v.TenantID())
	}
	// 租户视图应返回全量数据范围
	if ds := v.DataScope(); ds == nil || len(ds) == 0 {
		t.Fatal("tenant DataScope should not be empty")
	}
}

func TestFromClaimsNil(t *testing.T) {
	if got := FromClaims(nil); got != Default {
		t.Fatal("FromClaims(nil) must return Default")
	}
}