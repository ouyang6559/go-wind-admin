// login_policy repository（骨架）
pub struct LoginPolicyRepo { db: sqlx::AnyPool }

impl LoginPolicyRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
