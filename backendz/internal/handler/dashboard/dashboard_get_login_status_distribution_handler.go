// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package dashboard

import (
	"net/http"

	"github.com/zeromicro/go-zero/rest/httpx"
	xhttp "github.com/zeromicro/x/http"
	"go-wind-admin/backendz/internal/logic/dashboard"
	"go-wind-admin/backendz/internal/svc"
)

func DashboardGetLoginStatusDistributionHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		l := dashboard.NewDashboardGetLoginStatusDistributionLogic(r.Context(), svcCtx)
		resp, err := l.DashboardGetLoginStatusDistribution()
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
		} else {
			httpx.OkJsonCtx(r.Context(), w, resp)
		}
	}
}
