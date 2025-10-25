#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SnapshotAggregate {
    pub id: i64,
    pub snapshot_id: i64,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub file_count: i64,
    pub total_size_bytes: i64,
    pub created_at: String,
    pub updated_at: String,
}

impl SnapshotAggregate {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i64,
        snapshot_id: i64,
        code: String,
        name: String,
        description: Option<String>,
        file_count: i64,
        total_size_bytes: i64,
        created_at: String,
        updated_at: String,
    ) -> Self {
        Self {
            id,
            snapshot_id,
            code,
            name,
            description,
            file_count,
            total_size_bytes,
            created_at,
            updated_at,
        }
    }
}
