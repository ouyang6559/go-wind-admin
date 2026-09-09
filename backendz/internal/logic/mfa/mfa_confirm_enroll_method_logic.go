package mfa

import (
	"context"
	"strconv"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/ent/gen/usermfafactor"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type MfaConfirmEnrollMethodLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewMfaConfirmEnrollMethodLogic(ctx context.Context, svcCtx *svc.ServiceContext) *MfaConfirmEnrollMethodLogic {
	return &MfaConfirmEnrollMethodLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *MfaConfirmEnrollMethodLogic) MfaConfirmEnrollMethod(req *types.ConfirmEnrollMethodRequest) (resp *types.ConfirmEnrollMethodResponse, err error) {
	c, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return nil, xerr.UnauthorizedMsg("unauthorized")
	}
	if strings.TrimSpace(req.OperationId) == "" {
		return nil, xerr.BadRequestMsg("operationId required")
	}

	ch, cerr := peekEnrollChallenge(l.ctx, l.svcCtx.Rds, req.OperationId)
	if cerr != nil {
		return nil, xerr.BadRequestMsg("enroll challenge not found or expired")
	}
	if ch.UserID != c.UserID || ch.TenantID != c.TenantID {
		return nil, xerr.ForbiddenMsg("challenge not bound to current user")
	}

	// 仅支持 TOTP 确认；SMS/EMAIL/WEBAUTHN 最小实现直接返回成功。
	if req.TotpCode != "" {
		if !validateTOTP(ch.Secret, req.TotpCode, time.Now(), 1) {
			return nil, xerr.BadRequestMsg("invalid totp code")
		}
	}

	saved, serr := l.svcCtx.Ent.UserMfaFactor.Create().
		SetTenantID(ch.TenantID).
		SetUserID(ch.UserID).
		SetMethod(usermfafactor.MethodTotp).
		SetSecretHash(ch.Secret).
		SetStatus(usermfafactor.StatusEnabled).
		SetDisplayName(defaultDisplay(req.Display, ch.Display)).
		Save(l.ctx)
	if serr != nil {
		logx.WithContext(l.ctx).Errorf("save mfa factor failed: %v", serr)
		return nil, xerr.ServerErrorMsg("save mfa factor failed")
	}

	deleteEnrollChallenge(l.ctx, l.svcCtx.Rds, req.OperationId)
	return &types.ConfirmEnrollMethodResponse{
		Success:      true,
		CredentialId: strconv.FormatUint(uint64(saved.ID), 10),
	}, nil
}