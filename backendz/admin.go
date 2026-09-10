// Code scaffolded by goctl. Safe to edit.
// goctl 1.10.1

package main

import (
	"flag"
	"fmt"

	"go-wind-admin/backendz/internal/config"
	"go-wind-admin/backendz/internal/handler"
	"go-wind-admin/backendz/internal/middleware"
	"go-wind-admin/backendz/internal/pkassist"
	"go-wind-admin/backendz/internal/svc"

	// 注册 ent schema 运行期默认值（枚举/时长等），必须在创建 client 前导入。
	_ "go-wind-admin/backendz/internal/ent/gen/runtime"

	"github.com/zeromicro/go-zero/core/conf"
	"github.com/zeromicro/go-zero/rest"
)

var configFile = flag.String("f", "etc/admin-api.yaml", "the config file")

func main() {
	flag.Parse()

	var c config.Config
	conf.MustLoad(*configFile, &c)

	server := rest.MustNewServer(c.RestConf)
	defer server.Stop()

	ctx := svc.NewServiceContext(c)
	handler.RegisterHandlers(server, ctx)
	// 补注册 goctl 无法表达的「资源:动作」冒号字面路由（Kratos 契约迁移，见 colon_routes.go）
	handler.RegisterColonRoutes(server, ctx)

	// 全局中间件：注入原始请求（供 logic 读头/IP），再做 JWT 鉴权（白名单放行）
	server.Use(middleware.RequestCtx())
	server.Use(middleware.Auth(ctx))

	if err := pkassist.Seed(ctx); err != nil {
		panic(err)
	}

	fmt.Printf("Starting server at %s:%d...\n", c.Host, c.Port)
	server.Start()
}
