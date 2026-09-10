package middleware

import (
	"context"
	"net"
	"net/http"
	"time"

	"go-wind-admin/backendz/internal/auditlog"
	"go-wind-admin/backendz/internal/svc"
)

const auditRequestIDKey ctxKey = "audit-request-id"

// Audit 返回 API 审计中间件：仅对通过鉴权的请求记录 ApiAuditLog（best-effort，不阻断业务）。
// 必须注册在 Auth 之后，这样只有携带有效 JWT（上下文已有 claims）的请求才会被审计，
// 从而避免把验证码/登录等公开路径的无关流量写入审计表。
func Audit(svcCtx *svc.ServiceContext) func(next http.HandlerFunc) http.HandlerFunc {
	return func(next http.HandlerFunc) http.HandlerFunc {
		return func(w http.ResponseWriter, r *http.Request) {
			claims, ok := ClaimsFromContext(r.Context())
			if !ok {
				next.ServeHTTP(w, r)
				return
			}

			ctx := r.Context()
			requestID := auditlog.NewRequestID()
			ctx = context.WithValue(ctx, auditRequestIDKey, requestID)

			rec := &statusRecorder{ResponseWriter: w}
			start := time.Now()
			next.ServeHTTP(rec, r.WithContext(ctx))
			latency := uint32(time.Since(start).Milliseconds())

			status := rec.status
			if status == 0 {
				status = http.StatusOK
			}
			success := status >= 200 && status < 400
			reason := ""
			if !success {
				reason = http.StatusText(status)
			}

			auditlog.WriteAPI(ctx, svcCtx, auditlog.APIRequest{
				UserID:     claims.UserID,
				Username:   claims.Username,
				IP:         clientIP(r),
				Method:     r.Method,
				Path:       r.URL.Path,
				RequestURI: r.URL.RequestURI(),
				RequestID:  requestID,
				LatencyMS:  latency,
				Success:    success,
				StatusCode: uint32(status),
				Reason:     reason,
			})
		}
	}
}

// AuditRequestIDFromContext 读取本次请求生成的审计请求 ID（若中间件已启用）。
func AuditRequestIDFromContext(ctx context.Context) string {
	v, _ := ctx.Value(auditRequestIDKey).(string)
	return v
}

// statusRecorder 包装 ResponseWriter，捕获响应状态码，供 API 审计使用。
type statusRecorder struct {
	http.ResponseWriter
	status int
}

func (r *statusRecorder) WriteHeader(code int) {
	r.status = code
	r.ResponseWriter.WriteHeader(code)
}

func (r *statusRecorder) Write(b []byte) (int, error) {
	if r.status == 0 {
		r.status = http.StatusOK
	}
	return r.ResponseWriter.Write(b)
}

// clientIP 提取请求来源 IP（去掉端口）。
func clientIP(r *http.Request) string {
	if r == nil {
		return ""
	}
	if host, _, err := net.SplitHostPort(r.RemoteAddr); err == nil {
		return host
	}
	return r.RemoteAddr
}