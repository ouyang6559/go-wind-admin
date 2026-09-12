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

    // 启动内嵌任务调度器：注册启用中的 PERIODIC 任务 + 系统级常驻任务
    {
        let sched = state.scheduler.clone();
        tokio::spawn(async move {
            if let Err(e) = sched.start_all().await {
                tracing::warn!(error = %e, "task scheduler start_all failed (continuing)");
            }
        });
    }

    // 4. Router（带 ConnectInfo，登录链路记录客户端 IP）+ 操作审计中间件
    // 先用 with_state 注入的中介器需要 AppState；Layer 须在 with_state 之前应用于路由。
    let sse_state = state.clone();
    let app = build_router()
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            backendr::middleware::tenant_access::tenant_gate,
        ))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            backendr::middleware::audit::audit,
        ))
        .with_state(state);

    // 5. 监听
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "listening");

    // 6. SSE 网关独立监听（默认 :7789，配置 off 关闭）；失败不阻断主服务
    tokio::spawn(async move {
        if let Err(e) = serve_sse(&sse_state).await {
            tracing::warn!(error = %e, "sse gateway stopped (main service continues)");
        }
    });

    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;

    Ok(())
}

/// SSE 网关（对齐 Go :7789 `/events`）：未配置（GW_ADMIN_SSE_ADDR=off）时静默跳过，
/// 与 Go「SSE 未配置时 NewSseServer 返回 nil」一致。
async fn serve_sse(state: &backendr::state::AppState) -> anyhow::Result<()> {
    let Some(addr) = state.config.sse_addr.clone() else {
        tracing::info!("sse gateway disabled (GW_ADMIN_SSE_ADDR=off)");
        return Ok(());
    };
    let app = backendr::sse::build_router().with_state(state.clone());
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "sse gateway listening (/events)");
    axum::serve(listener, app).await?;
    Ok(())
}
