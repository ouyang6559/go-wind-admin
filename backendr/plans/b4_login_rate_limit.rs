//! B4 脚手架：登录限流 + login_policy 闸门。
//!
//! 目标：补齐 Go 的双维度失败计数锁定（IP + 用户名）与 `sys_login_policy`
//! 表驱动的全局/用户定向策略，并整合进登录链路。
//!
//! 现状接线点：`src/handlers/authentication.rs` `authentication_login`（约 L211）
//! 目前只做「验证码 + 凭证校验」＋MFA 闸门，未做失败锁定与策略判定。
//!
//! Redis 键（沿用现有前缀习惯）：
//! - 失败计数：`gw:loginfail:ip:{ip}`、`gw:loginfail:user:{username}`（INCR + EXPIRE）
//! - 锁定标记：`gw:lock:ip:{ip}`、`gw:lock:user:{username}`
//!
//! 对齐 Go：失败计数达阈值 → 锁定一段时间（threshold / window / lock_seconds 来自 policy）。

use std::time::Duration;

/// 登录尝试结果。
pub enum Attempt {
    Denied { reason: String },
    Allowed,
}

/// 登录策略判定结果（对齐 `LoginPolicy` DTO 语义）。
#[derive(Debug, Clone, Default)]
pub struct Policy {
    pub max_failed_attempts: i32,
    pub lock_seconds: i64,
    pub lock_enabled: bool,
    pub notify_failed: bool,
}

/// Redis 计数缓存访问（用 `crate::state` 现有 RedisPool；此处仅留签名）。
pub struct FailCounter<'a> {
    _db: &'a crate::state::AppState,
}

impl<'a> FailCounter<'a> {
    pub fn new(db: &'a crate::state::AppState) -> Self {
        FailCounter { _db: db }
    }

    /// 记录一次失败：incr ip / user 两把计数钥匙，超过阈值则上锁。
    #[allow(unused_variables)]
    pub async fn record_failed(&self, ip: &str, username: &str, policy: &Policy) {
        // redis INCR gw:loginfail:ip:{ip} → if n >= max_failed_attempts { SET gw:lock:ip:{ip} EX lock_seconds }
        // redis INCR gw:loginfail:user:{username} → 同上用户名锁
        // policy.lock_enabled=false 或 lock_seconds=0 时不落锁
    }

    /// 登录前查询是否被锁定；命中返回拒绝原因。
    #[allow(unused_variables)]
    pub async fn check_locked(&self, ip: &str, username: &str) -> Option<String> {
        // EXISTS gw:lock:ip:{ip} / gw:lock:user:{username} → 返回「尝试过多，请稍后再试」
        None
    }

    /// 登录成功清零计数。
    #[allow(unused_variables)]
    pub async fn reset(&self, ip: &str, username: &str) {
        // DEL gw:loginfail:ip:{ip} gw:loginfail:user:{username} gw:lock:*
    }
}

/// 从 `sys_login_policy` 装载有效策略（全局 + 定向 username / ip）。
/// 读取 `src/handlers/login_policy.rs` 已实现的 CRUD 底层表。
#[allow(unused_variables)]
pub async fn load_effective_policy(
    state: &crate::state::AppState,
    username: &str,
    ip: &str,
) -> Policy {
    // SELECT ... FROM sys_login_policy WHERE (scope='global' OR scope='user' AND matching=username
    //        OR scope='ip' AND matching=ip) AND enabled=true
    // 取最严格组合；无记录则给 Go 默认（如 max_failed_attempts=5, lock_seconds=300）
    let _ = Duration::from_secs(0);
    Policy {
        max_failed_attempts: 5,
        lock_seconds: 300,
        lock_enabled: true,
        notify_failed: false,
    }
}

/// 在 `authentication_login` 中这样接入（示意）：
/// ```ignore
/// // 1) 策略装载 + 锁定检查（在验证码校验前后均可）
/// let policy = load_effective_policy(&state, &body.username, &client_ip).await;
/// let counter = FailCounter::new(&state);
/// if let Some(reason) = counter.check_locked(&client_ip, &body.username).await {
///     return Err(AppError::Validation(reason));   // 或滑窗组合的限流响应
/// }
/// // 2) 凭证/验证码失败 →
/// counter.record_failed(&client_ip, &body.username, &policy).await;
/// return Err(...);
/// // 3) 成功 →
/// counter.reset(&client_ip, &body.username).await;
/// ```
/// 同时可叠加纯 IP 滑动窗口限流（如 60s 内 N 次）作为浅层防线，与上面的阈值锁互补。