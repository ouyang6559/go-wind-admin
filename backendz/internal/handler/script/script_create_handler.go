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

func ScriptCreateHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var req types.CreateScriptRequest
		if err := httpx.Parse(r, &req); err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
			return
		}

		l := script.NewScriptCreateLogic(r.Context(), svcCtx)
		err := l.ScriptCreate(&req)
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
		} else {
			xhttp.JsonBaseResponseCtx(r.Context(), w, nil)
		}
	}
}
