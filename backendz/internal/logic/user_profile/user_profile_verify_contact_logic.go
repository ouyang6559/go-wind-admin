// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package user_profile

import (
	"context"
	"strings"

	"go-wind-admin/backendz/internal/ent/gen/usercredential"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type UserProfileVerifyContactLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserProfileVerifyContactLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserProfileVerifyContactLogic {
	return &UserProfileVerifyContactLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserProfileVerifyContactLogic) UserProfileVerifyContact(req *types.VerifyContactRequest) error {
	c, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return xerr.UnauthorizedMsg("unauthorized")
	}

	email := strings.TrimSpace(req.Email.Email)
	code := strings.TrimSpace(req.Email.Code)
	if email == "" || code == "" {
		return xerr.BadRequestMsg("contact and code are required")
	}

	stored, err := l.svcCtx.Rds.GetCtx(l.ctx, bindContactKey(email))
	if err != nil || stored == "" || stored != code {
		return xerr.BadRequestMsg("invalid or expired verification code")
	}
	l.svcCtx.Rds.DelCtx(l.ctx, bindContactKey(email))

	// 写入 EMAIL 登录凭证（identifier=邮箱；credential 为占位哈希，不用于密码校验）
	if _, cerr := l.svcCtx.Ent.UserCredential.Create().
		SetTenantID(c.TenantID).
		SetUserID(c.UserID).
		SetIdentityType(usercredential.IdentityTypeEmail).
		SetIdentifier(email).
		SetCredentialType(usercredential.CredentialTypePasswordHash).
		SetCredential(dummyPasswordHash).
		SetIsPrimary(false).
		SetStatus(usercredential.StatusEnabled).
		Save(l.ctx); cerr != nil {
		logx.WithContext(l.ctx).Errorf("bind contact: create email credential failed: %v", cerr)
		return xerr.ServerErrorMsg("bind email failed")
	}
	return nil
}
