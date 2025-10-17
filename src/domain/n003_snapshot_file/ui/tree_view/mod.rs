use eframe::egui;
use rusqlite::Connection;

use crate::domain::n002_snapshot::repository as snapshot_repo;
use crate::domain::n002_snapshot::ui::picker::{ui_snapshot_picker, SnapshotPickerState};
use crate::domain::n003_snapshot_file::repository as repo;

#[derive(Default)]
pub struct TreeViewState {
    pub snapshot_id: Option<i64>,
    pub snapshot_name: String,
    pub snapshot_picker: SnapshotPickerState,
    pub status: String,
}

impl TreeViewState {
    pub fn open_picker(&mut self) {
        self.snapshot_picker.open();
    }
}

pub fn ui_tree_view(ui: &mut egui::Ui, conn: &Connection, state: &mut TreeViewState) {
    // Snapshot picker window
    if let Some(selected_id) = ui_snapshot_picker(ui, conn, &mut state.snapshot_picker) {
        state.snapshot_id = Some(selected_id);

        // Load snapshot name
        if let Ok(snapshot) = snapshot_repo::get_by_id(conn, selected_id) {
            state.snapshot_name = snapshot.name;
        }
    }

    // Selection controls
    ui.horizontal(|ui| {
        ui.label("Selected Snapshot:");
        if state.snapshot_id.is_some() {
            ui.label(egui::RichText::new(&state.snapshot_name).strong());
        } else {
            ui.label(egui::RichText::new("None").weak());
        }

        if ui.button("Select...").clicked() {
            state.open_picker();
        }
    });

    ui.add_space(12.0);

    // File tree display
    if let Some(snapshot_id) = state.snapshot_id {
        match repo::list_by_snapshot(conn, snapshot_id) {
            Ok(files) => {
                if files.is_empty() {
                    ui.label(egui::RichText::new("No files found. Run scan to populate.").weak());
                } else {
                    ui.label(format!("Total items: {}", files.len()));
                    ui.add_space(6.0);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for file in files.iter() {
                            let icon = if file.is_directory { "📁" } else { "📄" };
                            let size_str = if file.is_directory {
                                String::new()
                            } else {
                                format!(" - {} bytes", file.size_bytes)
                            };
                            ui.label(format!("{} {}{}", icon, file.path, size_str));
                        }
                    });
                }
            }
            Err(e) => {
                ui.colored_label(
                    egui::Color32::LIGHT_RED,
                    format!("Error loading files: {}", e),
                );
            }
        }
    } else {
        ui.label(egui::RichText::new("Select a snapshot to view its files").weak());
    }

    if !state.status.is_empty() {
        ui.add_space(12.0);
        ui.colored_label(egui::Color32::LIGHT_BLUE, &state.status);
    }
}
