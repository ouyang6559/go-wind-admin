// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package authentication

import (
	"encoding/json"
	"net/http"

	xhttp "github.com/zeromicro/x/http"
	"go-wind-admin/backendz/internal/logic/authentication"
	"go-wind-admin/backendz/internal/svc"
)

func AuthenticationGenerateCaptchaHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		l := authentication.NewAuthenticationGenerateCaptchaLogic(r.Context(), svcCtx)
		resp, err := l.AuthenticationGenerateCaptcha()
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
			return
		}
		// 与 backend（kratos）保持一致：验证码接口成功时平铺输出 {captchaId, imageBase64}，
		// 不再套 {code,msg,data} 外壳，前端能直接解包 data（见 docs/backendz-frontend-接口格式审计.md P0-1 单接口修复）
		w.Header().Set("Content-Type", "application/json; charset=utf-8")
		if err := json.NewEncoder(w).Encode(resp); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	}
}
