// permission repository（骨架）
pub struct PermissionRepo { db: sqlx::AnyPool }

impl PermissionRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
