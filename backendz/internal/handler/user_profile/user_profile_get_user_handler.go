// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package user_profile

import (
	"encoding/json"
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
			return
		}
		// 与 backend（kratos）保持一致：GET /admin/v1/me 平铺输出 user 对象，
		// 不再套 {code,msg,data} 外壳，否则前端 fetchUserProfile 读不到 uid/roles/homePath。
		w.Header().Set("Content-Type", "application/json; charset=utf-8")
		if err := json.NewEncoder(w).Encode(resp); err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
		}
	}
}
