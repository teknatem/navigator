use eframe::egui;
use rusqlite::Connection;

use crate::domain::n001_project::ui::picker::{ui_project_picker, ProjectPickerState};
use crate::domain::n001_project::repository as project_repo;

#[derive(Default, Clone)]
pub struct SnapshotDetailsState {
    pub name: String,
    pub project_id: i64,
    pub comment: String,
    pub files_count: i64,
    pub dirs_count: i64,
    pub files_size_bytes: i64,
    pub loc_count: i64,
    pub scanned_at: String,
}

impl SnapshotDetailsState {
    pub fn from_snapshot(
        name: &str,
        project_id: i64,
        comment: Option<&str>,
        files_count: i64,
        dirs_count: i64,
        files_size_bytes: i64,
        loc_count: i64,
        scanned_at: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            project_id,
            comment: comment.unwrap_or("").to_string(),
            files_count,
            dirs_count,
            files_size_bytes,
            loc_count,
            scanned_at: scanned_at.to_string(),
        }
    }
}

// Details form renders fields and integrates a Project picker inline
pub fn details_form(
    ui: &mut egui::Ui,
    conn: &Connection,
    picker: &mut ProjectPickerState,
    state: &mut SnapshotDetailsState,
) {
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut state.name);
    });

    // Show current project name with ID and allow picking
    let proj_name = if state.project_id > 0 {
        match project_repo::get_by_id(conn, state.project_id) {
            Ok(p) => p.name,
            Err(_) => String::from("<unknown>"),
        }
    } else {
        String::from("<none>")
    };
    ui.horizontal(|ui| {
        ui.label(format!("Project: {} (ID: {})", proj_name, state.project_id));
        if ui.button("Pick Project...").clicked() {
            picker.open();
        }
    });
    if let Some(pid) = ui_project_picker(ui, conn, picker) {
        state.project_id = pid;
    }

    ui.horizontal(|ui| {
        ui.label("Comment:");
        ui.text_edit_singleline(&mut state.comment);
    });

    ui.horizontal(|ui| {
        ui.label("Files:");
        ui.add(egui::DragValue::new(&mut state.files_count).clamp_range(0..=i64::MAX));
        ui.label("Dirs:");
        ui.add(egui::DragValue::new(&mut state.dirs_count).clamp_range(0..=i64::MAX));
    });

    ui.horizontal(|ui| {
        ui.label("Size(bytes):");
        ui.add(egui::DragValue::new(&mut state.files_size_bytes).clamp_range(0..=i64::MAX));
        ui.label("LOC:");
        ui.add(egui::DragValue::new(&mut state.loc_count).clamp_range(0..=i64::MAX));
    });

    ui.horizontal(|ui| {
        ui.label("Scanned At:");
        ui.text_edit_singleline(&mut state.scanned_at);
    });
}
