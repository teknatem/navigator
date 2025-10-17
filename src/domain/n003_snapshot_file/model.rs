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
    pub file_extension: Option<String>,
    pub crate_layer: Option<String>,
    pub artifact_type: Option<String>,
    pub artifact_id: Option<String>,
    pub artifact_name: Option<String>,
    pub role: Option<String>,
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
        file_extension: Option<String>,
        crate_layer: Option<String>,
        artifact_type: Option<String>,
        artifact_id: Option<String>,
        artifact_name: Option<String>,
        role: Option<String>,
    ) -> Self {
        Self {
            id,
            snapshot_id,
            parent_id,
            name,
            path,
            size_bytes,
            is_directory,
            file_extension,
            crate_layer,
            artifact_type,
            artifact_id,
            artifact_name,
            role,
        }
    }
}
