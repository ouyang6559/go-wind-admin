// @generated
//! 路由聚合：每个业务模块一个子文件，统一挂载到 /admin/v1 下。
pub mod admin_portal;
pub mod api;
pub mod api_audit_log;
pub mod authentication;
pub mod dashboard;
pub mod data_access_audit_log;
pub mod dict_entry;
pub mod dict_type;
pub mod file;
pub mod file_transfer;
pub mod internal_message;
pub mod internal_message_category;
pub mod internal_message_recipient;
pub mod language;
pub mod login_audit_log;
pub mod login_policy;
pub mod menu;
pub mod mfa;
pub mod operation_audit_log;
pub mod org_unit;
pub mod permission;
pub mod permission_audit_log;
pub mod permission_group;
pub mod plan;
pub mod plan_module;
pub mod plan_quota;
pub mod policy_evaluation_log;
pub mod position;
pub mod redis_cache_monitor;
pub mod role;
pub mod task;
pub mod tenant;
pub mod user;
pub mod user_profile;

use axum::Router;
use crate::state::AppState;

pub fn build_router() -> Router<AppState> {
    let mut r = Router::new();
    r = r.nest("/admin/v1", self_router());
    r = r.merge(crate::routes::notification_channel::build());
    r = r.merge(crate::routes::online_session::build());
    r = r.merge(crate::routes::script::build());
    r = r.merge(crate::routes::script_log::build());
    r = r.merge(crate::routes::server_monitor::build());
    r
}

// 每个模块的 Router 携带完整路径（含 /admin/v1），这里仅作聚合。
fn self_router() -> Router<AppState> {
    let mut r = Router::new();
    r = r.merge(crate::routes::admin_portal::build());
    r = r.merge(crate::routes::api::build());
    r = r.merge(crate::routes::api_audit_log::build());
    r = r.merge(crate::routes::authentication::build());
    r = r.merge(crate::routes::dashboard::build());
    r = r.merge(crate::routes::data_access_audit_log::build());
    r = r.merge(crate::routes::dict_entry::build());
    r = r.merge(crate::routes::dict_type::build());
    r = r.merge(crate::routes::file::build());
    r = r.merge(crate::routes::file_transfer::build());
    r = r.merge(crate::routes::internal_message::build());
    r = r.merge(crate::routes::internal_message_category::build());
    r = r.merge(crate::routes::internal_message_recipient::build());
    r = r.merge(crate::routes::language::build());
    r = r.merge(crate::routes::login_audit_log::build());
    r = r.merge(crate::routes::login_policy::build());
    r = r.merge(crate::routes::menu::build());
    r = r.merge(crate::routes::mfa::build());
    r = r.merge(crate::routes::operation_audit_log::build());
    r = r.merge(crate::routes::org_unit::build());
    r = r.merge(crate::routes::permission::build());
    r = r.merge(crate::routes::permission_audit_log::build());
    r = r.merge(crate::routes::permission_group::build());
    r = r.merge(crate::routes::plan::build());
    r = r.merge(crate::routes::plan_module::build());
    r = r.merge(crate::routes::plan_quota::build());
    r = r.merge(crate::routes::policy_evaluation_log::build());
    r = r.merge(crate::routes::position::build());
    r = r.merge(crate::routes::redis_cache_monitor::build());
    r = r.merge(crate::routes::role::build());
    r = r.merge(crate::routes::task::build());
    r = r.merge(crate::routes::tenant::build());
    r = r.merge(crate::routes::user::build());
    r = r.merge(crate::routes::user_profile::build());
    r = r.merge(crate::routes::notification_channel::build());
    r = r.merge(crate::routes::online_session::build());
    r = r.merge(crate::routes::script::build());
    r = r.merge(crate::routes::script_log::build());
    r = r.merge(crate::routes::server_monitor::build());
    r
}
pub mod notification_channel;
pub mod online_session;
pub mod script;
pub mod script_log;
pub mod server_monitor;
