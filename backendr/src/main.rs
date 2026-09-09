//! GoWind Admin — Rust/axum 服务入口。
//!
//! 启动流程：加载 .env -> 初始化 tracing -> 构建 AppState（尽力连接池）
//! -> 组装 Router -> 绑定监听地址。

use backendr::config::AppConfig;
use backendr::routes::build_router;
use backendr::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. 环境变量
    dotenvy::dotenv().ok();

    // 2. 日志
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "backendr=info,tower_http=info,axum::rejection=trace".into()),
        )
        .with_target(true)
        .init();

    // 3. 配置 + 状态
    let config = AppConfig::from_env();
    let addr = config.http_addr.clone();
    let state = AppState::new(config).await;
    tracing::info!(meta = "GoWind Admin backendr (axum) start", addr = %addr);

    // 4. Router
    let app = build_router().with_state(state);

    // 5. 监听
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "listening");
    axum::serve(listener, app).await?;

    Ok(())
}