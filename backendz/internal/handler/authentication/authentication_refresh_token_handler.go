// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package authentication

import (
	"encoding/json"
	"net/http"

	"github.com/zeromicro/go-zero/rest/httpx"
	xhttp "github.com/zeromicro/x/http"
	"go-wind-admin/backendz/internal/logic/authentication"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
	"go-wind-admin/backendz/internal/xerr"
)

func AuthenticationRefreshTokenHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var req types.LoginRequest
		if err := httpx.Parse(r, &req); err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
			return
		}

		// 与 kratos backend 保持一致：仅接受 refresh_token 授权类型，且 refresh token
		// 一律从 HttpOnly cookie 读取（前端刷新请求体不含 refresh_token）。
		if req.GrantType != "refresh_token" {
			xhttp.JsonBaseResponseCtx(r.Context(), w, xerr.InvalidGrantTypeMsg())
			return
		}
		req.RefreshToken = middleware.ReadRefreshTokenCookie(r)

		l := authentication.NewAuthenticationRefreshTokenLogic(r.Context(), svcCtx)
		resp, err := l.AuthenticationRefreshToken(&req)
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
			return
		}
		// 刷新成功后轮换 refresh token 并重写 cookie（header 先于 body 落盘）
		middleware.SetRefreshCookies(w, r, resp.RefreshToken, resp.RefreshExpiresIn)
		// 与登录一致：平铺输出 {access_token, expires_in, ...}，不套 {code,msg,data} 外壳，
		// 否则前端 auth store 读取 response.access_token 取不到令牌。
		w.Header().Set("Content-Type", "application/json; charset=utf-8")
		if err := json.NewEncoder(w).Encode(resp); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	}
}
