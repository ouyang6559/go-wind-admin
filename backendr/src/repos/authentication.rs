// authentication repository（骨架）
pub struct AuthenticationRepo { db: sqlx::AnyPool }

impl AuthenticationRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
