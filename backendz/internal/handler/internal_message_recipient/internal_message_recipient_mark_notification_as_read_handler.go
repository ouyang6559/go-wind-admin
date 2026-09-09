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

func InternalMessageRecipientMarkNotificationAsReadHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		var req types.MarkNotificationAsReadRequest
		if err := httpx.Parse(r, &req); err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
			return
		}

		l := internal_message_recipient.NewInternalMessageRecipientMarkNotificationAsReadLogic(r.Context(), svcCtx)
		err := l.InternalMessageRecipientMarkNotificationAsRead(&req)
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
		} else {
			xhttp.JsonBaseResponseCtx(r.Context(), w, nil)
		}
	}
}
