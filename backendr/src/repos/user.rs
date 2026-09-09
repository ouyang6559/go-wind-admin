// user repository（骨架）
pub struct UserRepo { db: sqlx::AnyPool }

impl UserRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
