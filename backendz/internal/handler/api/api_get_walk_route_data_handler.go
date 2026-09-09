// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package api

import (
	"net/http"

	xhttp "github.com/zeromicro/x/http"
	"go-wind-admin/backendz/internal/logic/api"
	"go-wind-admin/backendz/internal/svc"
)

func ApiGetWalkRouteDataHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		l := api.NewApiGetWalkRouteDataLogic(r.Context(), svcCtx)
		resp, err := l.ApiGetWalkRouteData()
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
		} else {
			xhttp.JsonBaseResponseCtx(r.Context(), w, resp)
		}
	}
}
