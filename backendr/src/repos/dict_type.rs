// dict_type repository（骨架）
pub struct DictTypeRepo { db: sqlx::AnyPool }

impl DictTypeRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
