// Package middleware 提供 JWT 鉴权中间件与登录态 Cookie 辅助。
package middleware

import (
	"bytes"
	"io"
	"net/http"
	"strings"
)

// NormalizeJsonBody 兜底修正「无意义」的 JSON 请求体：
// 当 body 实为字面量 null / 空白时，视为无请求体（清空 Content-Type 并置空 body）。
//
// 背景：react 端 axios transport 对 DELETE/GET 这类无数据请求，在全局默认
// `Content-Type: application/json` 下会把 `data: null` 序列化成字面量字符串 "null"
// 发送。go-zero 的 httpx.Parse 仅在 `ContentLength > 0 && Content-Type 为 json`
// 时才做 JSON 绑定，于是 "null" 被绑定进 handler 的 req 结构体，触发
// ent 「unsupported type on setting field value」，导致所有 UI 单行删除 100% 失败。
// kratos 后端容错该 body，故行为一致：这里同样将其归一化为无请求体。
func NormalizeJsonBody() func(next http.HandlerFunc) http.HandlerFunc {
	return func(next http.HandlerFunc) http.HandlerFunc {
		return func(w http.ResponseWriter, r *http.Request) {
			if r.Body == nil || strings.TrimSpace(r.Header.Get("Content-Type")) == "" {
				next.ServeHTTP(w, r)
				return
			}
			// 仅处理 application/json 请求体；其余（form/multipart 上传等）不干预。
			if !strings.HasPrefix(strings.ToLower(r.Header.Get("Content-Type")), "application/json") {
				next.ServeHTTP(w, r)
				return
			}

			buf, err := io.ReadAll(r.Body)
			if err != nil {
				// body 读失败：保留原样，交由后续 handler 自行报错（不吞错）。
				next.ServeHTTP(w, r)
				return
			}
			trimmed := bytes.TrimSpace(buf)
			if len(trimmed) == 0 || bytes.Equal(trimmed, []byte("null")) {
				// 命中「无意义 body」：填充不存在的请求，让 httpx.Parse 走无 body 分支。
				r.Body = http.NoBody
				r.ContentLength = 0
				r.Header.Del("Content-Type")
			} else {
				// 有实际 JSON 内容，还原切回 body 供 handler 正常解析。
				r.Body = io.NopCloser(bytes.NewReader(buf))
			}
			next.ServeHTTP(w, r)
		}
	}
}