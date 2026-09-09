// Package pkassist 应用启动辅助：种子初始化（默认管理员）。
package pkassist

import (
	"context"

	"entgo.io/ent/privacy"
	gocrudviewer "github.com/tx7do/go-crud/viewer"

	"go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/ent/gen/usercredential"
	"go-wind-admin/backendz/internal/pkg/password"
	"go-wind-admin/backendz/internal/pkg/viewer"
	"go-wind-admin/backendz/internal/svc"
)

// Seed 初始化默认数据：若系统不存在 admin 用户则创建（admin / admin）。幂等可重复调用。
func Seed(s *svc.ServiceContext) error {
	// 平台上下文：使 tenant 隐私策略放行（IsPlatformContext），否则缺 viewer 会 fail-closed。
	ctx := gocrudviewer.WithContext(context.Background(), viewer.Default)

	exists, err := s.Ent.User.Query().Where(user.UsernameEQ("admin")).Exist(ctx)
	if err != nil {
		return err
	}
	if exists {
		return nil
	}

	hash, err := password.Hash("admin")
	if err != nil {
		return err
	}

	// 种子写入注入 Allow 决策上下文，避免被 schema 策略拦截。
	allowCtx := privacy.DecisionContext(ctx, privacy.Allow)

	u, err := s.Ent.User.Create().
		SetTenantID(0).
		SetUsername("admin").
		SetNickname("Administrator").
		SetStatus(user.StatusNormal).
		Save(allowCtx)
	if err != nil {
		return err
	}

	_, err = s.Ent.UserCredential.Create().
		SetTenantID(0).
		SetUserID(u.ID).
		SetIdentityType(usercredential.IdentityTypeUsername).
		SetIdentifier("admin").
		SetCredentialType(usercredential.CredentialTypePasswordHash).
		SetCredential(hash).
		SetIsPrimary(true).
		SetStatus(usercredential.StatusEnabled).
		Save(allowCtx)
	return err
}