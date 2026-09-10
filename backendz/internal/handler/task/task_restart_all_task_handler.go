// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package task

import (
	"net/http"

	"github.com/zeromicro/go-zero/rest/httpx"
	xhttp "github.com/zeromicro/x/http"
	"go-wind-admin/backendz/internal/logic/task"
	"go-wind-admin/backendz/internal/svc"
)

func TaskRestartAllTaskHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		l := task.NewTaskRestartAllTaskLogic(r.Context(), svcCtx)
		resp, err := l.TaskRestartAllTask()
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
		} else {
			httpx.OkJsonCtx(r.Context(), w, resp)
		}
	}
}
