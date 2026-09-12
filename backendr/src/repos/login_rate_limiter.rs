// login_rate_limiter：基于 Redis 的登录失败计数与锁定（对齐 Go login_rate_limiter.go）。
//
// 按 IP 与用户名双维度限流：任一维度失败计数达到阈值即锁定一个锁定窗口；
// 成功后清零。Redis 不可用时各操作 fail-open（不阻断登录）——限流属防御性增强，
// 不应让 Redis 故障拖垮登录整体可用性（与 Go 一致）。
//
// 阈值 / 窗口对齐 Go：登录失败阈值 5，锁定窗口 15 分钟，失败计数器 TTL=锁定窗口。

use redis::AsyncCommands;

use crate::error::AppError;

pub(crate) const FAIL_THRESHOLD: i64 = 5;
/// 锁定窗口（秒）：15 分钟
pub(crate) const LOCKOUT_SECS: u64 = 15 * 60;

/// 登录失败限流器。持有一个 redis::Client，未配置 Redis 时由调用方直接跳过。
pub struct LoginRateLimiter {
    client: redis::Client,
}

/// 原子「判定是否已锁定 → 自增失败计数 → 设置 TTL」Lua 脚本（语义对齐 Go）。
/// 已锁定（当前计数 >= 阈值）则不再自增并返回 {1, current}；否则自增并在
/// 首次失败时设置过期，返回 {0, current}。
fn incr_if_not_locked_script() -> redis::Script {
    redis::Script::new(
        r#"
        local key = KEYS[1]
        local threshold = tonumber(ARGV[1])
        local ttl = tonumber(ARGV[2])

        local current = tonumber(redis.call('GET', key) or "0")
        if current >= threshold then
            return {1, current}
        end

        current = redis.call('INCR', key)
        if current == 1 then
            redis.call('EXPIRE', key, ttl)
        end
        return {0, current}
        "#,
    )
}

impl LoginRateLimiter {
    pub fn new(client: redis::Client) -> Self {
        Self { client }
    }

    /// 按 IP 与用户名生成两个维度的失败计数键。空值维度跳过
    /// （例如内网无 X-Forwarded-For 时仅有用户名维度）。
    fn fail_keys(ip: &str, username: &str) -> Vec<String> {
        let mut keys = Vec::with_capacity(2);
        if !ip.is_empty() {
            keys.push(format!("gowind:login:fail:ip:{}", ip));
        }
        if !username.is_empty() {
            keys.push(format!("gowind:login:fail:user:{}", username));
        }
        keys
    }

    /// 登录前前置拦截：查询当前 IP/用户名是否处于锁定状态（不自增计数）。
    /// Redis 错误返回 Err，调用方应 fail-open（记日志继续）。
    pub async fn is_locked(&self, ip: &str, username: &str) -> Result<bool, AppError> {
        let mut con = self
            .client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| AppError::Redis(e))?;
        for key in Self::fail_keys(ip, username) {
            let cnt: Option<i64> = con.get(&key).await.map_err(AppError::Redis)?;
            if cnt.unwrap_or(0) >= FAIL_THRESHOLD {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// 登录失败时自增计数（按 IP + 用户名双维度），并返回是否已触发锁定。
    /// 任一维度超阈值即视为锁定。Redis 错误返回 Err。
    pub async fn check_and_incr(&self, ip: &str, username: &str) -> Result<bool, AppError> {
        let mut con = self
            .client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| AppError::Redis(e))?;
        let keys = Self::fail_keys(ip, username);
        let mut script = incr_if_not_locked_script();
        for key in keys {
            let res: (i64, i64) = script
                .key(key)
                .arg(FAIL_THRESHOLD)
                .arg(LOCKOUT_SECS)
                .invoke_async(&mut con)
                .await
                .map_err(AppError::Redis)?;
            if res.0 == 1 {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// 登录成功后清零失败计数（best-effort）。
    pub async fn reset(&self, ip: &str, username: &str) -> Result<(), AppError> {
        let mut con = self
            .client
            .get_multiplexed_tokio_connection()
            .await
            .map_err(|e| AppError::Redis(e))?;
        for key in Self::fail_keys(ip, username) {
            let _: () = con.del(&key).await.map_err(AppError::Redis)?;
        }
        Ok(())
    }
}