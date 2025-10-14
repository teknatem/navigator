use eframe::egui;
use rusqlite::Connection;

use crate::domain::n001_project::model::Project;
use crate::domain::n001_project::repository as repo;

#[derive(Default)]
pub struct ProjectPickerState {
    pub open: bool,
    pub search: String,
    pub selected_id: Option<i64>,
    pub status: String,
}

impl ProjectPickerState {
    pub fn open(&mut self) {
        self.open = true;
        self.selected_id = None;
        self.status.clear();
    }
}

/// Renders a modal window to pick a project.
/// Returns Some(id) in the frame when user confirms selection; otherwise None.
pub fn ui_project_picker(
    ui: &mut egui::Ui,
    conn: &Connection,
    state: &mut ProjectPickerState,
) -> Option<i64> {
    if !state.open {
        return None;
    }

    let mut result: Option<i64> = None;
    let mut open = true;

    egui::Window::new("Pick Project")
        .collapsible(false)
        .resizable(true)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            // Search row
            ui.horizontal(|ui| {
                ui.label("Search:");
                let resp = ui.text_edit_singleline(&mut state.search);
                if resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    // no-op: filter applies automatically
                }
            });

            ui.add_space(6.0);

            // Load projects
            let projects: Vec<Project> = match repo::list_all(conn) {
                Ok(list) => list,
                Err(e) => {
                    state.status = format!("Error loading projects: {}", e);
                    Vec::new()
                }
            };

            // Filter
            let query = state.search.to_lowercase();
            let filtered = if query.is_empty() {
                projects
            } else {
                projects
                    .into_iter()
                    .filter(|p| {
                        p.name.to_lowercase().contains(&query)
                            || p.root_path.to_lowercase().contains(&query)
                            || p
                                .description
                                .as_deref()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&query)
                    })
                    .collect::<Vec<_>>()
            };

            // List
            ui.group(|ui| {
                ui.set_min_height(200.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for p in filtered.iter() {
                        let is_selected = state.selected_id == Some(p.id);
                        let mut label = egui::RichText::new(format!("{}", p.name));
                        if is_selected {
                            label = label.strong();
                        }
                        let response = ui.selectable_label(is_selected, label);
                        let clicked = response.clicked();
                        let double_clicked = response.double_clicked();
                        // optional: could show tooltip here if desired

                        if clicked {
                            state.selected_id = Some(p.id);
                        }
                        if double_clicked {
                            state.selected_id = Some(p.id);
                            result = state.selected_id;
                        }
                    }
                });
            });

            ui.add_space(6.0);

            // Status
            if !state.status.is_empty() {
                ui.colored_label(egui::Color32::LIGHT_RED, &state.status);
            }

            ui.add_space(6.0);

            // Actions
            ui.horizontal(|ui| {
                let can_select = state.selected_id.is_some();
                if ui.add_enabled(can_select, egui::Button::new("Select")).clicked() {
                    result = state.selected_id;
                }
                if ui.button("Cancel").clicked() {
                    state.selected_id = None;
                    result = None;
                    state.status.clear();
                    state.open = false;
                }
            });
        });

    if !open {
        state.open = false;
    }

    if let Some(id) = result {
        state.open = false;
        return Some(id);
    }

    None
}
