package authentication

import (
	"context"
	"net"

	"go-wind-admin/backendz/internal/auditlog"
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
	var userID uint32
	var username string
	claims, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		logx.WithContext(l.ctx).Infof("logout without authenticated claims")
	} else {
		userID = claims.UserID
		username = claims.Username
	}

	// 登出审计（best-effort，失败仅记日志不阻断登出）。
	auditlog.WriteLogout(l.ctx, l.svcCtx, userID, username, l.clientIP())
	return nil
}

func (l *AuthenticationLogoutLogic) clientIP() string {
	r, ok := middleware.RequestFromContext(l.ctx)
	if !ok {
		return ""
	}
	if host, _, err := net.SplitHostPort(r.RemoteAddr); err == nil {
		return host
	}
	return r.RemoteAddr
}