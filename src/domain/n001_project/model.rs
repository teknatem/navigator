#[derive(Debug, Clone)]
pub struct Project {
    pub id: i64,
    pub root_path: String,
    pub name: String,
    pub description: Option<String>,
}

impl Project {
    pub fn new(id: i64, root_path: String, name: String, description: Option<String>) -> Self {
        Self { id, root_path, name, description }
    }
}

