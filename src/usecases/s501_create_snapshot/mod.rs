mod gitignore;
mod scanner;

use eframe::egui;
use rusqlite::Connection;
use std::path::PathBuf;

use crate::domain::n002_snapshot::ui::picker::{ui_snapshot_picker, SnapshotPickerState};
use crate::domain::n002_snapshot::repository as snapshot_repo;
use crate::domain::n001_project::repository as project_repo;
use scanner::{scan_directory, ScanProgress};

#[derive(Default)]
pub struct ScanSnapshotState {
    pub snapshot_picker: SnapshotPickerState,
    pub selected_snapshot_id: Option<i64>,
    pub selected_snapshot_name: String,
    pub project_root_path: String,
    pub status: String,
    pub is_scanning: bool,
    pub progress_files: usize,
    pub progress_dirs: usize,
    pub progress_current: String,
}

impl ScanSnapshotState {
    pub fn open_picker(&mut self) {
        self.snapshot_picker.open();
    }
}

pub fn ui_scan_snapshot(ui: &mut egui::Ui, conn: &Connection, state: &mut ScanSnapshotState) {
    ui.heading("Scan Snapshot");
    ui.add_space(12.0);
    
    // Snapshot picker
    if let Some(snapshot_id) = ui_snapshot_picker(ui, conn, &mut state.snapshot_picker) {
        state.selected_snapshot_id = Some(snapshot_id);
        
        // Load snapshot details
        if let Ok(snapshot) = snapshot_repo::get_by_id(conn, snapshot_id) {
            state.selected_snapshot_name = snapshot.name.clone();
            
            // Load project to get root path
            if let Ok(project) = project_repo::get_by_id(conn, snapshot.project_id) {
                state.project_root_path = project.root_path;
            }
        }
    }
    
    ui.horizontal(|ui| {
        ui.label("Selected Snapshot:");
        if state.selected_snapshot_id.is_some() {
            ui.label(egui::RichText::new(&state.selected_snapshot_name).strong());
        } else {
            ui.label(egui::RichText::new("None").weak());
        }
        
        if ui.button("Select...").clicked() {
            state.open_picker();
        }
    });
    
    if !state.project_root_path.is_empty() {
        ui.label(format!("Project Path: {}", state.project_root_path));
    }
    
    ui.add_space(12.0);
    
    // Scan button
    let can_scan = state.selected_snapshot_id.is_some() && !state.is_scanning;
    if ui.add_enabled(can_scan, egui::Button::new("Scan")).clicked() {
        start_scan(conn, state);
    }
    
    ui.add_space(12.0);
    
    // Progress display
    if state.is_scanning || state.progress_files > 0 || state.progress_dirs > 0 {
        ui.group(|ui| {
            ui.label(egui::RichText::new("Scan Progress").strong());
            ui.label(format!("Files scanned: {}", state.progress_files));
            ui.label(format!("Directories scanned: {}", state.progress_dirs));
            if !state.progress_current.is_empty() {
                ui.label(format!("Current: {}", state.progress_current));
            }
            
            if state.is_scanning {
                ui.add_space(6.0);
                ui.spinner();
            }
        });
    }
    
    ui.add_space(12.0);
    
    // Status messages
    if !state.status.is_empty() {
        let color = if state.status.starts_with("Error") || state.status.starts_with("Failed") {
            egui::Color32::LIGHT_RED
        } else if state.status.starts_with("Success") || state.status.starts_with("Completed") {
            egui::Color32::LIGHT_GREEN
        } else {
            egui::Color32::LIGHT_BLUE
        };
        
        ui.colored_label(color, &state.status);
    }
}

fn start_scan(conn: &Connection, state: &mut ScanSnapshotState) {
    if let Some(snapshot_id) = state.selected_snapshot_id {
        state.is_scanning = true;
        state.progress_files = 0;
        state.progress_dirs = 0;
        state.progress_current = String::new();
        state.status = "Starting scan...".to_string();
        
        let root_path = PathBuf::from(&state.project_root_path);
        
        // Perform scan with progress callback
        let result = scan_directory(
            conn,
            snapshot_id,
            &root_path,
            |progress: ScanProgress| {
                // Note: In a real async implementation, this would update the UI
                // For now, we'll just collect the final state
                state.progress_files = progress.files_scanned;
                state.progress_dirs = progress.dirs_scanned;
                state.progress_current = progress.current_path;
            },
        );
        
        state.is_scanning = false;
        
        match result {
            Ok(scan_result) => {
                state.status = format!(
                    "Completed! Files: {}, Dirs: {}, Total Size: {} bytes",
                    scan_result.files_count,
                    scan_result.dirs_count,
                    scan_result.total_size
                );
            }
            Err(e) => {
                state.status = format!("Error: {}", e);
            }
        }
    }
}

