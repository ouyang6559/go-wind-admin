package script

import (
	"context"
	"fmt"
	"strings"
	"time"

	"go-wind-admin/app/admin/service/internal/data/ent"
	scriptV1 "go-wind-admin/api/gen/go/script/service/v1"
	"go-wind-admin/pkg/scripting"
)

// 实体生命周期钩子桥：
//
// 经 ent 的 client.Use() 挂载全局 mutation hook，在实体 Create/Update/Delete
// 落库成功后异步触发形如 "<entity>.after_<op>" 的钩子点（如 user.after_create）。
// 只暴露 after 类钩子（异步、不阻塞业务、不可回滚）——before 类同步钩子需要
// 拦截/修改/否决变更，涉及时序与部分失败语义，待 after 类磨熟后另行开放。
//
// 执行隔离：钩子在独立 goroutine 中经 Runtime 执行，不阻塞业务写入路径；
// 单脚本失败仅记日志/审计，绝不影响业务结果。

// EntityHooksMapping ent 实体类型名（m.Type()，PascalCase）→ 钩子点前缀（小写）。
// 只收录有明确业务意义的实体，避免钩子风暴；新增实体在此登记即可生效。
var EntityHooksMapping = map[string]string{
	"User":                "user",
	"Tenant":              "tenant",
	"Role":                "role",
	"InternalMessage":     "internal_message",
	"NotificationChannel": "notification_channel",
}

// ScriptHookInvoker 由 app 层注入（包装 Runtime.ExecuteHook 的异步调用），
// 避免本桥依赖具体运行时实现。
type ScriptHookInvoker func(ctx context.Context, hookPoint string, data map[string]any)

// AttachEntityHooks 在 ent client 上挂载全局生命周期钩子。
func AttachEntityHooks(client *ent.Client, invoker ScriptHookInvoker) {
	client.Use(newEntityHook(invoker))
}

// newEntityHook 构造 ent 全局 hook：变更成功后按实体类型/操作触发对应钩子点。
func newEntityHook(invoker ScriptHookInvoker) ent.Hook {
	h := func(next ent.Mutator) ent.Mutator {
		return ent.MutateFunc(func(ctx context.Context, m ent.Mutation) (ent.Value, error) {
			fmt.Printf("[script-hook] hook entered: type=%s op=%s\n", m.Type(), m.Op())
			value, err := next.Mutate(ctx, m)

			// 只在变更成功后触发（err == nil）
			if err == nil && invoker != nil {
				typ := m.Type()
				prefix, ok := EntityHooksMapping[typ]
				if ok {
					op := mutationOpName(m.Op())
					hookPoint := prefix + ".after_" + op

					payload := map[string]any{
						"entity": typ,
						"op":     op,
						"id":     mutationID(m),
					}

					go func() {
						fmt.Printf("[script-hook] goroutine fired: %s payload=%v\n", hookPoint, payload)
						defer func() {
							// 钩子 goroutine 的兜底防护：脚本 panic 不能带走业务进程
							if r := recover(); r != nil {
								fmt.Printf("[script-hook] panic in %s: %v\n", hookPoint, r)
							}
						}()

						// 带超时的独立上下文：不继承请求取消信号（业务返回后钩子仍应跑完）
						hookCtx, cancel := context.WithTimeout(context.WithoutCancel(ctx), 30*time.Second)
						defer cancel()
						invoker(hookCtx, hookPoint, payload)
					}()
				}
			}

			return value, err
		})
	}
	return ent.Hook(h)
}

// mutationOpName 把 ent Op 映射为钩子点命名中的操作名。
func mutationOpName(op ent.Op) string {
	switch op {
	case ent.OpCreate:
		return "create"
	case ent.OpUpdate | ent.OpUpdateOne:
		return "update"
	case ent.OpDelete:
		return "delete"
	case ent.OpDeleteOne:
		return "delete"
	default:
		return strings.ToLower(op.String())
	}
}

// mutationID 从 mutation 尽力提取实体 ID（取不到为 0）。
func mutationID(m ent.Mutation) uint32 {
	type idGetter interface{ ID() (uint32, bool) }
	if g, ok := m.(idGetter); ok {
		if id, exists := g.ID(); exists {
			return id
		}
	}
	return 0
}

// ExecuteHookPayload 组装脚本执行上下文并同步触发钩子点（Runtime 侧入口）。
// 异步路径经 ScriptHookInvoker 间接调用本方法。
func (r *Runtime) InvokeEntityHook(hookPoint string, payload map[string]any) error {
	eng := r.engineForHookPoint(hookPoint)
	if eng == nil {
		return scriptV1.ErrorNotFound("no scripts mounted on hook point: %s", hookPoint)
	}

	execCtx := scripting.NewContext(hookPoint)
	for k, v := range payload {
		execCtx.Set(k, v)
	}

	// 钩子上下文把实体信息放在顶层（脚本用 ctx.get("entity") / ctx.get("id") 读取）
	return eng.ExecuteHook(context.Background(), hookPoint, execCtx)
}

// engineForHookPoint 返回挂载了指定钩子点脚本的引擎（任一语言命中即可）。
// 无挂载时返回 nil（业务侧据此完全跳过执行开销）。
func (r *Runtime) engineForHookPoint(hookPoint string) *scripting.Engine {
	r.mu.RLock()
	defer r.mu.RUnlock()
	for _, eng := range r.engines {
		for _, hp := range eng.HookPoints() {
			if hp.Name == hookPoint && (hp.ScriptCount > 0 || hp.CallbackCount > 0) {
				return eng
			}
		}
	}
	return nil
}
