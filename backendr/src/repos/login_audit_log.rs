// login_audit_log repository（骨架）
pub struct LoginAuditLogRepo { db: sqlx::AnyPool }

impl LoginAuditLogRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
