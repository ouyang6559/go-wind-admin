// internal_message repository（骨架）
pub struct InternalMessageRepo { db: sqlx::AnyPool }

impl InternalMessageRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
