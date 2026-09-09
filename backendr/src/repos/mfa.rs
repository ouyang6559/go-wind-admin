// mfa repository（骨架）
pub struct MfaRepo { db: sqlx::AnyPool }

impl MfaRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
