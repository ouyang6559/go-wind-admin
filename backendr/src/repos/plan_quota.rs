// plan_quota repository（骨架）
pub struct PlanQuotaRepo { db: sqlx::AnyPool }

impl PlanQuotaRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
