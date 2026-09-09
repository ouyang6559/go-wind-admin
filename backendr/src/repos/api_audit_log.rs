// api_audit_log repository（骨架）
pub struct ApiAuditLogRepo { db: sqlx::AnyPool }

impl ApiAuditLogRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
