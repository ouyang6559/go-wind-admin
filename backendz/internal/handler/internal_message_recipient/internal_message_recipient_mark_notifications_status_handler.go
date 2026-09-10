// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2

package internal_message_recipient

import (
	"net/http"

	"github.com/zeromicro/go-zero/rest/httpx"
	xhttp "github.com/zeromicro/x/http"
	"go-wind-admin/backendz/internal/logic/internal_message_recipient"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/types"
)

// 成功响应与 Kratos 主后端对齐：直接返回扁平 proto 消息体（如 {"items":[],"total":"0"}），
// 不包 {code,msg,data} 信封，保证 react/vue-element/vue-vben 三层前端零改动。
// 仅错误路径通过 xhttp 输出 {code,msg}。

func InternalMessageRecipientMarkNotificationsStatusHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var req types.MarkNotificationsStatusRequest
		if err := httpx.Parse(r, &req); err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
			return
		}

		l := internal_message_recipient.NewInternalMessageRecipientMarkNotificationsStatusLogic(r.Context(), svcCtx)
		err := l.InternalMessageRecipientMarkNotificationsStatus(&req)
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
		} else {
			xhttp.JsonBaseResponseCtx(r.Context(), w, nil)
		}
	}
}
