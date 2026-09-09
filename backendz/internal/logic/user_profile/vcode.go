// 用户画像模块的验证码与常量辅助。
package user_profile

import (
	"crypto/rand"
	"fmt"
	"math/big"
	"time"
)

const bindContactTTL = 10 * time.Minute

const bindContactKeyPrefix = "gowind:bind_contact:"

// dummyPasswordHash 合法 bcrypt 占位（EMAIL 凭证不用于密码校验）。
const dummyPasswordHash = "$2a$10$1sbpKmhQDpXLHnDnEQ1nLe3oOnYyP2bUJyqHcX2T0Fq1qfyoXOrPm"

// bindContactKey 构造绑定联系方式的验证码 Redis 键。
func bindContactKey(contact string) string { return bindContactKeyPrefix + contact }

// generateVCode 生成 6 位数字验证码；随机源异常时回退为全 0。
func generateVCode() string {
	n, err := rand.Int(rand.Reader, big.NewInt(1000000))
	if err != nil {
		return "000000"
	}
	return fmt.Sprintf("%06d", n.Int64())
}
