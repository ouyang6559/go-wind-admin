package pkassist

import (
	"go-wind-admin/backendz/internal/ent/gen/menu"
	"go-wind-admin/backendz/internal/ent/gen/role"
	"go-wind-admin/backendz/internal/ent/gen/rolemetadata"
	"go-wind-admin/backendz/internal/ent/schema"
)

// 默认静态种子数据。数值/文案与 backend（Kratos）pkg/constants/default_data.go 对齐，
// 仅把 proto 类型替换为 backendz 的 ent 结构体/枚举。字段取舍见各 struct 注释。

// 角色 code（与 backend/pkg/constants/role.go 对齐）
const (
	roleCodePlatformAdmin       = "platform:admin"
	roleCodeTenantAdminTemplate = "template:tenant:manager"
	roleNameTenantManager       = "租户管理员"
)

// 权限 code（与 backend/pkg/constants/permission.go 对齐）
const (
	permCodeAccessBackend = "sys:access_backend"
	permCodePlatformAdmin = "sys:platform_admin"
	permCodeTenantManager = "sys:tenant_manager"
	permCodeManageTenants = "sys:manage_tenants"
	permCodeAuditLogs     = "sys:audit_logs"
)

// 权限模块标识
const permissionModule = "sys"

// 菜单 authority
var (
	authorityBoth     = []string{"sys:platform_admin", "sys:tenant_manager"}
	authorityPlatform = []string{"sys:platform_admin"}
)

func int32Ptr(v int32) *int32 { return &v }
func boolPtr(v bool) *bool    { return &v }

// ===== 语言 =====

type seedLanguage struct {
	code, name, native string
	isDefault          bool
	sortOrder          uint32
}

var defaultLanguages = []seedLanguage{
	{code: "zh-CN", name: "中文（简体）", native: "简体中文", isDefault: true, sortOrder: 0},
	{code: "zh-TW", name: "中文（繁体）", native: "繁體中文", sortOrder: 100},
	{code: "en-US", name: "英语", native: "English", sortOrder: 1},
	{code: "ja-JP", name: "日语", native: "日本語", sortOrder: 100},
	{code: "ko-KR", name: "韩语", native: "한국어", sortOrder: 100},
	{code: "es-ES", name: "西班牙语", native: "Español", sortOrder: 100},
	{code: "fr-FR", name: "法语", native: "Français", sortOrder: 100},
}

// ===== 权限组（树形）=====

type seedPermissionGroup struct {
	name       string
	module     string
	path       string
	parentName string // 空表示根节点
	sortOrder  uint32
}

var defaultPermissionGroups = []seedPermissionGroup{
	{name: "系统管理", module: permissionModule, path: "", sortOrder: 1},
	{name: "系统权限", module: permissionModule, path: "/1/2/", parentName: "系统管理", sortOrder: 1},
	{name: "租户管理", module: permissionModule, path: "/1/3/", parentName: "系统管理", sortOrder: 2},
	{name: "审计管理", module: permissionModule, path: "/1/4/", parentName: "系统管理", sortOrder: 3},
	{name: "安全策略", module: permissionModule, path: "/1/5/", parentName: "系统管理", sortOrder: 4},
}

// ===== 权限点 =====

type seedPermission struct {
	name, desc, code, groupName string
}

var defaultPermissions = []seedPermission{
	{name: "访问后台", code: permCodeAccessBackend, groupName: "系统权限", desc: "允许用户访问系统后台管理界面"},
	{name: "平台管理员权限", code: permCodePlatformAdmin, groupName: "系统权限", desc: "拥有系统所有功能的操作权限，可管理租户、用户、角色及所有资源"},
	{name: "租户管理员权限", code: permCodeTenantManager, groupName: "租户管理", desc: "拥有租户内所有功能的操作权限，可管理用户、角色及租户内所有资源"},
	{name: "管理租户", code: permCodeManageTenants, groupName: "租户管理", desc: "允许创建/修改/删除租户"},
	{name: "查看审计日志", code: permCodeAuditLogs, groupName: "审计管理", desc: "允许查看系统操作日志"},
}

// ===== 角色 =====

type seedRole struct {
	name            string
	code            string
	desc            string
	isProtected     bool
	typ             role.Type
	sortOrder       uint32
	permissionCodes []string
}

