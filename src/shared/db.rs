use std::path::Path;

use rusqlite::{Connection, Result as SqlResult};

use crate::app_settings::AppSettings;
use crate::domain::n001_project::repository as project_repo;
use crate::domain::n002_snapshot::repository as snapshot_repo;
use crate::domain::n003_snapshot_file::repository as snapshot_file_repo;

pub const DB_PATH: &str = "navigator.db";

/// Open database by path, creating it (and initializing tables) if missing.
pub fn open_or_create(db_path: &str) -> SqlResult<Connection> {
    let db_exists = Path::new(db_path).exists();

    if db_exists {
        println!("Database found: {}", db_path);
    } else {
        println!("Database not found. Creating new database: {}", db_path);
    }

    let conn = Connection::open(db_path)?;

    // Ensure all tables exist (idempotent)
    init_database(&conn)?;

    if !db_exists {
        println!("Database created successfully with all tables");
    }

    Ok(conn)
}

/// Create all required tables (idempotent).
pub fn init_database(conn: &Connection) -> SqlResult<()> {
    // Demo/sample table kept for backward compatibility with existing UI
    conn.execute(
        "CREATE TABLE IF NOT EXISTS items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL
        )",
        [],
    )?;
    println!("  ✓ Table 'items' initialized");

    // App settings table
    AppSettings::init_table(conn)?;
    println!("  ✓ Table 'settings' initialized");

    // Domain: n001_project aggregate table
    project_repo::init_table(conn)?;
    println!("  ✓ Table 'n001_project' initialized");

    // Domain: n002_snapshot aggregate table
    snapshot_repo::init_table(conn)?;
    println!("  ✓ Table 'n002_snapshot' initialized");

    // Domain: n003_snapshot_file aggregate table
    snapshot_file_repo::init_table(conn)?;
    println!("  ✓ Table 'n003_snapshot_file' initialized");

    Ok(())
}





