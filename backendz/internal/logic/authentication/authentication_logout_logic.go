package authentication

import (
	"context"

	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"

	"github.com/zeromicro/go-zero/core/logx"
)

type AuthenticationLogoutLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewAuthenticationLogoutLogic(ctx context.Context, svcCtx *svc.ServiceContext) *AuthenticationLogoutLogic {
	return &AuthenticationLogoutLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *AuthenticationLogoutLogic) AuthenticationLogout() error {
	// 成功解析到身份即视为登出完成（JWT 无状态；如需服务端吊销可在此接入黑名单）。
	_, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		logx.WithContext(l.ctx).Infof("logout without authenticated claims")
	}
	return nil
}