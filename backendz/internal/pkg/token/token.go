// Package token 签发与解析用户 JWT 访问令牌与刷新令牌。
package token

import (
	"errors"
	"time"

	"github.com/golang-jwt/jwt/v4"
)

// Claim 访问令牌载荷。
type Claim struct {
	UserID   uint32 `json:"uid"`
	TenantID uint32 `json:"tid"`
	Username string `json:"username"`
	ClientID string `json:"client_id,omitempty"`
	DeviceID string `json:"device_id,omitempty"`
	jwt.RegisteredClaims
}

// TokenManager 令牌签发与校验。
type TokenManager struct {
	accessSecret  []byte
	accessExpire  time.Duration
	refreshSecret []byte
	refreshExpire time.Duration
}

func NewTokenManager(accessSecret string, accessExpire, refreshExpire time.Duration) *TokenManager {
	return &TokenManager{
		accessSecret:  []byte(accessSecret),
		accessExpire:  accessExpire,
		refreshSecret: []byte(accessSecret + ":refresh"),
		refreshExpire: refreshExpire,
	}
}

// CreateAccessToken 生成访问令牌。
func (m *TokenManager) CreateAccessToken(uid, tid uint32, username, clientID, deviceID string) (string, error) {
	return m.sign(m.accessSecret, m.accessExpire, uid, tid, username, clientID, deviceID)
}

// CreateRefreshToken 生成刷新令牌。
func (m *TokenManager) CreateRefreshToken(uid, tid uint32, username, clientID, deviceID string) (string, error) {
	return m.sign(m.refreshSecret, m.refreshExpire, uid, tid, username, clientID, deviceID)
}

func (m *TokenManager) sign(secret []byte, expire time.Duration, uid, tid uint32, username, clientID, deviceID string) (string, error) {
	now := time.Now()
	claims := Claim{
		UserID:   uid,
		TenantID: tid,
		Username: username,
		ClientID: clientID,
		DeviceID: deviceID,
		RegisteredClaims: jwt.RegisteredClaims{
			ExpiresAt: jwt.NewNumericDate(now.Add(expire)),
			IssuedAt:  jwt.NewNumericDate(now),
		},
	}
	t := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return t.SignedString(secret)
}

// ParseAccessToken 校验访问令牌并返回载荷。
func (m *TokenManager) ParseAccessToken(tokenStr string) (*Claim, error) {
	return m.parse(m.accessSecret, tokenStr)
}

// ParseRefreshToken 校验刷新令牌并返回载荷。
func (m *TokenManager) ParseRefreshToken(tokenStr string) (*Claim, error) {
	return m.parse(m.refreshSecret, tokenStr)
}

func (m *TokenManager) parse(secret []byte, tokenStr string) (*Claim, error) {
	claims := &Claim{}
	t, err := jwt.ParseWithClaims(tokenStr, claims, func(t *jwt.Token) (any, error) {
		if _, ok := t.Method.(*jwt.SigningMethodHMAC); !ok {
			return nil, errors.New("unexpected signing method")
		}
		return secret, nil
	})
	if err != nil {
		return nil, err
	}
	if !t.Valid {
		return nil, errors.New("invalid token")
	}
	return claims, nil
}

// AccessExpiresIn 访问令牌有效期（秒）。
func (m *TokenManager) AccessExpiresIn() int64 {
	return int64(m.accessExpire.Seconds())
}

// RefreshExpiresIn 刷新令牌有效期（秒）。
func (m *TokenManager) RefreshExpiresIn() int64 {
	return int64(m.refreshExpire.Seconds())
}