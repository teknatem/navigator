use eframe::egui;
use egui_extras::{Column, TableBuilder};
use egui::{Color32, Stroke, Vec2, RichText, TextStyle};
use rusqlite::Connection;

use crate::domain::n001_project::model::Project;
use crate::domain::n001_project::repository as repo;
use crate::domain::n001_project::ui::details::{details_form, ProjectDetailsState};

#[derive(Default)]
pub struct ProjectsListState {
    // Modals
    pub show_create: bool,
    pub show_edit: bool,
    pub show_view: bool,
    pub editing_id: Option<i64>,
    pub create_details: ProjectDetailsState,
    pub edit_details: ProjectDetailsState,
    pub view_details: ProjectDetailsState,

    pub status: String,
}

pub fn ui_projects_list(ui: &mut egui::Ui, conn: &Connection, state: &mut ProjectsListState) {
    // Top buttons only (no headings)
    ui.horizontal(|ui| {
        if ui.button("Add").clicked() {
            state.show_create = true;
            state.create_details = ProjectDetailsState::default();
        }
    });

    ui.add_space(6.0);

    // Load list
    let projects: Vec<Project> = match repo::list_all(conn) {
        Ok(list) => list,
        Err(e) => {
            ui.colored_label(
                egui::Color32::LIGHT_RED,
                format!("Error loading projects: {}", e),
            );
            return;
        }
    };

    // Ensure table uses full available width and recomputes each frame
    ui.set_width(ui.available_width());

    // Font sizes (20% smaller)
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

    // Table: 1 row = 1 item; with headers and dark horizontal lines
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .column(Column::remainder()) // Name
        .column(Column::remainder()) // Path
        .column(Column::remainder()) // Description
        .column(Column::auto()) // Actions
        .header(24.0, |mut header| {
            header.col(|ui| {
                let rect = ui.max_rect();
                ui.painter().rect_filled(rect, 0.0, Color32::from_gray(45));
                ui.label(RichText::new("Name").size(header_size));
            });
            header.col(|ui| {
                let rect = ui.max_rect();
                ui.painter().rect_filled(rect, 0.0, Color32::from_gray(45));
                ui.label(RichText::new("Path").size(header_size));
            });
            header.col(|ui| {
                let rect = ui.max_rect();
                ui.painter().rect_filled(rect, 0.0, Color32::from_gray(45));
                ui.label(RichText::new("Description").size(header_size));
            });
            header.col(|ui| {
                let rect = ui.max_rect();
                ui.painter().rect_filled(rect, 0.0, Color32::from_gray(45));
                ui.label(RichText::new("Actions").size(header_size));
            });
        })
        .body(|mut body| {
            for p in projects.iter() {
                let id = p.id;
                body.row(26.0, |mut row| {
                    row.col(|ui| { ui.label(RichText::new(&p.name).size(row_size)); });
                    row.col(|ui| { ui.label(RichText::new(&p.root_path).size(row_size)); });
                    row.col(|ui| {
                        let desc = p.description.as_deref().unwrap_or("");
                        let short = if desc.len() > 80 { format!("{}…", &desc[..80]) } else { desc.to_string() };
                        ui.label(RichText::new(short).size(row_size));
                    });
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            if ui.button("View").clicked() {
                                match repo::get_by_id(conn, id) {
                                    Ok(p) => {
                                        state.view_details = ProjectDetailsState::from_values(
                                            &p.name,
                                            &p.root_path,
                                            p.description.as_deref(),
                                        );
                                        state.show_view = true;
                                    }
                                    Err(e) => state.status = format!("Error loading project: {}", e),
                                }
                            }
                            if ui.button("Edit").clicked() {
                                match repo::get_by_id(conn, id) {
                                    Ok(p) => {
                                        state.editing_id = Some(p.id);
                                        state.edit_details = ProjectDetailsState::from_values(
                                            &p.name,
                                            &p.root_path,
                                            p.description.as_deref(),
                                        );
                                        state.show_edit = true;
                                    }
                                    Err(e) => state.status = format!("Error loading project: {}", e),
                                }
                            }
                            if ui.button("Delete").clicked() {
                                match repo::delete(conn, id) {
                                    Ok(_) => state.status = format!("Deleted project ID {}", id),
                                    Err(e) => state.status = format!("Error deleting: {}", e),
                                }
                            }
                        });
                    });
                });
                // dark horizontal line under each row
                body.row(2.0, |mut row| {
                    // draw line across columns by painting in each cell
                    for _i in 0..4 {
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

    // Create project window
    if state.show_create {
        let mut open = true;
        egui::Window::new("Add Project")
            .id(egui::Id::new("n001_add_project"))
            .open(&mut open)
            .show(ui.ctx(), |ui| {
            details_form(ui, &mut state.create_details);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    let name = state.create_details.name.trim();
                    let path = state.create_details.root_path.trim();
                    let desc_opt = if state.create_details.description.trim().is_empty() {
                        None
                    } else {
                        Some(state.create_details.description.trim())
                    };
                    if name.is_empty() || path.is_empty() {
                        state.status = "Name and path required".to_string();
                    } else {
                        match repo::create(conn, name, path, desc_opt) {
                            Ok(id) => {
                                state.status = format!("Project created (ID={})", id);
                                state.show_create = false;
                                state.create_details = ProjectDetailsState::default();
                            }
                            Err(e) => state.status = format!("Error creating: {}", e),
                        }
                    }
                }
                if ui.button("Cancel").clicked() {
                    state.show_create = false;
                }
            });
        });
        if !open {
            state.show_create = false;
        }
    }

    // View project window
    if state.show_view {
        let mut open = true;
        egui::Window::new("View Project")
            .id(egui::Id::new("n001_view_project"))
            .open(&mut open)
            .show(ui.ctx(), |ui| {
            ui.label(format!("Name: {}", state.view_details.name));
            ui.label(format!("Root Path: {}", state.view_details.root_path));
            ui.label(format!("Description: {}", state.view_details.description));
            ui.add_space(6.0);
            if ui.button("Close").clicked() {
                state.show_view = false;
            }
        });
        if !open {
            state.show_view = false;
        }
    }

    // Edit project window
    if state.show_edit {
        let mut open = true;
        egui::Window::new("Edit Project")
            .id(egui::Id::new("n001_edit_project"))
            .open(&mut open)
            .show(ui.ctx(), |ui| {
            details_form(ui, &mut state.edit_details);
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    if let Some(id) = state.editing_id {
                        let name = state.edit_details.name.trim().to_string();
                        let path = state.edit_details.root_path.trim().to_string();
                        let desc = if state.edit_details.description.trim().is_empty() {
                            None
                        } else {
                            Some(state.edit_details.description.trim().to_string())
                        };
                        if name.is_empty() || path.is_empty() {
                            state.status = "Name and path required".to_string();
                        } else {
                            let updated = Project { id, name, root_path: path, description: desc };
                            match repo::update(conn, &updated) {
                                Ok(_) => {
                                    state.status = "Project updated".to_string();
                                    state.show_edit = false;
                                    state.editing_id = None;
                                }
                                Err(e) => state.status = format!("Error updating: {}", e),
                            }
                        }
                    }
                }
                if ui.button("Cancel").clicked() {
                    state.show_edit = false;
                    state.editing_id = None;
                }
            });
        });
        if !open {
            state.show_edit = false;
            state.editing_id = None;
        }
    }
}
