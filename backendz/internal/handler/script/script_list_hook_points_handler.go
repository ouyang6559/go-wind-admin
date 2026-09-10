// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package script

import (
	"net/http"

	"github.com/zeromicro/go-zero/rest/httpx"
	xhttp "github.com/zeromicro/x/http"
	"go-wind-admin/backendz/internal/logic/script"
	"go-wind-admin/backendz/internal/svc"
)

func ScriptListHookPointsHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		l := script.NewScriptListHookPointsLogic(r.Context(), svcCtx)
		resp, err := l.ScriptListHookPoints()
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
		} else {
			httpx.OkJsonCtx(r.Context(), w, resp)
		}
	}
}
