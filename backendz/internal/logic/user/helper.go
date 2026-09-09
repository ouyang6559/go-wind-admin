// 用户模块共享的凭证与删除辅助逻辑。
package user

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen"
	"go-wind-admin/backendz/internal/ent/gen/usercredential"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/pkg/password"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

// resetCredentialUsername 将指定用户的「用户名」登录凭证的密码哈希重置为新口令。
func resetCredentialUsername(ctx context.Context, svcCtx *svc.ServiceContext, username *string, plain string) error {
	if username == nil || *username == "" {
		return xerr.BadRequestMsg("username is required")
	}
	if plain == "" {
		return xerr.BadRequestMsg("new password is required")
	}
	hash, err := password.Hash(plain)
	if err != nil {
		logx.WithContext(ctx).Errorf("hash password failed: %v", err)
		return xerr.ServerErrorMsg("hash password failed")
	}
	if _, err := svcCtx.Ent.UserCredential.Update().
		Where(
			usercredential.IdentifierEQ(*username),
			usercredential.IdentityTypeEQ(usercredential.IdentityTypeUsername),
			usercredential.DeletedAtIsNil(),
		).
		SetCredential(hash).
		SetUpdatedAt(time.Now()).
		Save(ctx); err != nil {
		logx.WithContext(ctx).Errorf("reset credential failed: %v", err)
		return xerr.ServerErrorMsg("reset user password failed")
	}
	return nil
}

// softDeleteUser 软删除用户：禁止删除平台管理员(id=1)与自身。
func softDeleteUser(ctx context.Context, svcCtx *svc.ServiceContext, u *gen.User) error {
	if u.ID == 1 {
		return xerr.BadRequestMsg("default admin cannot be deleted")
	}
	var operatorID uint32
	if c, ok := middleware.ClaimsFromContext(ctx); ok {
		operatorID = c.UserID
		if u.ID == c.UserID {
			return xerr.BadRequestMsg("cannot delete yourself")
		}
	}
	now := time.Now()
	if err := svcCtx.Ent.User.UpdateOneID(u.ID).
		SetDeletedAt(now).
		SetDeletedBy(operatorID).
		Exec(ctx); err != nil {
		logx.WithContext(ctx).Errorf("soft delete user failed: %v", err)
		return xerr.ServerErrorMsg("soft delete user failed")
	}
	return nil
}
