package token

import (
	"testing"
	"time"
)

func newManager() *TokenManager {
	return NewTokenManager("test-secret-key", time.Hour, 30*24*time.Hour)
}

func TestCreateParseAccessToken(t *testing.T) {
	m := newManager()
	raw, err := m.CreateAccessToken(1, 2, "alice", "web", "dev-1")
	if err != nil {
		t.Fatalf("CreateAccessToken err: %v", err)
	}
	if raw == "" {
		t.Fatal("empty token")
	}
	c, err := m.ParseAccessToken(raw)
	if err != nil {
		t.Fatalf("ParseAccessToken err: %v", err)
	}
	if c.UserID != 1 || c.TenantID != 2 {
		t.Fatalf("claim mismatch: uid=%d tid=%d", c.UserID, c.TenantID)
	}
	if c.Username != "alice" || c.ClientID != "web" || c.DeviceID != "dev-1" {
		t.Fatalf("claim string mismatch: %+v", c)
	}
	if !c.VerifyExpiresAt(time.Now(), false) {
		t.Fatal("access token should not be expired")
	}
}

func TestRefreshTokenSeparateSecret(t *testing.T) {
	m := newManager()
	raw, err := m.CreateRefreshToken(7, 0, "bob", "", "")
	if err != nil {
		t.Fatalf("CreateRefreshToken err: %v", err)
	}
	c, err := m.ParseRefreshToken(raw)
	if err != nil {
		t.Fatalf("ParseRefreshToken err: %v", err)
	}
	if c.UserID != 7 || c.TenantID != 0 {
		t.Fatalf("claim mismatch: %+v", c)
	}
	// 访问令牌密钥与刷新令牌不同：用访问解析刷新令牌应失败
	if _, err := m.ParseAccessToken(raw); err == nil {
		t.Fatal("ParseAccessToken must reject a refresh token (different secret)")
	}
}

func TestParseRejectsTamperedToken(t *testing.T) {
	m := newManager()
	raw, _ := m.CreateAccessToken(1, 0, "alice", "", "")
	tampered := raw[:len(raw)-4] + "xxxx"
	if _, err := m.ParseAccessToken(tampered); err == nil {
		t.Fatal("tampered token must fail parsing")
	}
	if _, err := m.ParseAccessToken("garbage.token.value"); err == nil {
		t.Fatal("garbage token must fail parsing")
	}
	if _, err := m.ParseAccessToken(""); err == nil {
		t.Fatal("empty token must fail parsing")
	}
}

func TestAccessExpiresIn(t *testing.T) {
	m := newManager()
	if got := m.AccessExpiresIn(); got != int64(time.Hour.Seconds()) {
		t.Fatalf("AccessExpiresIn=%d", got)
	}
	if got := m.RefreshExpiresIn(); got != int64((30 * 24 * time.Hour).Seconds()) {
		t.Fatalf("RefreshExpiresIn=%d", got)
	}
}