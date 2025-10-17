use rusqlite::{params, Connection, Result as SqlResult};

use super::model::SnapshotFile;

pub fn init_table(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS n003_snapshot_file (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snapshot_id INTEGER NOT NULL,
            parent_id INTEGER,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            is_directory INTEGER NOT NULL,
            FOREIGN KEY (snapshot_id) REFERENCES n002_snapshot(id),
            FOREIGN KEY (parent_id) REFERENCES n003_snapshot_file(id)
        )",
        [],
    )?;

    // Create indexes for better query performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_snapshot_file_snapshot_id 
         ON n003_snapshot_file(snapshot_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_snapshot_file_parent_id 
         ON n003_snapshot_file(parent_id)",
        [],
    )?;

    Ok(())
}

pub fn create(
    conn: &Connection,
    snapshot_id: i64,
    parent_id: Option<i64>,
    name: &str,
    path: &str,
    size_bytes: i64,
    is_directory: bool,
) -> SqlResult<i64> {
    conn.execute(
        "INSERT INTO n003_snapshot_file (snapshot_id, parent_id, name, path, size_bytes, is_directory)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![snapshot_id, parent_id, name, path, size_bytes, if is_directory { 1 } else { 0 }],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn list_by_snapshot(conn: &Connection, snapshot_id: i64) -> SqlResult<Vec<SnapshotFile>> {
    let mut stmt = conn.prepare(
        "SELECT id, snapshot_id, parent_id, name, path, size_bytes, is_directory
         FROM n003_snapshot_file
         WHERE snapshot_id = ?1
         ORDER BY path",
    )?;

    let rows = stmt.query_map([snapshot_id], |row| {
        Ok(SnapshotFile::new(
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, Option<i64>>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, String>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)? != 0,
        ))
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn delete_by_snapshot(conn: &Connection, snapshot_id: i64) -> SqlResult<()> {
    conn.execute(
        "DELETE FROM n003_snapshot_file WHERE snapshot_id = ?1",
        [snapshot_id],
    )?;
    Ok(())
}

pub fn count_files_and_dirs(conn: &Connection, snapshot_id: i64) -> SqlResult<(i64, i64)> {
    let files_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM n003_snapshot_file 
         WHERE snapshot_id = ?1 AND is_directory = 0",
        [snapshot_id],
        |row| row.get(0),
    )?;

    let dirs_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM n003_snapshot_file 
         WHERE snapshot_id = ?1 AND is_directory = 1",
        [snapshot_id],
        |row| row.get(0),
    )?;

    Ok((files_count, dirs_count))
}

pub fn sum_file_sizes(conn: &Connection, snapshot_id: i64) -> SqlResult<i64> {
    let total_size: i64 = conn.query_row(
        "SELECT COALESCE(SUM(size_bytes), 0) FROM n003_snapshot_file 
         WHERE snapshot_id = ?1 AND is_directory = 0",
        [snapshot_id],
        |row| row.get(0),
    )?;

    Ok(total_size)
}
