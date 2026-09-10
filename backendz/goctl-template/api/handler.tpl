// Code scaffolded by goctl. Safe to edit.
// goctl {{.version}}

package {{.PkgName}}

import (
	"net/http"

	{{if or .HasRequest .HasResp}}"github.com/zeromicro/go-zero/rest/httpx"
	{{end}}xhttp "github.com/zeromicro/x/http"
	{{.ImportPackages}}
)

// 成功响应与 Kratos 主后端对齐：直接返回扁平 proto 消息体（如 {"items":[],"total":"0"}），
// 不包 {code,msg,data} 信封，保证 react/vue-element/vue-vben 三层前端零改动。
// 仅错误路径通过 xhttp 输出 {code,msg}。

{{if .HasDoc}}{{.Doc}}{{end}}
func {{.HandlerName}}(svcCtx *svc.ServiceContext) http.HandlerFunc {
	return func(w http.ResponseWriter, r *http.Request) {
		{{if .HasRequest}}var req types.{{.RequestType}}
		if err := httpx.Parse(r, &req); err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
			return
		}

		{{end}}l := {{.LogicName}}.New{{.LogicType}}(r.Context(), svcCtx)
		{{if .HasResp}}resp, {{end}}err := l.{{.Call}}({{if .HasRequest}}&req{{end}})
		if err != nil {
			xhttp.JsonBaseResponseCtx(r.Context(), w, err)
		} else {
			{{if .HasResp}}httpx.OkJsonCtx(r.Context(), w, resp){{else}}xhttp.JsonBaseResponseCtx(r.Context(), w, nil){{end}}
		}
	}
}