var defaultRoles = []seedRole{
	{
		name:        "平台管理员",
		code:        roleCodePlatformAdmin,
		desc:        "拥有系统所有功能的操作权限，可管理租户、用户、角色及所有资源",
		isProtected: true,
		typ:         role.TypeSystem,
		sortOrder:   1,
		permissionCodes: []string{
			permCodeAccessBackend, permCodePlatformAdmin, permCodeManageTenants,
		},
	},
	{
		name:        roleNameTenantManager + "模板",
		code:        roleCodeTenantAdminTemplate,
		desc:        "租户管理员角色，拥有租户内所有功能的操作权限，可管理用户、角色及租户内所有资源",
		isProtected: true,
		typ:         role.TypeTemplate,
		sortOrder:   2,
		permissionCodes: []string{
			permCodeAccessBackend, permCodeTenantManager,
		},
	},
}

// ===== 角色元数据 =====

type seedRoleMetadata struct {
	roleCode    string
	isTemplate  bool
	templateFor string
	scope       rolemetadata.Scope
	syncPolicy  rolemetadata.SyncPolicy
}

var seedRoleMetadatas = []seedRoleMetadata{
	{
		roleCode:   roleCodePlatformAdmin,
		isTemplate: false,
		scope:      rolemetadata.ScopePlatform,
		syncPolicy: rolemetadata.SyncPolicyAuto,
	},
	{
		roleCode:    roleCodeTenantAdminTemplate,
		isTemplate:  true,
		templateFor: "tenant:manager",
		scope:       rolemetadata.ScopeTenant,
		syncPolicy:  rolemetadata.SyncPolicyAuto,
	},
}

func metadataByRole(code string) (seedRoleMetadata, bool) {
	for _, m := range seedRoleMetadatas {
		if m.roleCode == code {
			return m, true
		}
	}
	return seedRoleMetadata{}, false
}

// ===== 菜单（树形，33 条）=====

type seedMenu struct {
	name       string
	parentName string // 空表示根节点
	typ        menu.Type
	path       string
	redirect   string
	component  string
	hasModule  bool
	module     menu.Module
	meta       *schema.MenuMeta
}

