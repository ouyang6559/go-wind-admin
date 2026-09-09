package authentication

import (
	"context"
	"strconv"
	"strings"

	"go-wind-admin/backendz/internal/ent/gen/user"
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

	accessToken, aerr := l.svcCtx.Token.CreateAccessToken(u.ID, *u.TenantID, usernameVal, req.ClientId, req.DeviceId)
	if aerr != nil {
		return nil, xerr.ServerErrorMsg("create access token failed")
	}
	newRefresh, rerr := l.svcCtx.Token.CreateRefreshToken(u.ID, *u.TenantID, usernameVal, req.ClientId, req.DeviceId)
	if rerr != nil {
		return nil, xerr.ServerErrorMsg("create refresh token failed")
	}

	return &types.LoginResponse{
		TokenType:        "Bearer",
		AccessToken:      accessToken,
		ExpiresIn:        strconv.FormatInt(l.svcCtx.Token.AccessExpiresIn(), 10),
		RefreshToken:     newRefresh,
		RefreshExpiresIn: strconv.FormatInt(l.svcCtx.Token.RefreshExpiresIn(), 10),
	}, nil
}