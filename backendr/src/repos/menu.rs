// menu repository（骨架）
pub struct MenuRepo { db: sqlx::AnyPool }

impl MenuRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
