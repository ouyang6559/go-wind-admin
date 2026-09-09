// permission_audit_log repository（骨架）
pub struct PermissionAuditLogRepo { db: sqlx::AnyPool }

impl PermissionAuditLogRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
