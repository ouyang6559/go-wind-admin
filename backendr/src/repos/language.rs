// language repository（骨架）
pub struct LanguageRepo { db: sqlx::AnyPool }

impl LanguageRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
