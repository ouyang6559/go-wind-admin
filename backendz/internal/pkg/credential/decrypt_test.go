package credential

import (
	"crypto/aes"
	"crypto/cipher"
	"encoding/base64"
	"strings"
	"testing"
)

// encryptPKCS7 与前端 react（CryptoJS AES-CBC-Pkcs7, key=iv=DefaultAESKey, base64）对齐的加密。
func encryptPKCS7(t *testing.T, plain string) string {
	t.Helper()
	key := []byte(DefaultAESKey)
	block, err := aes.NewCipher(key)
	if err != nil {
		t.Fatalf("NewCipher: %v", err)
	}
	bs := block.BlockSize()
	padLen := bs - len(plain)%bs
	data := append([]byte(plain), make([]byte, padLen)...)
	for i := len(plain); i < len(data); i++ {
		data[i] = byte(padLen)
	}
	ciphertext := make([]byte, len(data))
	cipher.NewCBCEncrypter(block, key[:bs]).CryptBlocks(ciphertext, data)
	return base64.StdEncoding.EncodeToString(ciphertext)
}

func TestResolvePlainPassword_decryptsAES(t *testing.T) {
	enc := encryptPKCS7(t, "admin")
	got := ResolvePlainPassword(enc)
	if got != "admin" {
		t.Fatalf("decrypt = %q, want %q", got, "admin")
	}
}

func TestResolvePlainPassword_tamperedFallsBackToRaw(t *testing.T) {
	// "admin" 非合法 base64（长度 5 非 4 倍数），解密失败应回退原值，用于兼容明文直传
	got := ResolvePlainPassword("admin")
	if got != "admin" {
		t.Fatalf("fallback = %q, want raw %q", got, "admin")
	}
}

func TestResolvePlainPassword_empty(t *testing.T) {
	if got := ResolvePlainPassword(""); got != "" {
		t.Fatalf("empty should passthrough, got %q", got)
	}
}

func TestResolvePlainPassword_longPlaintext(t *testing.T) {
	// 多块明文（>16 字节）也能正确解密还原
	plain := strings.Repeat("x", 33)
	enc := encryptPKCS7(t, plain)
	if got := ResolvePlainPassword(enc); got != plain {
		t.Fatalf("decrypt multi-block = %q", got)
	}
}

func TestDefaultAESKey_is16Bytes(t *testing.T) {
	if len(DefaultAESKey) != 16 {
		t.Fatalf("DefaultAESKey len = %d, must be 16 (AES-128)", len(DefaultAESKey))
	}
}