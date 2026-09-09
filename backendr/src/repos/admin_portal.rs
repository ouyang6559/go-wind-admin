// admin_portal repository（骨架）
pub struct AdminPortalRepo { db: sqlx::AnyPool }

impl AdminPortalRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
