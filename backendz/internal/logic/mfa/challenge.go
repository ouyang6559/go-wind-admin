package mfa

import (
	"context"
	"encoding/json"
	"fmt"

	"github.com/google/uuid"
	"github.com/zeromicro/go-zero/core/stores/redis"
)

// MFA 挑战/注册上下文缓存，基于 svcCtx.Rds（go-zero redis）。
// 所有操作单次有效（取即删/注册成功后删）。

const (
	// mfaChallengeTTL 注册/登录挑战上下文有效期（秒）。
	mfaChallengeTTL = 300
	enrollKeyFmt    = "mfa:enroll:%s"
	loginKeyFmt     = "mfa:login:%s"
)

// enrollChallenge 注册上下文：StartEnrollMethod 写入，ConfirmEnrollMethod 取出校验首码。
type enrollChallenge struct {
	Secret   string
	TenantID uint32
	UserID   uint32
	Display  string
}

// loginChallenge 登录挑战上下文：VerifyMFAChallenge 读取，验证通过后签发 token。
type loginChallenge struct {
	UserID     uint32
	TenantID   uint32
	Username   string
	ClientType string
}

func setEnrollChallenge(ctx context.Context, rds *redis.Redis, c *enrollChallenge) (string, error) {
	opID := uuid.NewString()
	raw, err := json.Marshal(c)
	if err != nil {
		return "", err
	}
	key := fmt.Sprintf(enrollKeyFmt, opID)
	if err := rds.SetexCtx(ctx, key, string(raw), mfaChallengeTTL); err != nil {
		return "", err
	}
	return opID, nil
}

func peekEnrollChallenge(ctx context.Context, rds *redis.Redis, opID string) (*enrollChallenge, error) {
	raw, err := rds.GetCtx(ctx, fmt.Sprintf(enrollKeyFmt, opID))
	if err != nil || raw == "" {
		return nil, fmt.Errorf("enroll challenge not found or expired")
	}
	var c enrollChallenge
	if err := json.Unmarshal([]byte(raw), &c); err != nil {
		return nil, err
	}
	return &c, nil
}

func deleteEnrollChallenge(ctx context.Context, rds *redis.Redis, opID string) {
	_, _ = rds.DelCtx(ctx, fmt.Sprintf(enrollKeyFmt, opID))
}

func setLoginChallenge(ctx context.Context, rds *redis.Redis, c *loginChallenge) (string, error) {
	opID := uuid.NewString()
	raw, err := json.Marshal(c)
	if err != nil {
		return "", err
	}
	if err := rds.SetexCtx(ctx, fmt.Sprintf(loginKeyFmt, opID), string(raw), mfaChallengeTTL); err != nil {
		return "", err
	}
	return opID, nil
}

func peekLoginChallenge(ctx context.Context, rds *redis.Redis, opID string) (*loginChallenge, error) {
	raw, err := rds.GetCtx(ctx, fmt.Sprintf(loginKeyFmt, opID))
	if err != nil || raw == "" {
		return nil, fmt.Errorf("login challenge not found or expired")
	}
	var c loginChallenge
	if err := json.Unmarshal([]byte(raw), &c); err != nil {
		return nil, err
	}
	return &c, nil
}

func deleteLoginChallenge(ctx context.Context, rds *redis.Redis, opID string) {
	_, _ = rds.DelCtx(ctx, fmt.Sprintf(loginKeyFmt, opID))
}