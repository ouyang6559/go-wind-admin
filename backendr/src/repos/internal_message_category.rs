// internal_message_category repository（骨架）
pub struct InternalMessageCategoryRepo { db: sqlx::AnyPool }

impl InternalMessageCategoryRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
