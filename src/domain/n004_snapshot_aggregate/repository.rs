use rusqlite::{params, Connection, Result as SqlResult};

use super::model::SnapshotAggregate;

pub fn init_table(conn: &Connection) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS n004_snapshot_aggregate (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            snapshot_id INTEGER NOT NULL,
            code TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            file_count INTEGER NOT NULL DEFAULT 0,
            total_size_bytes INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (snapshot_id) REFERENCES n002_snapshot(id)
        )",
        [],
    )?;

    // Create indexes for better query performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_snapshot_aggregate_snapshot_id 
         ON n004_snapshot_aggregate(snapshot_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_snapshot_aggregate_code 
         ON n004_snapshot_aggregate(code)",
        [],
    )?;

    Ok(())
}

pub fn create(
    conn: &Connection,
    snapshot_id: i64,
    code: &str,
    name: &str,
    description: Option<&str>,
    file_count: i64,
    total_size_bytes: i64,
) -> SqlResult<i64> {
    let now = chrono::Utc::now().to_rfc3339();
    
    conn.execute(
        "INSERT INTO n004_snapshot_aggregate (snapshot_id, code, name, description, file_count, total_size_bytes, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            snapshot_id, 
            code, 
            name, 
            description,
            file_count,
            total_size_bytes,
            &now,
            &now
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(
    conn: &Connection,
    id: i64,
    code: &str,
    name: &str,
    description: Option<&str>,
    file_count: i64,
    total_size_bytes: i64,
) -> SqlResult<()> {
    let now = chrono::Utc::now().to_rfc3339();
    
    conn.execute(
        "UPDATE n004_snapshot_aggregate 
         SET code = ?1, name = ?2, description = ?3, file_count = ?4, total_size_bytes = ?5, updated_at = ?6
         WHERE id = ?7",
        params![code, name, description, file_count, total_size_bytes, &now, id],
    )?;
    Ok(())
}

pub fn delete(conn: &Connection, id: i64) -> SqlResult<()> {
    conn.execute(
        "DELETE FROM n004_snapshot_aggregate WHERE id = ?1",
        [id],
    )?;
    Ok(())
}

pub fn delete_by_snapshot(conn: &Connection, snapshot_id: i64) -> SqlResult<()> {
    conn.execute(
        "DELETE FROM n004_snapshot_aggregate WHERE snapshot_id = ?1",
        [snapshot_id],
    )?;
    Ok(())
}

pub fn get_by_id(conn: &Connection, id: i64) -> SqlResult<Option<SnapshotAggregate>> {
    let mut stmt = conn.prepare(
        "SELECT id, snapshot_id, code, name, description, file_count, total_size_bytes, created_at, updated_at
         FROM n004_snapshot_aggregate
         WHERE id = ?1",
    )?;

    let mut rows = stmt.query_map([id], |row| {
        Ok(SnapshotAggregate::new(
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
        ))
    })?;

    match rows.next() {
        Some(row) => Ok(Some(row?)),
        None => Ok(None),
    }
}

pub fn list_by_snapshot(conn: &Connection, snapshot_id: i64) -> SqlResult<Vec<SnapshotAggregate>> {
    let mut stmt = conn.prepare(
        "SELECT id, snapshot_id, code, name, description, file_count, total_size_bytes, created_at, updated_at
         FROM n004_snapshot_aggregate
         WHERE snapshot_id = ?1
         ORDER BY code",
    )?;

    let rows = stmt.query_map([snapshot_id], |row| {
        Ok(SnapshotAggregate::new(
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
            row.get::<_, Option<String>>(4)?,
            row.get::<_, i64>(5)?,
            row.get::<_, i64>(6)?,
            row.get::<_, String>(7)?,
            row.get::<_, String>(8)?,
        ))
    })?;

    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn get_stats(conn: &Connection, snapshot_id: i64) -> SqlResult<(i64, i64, i64)> {
    let aggregate_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM n004_snapshot_aggregate 
         WHERE snapshot_id = ?1",
        [snapshot_id],
        |row| row.get(0),
    )?;

    let total_files: i64 = conn.query_row(
        "SELECT COALESCE(SUM(file_count), 0) FROM n004_snapshot_aggregate 
         WHERE snapshot_id = ?1",
        [snapshot_id],
        |row| row.get(0),
    )?;

    let total_size: i64 = conn.query_row(
        "SELECT COALESCE(SUM(total_size_bytes), 0) FROM n004_snapshot_aggregate 
         WHERE snapshot_id = ?1",
        [snapshot_id],
        |row| row.get(0),
    )?;

    Ok((aggregate_count, total_files, total_size))
}
