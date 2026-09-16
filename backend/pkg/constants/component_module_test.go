package constants

import (
	"testing"

	"github.com/stretchr/testify/assert"

	identityV1 "go-wind-admin/api/gen/go/identity/service/v1"
)

// TestComponentToModule 组件路径前缀 → 业务模块的归类必须与登记的前缀表
// 完全一致。该函数用于默认菜单与存量菜单的 module 字段回填，模块白名单
// 依赖该字段过滤租户可见菜单；前缀匹配错位会导致菜单在租户侧整页消失。
func TestComponentToModule(t *testing.T) {
	cases := []struct {
		name      string
		component string
		want      identityV1.Module
	}{
		// catalog 容器节点：不参与白名单过滤
		{"layout container", "BasicLayout", identityV1.Module_MODULE_UNSPECIFIED},
		{"empty component", "", identityV1.Module_MODULE_UNSPECIFIED},

		// 登记过的前缀，路径带具体页面
		{"dashboard page", "dashboard/analytics/index.vue", identityV1.Module_DASHBOARD},
		{"dashboard bare prefix", "dashboard/", identityV1.Module_DASHBOARD},
		{"opm page", "app/opm/user/list/index.vue", identityV1.Module_OPM},
		{"system page", "app/system/config/index.vue", identityV1.Module_SYSTEM},
		{"dict page", "app/dict/entry/index.vue", identityV1.Module_DICT},
		{"tenant page", "app/tenant/tenant/index.vue", identityV1.Module_TENANT},
		{"permission page", "app/permission/menu/index.vue", identityV1.Module_PERMISSION},
		{"file page", "app/file/list/index.vue", identityV1.Module_FILE},
		{"task page", "app/task/list/index.vue", identityV1.Module_TASK},

		// !!! 已知缺陷（钉住现状）：
		// "app/log/" 实际 8 字节，但分支条件用 component[:9] 与 8 字节字面量比较，
		// 恒为 false —— 死分支，全部日志审计组件落入 UNSPECIFIED；
		// "app/internal_message/" 实际 21 字节，但分支条件用 component[:20] 比较，
		// 恒为 false —— 死分支，内部消息组件同样落入 UNSPECIFIED。
		// 修复 ComponentToModule 后，以下两组期望值应翻转为
		// Module_LOG / Module_INTERNAL_MESSAGE。
		{"log page dead branch", "app/log/api_audit_log/index.vue", identityV1.Module_MODULE_UNSPECIFIED},
		{"internal message page dead branch", "app/internal_message/inbox/index.vue", identityV1.Module_MODULE_UNSPECIFIED},

		// 复合前缀：switch 顺序决定归入 system 而非 dict/file/task
		{"system dict page hits system first", "app/system/dict/index.vue", identityV1.Module_SYSTEM},
		{"system file page hits system first", "app/system/file/index.vue", identityV1.Module_SYSTEM},
		{"system task page hits system first", "app/system/task/index.vue", identityV1.Module_SYSTEM},

		// 未登记的前缀 / 前缀变体一律 UNSPECIFIED
		{"unknown top dir", "app/unknown/thing/index.vue", identityV1.Module_MODULE_UNSPECIFIED},
		{"prefix without slash", "app/opm", identityV1.Module_MODULE_UNSPECIFIED},
		{"prefix with extra letter", "app/opmx/user/index.vue", identityV1.Module_MODULE_UNSPECIFIED},
		{"dashboard with suffix letter", "dashboardx/index.vue", identityV1.Module_MODULE_UNSPECIFIED},
		{"case sensitive", "App/OPM/user/index.vue", identityV1.Module_MODULE_UNSPECIFIED},
		{"leading space not trimmed", " app/opm/user/index.vue", identityV1.Module_MODULE_UNSPECIFIED},
		{"plain text", "just-a-name", identityV1.Module_MODULE_UNSPECIFIED},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			assert.Equal(t, tc.want, ComponentToModule(tc.component))
		})
	}
}

