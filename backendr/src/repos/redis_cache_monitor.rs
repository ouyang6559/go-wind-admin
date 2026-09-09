// redis_cache_monitor repository（骨架）
pub struct RedisCacheMonitorRepo { db: sqlx::AnyPool }

impl RedisCacheMonitorRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
