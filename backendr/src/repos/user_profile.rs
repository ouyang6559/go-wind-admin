// user_profile repository（骨架）
pub struct UserProfileRepo { db: sqlx::AnyPool }

impl UserProfileRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
