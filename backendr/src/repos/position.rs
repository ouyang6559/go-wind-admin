// position repository（骨架）
pub struct PositionRepo { db: sqlx::AnyPool }

impl PositionRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
