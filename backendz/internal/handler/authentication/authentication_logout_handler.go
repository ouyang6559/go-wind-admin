// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package authentication

import (
	"net/http"

	xhttp "github.com/zeromicro/x/http"
	"go-wind-admin/backendz/internal/logic/authentication"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
)

func AuthenticationLogoutHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		// 无论登出逻辑是否成功，都清除 refresh token 相关 cookie（header 先于 body 落盘）
		middleware.ClearRefreshCookies(w, r)
		l := authentication.NewAuthenticationLogoutLogic(r.Context(), svcCtx)
		err := l.AuthenticationLogout()
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
		} else {
			xhttp.JsonBaseResponseCtx(r.Context(), w, nil)
		}
	}
}
