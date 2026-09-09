// Package middleware 提供 JWT 鉴权中间件。
package middleware

import (
	"context"
	"net/http"
	"strings"

	xhttp "github.com/zeromicro/x/http"

	"go-wind-admin/backendz/internal/pkg/token"
	"go-wind-admin/backendz/internal/pkg/viewer"
	"go-wind-admin/backendz/internal/svc"
	"go-wind-admin/backendz/internal/xerr"

	gocrudviewer "github.com/tx7do/go-crud/viewer"
)

type ctxKey string

const claimsContextKey ctxKey = "authn-claims"

// ClaimsFromContext 从上下文读取 JWT 载荷；未登录返回 (nil,false)。
func ClaimsFromContext(ctx context.Context) (*token.Claim, bool) {
	v, ok := ctx.Value(claimsContextKey).(*token.Claim)
	return v, ok
}

const requestContextKey ctxKey = "req-obj"

// RequestCtx 将原始 *http.Request 注入上下文，供 logic 层读取请求头/IP 等。
func RequestCtx() func(next http.HandlerFunc) http.HandlerFunc {
	return func(next http.HandlerFunc) http.HandlerFunc {
		return func(w http.ResponseWriter, r *http.Request) {
			ctx := context.WithValue(r.Context(), requestContextKey, r)
			next.ServeHTTP(w, r.WithContext(ctx))
		}
	}
}

// RequestFromContext 从上下文取回原始请求。
func RequestFromContext(ctx context.Context) (*http.Request, bool) {
	r, ok := ctx.Value(requestContextKey).(*http.Request)
	return r, ok
}

// publicPaths 免鉴权路径（登录验证码/登录/刷新/注册）。
var publicPaths = map[string]bool{
	"/admin/v1/captcha":        true,
	"/admin/v1/captcha/verify": true,
	"/admin/v1/login":          true,
	"/admin/v1/refresh-token":  true,
	"/admin/v1/register":       true,
}

// Auth 返回 JWT 鉴权中间件：白名单路径放行，其余路径校验 Bearer Token。
// 无论是否登录，都会向上下文注入 tx7do viewer（公开路径使用平台缺省视图，
// 认证请求使用 JWT 载荷派生的视图），供 ent 租户隐私策略消费。
func Auth(svcCtx *svc.ServiceContext) func(next http.HandlerFunc) http.HandlerFunc {
	return func(next http.HandlerFunc) http.HandlerFunc {
		return func(w http.ResponseWriter, r *http.Request) {
			ctx := r.Context()
			var claims *token.Claim

			if publicPaths[r.Method+" "+r.URL.Path] || publicPaths[r.URL.Path] {
				next.ServeHTTP(w, r.WithContext(gocrudviewer.WithContext(ctx, viewer.Default)))
				return
			}

			auth := r.Header.Get("Authorization")
			tokenStr := strings.TrimSpace(strings.TrimPrefix(auth, "Bearer "))
			if tokenStr == "" {
				xhttp.JsonBaseResponseCtx(ctx, w, xerr.UnauthorizedMsg("unauthorized"))
				return
			}

			var err error
			claims, err = svcCtx.Token.ParseAccessToken(tokenStr)
			if err != nil {
				xhttp.JsonBaseResponseCtx(ctx, w, xerr.UnauthorizedMsg("invalid or expired token"))
				return
			}

			ctx = context.WithValue(ctx, claimsContextKey, claims)
			ctx = gocrudviewer.WithContext(ctx, viewer.FromClaims(claims))
			next.ServeHTTP(w, r.WithContext(ctx))
		}
	}
}