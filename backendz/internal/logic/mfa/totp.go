package mfa

import (
	"crypto/hmac"
	"crypto/rand"
	"crypto/sha1"
	"crypto/subtle"
	"encoding/base32"
	"encoding/binary"
	"fmt"
	"io"
	"strings"
	"time"
)

// 本文件提供基于标准库的最小 TOTP（RFC 6238）实现。
// backendz 未引入 pquerna/otp，这里以 SHA1/6 位/30s 周期实现与常见认证器 App 兼容的
// 生成与校验；若需二维码等高级能力，可引入 pquerna/otp（未在 go.mod，需另行添加）。

// totpPeriod 单个时间窗口秒数。
const totpPeriod = 30

// generateTOTPSecret 生成 20 字节随机密钥，base32 编码（去填充）。
func generateTOTPSecret() (string, error) {
	raw := make([]byte, 20)
	if _, err := io.ReadFull(rand.Reader, raw); err != nil {
		return "", err
	}
	return strings.TrimRight(base32.StdEncoding.EncodeToString(raw), "="), nil
}

// totpCode 计算某时刻的 6 位 TOTP 码（HMAC-SHA1）。secret 为 base32（可含 '=' 填充）。
func totpCode(secret string, t time.Time) (string, error) {
	key, err := base32.StdEncoding.WithPadding(base32.NoPadding).
		DecodeString(strings.ToUpper(strings.TrimRight(secret, "=")))
	if err != nil {
		return "", err
	}
	counter := make([]byte, 8)
	binary.BigEndian.PutUint64(counter, uint64(t.Unix()/totpPeriod))

	mac := hmac.New(sha1.New, key)
	mac.Write(counter)
	hash := mac.Sum(nil)

	offset := hash[len(hash)-1] & 0x0f
	code := (uint32(hash[offset])&0x7f)<<24 |
		uint32(hash[offset+1])<<16 |
		uint32(hash[offset+2])<<8 |
		uint32(hash[offset+3])
	return fmt.Sprintf("%06d", code%1_000_000), nil
}

// validateTOTP 校验 TOTP 码，允许 ±skew 个时间窗口（防时钟漂移）。
func validateTOTP(secret, code string, now time.Time, skew int) bool {
	for i := -skew; i <= skew; i++ {
		at := now.Add(time.Duration(i*totpPeriod) * time.Second)
		c, err := totpCode(secret, at)
		if err == nil && subtle.ConstantTimeCompare([]byte(c), []byte(code)) == 1 {
			return true
		}
	}
	return false
}

// otpAuthURL 构造标准 otpauth URI，供认证器 App 导入。
func otpAuthURL(issuer, account, secret string) string {
	return fmt.Sprintf("otpauth://totp/%s:%s?secret=%s&issuer=%s&algorithm=SHA1&digits=6&period=%d",
		issuer, account, secret, issuer, totpPeriod)
}