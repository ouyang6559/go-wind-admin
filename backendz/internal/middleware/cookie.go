// Package middleware 提供 JWT 鉴权中间件与登录态 Cookie 辅助。
package middleware

import (
	"fmt"
	"net/http"
	"strings"
	"time"
)

// refresh token cookie 相关常量（与 kratos backend 保持一致）：
// refresh_token 以 HttpOnly Cookie 传输，Path 收窄到刷新端点，SameSite=Lax 阻断跨站 POST；
// refresh_exp 为非 HttpOnly 的过期时间戳 cookie，供前端定时器读取以调度主动刷新。
const (
	RefreshTokenCookieName = "refresh_token"
	RefreshExpCookieName   = "refresh_exp"
	RefreshCookiePath      = "/admin/v1/refresh-token"
)

// ResolveCookieSecure 按请求的实际传输层判断是否给 cookie 加 Secure：
// TLS 直连或经可信反代（X-Forwarded-Proto: https）→ true；明文 HTTP → false。
// 依据：带 Secure 的 cookie 在明文 HTTP 下会被浏览器拒收，导致 refresh token 无法落地。
func ResolveCookieSecure(r *http.Request) bool {
	if r == nil {
		return true
	}
	if r.TLS != nil {
		return true
	}
	return strings.EqualFold(r.Header.Get("X-Forwarded-Proto"), "https")
}

// SetRefreshCookies 将 refresh token 及其过期时间戳写入两个 Set-Cookie 响应头。
// refresh_token：HttpOnly（JS 不可读）+ Path 收窄到刷新端点（纵深防御，仅刷新请求携带）；
// refresh_exp：非 HttpOnly（前端定时器/静默恢复可读）+ Path=/（任意页面 JS 可读）。
// 两者 SameSite=Lax（localhost 不同端口属同站，dev 直连后端可落地），Secure 按 TLS 自适应。
func SetRefreshCookies(w http.ResponseWriter, r *http.Request, refreshToken string, refreshExpiresInSeconds int64) {
	secure := ResolveCookieSecure(r)

	rtCookie := &http.Cookie{
		Name:     RefreshTokenCookieName,
		Value:    refreshToken,
		Path:     RefreshCookiePath,
		MaxAge:   int(refreshExpiresInSeconds),
		Secure:   secure,
		HttpOnly: true,
		SameSite: http.SameSiteLaxMode,
	}
	http.SetCookie(w, rtCookie)

	expCookie := &http.Cookie{
		Name:     RefreshExpCookieName,
		Value:    fmt.Sprintf("%d", time.Now().Unix()+refreshExpiresInSeconds),
		Path:     "/",
		MaxAge:   int(refreshExpiresInSeconds),
		Secure:   secure,
		HttpOnly: false,
		SameSite: http.SameSiteLaxMode,
	}
	http.SetCookie(w, expCookie)
}

// ClearRefreshCookies 向响应头写入 Max-Age=0 的清除 cookie，使浏览器立即删除 refresh token 相关 cookie。
// Secure 按 TLS 自适应，与 SetRefreshCookies 同逻辑，保证属性匹配可删除。
func ClearRefreshCookies(w http.ResponseWriter, r *http.Request) {
	secure := ResolveCookieSecure(r)

	for _, c := range []*http.Cookie{
		{Name: RefreshTokenCookieName, Path: RefreshCookiePath, MaxAge: -1, Secure: secure, HttpOnly: true, SameSite: http.SameSiteLaxMode},
		{Name: RefreshExpCookieName, Path: "/", MaxAge: -1, Secure: secure, HttpOnly: false, SameSite: http.SameSiteLaxMode},
	} {
		http.SetCookie(w, c)
	}
}

// ReadRefreshTokenCookie 从请求 Cookie 中读取 HttpOnly 的 refresh_token（刷新端点专用）。
func ReadRefreshTokenCookie(r *http.Request) string {
	if r == nil {
		return ""
	}
	c, err := r.Cookie(RefreshTokenCookieName)
	if err != nil || c == nil {
		return ""
	}
	return c.Value
}