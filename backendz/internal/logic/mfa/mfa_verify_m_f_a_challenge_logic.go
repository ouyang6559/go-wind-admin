package mfa

import (
	"context"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/usermfafactor"
	"go-wind-admin/backendz/internal/pkg/session"
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

	accessToken, refreshToken, jti, terr := l.svcCtx.Token.CreateTokenPair(ch.UserID, ch.TenantID, ch.Username, ch.ClientType, "")
	if terr != nil {
		return nil, xerr.ServerErrorMsg("create token pair failed")
	}
	// 与登录/刷新一致：MFA 通过即视为一次登录，写入在线会话注册表（best-effort）。
	// 若不复用统一 jti + 会话记录，令牌对会被鉴权中间件的会话吊销检查误判为已下线。
	if serr := l.svcCtx.Session.Record(l.ctx, session.Meta{
		UID:        ch.UserID,
		TenantID:   ch.TenantID,
		Username:   ch.Username,
		JTI:        jti,
		ClientType: ch.ClientType,
		LoginAt:    time.Now(),
	}); serr != nil {
		logx.WithContext(l.ctx).Errorf("record mfa session for user [%d] failed: %v", ch.UserID, serr)
	}

	deleteLoginChallenge(l.ctx, l.svcCtx.Rds, req.OperationId)

	// MFA 通过即视为登录成功，与密码登录路径一致记录最近登录信息（对齐 backend
	// 3963e55c 两路径语义；best-effort，失败不阻断发令牌）。
	_, _ = l.svcCtx.Ent.User.UpdateOneID(ch.UserID).
		SetLastLoginAt(time.Now()).
		SetLastLoginIP(mfaClientIP(l.ctx)).
		Save(l.ctx)
	return &types.LoginResponse{
		TokenType:        "Bearer",
		AccessToken:      accessToken,
		ExpiresIn:        l.svcCtx.Token.AccessExpiresIn(),
		RefreshToken:     refreshToken,
		RefreshExpiresIn: l.svcCtx.Token.RefreshExpiresIn(),
	}, nil
}
