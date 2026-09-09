package authentication

import (
	"context"

	"github.com/zeromicro/go-zero/core/logx"

	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
)

type AuthenticationVerifyCaptchaLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewAuthenticationVerifyCaptchaLogic(ctx context.Context, svcCtx *svc.ServiceContext) *AuthenticationVerifyCaptchaLogic {
	return &AuthenticationVerifyCaptchaLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *AuthenticationVerifyCaptchaLogic) AuthenticationVerifyCaptcha(req *types.VerifyCaptchaRequest) (resp *types.VerifyCaptchaResponse, err error) {
	saved := l.svcCtx.CaptchaVerify(req.CaptchaId, req.UserInput)
	return &types.VerifyCaptchaResponse{Valid: saved}, nil
}