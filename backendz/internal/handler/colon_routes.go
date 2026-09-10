// Code scaffolded by goctl. Safe to edit.
// 注意：本文件是手写文件，请不要删除。它承载 goctl .api 无法表达的「冒号字面路由」。

package handler

import (
	"net/http"

	"go-wind-admin/backendz/internal/handler/permission"
	"go-wind-admin/backendz/internal/handler/task"
	"go-wind-admin/backendz/internal/handler/tenant"
	"go-wind-admin/backendz/internal/handler/user"
	"go-wind-admin/backendz/internal/svc"

	"github.com/zeromicro/go-zero/rest"
)

// RegisterColonRoutes 注册 Kratos openapi 的「资源:动作」冒号字面路由（代码转移，非无效代码）。
//
// 背景：goctl 的 .api 解析器不允许路径段内出现 ':'，无法在 desc/*.api 里声明
// 如 /admin/v1/tasks:control 这类 URL；但运行时 patRouter 将非开头冒号段视为
// **字面匹配**（与 /xxx/{id} 的路径参数不同）。因此这些路由在此手动补注册。
//
// 约束：本块不能带 rest.WithPrefix(...)，否则 go-zero 会在注册时用
// path.Join(prefix, path) 拼接而插入 '/'，改写为 tasks/:control 语义。
// handler 与 routes.go 中对应斜杠路由一致，均走全局 Auth 中间件。
//
// 调用点：admin.go 中在 handler.RegisterHandlers(...) 之后调用，
// 因此置于本文件而非生成的 routes.go，可在 goctl 重新生成后依然存活。
func RegisterColonRoutes(server *rest.Server, serverCtx *svc.ServiceContext) {
	server.AddRoutes(
		[]rest.Route{
			{
				Method:  http.MethodPost,
				Path:    "/admin/v1/permissions/sync:perms",
				Handler: permission.PermissionSyncPermissionsHandler(serverCtx),
			},
			{
				Method:  http.MethodPost,
				Path:    "/admin/v1/tasks:control",
				Handler: task.TaskControlTaskHandler(serverCtx),
			},
			{
				Method:  http.MethodPost,
				Path:    "/admin/v1/tasks:restart",
				Handler: task.TaskRestartAllTaskHandler(serverCtx),
			},
			{
				Method:  http.MethodPost,
				Path:    "/admin/v1/tasks:start",
				Handler: task.TaskStartAllTaskHandler(serverCtx),
			},
			{
				Method:  http.MethodPost,
				Path:    "/admin/v1/tasks:stop",
				Handler: task.TaskStopAllTaskHandler(serverCtx),
			},
			{
				Method:  http.MethodGet,
				Path:    "/admin/v1/tasks:type-names",
				Handler: task.TaskListTaskTypeNameHandler(serverCtx),
			},
			{
				Method:  http.MethodGet,
				Path:    "/admin/v1/tenants:exists",
				Handler: tenant.TenantTenantExistsHandler(serverCtx),
			},
			{
				Method:  http.MethodPost,
				Path:    "/admin/v1/tenants:with-admin",
				Handler: tenant.TenantCreateTenantWithAdminUserHandler(serverCtx),
			},
			{
				Method:  http.MethodGet,
				Path:    "/admin/v1/users:exists",
				Handler: user.UserUserExistsHandler(serverCtx),
			},
		},
	)
}
