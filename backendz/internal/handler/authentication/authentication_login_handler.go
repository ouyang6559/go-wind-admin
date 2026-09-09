// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package authentication

import (
	"encoding/json"
	"net/http"

	"github.com/zeromicro/go-zero/rest/httpx"
	xhttp "github.com/zeromicro/x/http"
	"go-wind-admin/backendz/internal/logic/authentication"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
)

func AuthenticationLoginHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var req types.LoginRequest
		if err := httpx.Parse(r, &req); err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
			return
		}

		l := authentication.NewAuthenticationLoginLogic(r.Context(), svcCtx)
		resp, err := l.AuthenticationLogin(&req)
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
			return
		}
		// 与 backend（kratos）保持一致：登录成功时平铺输出 {token_type, access_token, ...}，
		// 不再套 {code,msg,data} 外壳，否则前端 auth.ts 读取 response.access_token 取不到令牌。
		w.Header().Set("Content-Type", "application/json; charset=utf-8")
		if err := json.NewEncoder(w).Encode(resp); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	}
}
