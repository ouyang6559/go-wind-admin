// api repository（骨架）
pub struct ApiRepo { db: sqlx::AnyPool }

impl ApiRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
