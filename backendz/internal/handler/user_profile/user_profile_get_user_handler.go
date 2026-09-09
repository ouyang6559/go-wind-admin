// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package user_profile

import (
	"net/http"

	xhttp "github.com/zeromicro/x/http"
	"go-wind-admin/backendz/internal/logic/user_profile"
	"go-wind-admin/backendz/internal/svc"
)

func UserProfileGetUserHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		l := user_profile.NewUserProfileGetUserLogic(r.Context(), svcCtx)
		resp, err := l.UserProfileGetUser()
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
		} else {
			xhttp.JsonBaseResponseCtx(r.Context(), w, resp)
		}
	}
}
