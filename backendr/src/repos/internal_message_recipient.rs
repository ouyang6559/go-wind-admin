// internal_message_recipient repository（骨架）
pub struct InternalMessageRecipientRepo { db: sqlx::AnyPool }

impl InternalMessageRecipientRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
