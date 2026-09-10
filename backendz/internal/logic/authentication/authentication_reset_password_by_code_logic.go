// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package authentication

import (
	"context"
	"strings"
	"time"

	"entgo.io/ent/privacy"

	"go-wind-admin/backendz/internal/ent/gen/usercredential"
	"go-wind-admin/backendz/internal/pkg/credential"
	"go-wind-admin/backendz/internal/pkg/password"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type AuthenticationResetPasswordByCodeLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewAuthenticationResetPasswordByCodeLogic(ctx context.Context, svcCtx *svc.ServiceContext) *AuthenticationResetPasswordByCodeLogic {
	return &AuthenticationResetPasswordByCodeLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

// AuthenticationResetPasswordByCode 凭邮箱验证码重置密码（免鉴权）。
// 校验通过后更新该用户 USERNAME 凭证的密码，并吊销其全部在线会话。
func (l *AuthenticationResetPasswordByCodeLogic) AuthenticationResetPasswordByCode(req *types.ResetPasswordByCodeRequest) error {
	identifier := strings.TrimSpace(req.Identifier)
	code := strings.TrimSpace(req.Code)
	newPassword := req.NewPassword
	if identifier == "" || code == "" || newPassword == "" {
		return xerr.BadRequestMsg("identifier, code and new password are required")
	}

	// 单次有效验证码校验（比对失败同样删除，杜绝暴力重试）
	if !l.svcCtx.VCodeVerify(svc.VCodePurposeResetPassword, identifier, code) {
		return xerr.BadRequestMsg("invalid or expired verification code")
	}

	allowCtx := privacy.DecisionContext(l.ctx, privacy.Allow)

	// 通过 EMAIL 凭证反查用户 ID
	emailCred, cerr := l.svcCtx.Ent.UserCredential.Query().
		Where(
			usercredential.IdentityTypeEQ(usercredential.IdentityTypeEmail),
			usercredential.IdentifierEQ(identifier),
		).
		Only(allowCtx)
	if cerr != nil || emailCred.UserID == nil || *emailCred.UserID == 0 {
		return xerr.BadRequestMsg("invalid or expired verification code")
	}
	userID := *emailCred.UserID

	u, uerr := l.svcCtx.Ent.User.Get(allowCtx, userID)
	if uerr != nil || u.Username == nil || *u.Username == "" {
		return xerr.BadRequestMsg("invalid or expired verification code")
	}

	// 新密码 AES 密文传输，与登录同规（mock/内部测试可直接提交明文）
	plainPassword := credential.ResolvePlainPassword(newPassword)
	if plainPassword == "" {
		return xerr.BadRequestMsg("new password is empty")
	}
	hash, herr := password.Hash(plainPassword)
	if herr != nil {
		logx.WithContext(l.ctx).Errorf("reset password by code: hash new password for user [%d] failed: %v", userID, herr)
		return xerr.ServerErrorMsg("hash password failed")
	}

	// 更新该用户 USERNAME 凭证的密码（重置路径以用户名为标识落库）
	affected, uerr := l.svcCtx.Ent.UserCredential.Update().
		Where(
			usercredential.IdentityTypeEQ(usercredential.IdentityTypeUsername),
			usercredential.IdentifierEQ(*u.Username),
		).
		SetCredential(hash).
		SetUpdatedAt(time.Now()).
		Save(allowCtx)
	if uerr != nil {
		logx.WithContext(l.ctx).Errorf("reset password by code: update username credential for user [%d] failed: %v", userID, uerr)
		return xerr.ServerErrorMsg("reset password failed")
	}
	if affected == 0 {
		logx.WithContext(l.ctx).Errorf("reset password by code: no username credential matched for user [%d]", userID)
		return xerr.ServerErrorMsg("reset password failed")
	}

	// 重置后吊销该用户全部在线会话（best-effort：失败仅记录日志，不阻断重置结果）
	if n, serr := l.svcCtx.Session.RevokeByUser(l.ctx, userID); serr != nil {
		logx.WithContext(l.ctx).Errorf("revoke sessions after reset-by-code for user [%d] failed: %v", userID, serr)
	} else {
		logx.WithContext(l.ctx).Infof("revoke %d session(s) after reset-by-code for user [%d]", n, userID)
	}

	logx.WithContext(l.ctx).Infof("password reset by code done for user [%d]", userID)
	return nil
}