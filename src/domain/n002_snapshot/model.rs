#[derive(Debug, Clone)]
pub struct Snapshot {
    pub id: i64,
    pub name: String,
    pub project_id: i64,
    pub comment: Option<String>,
    pub files_count: i64,
    pub dirs_count: i64,
    pub files_size_bytes: i64,
    pub loc_count: i64,
    pub scanned_at: String,
}

impl Snapshot {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: i64,
        name: String,
        project_id: i64,
        comment: Option<String>,
        files_count: i64,
        dirs_count: i64,
        files_size_bytes: i64,
        loc_count: i64,
        scanned_at: String,
    ) -> Self {
        Self {
            id,
            name,
            project_id,
            comment,
            files_count,
            dirs_count,
            files_size_bytes,
            loc_count,
            scanned_at,
        }
    }
}
