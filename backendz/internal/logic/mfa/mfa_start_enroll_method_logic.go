package mfa

import (
	"context"
	"strconv"
	"strings"
	"time"

	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"

	"github.com/zeromicro/go-zero/core/logx"
)

type MfaStartEnrollMethodLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewMfaStartEnrollMethodLogic(ctx context.Context, svcCtx *svc.ServiceContext) *MfaStartEnrollMethodLogic {
	return &MfaStartEnrollMethodLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *MfaStartEnrollMethodLogic) MfaStartEnrollMethod(req *types.StartEnrollMethodRequest) (resp *types.StartEnrollMethodResponse, err error) {
	c, ok := middleware.ClaimsFromContext(l.ctx)
	if !ok {
		return nil, xerr.UnauthorizedMsg("unauthorized")
	}

	method := strings.ToUpper(strings.TrimSpace(req.Method))
	if method == "" {
		method = "TOTP"
	}
	if method != "TOTP" {
		// 非 TOTP（SMS/EMAIL/WEBAUTHN）在该网关最小实现下仅登记 context，不真正外呼。
		opID, cerr := setEnrollChallenge(l.ctx, l.svcCtx.Rds, &enrollChallenge{
			TenantID: c.TenantID,
			UserID:   c.UserID,
			Display:  method,
		})
		if cerr != nil {
			logx.WithContext(l.ctx).Errorf("store enroll challenge failed: %v", cerr)
			return nil, xerr.ServerErrorMsg("store enroll challenge failed")
		}
		return &types.StartEnrollMethodResponse{
			OperationId: opID,
			ExpiresAt:   time.Now().Add(mfaChallengeTTL * time.Second).Format(time.RFC3339),
		}, nil
	}

	secret, serr := generateTOTPSecret()
	if serr != nil {
		return nil, xerr.ServerErrorMsg("generate totp secret failed")
	}
	account := strconv.FormatUint(uint64(c.UserID), 10)
	opID, cerr := setEnrollChallenge(l.ctx, l.svcCtx.Rds, &enrollChallenge{
		Secret:   secret,
		TenantID: c.TenantID,
		UserID:   c.UserID,
		Display:  "TOTP",
	})
	if cerr != nil {
		logx.WithContext(l.ctx).Errorf("store enroll challenge failed: %v", cerr)
		return nil, xerr.ServerErrorMsg("store enroll challenge failed")
	}

	return &types.StartEnrollMethodResponse{
		Totp: types.TOTPResult{
			Secret:     secret,
			OtpAuthUrl: otpAuthURL(mfaIssuer, account, secret),
		},
		OperationId: opID,
		ExpiresAt:   time.Now().Add(mfaChallengeTTL * time.Second).Format(time.RFC3339),
	}, nil
}