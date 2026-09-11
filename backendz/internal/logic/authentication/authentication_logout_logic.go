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
	var userID uint32
	var username string
	claims, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		logx.WithContext(l.ctx).Infof("logout without authenticated claims")
	} else {
		userID = claims.UserID
		username = claims.Username
		// 吊销当前会话（按 jti 删除令牌对会话记录），使旧 access/refresh token 失效。
		// 与我的会话/踢下线/重置密码共用同一会话注册表，语义一致。
		if claims.ID != "" {
			if err := l.svcCtx.Session.Revoke(l.ctx, userID, claims.ID); err != nil {
				logx.WithContext(l.ctx).Errorf("revoke session for user [%d] jti [%s] failed: %v", userID, claims.ID, err)
			}
		}
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