var defaultMenus = []seedMenu{
	{name: "Dashboard", typ: menu.TypeCatalog, path: "/dashboard", component: "BasicLayout", meta: &schema.MenuMeta{Order: int32Ptr(-1), Title: "page.dashboard.title", Icon: "lucide:layout-dashboard", Authority: authorityBoth}},
	{name: "Analytics", parentName: "Dashboard", typ: menu.TypeMenu, path: "/analytics", component: "dashboard/analytics/index.vue", hasModule: true, module: menu.ModuleDashboard, meta: &schema.MenuMeta{Order: int32Ptr(-1), Title: "page.dashboard.analytics", Icon: "lucide:area-chart", Authority: authorityBoth, AffixTab: boolPtr(true)}},

	{name: "Profile", typ: menu.TypeCatalog, path: "/profile", component: "BasicLayout", meta: &schema.MenuMeta{Title: "menu.profile.settings", HideInMenu: boolPtr(true)}},
	{name: "ProfilePage", parentName: "Profile", typ: menu.TypeMenu, path: "/profile", component: "app/opm/user/profile/index.vue", hasModule: true, module: menu.ModuleOpm, meta: &schema.MenuMeta{Title: "menu.profile.settings", Icon: "lucide:user-pen", HideInMenu: boolPtr(true)}},

	{name: "Inbox", typ: menu.TypeCatalog, path: "/inbox", component: "BasicLayout", meta: &schema.MenuMeta{Title: "menu.profile.internalMessage", HideInMenu: boolPtr(true)}},
	{name: "InboxPage", parentName: "Inbox", typ: menu.TypeMenu, path: "/inbox", component: "app/internal_message/inbox/index.vue", hasModule: true, module: menu.ModuleInternalMessage, meta: &schema.MenuMeta{Title: "menu.profile.internalMessage", Icon: "lucide:message-circle-more", HideInMenu: boolPtr(true)}},

	{name: "TenantManagement", typ: menu.TypeCatalog, path: "/tenant", redirect: "/tenant/members", component: "BasicLayout", meta: &schema.MenuMeta{Order: int32Ptr(2000), Title: "menu.tenant.moduleName", Icon: "lucide:building-2", Authority: authorityPlatform}},
	{name: "TenantMemberManagement", parentName: "TenantManagement", typ: menu.TypeMenu, path: "members", component: "app/tenant/tenant/index.vue", hasModule: true, module: menu.ModuleTenant, meta: &schema.MenuMeta{Order: int32Ptr(1), Title: "menu.tenant.member", Icon: "lucide:users", Authority: authorityPlatform, AffixTab: boolPtr(true)}},

	{name: "OrganizationalPersonnelManagement", typ: menu.TypeCatalog, path: "/opm", redirect: "/opm/users", component: "BasicLayout", meta: &schema.MenuMeta{Order: int32Ptr(2001), Title: "menu.opm.moduleName", Icon: "lucide:users", KeepAlive: boolPtr(true), Authority: authorityBoth}},
	{name: "OrgUnitManagement", parentName: "OrganizationalPersonnelManagement", typ: menu.TypeMenu, path: "org-units", component: "app/opm/org_unit/index.vue", hasModule: true, module: menu.ModuleOpm, meta: &schema.MenuMeta{Order: int32Ptr(1), Title: "menu.opm.orgUnit", Icon: "lucide:layers", Authority: authorityBoth}},
	{name: "PositionManagement", parentName: "OrganizationalPersonnelManagement", typ: menu.TypeMenu, path: "positions", component: "app/opm/position/index.vue", hasModule: true, module: menu.ModuleOpm, meta: &schema.MenuMeta{Order: int32Ptr(2), Title: "menu.opm.position", Icon: "lucide:briefcase", Authority: authorityBoth}},
	{name: "UserManagement", parentName: "OrganizationalPersonnelManagement", typ: menu.TypeMenu, path: "users", component: "app/opm/user/list/index.vue", hasModule: true, module: menu.ModuleOpm, meta: &schema.MenuMeta{Order: int32Ptr(3), Title: "menu.opm.user", Icon: "lucide:user", Authority: authorityBoth}},
	{name: "UserDetail", parentName: "OrganizationalPersonnelManagement", typ: menu.TypeMenu, path: "users/detail/:id", component: "app/opm/user/detail/index.vue", hasModule: true, module: menu.ModuleOpm, meta: &schema.MenuMeta{Title: "menu.opm.userDetail", Authority: authorityBoth, HideInMenu: boolPtr(true)}},

	{name: "PermissionManagement", typ: menu.TypeCatalog, path: "/permission", redirect: "/permission/codes", component: "BasicLayout", meta: &schema.MenuMeta{Order: int32Ptr(2002), Title: "menu.permission.moduleName", Icon: "lucide:shield-check", KeepAlive: boolPtr(true), Authority: authorityBoth}},
	{name: "PermissionPointManagement", parentName: "PermissionManagement", typ: menu.TypeMenu, path: "codes", component: "app/permission/permission/index.vue", hasModule: true, module: menu.ModulePermission, meta: &schema.MenuMeta{Order: int32Ptr(1), Title: "menu.permission.permission", Icon: "lucide:shield-ellipsis", Authority: authorityPlatform}},
	{name: "RoleManagement", parentName: "PermissionManagement", typ: menu.TypeMenu, path: "roles", component: "app/permission/role/index.vue", hasModule: true, module: menu.ModulePermission, meta: &schema.MenuMeta{Order: int32Ptr(2), Title: "menu.permission.role", Icon: "lucide:shield-user", Authority: authorityBoth}},
	{name: "MenuManagement", parentName: "PermissionManagement", typ: menu.TypeMenu, path: "menus", component: "app/permission/menu/index.vue", hasModule: true, module: menu.ModulePermission, meta: &schema.MenuMeta{Order: int32Ptr(1), Title: "menu.permission.menu", Icon: "lucide:square-menu", Authority: authorityPlatform}},
	{name: "APIManagement", parentName: "PermissionManagement", typ: menu.TypeMenu, path: "apis", component: "app/permission/api/index.vue", hasModule: true, module: menu.ModulePermission, meta: &schema.MenuMeta{Order: int32Ptr(2), Title: "menu.permission.api", Icon: "lucide:route", Authority: authorityPlatform}},

	{name: "InternalMessageManagement", typ: menu.TypeCatalog, path: "/internal-message", redirect: "/internal-message/messages", component: "BasicLayout", meta: &schema.MenuMeta{Order: int32Ptr(2003), Title: "menu.internalMessage.moduleName", Icon: "lucide:mail", KeepAlive: boolPtr(true), Authority: authorityBoth}},
	{name: "InternalMessageList", parentName: "InternalMessageManagement", typ: menu.TypeMenu, path: "messages", component: "app/internal_message/message/index.vue", hasModule: true, module: menu.ModuleInternalMessage, meta: &schema.MenuMeta{Order: int32Ptr(1), Title: "menu.internalMessage.internalMessage", Icon: "lucide:message-circle-more", Authority: authorityBoth}},
	{name: "InternalMessageCategoryManagement", parentName: "InternalMessageManagement", typ: menu.TypeMenu, path: "categories", component: "app/internal_message/category/index.vue", hasModule: true, module: menu.ModuleInternalMessage, meta: &schema.MenuMeta{Order: int32Ptr(2), Title: "menu.internalMessage.internalMessageCategory", Icon: "lucide:calendar-check", Authority: authorityPlatform}},

	{name: "LogAuditManagement", typ: menu.TypeCatalog, path: "/log", redirect: "/log/login-audit-logs", component: "BasicLayout", meta: &schema.MenuMeta{Order: int32Ptr(2004), Title: "menu.log.moduleName", Icon: "lucide:logs", KeepAlive: boolPtr(true), Authority: authorityPlatform}},
	{name: "LoginAuditLog", parentName: "LogAuditManagement", typ: menu.TypeMenu, path: "login-audit-logs", component: "app/log/login_audit_log/index.vue", hasModule: true, module: menu.ModuleLog, meta: &schema.MenuMeta{Order: int32Ptr(1), Title: "menu.log.loginAuditLog", Icon: "lucide:user-lock", Authority: authorityPlatform}},
	{name: "ApiAuditLog", parentName: "LogAuditManagement", typ: menu.TypeMenu, path: "api-audit-logs", component: "app/log/api_audit_log/index.vue", hasModule: true, module: menu.ModuleLog, meta: &schema.MenuMeta{Order: int32Ptr(2), Title: "menu.log.apiAuditLog", Icon: "lucide:file-clock", Authority: authorityPlatform}},
	{name: "OperationAuditLog", parentName: "LogAuditManagement", typ: menu.TypeMenu, path: "operation-audit-logs", component: "app/log/operation_audit_log/index.vue", hasModule: true, module: menu.ModuleLog, meta: &schema.MenuMeta{Order: int32Ptr(3), Title: "menu.log.operationAuditLog", Icon: "lucide:shield-ellipsis", Authority: authorityPlatform}},
	{name: "DataAccessAuditLog", parentName: "LogAuditManagement", typ: menu.TypeMenu, path: "data-access-audit-logs", component: "app/log/data_access_audit_log/index.vue", hasModule: true, module: menu.ModuleLog, meta: &schema.MenuMeta{Order: int32Ptr(4), Title: "menu.log.dataAccessAuditLog", Icon: "lucide:shield-check", Authority: authorityPlatform}},
	{name: "PermissionAuditLog", parentName: "LogAuditManagement", typ: menu.TypeMenu, path: "permission-audit-logs", component: "app/log/permission_audit_log/index.vue", hasModule: true, module: menu.ModuleLog, meta: &schema.MenuMeta{Order: int32Ptr(5), Title: "menu.log.permissionAuditLog", Icon: "lucide:shield-alert", Authority: authorityPlatform}},

	{name: "System", typ: menu.TypeCatalog, path: "/system", redirect: "/system/menus", component: "BasicLayout", meta: &schema.MenuMeta{Order: int32Ptr(2005), Title: "menu.system.moduleName", Icon: "lucide:settings", KeepAlive: boolPtr(true), Authority: authorityBoth}},
	{name: "DictManagement", parentName: "System", typ: menu.TypeMenu, path: "dict", component: "app/system/dict/index.vue", hasModule: true, module: menu.ModuleSystem, meta: &schema.MenuMeta{Order: int32Ptr(3), Title: "menu.system.dict", Icon: "lucide:library-big", Authority: authorityPlatform}},
	{name: "FileManagement", parentName: "System", typ: menu.TypeMenu, path: "files", component: "app/system/file/index.vue", hasModule: true, module: menu.ModuleSystem, meta: &schema.MenuMeta{Order: int32Ptr(4), Title: "menu.system.file", Icon: "lucide:file-search", Authority: authorityBoth}},
	{name: "TaskManagement", parentName: "System", typ: menu.TypeMenu, path: "tasks", component: "app/system/task/index.vue", hasModule: true, module: menu.ModuleSystem, meta: &schema.MenuMeta{Order: int32Ptr(5), Title: "menu.system.task", Icon: "lucide:list-todo", Authority: authorityBoth}},
	{name: "LoginPolicyManagement", parentName: "System", typ: menu.TypeMenu, path: "login-policies", component: "app/system/login_policy/index.vue", hasModule: true, module: menu.ModuleSystem, meta: &schema.MenuMeta{Order: int32Ptr(6), Title: "menu.system.loginPolicy", Icon: "lucide:shield-x", Authority: authorityPlatform}},
	{name: "LanguageManagement", parentName: "System", typ: menu.TypeMenu, path: "languages", component: "app/system/language/index.vue", hasModule: true, module: menu.ModuleSystem, meta: &schema.MenuMeta{Order: int32Ptr(7), Title: "menu.system.language", Icon: "lucide:globe", Authority: authorityPlatform}},
}
