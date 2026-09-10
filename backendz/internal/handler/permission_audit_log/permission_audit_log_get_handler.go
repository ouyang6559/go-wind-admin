// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package permission_audit_log

import (
	"net/http"

	"github.com/zeromicro/go-zero/rest/httpx"
	xhttp "github.com/zeromicro/x/http"
	"go-wind-admin/backendz/internal/logic/permission_audit_log"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
)

func PermissionAuditLogGetHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var req types.PermissionAuditLogGetReq
		if err := httpx.Parse(r, &req); err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
			return
		}

		l := permission_audit_log.NewPermissionAuditLogGetLogic(r.Context(), svcCtx)
		resp, err := l.PermissionAuditLogGet(&req)
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
		} else {
			httpx.OkJsonCtx(r.Context(), w, resp)
		}
	}
}
