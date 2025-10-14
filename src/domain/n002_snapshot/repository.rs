use rusqlite::{Connection, Result as SqlResult};
use rusqlite::params;

use super::model::Snapshot;

pub fn init_table(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS n002_snapshot (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            project_id INTEGER NOT NULL,
            comment TEXT,
            files_count INTEGER NOT NULL,
            dirs_count INTEGER NOT NULL,
            files_size_bytes INTEGER NOT NULL,
            loc_count INTEGER NOT NULL,
            scanned_at TEXT NOT NULL
        )",
        [],
    )?;
    Ok(())
}

pub fn list_all(conn: &Connection) -> SqlResult<Vec<Snapshot>> {
    let mut stmt = conn.prepare(
        "SELECT id, name, project_id, comment, files_count, dirs_count, files_size_bytes, loc_count, scanned_at
         FROM n002_snapshot ORDER BY id DESC",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(Snapshot::new(
            row.get::<_, i64>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, i64>(2)?,
            row.get::<_, Option<String>>(3)?,
            row.get::<_, i64>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, i64>(7)?,
            row.get::<_, String>(8)?,
        ))
    })?;

    let mut out = Vec::new();
    for r in rows { out.push(r?); }
    Ok(out)
}

pub fn create(
    conn: &Connection,
    name: &str,
    project_id: i64,
    comment: Option<&str>,
    files_count: i64,
    dirs_count: i64,
    files_size_bytes: i64,
    loc_count: i64,
    scanned_at: &str,
) -> SqlResult<i64> {
    conn.execute(
        "INSERT INTO n002_snapshot (name, project_id, comment, files_count, dirs_count, files_size_bytes, loc_count, scanned_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![name, project_id, comment, files_count, dirs_count, files_size_bytes, loc_count, scanned_at],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get_by_id(conn: &Connection, id: i64) -> SqlResult<Snapshot> {
    conn.query_row(
        "SELECT id, name, project_id, comment, files_count, dirs_count, files_size_bytes, loc_count, scanned_at
         FROM n002_snapshot WHERE id = ?1",
        [id],
        |row| {
            Ok(Snapshot::new(
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, i64>(5)?,
                row.get::<_, i64>(6)?,
                row.get::<_, i64>(7)?,
                row.get::<_, String>(8)?,
            ))
        },
    )
}

pub fn update(conn: &Connection, snap: &Snapshot) -> SqlResult<()> {
    conn.execute(
        "UPDATE n002_snapshot
         SET name = ?1, project_id = ?2, comment = ?3, files_count = ?4, dirs_count = ?5, files_size_bytes = ?6, loc_count = ?7, scanned_at = ?8
         WHERE id = ?9",
        params![
            snap.name,
            snap.project_id,
            snap.comment,
            snap.files_count,
            snap.dirs_count,
            snap.files_size_bytes,
            snap.loc_count,
            snap.scanned_at,
            snap.id
        ],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> SqlResult<()> {
    conn.execute("DELETE FROM n002_snapshot WHERE id = ?1", [id])?;
    Ok(())
}
