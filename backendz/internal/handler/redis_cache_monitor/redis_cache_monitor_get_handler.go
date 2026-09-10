// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package redis_cache_monitor

import (
	"net/http"

	"github.com/zeromicro/go-zero/rest/httpx"
	xhttp "github.com/zeromicro/x/http"
	"go-wind-admin/backendz/internal/logic/redis_cache_monitor"
	"go-wind-admin/backendz/internal/svc"
)

func RedisCacheMonitorGetHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		l := redis_cache_monitor.NewRedisCacheMonitorGetLogic(r.Context(), svcCtx)
		resp, err := l.RedisCacheMonitorGet()
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
		} else {
			httpx.OkJsonCtx(r.Context(), w, resp)
		}
	}
}
