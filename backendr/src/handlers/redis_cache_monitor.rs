// redis_cache_monitor 模块 handlers：返回 Redis 运行时只读监控视图（INFO/DBSIZE/SLOWLOG）。
// 纯透传至 repo，对齐 Go `RedisCacheMonitorService.Get` 的极简形态；fail-soft，不抛业务错误。

use axum::extract::State;
use axum::response::IntoResponse;

use crate::error::AppError;
use crate::repos::redis_cache_monitor::RedisCacheMonitorRepo;
use crate::response::json_ok;
use crate::state::AppState;

pub async fn redis_cache_monitor_get(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    // GET /redis-cache-monitor  查询缓存监控信息
    let repo = RedisCacheMonitorRepo::new();
    let info = repo.get_info(state.redis.as_ref()).await;
    Ok(json_ok(info))
}