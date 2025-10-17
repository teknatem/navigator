use rusqlite::Connection;
use std::fs;
use std::path::Path;

use super::gitignore::GitignoreParser;
use crate::domain::n002_snapshot::repository as snapshot_repo;
use crate::domain::n003_snapshot_file::repository as file_repo;

pub struct ScanProgress {
    pub files_scanned: usize,
    pub dirs_scanned: usize,
    pub current_path: String,
}

pub struct ScanResult {
    pub files_count: i64,
    pub dirs_count: i64,
    pub total_size: i64,
}

pub fn scan_directory<F>(
    conn: &Connection,
    snapshot_id: i64,
    root_path: &Path,
    mut progress_callback: F,
) -> Result<ScanResult, String>
where
    F: FnMut(ScanProgress),
{
    // Check if .gitignore exists
    let gitignore_path = root_path.join(".gitignore");
    if !gitignore_path.exists() {
        return Err("File .gitignore not found in project root".to_string());
    }

    // Parse .gitignore
    let gitignore = GitignoreParser::from_file(&gitignore_path)?;

    // Delete existing file records for this snapshot
    file_repo::delete_by_snapshot(conn, snapshot_id)
        .map_err(|e| format!("Failed to delete existing records: {}", e))?;

    let mut progress = ScanProgress {
        files_scanned: 0,
        dirs_scanned: 0,
        current_path: String::new(),
    };

    // Start scanning from root
    scan_recursive(
        conn,
        snapshot_id,
        root_path,
        root_path,
        None,
        &gitignore,
        &mut progress,
        &mut progress_callback,
    )?;

    // Calculate statistics
    let (files_count, dirs_count) = file_repo::count_files_and_dirs(conn, snapshot_id)
        .map_err(|e| format!("Failed to count files: {}", e))?;

    let total_size = file_repo::sum_file_sizes(conn, snapshot_id)
        .map_err(|e| format!("Failed to sum file sizes: {}", e))?;

    // Update snapshot aggregate with new counts
    if let Ok(mut snapshot) = snapshot_repo::get_by_id(conn, snapshot_id) {
        snapshot.files_count = files_count;
        snapshot.dirs_count = dirs_count;
        snapshot.files_size_bytes = total_size;
        // Keep loc_count unchanged for now

        let _ = snapshot_repo::update(conn, &snapshot);
    }

    Ok(ScanResult {
        files_count,
        dirs_count,
        total_size,
    })
}

fn scan_recursive<F>(
    conn: &Connection,
    snapshot_id: i64,
    root_path: &Path,
    current_path: &Path,
    parent_id: Option<i64>,
    gitignore: &GitignoreParser,
    progress: &mut ScanProgress,
    progress_callback: &mut F,
) -> Result<(), String>
where
    F: FnMut(ScanProgress),
{
    let entries = fs::read_dir(current_path)
        .map_err(|e| format!("Failed to read directory {:?}: {}", current_path, e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        let metadata = entry
            .metadata()
            .map_err(|e| format!("Failed to read metadata for {:?}: {}", path, e))?;

        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| format!("Invalid filename: {:?}", path))?
            .to_string();

        let is_directory = metadata.is_dir();

        // Always skip .git directory
        if is_directory && name == ".git" {
            continue;
        }

        // Calculate relative path from root
        let rel_path = path
            .strip_prefix(root_path)
            .map_err(|e| format!("Failed to get relative path: {}", e))?
            .to_str()
            .ok_or_else(|| "Invalid path encoding".to_string())?
            .to_string();

        // Check if this path should be ignored
        if gitignore.is_ignored(&rel_path, is_directory) {
            continue;
        }

        // Get file size (0 for directories)
        let size_bytes = if is_directory {
            0
        } else {
            metadata.len() as i64
        };

        // Insert record into database
        let file_id = file_repo::create(
            conn,
            snapshot_id,
            parent_id,
            &name,
            &rel_path,
            size_bytes,
            is_directory,
        )
        .map_err(|e| format!("Failed to insert file record: {}", e))?;

        // Update progress
        if is_directory {
            progress.dirs_scanned += 1;
        } else {
            progress.files_scanned += 1;
        }
        progress.current_path = rel_path.clone();
        progress_callback(ScanProgress {
            files_scanned: progress.files_scanned,
            dirs_scanned: progress.dirs_scanned,
            current_path: progress.current_path.clone(),
        });

        // Recursively scan subdirectories
        if is_directory {
            scan_recursive(
                conn,
                snapshot_id,
                root_path,
                &path,
                Some(file_id),
                gitignore,
                progress,
                progress_callback,
            )?;
        }
    }

    Ok(())
}
