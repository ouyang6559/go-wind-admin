// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package admin_portal

import (
	"net/http"

	"github.com/zeromicro/go-zero/rest/httpx"
	xhttp "github.com/zeromicro/x/http"
	"go-wind-admin/backendz/internal/logic/admin_portal"
	"go-wind-admin/backendz/internal/svc"
)

func AdminPortalGetInitialContextHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		l := admin_portal.NewAdminPortalGetInitialContextLogic(r.Context(), svcCtx)
		resp, err := l.AdminPortalGetInitialContext()
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
		} else {
			httpx.OkJsonCtx(r.Context(), w, resp)
		}
	}
}
