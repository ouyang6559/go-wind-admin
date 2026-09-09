//! 应用配置：从环境变量加载，构建 `AppConfig`。
//!
//! 命名与 go-zero backendz 的 `etc/admin.yaml` 字段一一对应，
//! 便于后续双端对齐。字段缺失时提供开发默认值。

use std::env;

/// 运行环境
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Env {
    Dev,
    Test,
    Prod,
}

impl Env {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "prod" | "production" => Env::Prod,
            "test" | "testing" => Env::Test,
            _ => Env::Dev,
        }
    }
}

/// 全局配置（线程安全，启动时一次构建，运行期只读）
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// 运行环境
    pub env: Env,
    /// HTTP 监听地址，如 `0.0.0.0:7666`
    pub http_addr: String,
    /// SQL 数据库 DSN（sqlx AnyPool 支持 postgres/mysql/sqlite）
    pub database_url: Option<String>,
    /// Redis 连接地址，如 `redis://127.0.0.1:6379`
    pub redis_url: Option<String>,
    /// JWT 签名密钥
    pub jwt_secret: String,
    /// 默认租户标识（多租户隔离，tenantScoped 过滤用）
    pub default_tenant_id: i64,
    /// 每个请求的上传限制（字节）
    pub max_body_bytes: usize,
}

impl AppConfig {
    /// 从环境变量加载。首次调用前可用 `dotenvy::dotenv()` 读取 `.env`。
    pub fn from_env() -> Self {
        AppConfig {
            env: Env::from_str(&env_or("GW_ADMIN_ENV", "dev")),
            http_addr: env_or("GW_ADMIN_HTTP_ADDR", "0.0.0.0:7666"),
            database_url: env_or_opt("GW_ADMIN_DATABASE_URL"),
            redis_url: env_or_opt("GW_ADMIN_REDIS_URL"),
            jwt_secret: env_or("GW_ADMIN_JWT_SECRET", "dev-secret-change-me"),
            default_tenant_id: env_or("GW_ADMIN_DEFAULT_TENANT_ID", "1")
                .parse()
                .unwrap_or(1),
            max_body_bytes: env_or("GW_ADMIN_MAX_BODY_BYTES", "10485760")
                .parse()
                .unwrap_or(10 * 1024 * 1024),
        }
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_or_opt(key: &str) -> Option<String> {
    match env::var(key) {
        Ok(v) if !v.is_empty() => Some(v),
        _ => None,
    }
}