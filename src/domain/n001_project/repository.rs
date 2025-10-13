use rusqlite::{params, Connection, Result as SqlResult};

use super::model::Project;

pub fn init_table(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS n001_project (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            root_path TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT
        )",
        [],
    )?;
    Ok(())
}

pub fn create(conn: &Connection, name: &str, root_path: &str, description: Option<&str>) -> SqlResult<i64> {
    conn.execute(
        "INSERT INTO n001_project (name, root_path, description) VALUES (?1, ?2, ?3)",
        params![name, root_path, description],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_all(conn: &Connection) -> SqlResult<Vec<Project>> {
    let mut stmt = conn.prepare(
        "SELECT id, root_path, name, description FROM n001_project ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Project::new(
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, Option<String>>(3)?,
        ))
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get_by_id(conn: &Connection, id: i64) -> SqlResult<Project> {
    conn.query_row(
        "SELECT id, root_path, name, description FROM n001_project WHERE id = ?1",
        [id],
        |row| {
            Ok(Project::new(
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, Option<String>>(3)?,
            ))
        },
    )
}

pub fn update(conn: &Connection, project: &Project) -> SqlResult<()> {
    conn.execute(
        "UPDATE n001_project SET name = ?1, root_path = ?2, description = ?3 WHERE id = ?4",
        params![project.name, project.root_path, project.description, project.id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> SqlResult<()> {
    conn.execute("DELETE FROM n001_project WHERE id = ?1", [id])?;
    Ok(())
}

