package pkassist

import (
	"context"
	"fmt"
	"strings"

	"entgo.io/ent/privacy"
	gocrudviewer "github.com/tx7do/go-crud/viewer"

	"go-wind-admin/backendz/internal/ent/gen/api"
	"go-wind-admin/backendz/internal/pkg/viewer"
	"go-wind-admin/backendz/internal/svc"
)

// SyncApis 从已注册路由清单同步 API 资源表（sys_apis）。
// 数据源：svcCtx.Routes（admin.go 在 RegisterHandlers/RegisterColonRoutes 后从 rest.Server.Routes()
// 收集，含 WithPrefix 分组拼接后的完整路径，覆盖全部真实路由）。
// 按 (method, path) 判重、增量补齐，幂等可重复调用。供启动播种与管理页「接口同步」共用。
func SyncApis(s *svc.ServiceContext) error {
	ctx := gocrudviewer.WithContext(context.Background(), viewer.Default)
	allowCtx := privacy.DecisionContext(ctx, privacy.Allow)
	return syncApis(ctx, allowCtx, s)
}

func syncApis(ctx, allowCtx context.Context, s *svc.ServiceContext) error {
	if len(s.Routes) == 0 {
		return fmt.Errorf("sync apis: no routes collected from rest.Server")
	}

	// 现有 (method, path) 集合，缺失才插入。
	existing := map[string]bool{}
	apis, err := s.Ent.Api.Query().All(ctx)
	if err != nil {
		return err
	}
	for _, a := range apis {
		if a.Path != nil && a.Method != nil {
			existing[*a.Method+" "+*a.Path] = true
		}
	}

	created := 0
	for _, rt := range s.Routes {
		path := strings.TrimSpace(rt.Path)
		method := strings.ToUpper(strings.TrimSpace(rt.Method))
		if path == "" || method == "" {
			continue
		}
		key := method + " " + path
		if existing[key] {
			continue
		}
		if _, err := s.Ent.Api.Create().
			SetPath(path).
			SetMethod(method).
			SetOperation(operationFromPath(path)).
			SetScope(api.ScopeAdmin).
			SetStatus(api.StatusOn).
			Save(allowCtx); err != nil {
			return fmt.Errorf("sync api %s %s: %w", method, path, err)
		}
		existing[key] = true
		created++
	}
	return nil
}

// operationFromPath 从路径末段推断接口操作名（不含路径参数冒号）。
func operationFromPath(path string) string {
	seg := strings.TrimSuffix(path, "/")
	if i := strings.LastIndex(seg, "/"); i >= 0 {
		seg = seg[i+1:]
	}
	seg = strings.TrimPrefix(seg, ":")
	if seg == "" {
		return "root"
	}
	return seg
}
