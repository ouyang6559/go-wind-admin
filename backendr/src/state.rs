//! 应用运行时状态（`AppState`）。
//!
//! 通过 `axum::State` 注入到所有 handler 中，承载连接池与配置。
//! DB / Redis 连接失败时**记录告警而非 panic**，保证骨架可先行启动、
//! 无外部依赖也能跑通 `/admin/v1` 路由（返回 NotImplemented）。

use std::sync::Arc;

use crate::config::AppConfig;

/// 全局共享状态
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    /// SQL 连接池（AnyPool，支持 postgres/mysql/sqlite）
    pub db: Option<sqlx::AnyPool>,
    /// Redis 客户端（连接在使用时按需建立）
    pub redis: Option<redis::Client>,
    /// JWT 密钥
    pub jwt_secret: String,
}

impl AppState {
    /// 构建状态：尽力连接 DB / 初始化 Redis，失败仅告警。
    pub async fn new(config: AppConfig) -> Self {
        // sqlx::any 要求先注册已编译进二进制（按 feature）的驱动，否则连接时 panic
        sqlx::any::install_default_drivers();
        let db = match &config.database_url {
            Some(url) => match sqlx::AnyPool::connect(url).await {
                Ok(pool) => Some(pool),
                Err(e) => {
                    tracing::warn!(error = %e, "database connect skipped (startup best-effort)");
                    None
                }
            },
            None => None,
        };

        let redis = match &config.redis_url {
            Some(url) => match redis::Client::open(url.clone()) {
                Ok(client) => Some(client),
                Err(e) => {
                    tracing::warn!(error = %e, "redis client init skipped (startup best-effort)");
                    None
                }
            },
            None => None,
        };

        let jwt_secret = config.jwt_secret.clone();
        AppState {
            config: Arc::new(config),
            db,
            redis,
            jwt_secret,
        }
    }
}