// TestDefaultMenusModuleBackfillInvariant 默认菜单种子数据与归类函数的
// 一致性（除下方钉住的已知缺陷外）：非容器菜单的组件必须能归入一个
// 已定义的业务模块，否则该菜单会被模块白名单当作 UNSPECIFIED 过滤掉，
// 租户侧凭空丢页面；容器节点必须归为 UNSPECIFIED。
func TestDefaultMenusModuleBackfillInvariant(t *testing.T) {
	for _, menu := range DefaultMenus {
		component := menu.GetComponent()
		module := ComponentToModule(component)
		if component == "" || component == "BasicLayout" {
			assert.Equal(t, identityV1.Module_MODULE_UNSPECIFIED, module,
				"容器菜单 %q 的组件应归为 UNSPECIFIED", component)
			continue
		}
		if isKnownOffByOneComponent(component) {
			continue
		}
		_, defined := identityV1.Module_name[int32(module)]
		assert.True(t, defined,
			"菜单组件 %q 归类到未定义模块值 %d", component, module)
		assert.NotEqual(t, identityV1.Module_MODULE_UNSPECIFIED, module,
			"非容器菜单组件 %q 未被 ComponentToModule 登记，租户白名单会过滤掉该菜单；请登记前缀或修正组件路径", component)
	}
}

// knownOffByOneComponents 已知因 ComponentToModule 两个 off-by-one 死分支
// （"app/log/" 分支用 [:9] 比较 8 字节字面量、"app/internal_message/" 分支
// 用 [:20] 比较 21 字节字面量，均恒为 false）而归类为 UNSPECIFIED 的
// 默认菜单组件全集（日志审计 7 页 + 内部消息 3 页）。这些菜单的 module
// 字段在入库时被置空，租户侧模块白名单过滤会把它们整体隐藏。
func knownOffByOneComponents() map[string]bool {
	return map[string]bool{
		"app/log/api_audit_log/index.vue":         true,
		"app/log/data_access_audit_log/index.vue": true,
		"app/log/login_audit_log/index.vue":       true,
		"app/log/operation_audit_log/index.vue":   true,
		"app/log/permission_audit_log/index.vue":  true,
		"app/log/policy_evaluation_log/index.vue": true,
		"app/log/redis_cache_monitor/index.vue":   true,
		"app/internal_message/category/index.vue":  true,
		"app/internal_message/inbox/index.vue":     true,
		"app/internal_message/message/index.vue":   true,
	}
}

func isKnownOffByOneComponent(component string) bool {
	return knownOffByOneComponents()[component]
}

// TestDefaultMenusUnmappedComponentsOffByOneRegression 钉死当前因
// ComponentToModule off-by-one 死分支而归类为 UNSPECIFIED 的默认菜单组件
// 全集。两个方向都会触发本测试：
//  1. 修复死分支后（或新菜单组件未登记时），实际集合与钉死集合不一致，
//     测试失败，提示翻转 TestComponentToModule 的期望值并同步本清单；
//  2. 该清单本身防止"以为只有一两个菜单受影响"的误判扩散。
func TestDefaultMenusUnmappedComponentsOffByOneRegression(t *testing.T) {
	var unmapped []string
	for _, menu := range DefaultMenus {
		component := menu.GetComponent()
		if component == "" || component == "BasicLayout" {
			continue
		}
		if ComponentToModule(component) == identityV1.Module_MODULE_UNSPECIFIED {
			unmapped = append(unmapped, component)
		}
	}
	expected := make([]string, 0, len(knownOffByOneComponents()))
	for c := range knownOffByOneComponents() {
		expected = append(expected, c)
	}
	assert.ElementsMatch(t, expected, unmapped,
		"归类为 UNSPECIFIED 的非容器菜单组件集合发生变化；若已修复 ComponentToModule 的 off-by-one 死分支，请同步更新本清单与 TestComponentToModule 的期望值")
}
