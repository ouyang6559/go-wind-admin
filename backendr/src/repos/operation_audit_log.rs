// operation_audit_log repository（骨架）
pub struct OperationAuditLogRepo { db: sqlx::AnyPool }

impl OperationAuditLogRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
