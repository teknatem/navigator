use rusqlite::Connection;
use std::fs;
use std::path::Path;

use super::gitignore::GitignoreParser;
use crate::domain::n002_snapshot::repository as snapshot_repo;
use crate::domain::n003_snapshot_file::repository as file_repo;

/// Extract file extension from filename
fn extract_file_extension(name: &str) -> Option<String> {
    if let Some(dot_pos) = name.rfind('.') {
        let ext = &name[dot_pos + 1..];
        if !ext.is_empty() {
            return Some(ext.to_string());
        }
    }
    None
}

/// Extract crate layer from path (backend, frontend, contracts)
fn extract_crate_layer(path: &str) -> Option<String> {
    let path_lower = path.to_lowercase().replace('\\', "/");

    if path_lower.contains("crates/backend") || path_lower.contains("crates\\backend") {
        return Some("backend".to_string());
    }
    if path_lower.contains("crates/frontend") || path_lower.contains("crates\\frontend") {
        return Some("frontend".to_string());
    }
    if path_lower.contains("crates/contracts") || path_lower.contains("crates\\contracts") {
        return Some("contracts".to_string());
    }

    None
}

/// Extract artifact type from path (domain, usecase, shared)
fn extract_artifact_type(path: &str) -> Option<String> {
    let path_normalized = path.replace('\\', "/");

    if path_normalized.contains("/domain/") {
        return Some("domain".to_string());
    }
    if path_normalized.contains("/usecases/") {
        return Some("usecase".to_string());
    }
    if path_normalized.contains("/shared/") {
        return Some("shared".to_string());
    }

    None
}

/// Extract artifact id and name from path (e.g., "n001_project" -> id: "n001_", name: "project")
fn extract_artifact_id_name(path: &str) -> (Option<String>, Option<String>) {
    let path_normalized = path.replace('\\', "/");
    let segments: Vec<&str> = path_normalized.split('/').collect();

    for segment in segments {
        // Look for patterns like n001_, s501_, etc.
        if let Some(underscore_pos) = segment.find('_') {
            let prefix = &segment[..=underscore_pos];

            // Check if prefix matches pattern: letter(s) + digits + underscore
            let chars: Vec<char> = prefix.chars().collect();
            if chars.len() >= 3 {
                let has_letter_start = chars[0].is_alphabetic();
                let has_digits = chars.iter().skip(1).take_while(|c| c.is_numeric()).count() > 0;
                let ends_with_underscore = chars.last() == Some(&'_');

                if has_letter_start && has_digits && ends_with_underscore {
                    let id = prefix.to_string();
                    let name = segment[underscore_pos + 1..].to_string();
                    return (Some(id), Some(name));
                }
            }
        }
    }

    (None, None)
}

/// Extract role from path and filename
fn extract_role(path: &str, filename: &str, is_directory: bool) -> Option<String> {
    if is_directory {
        return None;
    }

    // Check if any parent directory is named "ui"
    let path_normalized = path.replace('\\', "/");
    let segments: Vec<&str> = path_normalized.split('/').collect();

    for segment in &segments {
        if *segment == "ui" {
            return Some("ui".to_string());
        }
    }

    // Check filename patterns
    match filename {
        "model.rs" => Some("model".to_string()),
        "repository.rs" => Some("repository".to_string()),
        "service.rs" => Some("service".to_string()),
        _ => None,
    }
}

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

        // Parse metadata
        let file_extension = if !is_directory {
            extract_file_extension(&name)
        } else {
            None
        };
        let crate_layer = extract_crate_layer(&rel_path);
        let artifact_type = extract_artifact_type(&rel_path);
        let (artifact_id, artifact_name) = extract_artifact_id_name(&rel_path);
        let role = extract_role(&rel_path, &name, is_directory);

        // Insert record into database
        let file_id = file_repo::create(
            conn,
            snapshot_id,
            parent_id,
            &name,
            &rel_path,
            size_bytes,
            is_directory,
            file_extension.as_deref(),
            crate_layer.as_deref(),
            artifact_type.as_deref(),
            artifact_id.as_deref(),
            artifact_name.as_deref(),
            role.as_deref(),
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
