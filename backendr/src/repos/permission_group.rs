// permission_group repository（骨架）
pub struct PermissionGroupRepo { db: sqlx::AnyPool }

impl PermissionGroupRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
