// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.2
// 自定义：与 backend（kratos sse /events）保持一致，按用户订阅并推送 event: notification。

package events

import (
	"fmt"
	"net/http"
	"strconv"
	"time"

	"github.com/zeromicro/go-zero/core/logc"

	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/svc"
)

const (
	// subscribeBuffer 事件通道缓冲大小（供告警用，避免慢客户端阻塞发送方）。
	subscribeBuffer = 16
	// keepAliveInterval 心跳间隔：以 SSE 注释帧维持代理/网关长连接，避免空闲连接被回收。
	keepAliveInterval = 15 * time.Second
)

func StreamEventsHandler(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		// 全局 Auth 中间件已完成 Bearer 鉴权并注入载荷；此处仅做 stream 越权校验。
		claims, ok := middleware.ClaimsFromContext(r.Context())
		if !ok {
			http.Error(w, "unauthorized", http.StatusUnauthorized)
			return
		}

		// ?stream=<userId> 必须与访问令牌一致，防止订阅他人事件流。
		streamParam := r.URL.Query().Get("stream")
		uid, err := strconv.ParseUint(streamParam, 10, 32)
		if streamParam == "" || err != nil || uint32(uid) != claims.UserID {
			http.Error(w, "stream user mismatch", http.StatusForbidden)
			return
		}

		flusher, ok := w.(http.Flusher)
		if !ok {
			logc.Errorf(r.Context(), "[sse] response writer is not a flusher")
			return
		}

		// 订阅该用户的事件流；连接关闭时务必注销并关闭通道。
		ch := svcCtx.Sse.Subscribe(claims.UserID)
		defer svcCtx.Sse.Unsubscribe(claims.UserID, ch)

		// rest.WithSSE() 已在引擎层写入 Content-Type/Cache-Control/Connection，
		// 这里补充 headers 以防直接调用该处理器时缺头。
		w.Header().Set("Content-Type", "text/event-stream")
		w.Header().Set("Cache-Control", "no-cache")
		w.Header().Set("Connection", "keep-alive")

		keepAlive := time.NewTicker(keepAliveInterval)
		defer keepAlive.Stop()

		for {
			select {
			case payload := <-ch:
				// 与 backend 一致：站内信通知事件名为 notification，data 为收件 JSON。
				if _, err := fmt.Fprintf(w, "event: notification\ndata: %s\n\n", string(payload)); err != nil {
					logc.Errorf(r.Context(), "[sse] write notification failed: %v", err)
					return
				}
				flusher.Flush()
			case <-keepAlive.C:
				// SSE 注释帧心跳。
				if _, err := fmt.Fprint(w, ": ping\n\n"); err != nil {
					return
				}
				flusher.Flush()
			case <-r.Context().Done():
				return
			}
		}
	}
}
