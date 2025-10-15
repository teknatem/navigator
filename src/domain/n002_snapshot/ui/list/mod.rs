use std::collections::HashMap;

use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui::{Color32, RichText, TextStyle, Vec2, Stroke};
use rusqlite::Connection;

use crate::domain::n002_snapshot::repository as repo;
use crate::domain::n002_snapshot::model::Snapshot;
use crate::domain::n002_snapshot::ui::details::{details_form, SnapshotDetailsState};
use crate::domain::n001_project::ui::picker::ProjectPickerState;
use crate::domain::n001_project::repository as project_repo;

#[derive(Default)]
pub struct SnapshotsListState {
    // Modals
    pub show_create: bool,
    pub show_edit: bool,
    pub show_view: bool,
    pub editing_id: Option<i64>,
    pub create_details: SnapshotDetailsState,
    pub edit_details: SnapshotDetailsState,
    pub view_details: SnapshotDetailsState,

    // Picker for selecting a project
    pub project_picker: ProjectPickerState,

    pub status: String,
}

pub fn ui_snapshots_list(ui: &mut egui::Ui, conn: &Connection, state: &mut SnapshotsListState) {
    // Top buttons
    ui.horizontal(|ui| {
        if ui.button("Add").clicked() {
            state.show_create = true;
            state.create_details = SnapshotDetailsState::default();
        }
    });

    ui.add_space(6.0);

    // Load snapshots
    let snapshots: Vec<Snapshot> = match repo::list_all(conn) {
        Ok(list) => list,
        Err(e) => {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                format!("Error loading snapshots: {}", e),
            );
            return;
        }
    };

    // Load projects map for name lookup
    let project_names: HashMap<i64, String> = match project_repo::list_all(conn) {
        Ok(list) => list.into_iter().map(|p| (p.id, p.name)).collect(),
        Err(_) => HashMap::new(),
    };

    // Ensure table uses full available width
    ui.set_width(ui.available_width());

    // Font sizes
    let heading_base = ui
        .style()
        .text_styles
        .get(&TextStyle::Heading)
        .map(|f| f.size)
        .unwrap_or(18.0);
    let body_base = ui
        .style()
        .text_styles
        .get(&TextStyle::Body)
        .map(|f| f.size)
        .unwrap_or(14.0);
    let header_size = heading_base * 0.8;
    let row_size = body_base * 0.8;

    // Use remainder columns so the table adjusts proportionally with width
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .column(Column::remainder()) // Name
        .column(Column::remainder()) // Project
        .column(Column::remainder()) // Files
        .column(Column::remainder()) // Dirs
        .column(Column::remainder()) // Size
        .column(Column::remainder()) // LOC
        .column(Column::remainder()) // Scanned At
        .column(Column::remainder()) // Comment
        .column(Column::auto())      // Actions
        .header(24.0, |mut header| {
            for title in [
                "Name", "Project", "Files", "Dirs", "Size(bytes)", "LOC", "Scanned At", "Comment", "Actions",
            ] {
                header.col(|ui| {
                    let rect = ui.max_rect();
                    ui.painter().rect_filled(rect, 0.0, Color32::from_gray(45));
                    ui.label(RichText::new(title).size(header_size));
                });
            }
        })
        .body(|mut body| {
            for s in snapshots.iter() {
                body.row(26.0, |mut row| {
                    row.col(|ui| { ui.label(RichText::new(&s.name).size(row_size)); });
                    row.col(|ui| {
                        let pname = project_names
                            .get(&s.project_id)
                            .map(|s| s.as_str())
                            .unwrap_or("<unknown>");
                        ui.label(RichText::new(format!("{} (ID: {})", pname, s.project_id)).size(row_size));
                    });
                    row.col(|ui| { ui.label(RichText::new(format!("{}", s.files_count)).size(row_size)); });
                    row.col(|ui| { ui.label(RichText::new(format!("{}", s.dirs_count)).size(row_size)); });
                    row.col(|ui| { ui.label(RichText::new(format!("{}", s.files_size_bytes)).size(row_size)); });
                    row.col(|ui| { ui.label(RichText::new(format!("{}", s.loc_count)).size(row_size)); });
                    row.col(|ui| { ui.label(RichText::new(&s.scanned_at).size(row_size)); });
                    row.col(|ui| { ui.label(RichText::new(s.comment.as_deref().unwrap_or("")).size(row_size)); });
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            if ui.button("View").clicked() {
                                let id = s.id;
                                match repo::get_by_id(conn, id) {
                                    Ok(snap) => {
                                        state.view_details = SnapshotDetailsState::from_snapshot(
                                            &snap.name,
                                            snap.project_id,
                                            snap.comment.as_deref(),
                                            snap.files_count,
                                            snap.dirs_count,
                                            snap.files_size_bytes,
                                            snap.loc_count,
                                            &snap.scanned_at,
                                        );
                                        state.show_view = true;
                                    }
                                    Err(e) => state.status = format!("Error loading snapshot: {}", e),
                                }
                            }
                            if ui.button("Edit").clicked() {
                                let id = s.id;
                                match repo::get_by_id(conn, id) {
                                    Ok(snap) => {
                                        state.editing_id = Some(snap.id);
                                        state.edit_details = SnapshotDetailsState::from_snapshot(
                                            &snap.name,
                                            snap.project_id,
                                            snap.comment.as_deref(),
                                            snap.files_count,
                                            snap.dirs_count,
                                            snap.files_size_bytes,
                                            snap.loc_count,
                                            &snap.scanned_at,
                                        );
                                        state.show_edit = true;
                                    }
                                    Err(e) => state.status = format!("Error loading snapshot: {}", e),
                                }
                            }
                            if ui.button("Delete").clicked() {
                                let id = s.id;
                                match repo::delete(conn, id) {
                                    Ok(_) => { state.status = format!("Deleted snapshot {}", id); }
                                    Err(e) => state.status = format!("Error deleting: {}", e),
                                }
                            }
                        });
                    });
                });
                // horizontal separator line
                body.row(2.0, |mut row| {
                    for _i in 0..9 {
                        row.col(|ui| {
                            let rect = ui.max_rect();
                            let y = rect.bottom();
                            ui.painter().hline(rect.x_range(), y, Stroke::new(1.0, Color32::from_gray(60)));
                            ui.allocate_space(Vec2::new(0.0, 0.0));
                        });
                    }
                });
            }
        });

    // Create snapshot window
    if state.show_create {
        let mut open = true;
        egui::Window::new("Add Snapshot")
            .id(egui::Id::new("n002_add_snapshot"))
            .open(&mut open)
            .show(ui.ctx(), |ui| {
            details_form(ui, conn, &mut state.project_picker, &mut state.create_details);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    let d = &state.create_details;
                    if d.name.trim().is_empty() || d.project_id <= 0 {
                        state.status = "Name and project required".to_string();
                    } else {
                        let comment_opt = if d.comment.trim().is_empty() { None } else { Some(d.comment.trim()) };
                        match repo::create(
                            conn,
                            d.name.trim(),
                            d.project_id,
                            comment_opt,
                            d.files_count,
                            d.dirs_count,
                            d.files_size_bytes,
                            d.loc_count,
                            d.scanned_at.trim(),
                        ) {
                            Ok(id) => {
                                state.status = format!("Snapshot created (ID={})", id);
                                state.show_create = false;
                                state.create_details = SnapshotDetailsState::default();
                            }
                            Err(e) => state.status = format!("Error creating: {}", e),
                        }
                    }
                }
                if ui.button("Cancel").clicked() { state.show_create = false; }
            });
        });
        if !open { state.show_create = false; }
    }

    // View snapshot window
    if state.show_view {
        let mut open = true;
        egui::Window::new("View Snapshot")
            .id(egui::Id::new("n002_view_snapshot"))
            .open(&mut open)
            .show(ui.ctx(), |ui| {
            let pname = project_names
                .get(&state.view_details.project_id)
                .map(|s| s.as_str())
                .unwrap_or("<unknown>");
            ui.label(format!("Name: {}", state.view_details.name));
            ui.label(format!("Project: {} (ID: {})", pname, state.view_details.project_id));
            ui.label(format!("Comment: {}", state.view_details.comment));
            ui.label(format!("Files: {}", state.view_details.files_count));
            ui.label(format!("Dirs: {}", state.view_details.dirs_count));
            ui.label(format!("Size(bytes): {}", state.view_details.files_size_bytes));
            ui.label(format!("LOC: {}", state.view_details.loc_count));
            ui.label(format!("Scanned At: {}", state.view_details.scanned_at));
            ui.add_space(6.0);
            if ui.button("Close").clicked() { state.show_view = false; }
        });
        if !open { state.show_view = false; }
    }

    // Edit snapshot window
    if state.show_edit {
        let mut open = true;
        egui::Window::new("Edit Snapshot")
            .id(egui::Id::new("n002_edit_snapshot"))
            .open(&mut open)
            .show(ui.ctx(), |ui| {
            details_form(ui, conn, &mut state.project_picker, &mut state.edit_details);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    if let Some(id) = state.editing_id {
                        let d = &state.edit_details;
                        if d.name.trim().is_empty() || d.project_id <= 0 {
                            state.status = "Name and project required".to_string();
                        } else {
                            let updated = Snapshot {
                                id,
                                name: d.name.trim().to_string(),
                                project_id: d.project_id,
                                comment: if d.comment.trim().is_empty() { None } else { Some(d.comment.trim().to_string()) },
                                files_count: d.files_count,
                                dirs_count: d.dirs_count,
                                files_size_bytes: d.files_size_bytes,
                                loc_count: d.loc_count,
                                scanned_at: d.scanned_at.trim().to_string(),
                            };
                            match repo::update(conn, &updated) {
                                Ok(_) => {
                                    state.status = "Snapshot updated".to_string();
                                    state.show_edit = false;
                                    state.editing_id = None;
                                }
                                Err(e) => state.status = format!("Error updating: {}", e),
                            }
                        }
                    }
                }
                if ui.button("Cancel").clicked() { state.show_edit = false; state.editing_id = None; }
            });
        });
        if !open { state.show_edit = false; state.editing_id = None; }
    }

    if !state.status.is_empty() {
        ui.add_space(6.0);
        ui.colored_label(egui::Color32::LIGHT_RED, &state.status);
    }
}
