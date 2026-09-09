// file repository（骨架）
pub struct FileRepo { db: sqlx::AnyPool }

impl FileRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
