// Package token 签发与解析用户 JWT 访问令牌与刷新令牌。
package token

import (
	"crypto/rand"
	"encoding/hex"
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
	return m.signWithJTI(m.accessSecret, m.accessExpire, uid, tid, username, clientID, deviceID, newJTI())
}

// CreateRefreshToken 生成刷新令牌。
func (m *TokenManager) CreateRefreshToken(uid, tid uint32, username, clientID, deviceID string) (string, error) {
	return m.signWithJTI(m.refreshSecret, m.refreshExpire, uid, tid, username, clientID, deviceID, newJTI())
}

// CreateTokenPair 生成一对令牌（访问+刷新）并返回本次令牌对统一的 jti。
// 同一对令牌共享同一个 jti，用于在线会话注册表中标识一次令牌签发（会话）。
func (m *TokenManager) CreateTokenPair(uid, tid uint32, username, clientID, deviceID string) (access, refresh, jti string, err error) {
	jti = newJTI()
	access, err = m.signWithJTI(m.accessSecret, m.accessExpire, uid, tid, username, clientID, deviceID, jti)
	if err != nil {
		return "", "", "", err
	}
	refresh, err = m.signWithJTI(m.refreshSecret, m.refreshExpire, uid, tid, username, clientID, deviceID, jti)
	if err != nil {
		return "", "", "", err
	}
	return access, refresh, jti, nil
}

func (m *TokenManager) signWithJTI(secret []byte, expire time.Duration, uid, tid uint32, username, clientID, deviceID, jti string) (string, error) {
	now := time.Now()
	claims := Claim{
		UserID:   uid,
		TenantID: tid,
		Username: username,
		ClientID: clientID,
		DeviceID: deviceID,
		RegisteredClaims: jwt.RegisteredClaims{
			ID:        jti,
			ExpiresAt: jwt.NewNumericDate(now.Add(expire)),
			IssuedAt:  jwt.NewNumericDate(now),
		},
	}
	t := jwt.NewWithClaims(jwt.SigningMethodHS256, claims)
	return t.SignedString(secret)
}

// newJTI 生成会话令牌对唯一 ID（32 字节随机 hex）。
func newJTI() string {
	buf := make([]byte, 32)
	if _, err := rand.Read(buf); err != nil {
		// 熵源不可用时回退到时间戳，仅用于保证不 panic（维持调用链不中断）
		return time.Now().Format("20060102150405.000000000")
	}
	return hex.EncodeToString(buf)
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
