// plan repository（骨架）
pub struct PlanRepo { db: sqlx::AnyPool }

impl PlanRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
