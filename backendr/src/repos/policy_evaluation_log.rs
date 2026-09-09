// policy_evaluation_log repository（骨架）
pub struct PolicyEvaluationLogRepo { db: sqlx::AnyPool }

impl PolicyEvaluationLogRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
