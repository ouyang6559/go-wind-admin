// tenant repository（骨架）
pub struct TenantRepo { db: sqlx::AnyPool }

impl TenantRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
