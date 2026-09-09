// role repository（骨架）
pub struct RoleRepo { db: sqlx::AnyPool }

impl RoleRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
