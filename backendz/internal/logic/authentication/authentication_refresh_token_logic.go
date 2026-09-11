package authentication

import (
	"context"
	"net"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/user"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/pkg/session"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type AuthenticationRefreshTokenLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewAuthenticationRefreshTokenLogic(ctx context.Context, svcCtx *svc.ServiceContext) *AuthenticationRefreshTokenLogic {
	return &AuthenticationRefreshTokenLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *AuthenticationRefreshTokenLogic) AuthenticationRefreshToken(req *types.LoginRequest) (resp *types.LoginResponse, err error) {
	refreshToken := strings.TrimSpace(req.RefreshToken)
	if refreshToken == "" {
		return nil, xerr.IncorrectRefreshTokenMsg()
	}

	claim, err := l.svcCtx.Token.ParseRefreshToken(refreshToken)
	if err != nil {
		return nil, xerr.IncorrectRefreshTokenMsg()
	}

	// 会话吊销检查：登出/踢下线会删除会话记录，refresh token 随之失效，不再换发新令牌。
	// 与登录写入一致的 best-effort 语义：Redis 异常时 fail-open（仅记日志），
	// 仅明确「会话不存在」时拒绝。
	if claim.ID != "" {
		exists, serr := l.svcCtx.Session.Exists(l.ctx, claim.UserID, claim.ID)
		if serr != nil {
			logx.WithContext(l.ctx).Errorf("check refresh session for user [%d] jti [%s] failed: %v", claim.UserID, claim.ID, serr)
		} else if !exists {
			return nil, xerr.IncorrectRefreshTokenMsg()
		}
	}

	u, uerr := l.svcCtx.Ent.User.Get(l.ctx, claim.UserID)
	if uerr != nil {
		return nil, xerr.IncorrectRefreshTokenMsg()
	}
	if u.Status == nil || *u.Status != user.StatusNormal {
		return nil, xerr.ForbiddenMsg("user is disabled")
	}

	if u.TenantID == nil {
		return nil, xerr.ForbiddenMsg("tenant missing")
	}

	usernameVal := ""
	if u.Username != nil {
		usernameVal = *u.Username
	}

	accessToken, newRefresh, jti, terr := l.svcCtx.Token.CreateTokenPair(u.ID, *u.TenantID, usernameVal, req.ClientId, req.DeviceId)
	if terr != nil {
		return nil, xerr.ServerErrorMsg("create token pair failed")
	}
	clientType := req.ClientType
	if strings.TrimSpace(clientType) == "" {
		clientType = req.ClientId
	}

	// 刷新轮换视为一次新会话：写入在线会话注册表（best-effort，失败仅记日志）
	if serr := l.svcCtx.Session.Record(l.ctx, session.Meta{
		UID:        u.ID,
		TenantID:   *u.TenantID,
		Username:   usernameVal,
		JTI:        jti,
		ClientType: clientType,
		IP:         l.clientIP(),
		UserAgent:  l.userAgent(),
		DeviceID:   req.DeviceId,
		LoginAt:    time.Now(),
	}); serr != nil {
		logx.WithContext(l.ctx).Errorf("record refresh session for user [%d] failed: %v", u.ID, serr)
	}

	return &types.LoginResponse{
		TokenType:        "Bearer",
		AccessToken:      accessToken,
		ExpiresIn:        l.svcCtx.Token.AccessExpiresIn(),
		RefreshToken:     newRefresh,
		RefreshExpiresIn: l.svcCtx.Token.RefreshExpiresIn(),
	}, nil
}

func (l *AuthenticationRefreshTokenLogic) clientIP() string {
	r, ok := middleware.RequestFromContext(l.ctx)
	if !ok {
		return ""
	}
	if host, _, err := net.SplitHostPort(r.RemoteAddr); err == nil {
		return host
	}
	return r.RemoteAddr
}

func (l *AuthenticationRefreshTokenLogic) userAgent() string {
	if r, ok := middleware.RequestFromContext(l.ctx); ok {
		return r.UserAgent()
	}
	return ""
}
