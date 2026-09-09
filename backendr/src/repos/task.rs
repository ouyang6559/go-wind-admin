// task repository（骨架）
pub struct TaskRepo { db: sqlx::AnyPool }

impl TaskRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
