// dashboard repository（骨架）
pub struct DashboardRepo { db: sqlx::AnyPool }

impl DashboardRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
