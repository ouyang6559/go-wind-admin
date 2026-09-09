// plan_module repository（骨架）
pub struct PlanModuleRepo { db: sqlx::AnyPool }

impl PlanModuleRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
