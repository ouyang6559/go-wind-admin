package authentication

import (
	"context"
	"strings"

	"github.com/mojocn/base64Captcha"
	"github.com/zeromicro/go-zero/core/logx"

	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"
)

type AuthenticationGenerateCaptchaLogic struct {
	logx.Logger
	ctx    context.Context
	svcCtx *svc.ServiceContext
}

func NewAuthenticationGenerateCaptchaLogic(ctx context.Context, svcCtx *svc.ServiceContext) *AuthenticationGenerateCaptchaLogic {
	return &AuthenticationGenerateCaptchaLogic{
		Logger: logx.WithContext(ctx),
		ctx:    ctx,
		svcCtx: svcCtx,
	}
}

func (l *AuthenticationGenerateCaptchaLogic) AuthenticationGenerateCaptcha() (resp *types.GenerateCaptchaResponse, err error) {
	// 4 位数字验证码图片
	driver := base64Captcha.NewDriverDigit(80, 240, 4, 0.7, 80)
	c := base64Captcha.NewCaptcha(driver, base64Captcha.DefaultMemStore)

	id, b64s, rawAnswer, cerr := c.Generate()
	if cerr != nil || b64s == "" {
		logx.Errorf("generate captcha failed: %v", cerr)
		return nil, xerr.ServerErrorMsg("generate captcha failed")
	}
	// 库返回的 b64s 已可能带 data URI 前缀，避免重复拼接
	const prefix = "data:image/png;base64,"
	if !strings.HasPrefix(b64s, prefix) {
		b64s = prefix + b64s
	}

	// 答案落 Redis（验证码为纯数字，取其中纯数字部分）
	answer := digitOnly(rawAnswer)
	if err := l.svcCtx.CaptchaSave(id, answer); err != nil {
		logx.Errorf("save captcha failed: %v", err)
		return nil, xerr.ServerErrorMsg("generate captcha failed")
	}

	return &types.GenerateCaptchaResponse{
		CaptchaId:   id,
		ImageBase64: b64s, // b64s 已带 data URI 前缀（见第 40-43 行处理）
	}, nil
}

func digitOnly(s string) string {
	var sb strings.Builder
	for _, r := range s {
		if r >= '0' && r <= '9' {
			sb.WriteRune(r)
		}
	}
	return sb.String()
}