#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SnapshotFile {
    pub id: i64,
    pub snapshot_id: i64,
    pub parent_id: Option<i64>,
    pub name: String,
    pub path: String,
    pub size_bytes: i64,
    pub is_directory: bool,
}

impl SnapshotFile {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i64,
        snapshot_id: i64,
        parent_id: Option<i64>,
        name: String,
        path: String,
        size_bytes: i64,
        is_directory: bool,
    ) -> Self {
        Self {
            id,
            snapshot_id,
            parent_id,
            name,
            path,
            size_bytes,
            is_directory,
        }
    }
}
