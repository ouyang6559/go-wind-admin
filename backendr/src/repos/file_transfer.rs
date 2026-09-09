// file_transfer repository（骨架）
pub struct FileTransferRepo { db: sqlx::AnyPool }

impl FileTransferRepo {
    pub fn new(db: sqlx::AnyPool) -> Self { Self { db } }
}
