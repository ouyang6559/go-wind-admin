// org_unit repository（骨架）
pub struct OrgUnitRepo { db: sqlx::AnyPool }

impl OrgUnitRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
