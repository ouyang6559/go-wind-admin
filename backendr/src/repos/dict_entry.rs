// dict_entry repository（骨架）
pub struct DictEntryRepo { db: sqlx::AnyPool }

impl DictEntryRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
