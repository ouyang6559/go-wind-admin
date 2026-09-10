// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package script

import (
	"net/http"

	"github.com/zeromicro/go-zero/rest/httpx"
	xhttp "github.com/zeromicro/x/http"
	"go-wind-admin/backendz/internal/logic/script"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
)

func ScriptCountHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var req types.PageRequest
		if err := httpx.Parse(r, &req); err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
			return
		}

		l := script.NewScriptCountLogic(r.Context(), svcCtx)
		resp, err := l.ScriptCount(&req)
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
		} else {
			httpx.OkJsonCtx(r.Context(), w, resp)
		}
	}
}
