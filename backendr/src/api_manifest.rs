//! 手工维护的 API 端点清单：Rust 无 proto/OpenAPI 注册表，作为全量重建 sys_apis 的依据。
//!
//! ⚠️ 运维约定：**新增端点时，必须同时在下方 `ENDPOINTS` 中登记**（path + method +
//! business_module），并在管理页触发「接口同步」（POST /apis/sync）触发全量重建，
//! 否则租户访问闸门会对该端点 fail-closed 403。
//!
//! path 不含 `/admin/v1` 前缀；business_module 为 sys_apis.business_module 的 DB 值
//! （DASHBOARD/OPM/.../TASK，对齐 Go pkg/constants/module_mapping.go）；未归类（如 script）
//! 记 None。
//!
//! 数据来源：backendr/src/routes/*.rs 内全部 `.route()` 注册（本清单覆盖所有接口，与
//! 2026-09-12 的 192/192 路由对齐 + Rust 特有别名，共 201 条记录）。

/// 单个端点元数据。
pub struct EndpointMeta {
    pub path: &'static str,
    pub method: &'static str,
    /// business_module 的 DB 值（如 "DASHBOARD"、"TASK"）；None 表示未归类（UNSPECIFIED）。
    pub business_module: Option<&'static str>,
}

/// 全量端点清单（手工维护）。按模块分组书写，保持可读性。
pub const ENDPOINTS: &[EndpointMeta] = &[
    // ---------- DASHBOARD ----------
    // admin_portal
    EndpointMeta { path: "/initial-context", method: "GET", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/perm-codes", method: "GET", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/routes", method: "GET", business_module: Some("DASHBOARD") },
    // authentication
    EndpointMeta { path: "/captcha", method: "GET", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/captcha/verify", method: "POST", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/login", method: "POST", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/logout", method: "POST", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/refresh-token", method: "POST", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/register", method: "POST", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/forgot-password", method: "POST", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/reset-password-by-code", method: "POST", business_module: Some("DASHBOARD") },
    // dashboard
    EndpointMeta { path: "/dashboard/login-status-distribution", method: "GET", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/dashboard/login-trend", method: "GET", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/dashboard/operation-action-distribution", method: "GET", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/dashboard/overview", method: "GET", business_module: Some("DASHBOARD") },
    // mfa
    EndpointMeta { path: "/mfa/disable", method: "POST", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/mfa/enroll/confirm", method: "POST", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/mfa/enroll/start", method: "POST", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/mfa/methods", method: "GET", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/mfa/status", method: "GET", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/mfa/verify", method: "POST", business_module: Some("DASHBOARD") },
    EndpointMeta { path: "/mfa/{credentialId}", method: "DELETE", business_module: Some("DASHBOARD") },

    // ---------- OPM ----------
    // user
    EndpointMeta { path: "/users:exists", method: "GET", business_module: Some("OPM") },
    EndpointMeta { path: "/users/exists", method: "GET", business_module: Some("OPM") },
    EndpointMeta { path: "/users", method: "GET", business_module: Some("OPM") },
    EndpointMeta { path: "/users", method: "POST", business_module: Some("OPM") },
    EndpointMeta { path: "/users/{id}", method: "GET", business_module: Some("OPM") },
    EndpointMeta { path: "/users/{id}", method: "PUT", business_module: Some("OPM") },
    EndpointMeta { path: "/users/{id}", method: "DELETE", business_module: Some("OPM") },
    EndpointMeta { path: "/users/{user_id}/password", method: "POST", business_module: Some("OPM") },
    EndpointMeta { path: "/users/username/{username}", method: "GET", business_module: Some("OPM") },
    EndpointMeta { path: "/users/username/{username}", method: "DELETE", business_module: Some("OPM") },
    // user_profile
    EndpointMeta { path: "/me", method: "GET", business_module: Some("OPM") },
    EndpointMeta { path: "/me", method: "PUT", business_module: Some("OPM") },
    EndpointMeta { path: "/me/avatar", method: "POST", business_module: Some("OPM") },
    EndpointMeta { path: "/me/avatar", method: "DELETE", business_module: Some("OPM") },
    EndpointMeta { path: "/me/contact", method: "POST", business_module: Some("OPM") },
    EndpointMeta { path: "/me/contact/verify", method: "POST", business_module: Some("OPM") },
    EndpointMeta { path: "/me/password", method: "POST", business_module: Some("OPM") },
    // org_unit
    EndpointMeta { path: "/org-units", method: "GET", business_module: Some("OPM") },
    EndpointMeta { path: "/org-units", method: "POST", business_module: Some("OPM") },
    EndpointMeta { path: "/org-units/{id}", method: "GET", business_module: Some("OPM") },
    EndpointMeta { path: "/org-units/{id}", method: "PUT", business_module: Some("OPM") },
    EndpointMeta { path: "/org-units/{id}", method: "DELETE", business_module: Some("OPM") },
    // position
    EndpointMeta { path: "/positions", method: "GET", business_module: Some("OPM") },
    EndpointMeta { path: "/positions", method: "POST", business_module: Some("OPM") },
    EndpointMeta { path: "/positions/{id}", method: "GET", business_module: Some("OPM") },
    EndpointMeta { path: "/positions/{id}", method: "PUT", business_module: Some("OPM") },
    EndpointMeta { path: "/positions/{id}", method: "DELETE", business_module: Some("OPM") },
    // role
    EndpointMeta { path: "/roles", method: "GET", business_module: Some("OPM") },
    EndpointMeta { path: "/roles", method: "POST", business_module: Some("OPM") },
    EndpointMeta { path: "/roles/{id}", method: "GET", business_module: Some("OPM") },
    EndpointMeta { path: "/roles/{id}", method: "PUT", business_module: Some("OPM") },
    EndpointMeta { path: "/roles/{id}", method: "DELETE", business_module: Some("OPM") },

    // ---------- PERMISSION ----------
    // menu
    EndpointMeta { path: "/menus", method: "GET", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/menus", method: "POST", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/menus/sync", method: "POST", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/menus/{id}", method: "GET", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/menus/{id}", method: "PUT", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/menus/{id}", method: "DELETE", business_module: Some("PERMISSION") },
    // api
    EndpointMeta { path: "/apis", method: "GET", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/apis", method: "POST", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/apis/sync", method: "POST", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/apis/walk-route", method: "GET", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/apis/{id}", method: "GET", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/apis/{id}", method: "PUT", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/apis/{id}", method: "DELETE", business_module: Some("PERMISSION") },
    // permission
    EndpointMeta { path: "/permissions", method: "GET", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/permissions", method: "POST", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/permissions/sync:perms", method: "POST", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/permissions/sync/perms", method: "POST", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/permissions/{id}", method: "GET", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/permissions/{id}", method: "PUT", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/permissions/{id}", method: "DELETE", business_module: Some("PERMISSION") },
    // permission_group
    EndpointMeta { path: "/permission-groups", method: "GET", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/permission-groups", method: "POST", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/permission-groups/{id}", method: "GET", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/permission-groups/{id}", method: "PUT", business_module: Some("PERMISSION") },
    EndpointMeta { path: "/permission-groups/{id}", method: "DELETE", business_module: Some("PERMISSION") },

    // ---------- DICT ----------
    // dict_type
    EndpointMeta { path: "/dict/types", method: "GET", business_module: Some("DICT") },
    EndpointMeta { path: "/dict/types", method: "POST", business_module: Some("DICT") },
    EndpointMeta { path: "/dict/types", method: "DELETE", business_module: Some("DICT") },
    EndpointMeta { path: "/dict/types/code/{code}", method: "GET", business_module: Some("DICT") },
    EndpointMeta { path: "/dict/types/{id}", method: "GET", business_module: Some("DICT") },
    EndpointMeta { path: "/dict/types/{id}", method: "PUT", business_module: Some("DICT") },
    // dict_entry
    EndpointMeta { path: "/dict/entries", method: "GET", business_module: Some("DICT") },
    EndpointMeta { path: "/dict/entries", method: "POST", business_module: Some("DICT") },
    EndpointMeta { path: "/dict/entries", method: "DELETE", business_module: Some("DICT") },
    EndpointMeta { path: "/dict/entries/by-type-code", method: "GET", business_module: Some("DICT") },
    EndpointMeta { path: "/dict/entries/{id}", method: "PUT", business_module: Some("DICT") },

    // ---------- SYSTEM ----------
    // access_key（OpenAPI 凭证；token 交换免鉴权）
    EndpointMeta { path: "/access-keys", method: "GET", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/access-keys", method: "POST", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/access-keys/{id}", method: "GET", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/access-keys/{id}", method: "PUT", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/access-keys/{id}", method: "DELETE", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/access-keys/{id}/secret", method: "PUT", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/access-keys/token", method: "POST", business_module: Some("SYSTEM") },
    // config（系统参数）
    EndpointMeta { path: "/configs", method: "GET", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/configs", method: "POST", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/configs/{id}", method: "GET", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/configs/{id}", method: "PUT", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/configs/{id}", method: "DELETE", business_module: Some("SYSTEM") },
    // language
    EndpointMeta { path: "/dict/langs", method: "GET", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/dict/langs", method: "POST", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/dict/langs", method: "DELETE", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/dict/langs/batch", method: "POST", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/dict/langs/{id}", method: "GET", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/dict/langs/{id}", method: "PUT", business_module: Some("SYSTEM") },
    // login_policy
    EndpointMeta { path: "/login-policies", method: "GET", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/login-policies", method: "POST", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/login-policies/{id}", method: "GET", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/login-policies/{id}", method: "PUT", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/login-policies/{id}", method: "DELETE", business_module: Some("SYSTEM") },
    // server_monitor
    EndpointMeta { path: "/server-monitor", method: "GET", business_module: Some("SYSTEM") },
    // notification_channel
    EndpointMeta { path: "/notification-channels", method: "GET", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/notification-channels", method: "POST", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/notification-channels/{id}", method: "GET", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/notification-channels/{id}", method: "PUT", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/notification-channels/{id}", method: "DELETE", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/notification-channels/{id}/send-test-email", method: "POST", business_module: Some("SYSTEM") },
    // online_session
    EndpointMeta { path: "/online-session/sessions", method: "GET", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/online-session/my-sessions", method: "GET", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/online-session/force-logout", method: "POST", business_module: Some("SYSTEM") },
    EndpointMeta { path: "/online-session/my-sessions/revoke", method: "POST", business_module: Some("SYSTEM") },

    // ---------- FILE ----------
    // file
    EndpointMeta { path: "/files", method: "GET", business_module: Some("FILE") },
    EndpointMeta { path: "/files", method: "POST", business_module: Some("FILE") },
    EndpointMeta { path: "/files/{id}", method: "GET", business_module: Some("FILE") },
    EndpointMeta { path: "/files/{id}", method: "PUT", business_module: Some("FILE") },
    EndpointMeta { path: "/files/{id}", method: "DELETE", business_module: Some("FILE") },
    // file_transfer
    EndpointMeta { path: "/file/download", method: "GET", business_module: Some("FILE") },
    EndpointMeta { path: "/file/upload", method: "PUT", business_module: Some("FILE") },
    EndpointMeta { path: "/file/upload", method: "POST", business_module: Some("FILE") },
    EndpointMeta { path: "/file/image", method: "GET", business_module: Some("FILE") },

    // ---------- TASK ----------
    EndpointMeta { path: "/tasks", method: "GET", business_module: Some("TASK") },
    EndpointMeta { path: "/tasks", method: "POST", business_module: Some("TASK") },
    EndpointMeta { path: "/tasks/type-name/{type_name}", method: "GET", business_module: Some("TASK") },
    EndpointMeta { path: "/tasks/{id}", method: "GET", business_module: Some("TASK") },
    EndpointMeta { path: "/tasks/{id}", method: "PUT", business_module: Some("TASK") },
    EndpointMeta { path: "/tasks/{id}", method: "DELETE", business_module: Some("TASK") },
    // 任务控制：proto 冒号风格
    EndpointMeta { path: "/tasks:control", method: "POST", business_module: Some("TASK") },
    EndpointMeta { path: "/tasks:restart", method: "POST", business_module: Some("TASK") },
    EndpointMeta { path: "/tasks:start", method: "POST", business_module: Some("TASK") },
    EndpointMeta { path: "/tasks:stop", method: "POST", business_module: Some("TASK") },
    EndpointMeta { path: "/tasks:type-names", method: "GET", business_module: Some("TASK") },
    // 任务控制：斜杠别名
    EndpointMeta { path: "/tasks/control", method: "POST", business_module: Some("TASK") },
    EndpointMeta { path: "/tasks/restart", method: "POST", business_module: Some("TASK") },
    EndpointMeta { path: "/tasks/start", method: "POST", business_module: Some("TASK") },
    EndpointMeta { path: "/tasks/stop", method: "POST", business_module: Some("TASK") },
    EndpointMeta { path: "/tasks/type-names", method: "GET", business_module: Some("TASK") },

    // ---------- TENANT ----------
    // tenant
    EndpointMeta { path: "/tenants:exists", method: "GET", business_module: Some("TENANT") },
    EndpointMeta { path: "/tenants/exists", method: "GET", business_module: Some("TENANT") },
    EndpointMeta { path: "/tenants", method: "GET", business_module: Some("TENANT") },
    EndpointMeta { path: "/tenants", method: "POST", business_module: Some("TENANT") },
    EndpointMeta { path: "/tenants/{id}", method: "GET", business_module: Some("TENANT") },
    EndpointMeta { path: "/tenants/{id}", method: "PUT", business_module: Some("TENANT") },
    EndpointMeta { path: "/tenants/{id}", method: "DELETE", business_module: Some("TENANT") },
    EndpointMeta { path: "/tenants/{id}/cleanup", method: "POST", business_module: Some("TENANT") },
    EndpointMeta { path: "/tenants/{id}/usage", method: "GET", business_module: Some("TENANT") },
    EndpointMeta { path: "/tenants:with-admin", method: "POST", business_module: Some("TENANT") },
    // plan
    EndpointMeta { path: "/plans", method: "GET", business_module: Some("TENANT") },
    EndpointMeta { path: "/plans", method: "POST", business_module: Some("TENANT") },
    EndpointMeta { path: "/plans", method: "DELETE", business_module: Some("TENANT") },
    EndpointMeta { path: "/plans/{id}", method: "GET", business_module: Some("TENANT") },
    EndpointMeta { path: "/plans/{id}", method: "PUT", business_module: Some("TENANT") },
    // plan_module
    EndpointMeta { path: "/plan-modules", method: "GET", business_module: Some("TENANT") },
    EndpointMeta { path: "/plan-modules", method: "POST", business_module: Some("TENANT") },
    EndpointMeta { path: "/plan-modules", method: "DELETE", business_module: Some("TENANT") },
    EndpointMeta { path: "/plan-modules/{id}", method: "GET", business_module: Some("TENANT") },
    EndpointMeta { path: "/plan-modules/{id}", method: "PUT", business_module: Some("TENANT") },
    // plan_quota
    EndpointMeta { path: "/plan-quotas", method: "GET", business_module: Some("TENANT") },
    EndpointMeta { path: "/plan-quotas", method: "POST", business_module: Some("TENANT") },
    EndpointMeta { path: "/plan-quotas", method: "DELETE", business_module: Some("TENANT") },
    EndpointMeta { path: "/plan-quotas/{id}", method: "PUT", business_module: Some("TENANT") },

    // ---------- LOG ----------
    // api_audit_log
    EndpointMeta { path: "/api-audit-logs", method: "GET", business_module: Some("LOG") },
    EndpointMeta { path: "/api-audit-logs/{id}", method: "GET", business_module: Some("LOG") },
    // login_audit_log
    EndpointMeta { path: "/login-audit-logs", method: "GET", business_module: Some("LOG") },
    EndpointMeta { path: "/login-audit-logs/{id}", method: "GET", business_module: Some("LOG") },
    // operation_audit_log
    EndpointMeta { path: "/operation-audit-logs", method: "GET", business_module: Some("LOG") },
    EndpointMeta { path: "/operation-audit-logs/{id}", method: "GET", business_module: Some("LOG") },
    // permission_audit_log
    EndpointMeta { path: "/permission-audit-logs", method: "GET", business_module: Some("LOG") },
    EndpointMeta { path: "/permission-audit-logs/{id}", method: "GET", business_module: Some("LOG") },
    // data_access_audit_log
    EndpointMeta { path: "/data-access-audit-logs", method: "GET", business_module: Some("LOG") },
    EndpointMeta { path: "/data-access-audit-logs/{id}", method: "GET", business_module: Some("LOG") },
    // policy_evaluation_log
    EndpointMeta { path: "/policy-evaluation-logs", method: "GET", business_module: Some("LOG") },
    EndpointMeta { path: "/policy-evaluation-logs/{id}", method: "GET", business_module: Some("LOG") },
    // redis_cache_monitor
    EndpointMeta { path: "/redis-cache-monitor", method: "GET", business_module: Some("LOG") },

    // ---------- INTERNAL_MESSAGE ----------
    // internal_message
    EndpointMeta { path: "/internal-message/messages", method: "GET", business_module: Some("INTERNAL_MESSAGE") },
    EndpointMeta { path: "/internal-message/messages/{id}", method: "GET", business_module: Some("INTERNAL_MESSAGE") },
    EndpointMeta { path: "/internal-message/messages/{id}", method: "PUT", business_module: Some("INTERNAL_MESSAGE") },
    EndpointMeta { path: "/internal-message/messages/{id}", method: "DELETE", business_module: Some("INTERNAL_MESSAGE") },
    EndpointMeta { path: "/internal-message/revoke", method: "POST", business_module: Some("INTERNAL_MESSAGE") },
    EndpointMeta { path: "/internal-message/send", method: "POST", business_module: Some("INTERNAL_MESSAGE") },
    // internal_message_category
    EndpointMeta { path: "/internal-message/categories", method: "GET", business_module: Some("INTERNAL_MESSAGE") },
    EndpointMeta { path: "/internal-message/categories", method: "POST", business_module: Some("INTERNAL_MESSAGE") },
    EndpointMeta { path: "/internal-message/categories/{id}", method: "GET", business_module: Some("INTERNAL_MESSAGE") },
    EndpointMeta { path: "/internal-message/categories/{id}", method: "PUT", business_module: Some("INTERNAL_MESSAGE") },
    EndpointMeta { path: "/internal-message/categories/{id}", method: "DELETE", business_module: Some("INTERNAL_MESSAGE") },
    // internal_message_recipient
    EndpointMeta { path: "/internal-message/inbox", method: "GET", business_module: Some("INTERNAL_MESSAGE") },
    EndpointMeta { path: "/internal-message/inbox/delete", method: "POST", business_module: Some("INTERNAL_MESSAGE") },
    EndpointMeta { path: "/internal-message/read", method: "POST", business_module: Some("INTERNAL_MESSAGE") },
    EndpointMeta { path: "/internal-message/status", method: "POST", business_module: Some("INTERNAL_MESSAGE") },

    // ---------- 未归类（Go module_mapping 无 script 服务，UNSPECIFIED） ----------
    // script
    EndpointMeta { path: "/scripts", method: "GET", business_module: None },
    EndpointMeta { path: "/scripts", method: "POST", business_module: None },
    EndpointMeta { path: "/scripts", method: "DELETE", business_module: None },
    EndpointMeta { path: "/scripts/count", method: "GET", business_module: None },
    EndpointMeta { path: "/scripts/name/{name}", method: "GET", business_module: None },
    EndpointMeta { path: "/scripts/{id}", method: "GET", business_module: None },
    EndpointMeta { path: "/scripts/{id}", method: "PUT", business_module: None },
    EndpointMeta { path: "/scripts/test_run", method: "POST", business_module: None },
    EndpointMeta { path: "/script/hooks", method: "GET", business_module: None },
    // script_log
    EndpointMeta { path: "/script/logs", method: "GET", business_module: None },
    EndpointMeta { path: "/script/logs/count", method: "GET", business_module: None },
    EndpointMeta { path: "/script/logs/purge", method: "POST", business_module: None },
];