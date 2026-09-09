// Package credential 提供对前端登录/修改密码所传加密密码的解密能力。
//
// 前端 react 用 CryptoJS 对密码做 AES-128-CBC 加密后 base64 传输，算法约定与旧
// backend（go-kratos）一致：key=iv=DefaultAESKey，PKCS#7 填充。改写到 backendz 后，
// 需在密码校验前先按该规则解密，才能与前端/旧后端契约兼容。
package credential

import (
	"crypto/aes"
	"crypto/cipher"
	"encoding/base64"
	"errors"
	"fmt"
)

// DefaultAESKey 与旧 backend（app/pkg/crypto.DefaultAESKey）保持一致，
// 前端 react 的 VITE_AES_KEY 亦为同值 "f51d66a73d8a0927"。
const DefaultAESKey = "f51d66a73d8a0927"

// ResolvePlainPassword 将前端传入（可能已加密）的密码还原为明文，供后续 bcrypt 校验。
//
// 兼容两种输入：
//  1. 前端约定格式：base64(AES-128-CBC(key=iv=DefaultAESKey, PKCS7, 明文密码))；
//  2. 纯明文（内部接口测试/运维工具直接提交 bcrypt 明文）。
//
// 规则：能成功 base64 解码且 AES 解密出非空结果时用明文，否则原样返回原始串。
// 依赖方在校验未命中时仍应兜底尝试原始串，避免密文被误判的边界场景。
func ResolvePlainPassword(ciphertext string) (plain string) {
	decrypted, err := aesDecryptPKCS5(ciphertext, []byte(DefaultAESKey))
	if err != nil || len(decrypted) == 0 {
		return ciphertext
	}
	return string(decrypted)
}

// aesDecryptPKCS5 执行 base64 + AES-128/192/256-CBC(PKCS7) 解密。
// 输入应为原始密文的 base64（CryptoJS 以 WordArray 作为 key 时不带 "Salted__" 头）。
func aesDecryptPKCS5(ciphertextB64 string, key []byte) ([]byte, error) {
	if ciphertextB64 == "" {
		return nil, errors.New("empty ciphertext")
	}
	switch len(key) {
	case 16, 24, 32:
	default:
		return nil, fmt.Errorf("invalid key length %d (want 16/24/32)", len(key))
	}

	data, err := base64.StdEncoding.DecodeString(ciphertextB64)
	if err != nil {
		return nil, err
	}

	block, err := aes.NewCipher(key)
	if err != nil {
		return nil, err
	}
	blockSize := block.BlockSize()
	if len(data) == 0 || len(data)%blockSize != 0 {
		return nil, errors.New("invalid ciphertext length: not multiple of block size")
	}

	// iv 未显式给出时取 key 前 blockSize 字节，与旧 backend AesDecrypt 行为一致
	iv := key[:blockSize]
	blockMode := cipher.NewCBCDecrypter(block, iv)
	dec := make([]byte, len(data))
	blockMode.CryptBlocks(dec, data)

	// PKCS7/5 去填充
	if len(dec) == 0 {
		return nil, errors.New("empty plaintext")
	}
	pad := int(dec[len(dec)-1])
	if pad == 0 || pad > blockSize || pad > len(dec) {
		return nil, errors.New("invalid padding length")
	}
	for _, b := range dec[len(dec)-pad:] {
		if int(b) != pad {
			return nil, errors.New("invalid padding bytes")
		}
	}
	return dec[:len(dec)-pad], nil
}