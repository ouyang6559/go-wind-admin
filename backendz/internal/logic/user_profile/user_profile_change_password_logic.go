// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package user_profile

import (
	"context"

	"go-wind-admin/backendz/internal/ent/gen/usercredential"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/pkg/password"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type UserProfileChangePasswordLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserProfileChangePasswordLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserProfileChangePasswordLogic {
	return &UserProfileChangePasswordLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserProfileChangePasswordLogic) UserProfileChangePassword(req *types.ChangePasswordRequest) error {
	c, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return xerr.UnauthorizedMsg("unauthorized")
	}
	if req.OldPassword == "" || req.NewPassword == "" {
		return xerr.BadRequestMsg("old and new password required")
	}

	// 取当前用户的「用户名」主凭证校验旧密码
	cred, err := l.svcCtx.Ent.UserCredential.Query().
		Where(
			usercredential.UserIDEQ(c.UserID),
			usercredential.IdentityTypeEQ(usercredential.IdentityTypeUsername),
			usercredential.StatusEQ(usercredential.StatusEnabled),
			usercredential.DeletedAtIsNil(),
		).
		First(l.ctx)
	if err != nil {
		logx.WithContext(l.ctx).Errorf("get current credential failed: %v", err)
		return xerr.InvalidPasswordMsg()
	}
	if cred.Credential == nil || !password.Verify(*cred.Credential, req.OldPassword) {
		return xerr.InvalidPasswordMsg()
	}

	hash, herr := password.Hash(req.NewPassword)
	if herr != nil {
		logx.WithContext(l.ctx).Errorf("hash new password failed: %v", herr)
		return xerr.ServerErrorMsg("hash new password failed")
	}
	if _, uerr := l.svcCtx.Ent.UserCredential.UpdateOneID(cred.ID).
		SetCredential(hash).
		Save(l.ctx); uerr != nil {
		logx.WithContext(l.ctx).Errorf("update credential failed: %v", uerr)
		return xerr.ServerErrorMsg("change password failed")
	}
	return nil
}
