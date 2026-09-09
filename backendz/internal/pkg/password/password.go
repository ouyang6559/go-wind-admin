// Package password 密码哈希工具（基于 bcrypt）。
package password

import "golang.org/x/crypto/bcrypt"

// Hash 对明文密码做 bcrypt 哈希。
func Hash(plain string) (string, error) {
	bytes, err := bcrypt.GenerateFromPassword([]byte(plain), bcrypt.DefaultCost)
	if err != nil {
		return "", err
	}
	return string(bytes), nil
}

// Verify 校验明文密码与 bcrypt 哈希是否匹配。
func Verify(hash, plain string) bool {
	return bcrypt.CompareHashAndPassword([]byte(hash), []byte(plain)) == nil
}