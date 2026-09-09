// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package user_profile

import (
	"context"
	"strings"

	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type UserProfileBindContactLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewUserProfileBindContactLogic(ctx context.Context, svcCtx *svc.ServiceContext) *UserProfileBindContactLogic {
	return &UserProfileBindContactLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *UserProfileBindContactLogic) UserProfileBindContact(req *types.BindContactRequest) error {
	// 当前仅支持邮箱绑定；二期接入短信后扩展 MOBILE。
	email := strings.TrimSpace(req.Email.Email)
	if email == "" {
		return xerr.BadRequestMsg("only email binding is supported")
	}

	code := generateVCode()
	if err := l.svcCtx.Rds.SetexCtx(l.ctx, bindContactKey(email), code, int(bindContactTTL.Seconds())); err != nil {
		logx.WithContext(l.ctx).Errorf("save bind contact vcode failed: %v", err)
		return xerr.ServerErrorMsg("save verification code failed")
	}
	// 网关未接入邮件渠道：验证码已写入 Redis，实际邮件投递由后续接入的邮件模块承担。
	logx.WithContext(l.ctx).Infof("bind contact: email [%s] vcode issued", email)
	return nil
}
