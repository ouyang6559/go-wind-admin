package mfa

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/usermfafactor"
	"go-wind-admin/backendz/internal/pkg/std"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type MfaVerifyMFAChallengeLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewMfaVerifyMFAChallengeLogic(ctx context.Context, svcCtx *svc.ServiceContext) *MfaVerifyMFAChallengeLogic {
	return &MfaVerifyMFAChallengeLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *MfaVerifyMFAChallengeLogic) MfaVerifyMFAChallenge(req *types.VerifyMFAChallengeRequest) (resp *types.LoginResponse, err error) {
	if req.OperationId == "" {
		return nil, xerr.BadRequestMsg("operationId required")
	}

	ch, cerr := peekLoginChallenge(l.ctx, l.svcCtx.Rds, req.OperationId)
	if cerr != nil {
		return nil, xerr.BadRequestMsg("mfa challenge not found or expired")
	}

	// 取用户已启用的 TOTP 因子进行校验。
	factor, ferr := l.svcCtx.Ent.UserMfaFactor.Query().
		Where(
			usermfafactor.TenantIDEQ(ch.TenantID),
			usermfafactor.UserIDEQ(ch.UserID),
			usermfafactor.MethodEQ(usermfafactor.MethodTotp),
			usermfafactor.StatusEQ(usermfafactor.StatusEnabled),
			usermfafactor.DeletedAtIsNil(),
		).First(l.ctx)
	if ferr != nil {
		return nil, xerr.BadRequestMsg("no mfa method enrolled")
	}

	secret := std.Str(factor.SecretHash)
	if req.TotpCode == "" || !validateTOTP(secret, req.TotpCode, time.Now(), 1) {
		return nil, xerr.BadRequestMsg("invalid totp code")
	}

	// 校验通过，记录使用时间并签发令牌。
	_, _ = l.svcCtx.Ent.UserMfaFactor.UpdateOneID(factor.ID).
		SetLastUsedAt(time.Now()).
		Save(l.ctx)

	accessToken, aerr := l.svcCtx.Token.CreateAccessToken(ch.UserID, ch.TenantID, ch.Username, ch.ClientType, "")
	if aerr != nil {
		return nil, xerr.ServerErrorMsg("create access token failed")
	}
	refreshToken, rerr := l.svcCtx.Token.CreateRefreshToken(ch.UserID, ch.TenantID, ch.Username, ch.ClientType, "")
	if rerr != nil {
		return nil, xerr.ServerErrorMsg("create refresh token failed")
	}

	deleteLoginChallenge(l.ctx, l.svcCtx.Rds, req.OperationId)
	return &types.LoginResponse{
		TokenType:        "Bearer",
		AccessToken:      accessToken,
		ExpiresIn:        l.svcCtx.Token.AccessExpiresIn(),
		RefreshToken:     refreshToken,
		RefreshExpiresIn: l.svcCtx.Token.RefreshExpiresIn(),
	}, nil
}