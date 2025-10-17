use eframe::egui;
use rusqlite::Connection;

use crate::domain::n002_snapshot::repository as snapshot_repo;
use crate::domain::n002_snapshot::ui::picker::{ui_snapshot_picker, SnapshotPickerState};
use crate::domain::n003_snapshot_file::model::SnapshotFile;
use crate::domain::n003_snapshot_file::repository as repo;

#[derive(Debug, Clone, Copy, PartialEq)]
enum SortColumn {
    Name,
    Path,
    Size,
    Extension,
    CrateLayer,
    ArtifactType,
    ArtifactId,
    ArtifactName,
    Role,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum SortDirection {
    Ascending,
    Descending,
}

#[derive(Default)]
pub struct ListState {
    pub snapshot_id: Option<i64>,
    pub snapshot_name: String,
    pub snapshot_picker: SnapshotPickerState,
    pub status: String,
    pub filter: String,
    sort_column: Option<SortColumn>,
    sort_direction: SortDirection,
}

impl Default for SortDirection {
    fn default() -> Self {
        SortDirection::Ascending
    }
}

impl ListState {
    pub fn open_picker(&mut self) {
        self.snapshot_picker.open();
    }
}

pub fn ui_list(ui: &mut egui::Ui, conn: &Connection, state: &mut ListState) {
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

    ui.add_space(8.0);

    // Filter input
    ui.horizontal(|ui| {
        ui.label("Filter:");
        ui.text_edit_singleline(&mut state.filter);
        if ui.button("Clear").clicked() {
            state.filter.clear();
        }
    });

    ui.add_space(8.0);

    // File list display
    if let Some(snapshot_id) = state.snapshot_id {
        match repo::list_by_snapshot(conn, snapshot_id) {
            Ok(mut files) => {
                if files.is_empty() {
                    ui.label(egui::RichText::new("No files found. Run scan to populate.").weak());
                } else {
                    // Apply filter
                    if !state.filter.is_empty() {
                        let filter_lower = state.filter.to_lowercase();
                        files.retain(|f| {
                            f.name.to_lowercase().contains(&filter_lower)
                                || f.path.to_lowercase().contains(&filter_lower)
                                || f.file_extension
                                    .as_ref()
                                    .map_or(false, |e| e.to_lowercase().contains(&filter_lower))
                                || f.crate_layer
                                    .as_ref()
                                    .map_or(false, |c| c.to_lowercase().contains(&filter_lower))
                                || f.artifact_type
                                    .as_ref()
                                    .map_or(false, |a| a.to_lowercase().contains(&filter_lower))
                                || f.artifact_id
                                    .as_ref()
                                    .map_or(false, |a| a.to_lowercase().contains(&filter_lower))
                                || f.artifact_name
                                    .as_ref()
                                    .map_or(false, |a| a.to_lowercase().contains(&filter_lower))
                                || f.role
                                    .as_ref()
                                    .map_or(false, |r| r.to_lowercase().contains(&filter_lower))
                        });
                    }

                    // Apply sorting
                    if let Some(sort_col) = state.sort_column {
                        files.sort_by(|a, b| {
                            let cmp = match sort_col {
                                SortColumn::Name => a.name.cmp(&b.name),
                                SortColumn::Path => a.path.cmp(&b.path),
                                SortColumn::Size => a.size_bytes.cmp(&b.size_bytes),
                                SortColumn::Extension => a
                                    .file_extension
                                    .as_ref()
                                    .map(|s| s.as_str())
                                    .unwrap_or("")
                                    .cmp(
                                        b.file_extension.as_ref().map(|s| s.as_str()).unwrap_or(""),
                                    ),
                                SortColumn::CrateLayer => a
                                    .crate_layer
                                    .as_ref()
                                    .map(|s| s.as_str())
                                    .unwrap_or("")
                                    .cmp(b.crate_layer.as_ref().map(|s| s.as_str()).unwrap_or("")),
                                SortColumn::ArtifactType => a
                                    .artifact_type
                                    .as_ref()
                                    .map(|s| s.as_str())
                                    .unwrap_or("")
                                    .cmp(
                                        b.artifact_type.as_ref().map(|s| s.as_str()).unwrap_or(""),
                                    ),
                                SortColumn::ArtifactId => a
                                    .artifact_id
                                    .as_ref()
                                    .map(|s| s.as_str())
                                    .unwrap_or("")
                                    .cmp(b.artifact_id.as_ref().map(|s| s.as_str()).unwrap_or("")),
                                SortColumn::ArtifactName => a
                                    .artifact_name
                                    .as_ref()
                                    .map(|s| s.as_str())
                                    .unwrap_or("")
                                    .cmp(
                                        b.artifact_name.as_ref().map(|s| s.as_str()).unwrap_or(""),
                                    ),
                                SortColumn::Role => a
                                    .role
                                    .as_ref()
                                    .map(|s| s.as_str())
                                    .unwrap_or("")
                                    .cmp(b.role.as_ref().map(|s| s.as_str()).unwrap_or("")),
                            };

                            match state.sort_direction {
                                SortDirection::Ascending => cmp,
                                SortDirection::Descending => cmp.reverse(),
                            }
                        });
                    }

                    ui.label(format!(
                        "Total items: {} (filtered: {})",
                        files.len(),
                        files.len()
                    ));
                    ui.add_space(6.0);

                    // Table with scrolling
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            render_table(ui, &files, state);
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

fn render_table(ui: &mut egui::Ui, files: &[SnapshotFile], state: &mut ListState) {
    use egui_extras::{Column, TableBuilder};

    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(40.0)) // Icon
        .column(Column::auto().at_least(150.0)) // Name
        .column(Column::auto().at_least(250.0)) // Path
        .column(Column::auto().at_least(80.0)) // Size
        .column(Column::auto().at_least(80.0)) // Extension
        .column(Column::auto().at_least(100.0)) // Crate Layer
        .column(Column::auto().at_least(100.0)) // Artifact Type
        .column(Column::auto().at_least(80.0)) // Artifact ID
        .column(Column::auto().at_least(150.0)) // Artifact Name
        .column(Column::auto().at_least(100.0)) // Role
        .header(20.0, |mut header| {
            header.col(|_ui| {});

            header.col(|ui| {
                if sortable_header(ui, "Name", state, SortColumn::Name) {
                    toggle_sort(state, SortColumn::Name);
                }
            });

            header.col(|ui| {
                if sortable_header(ui, "Path", state, SortColumn::Path) {
                    toggle_sort(state, SortColumn::Path);
                }
            });

            header.col(|ui| {
                if sortable_header(ui, "Size", state, SortColumn::Size) {
                    toggle_sort(state, SortColumn::Size);
                }
            });

            header.col(|ui| {
                if sortable_header(ui, "Ext", state, SortColumn::Extension) {
                    toggle_sort(state, SortColumn::Extension);
                }
            });

            header.col(|ui| {
                if sortable_header(ui, "Crate", state, SortColumn::CrateLayer) {
                    toggle_sort(state, SortColumn::CrateLayer);
                }
            });

            header.col(|ui| {
                if sortable_header(ui, "Type", state, SortColumn::ArtifactType) {
                    toggle_sort(state, SortColumn::ArtifactType);
                }
            });

            header.col(|ui| {
                if sortable_header(ui, "ID", state, SortColumn::ArtifactId) {
                    toggle_sort(state, SortColumn::ArtifactId);
                }
            });

            header.col(|ui| {
                if sortable_header(ui, "Artifact", state, SortColumn::ArtifactName) {
                    toggle_sort(state, SortColumn::ArtifactName);
                }
            });

            header.col(|ui| {
                if sortable_header(ui, "Role", state, SortColumn::Role) {
                    toggle_sort(state, SortColumn::Role);
                }
            });
        })
        .body(|mut body| {
            for file in files {
                body.row(18.0, |mut row| {
                    // Icon
                    row.col(|ui| {
                        let icon = if file.is_directory { "📁" } else { "📄" };
                        ui.label(icon);
                    });

                    // Name
                    row.col(|ui| {
                        ui.label(&file.name);
                    });

                    // Path
                    row.col(|ui| {
                        ui.label(&file.path);
                    });

                    // Size
                    row.col(|ui| {
                        if file.is_directory {
                            ui.label("");
                        } else {
                            ui.label(format_size(file.size_bytes));
                        }
                    });

                    // Extension
                    row.col(|ui| {
                        ui.label(file.file_extension.as_deref().unwrap_or(""));
                    });

                    // Crate Layer
                    row.col(|ui| {
                        ui.label(file.crate_layer.as_deref().unwrap_or(""));
                    });

                    // Artifact Type
                    row.col(|ui| {
                        ui.label(file.artifact_type.as_deref().unwrap_or(""));
                    });

                    // Artifact ID
                    row.col(|ui| {
                        ui.label(file.artifact_id.as_deref().unwrap_or(""));
                    });

                    // Artifact Name
                    row.col(|ui| {
                        ui.label(file.artifact_name.as_deref().unwrap_or(""));
                    });

                    // Role
                    row.col(|ui| {
                        ui.label(file.role.as_deref().unwrap_or(""));
                    });
                });
            }
        });
}

fn sortable_header(ui: &mut egui::Ui, text: &str, state: &ListState, column: SortColumn) -> bool {
    let is_sorted = state.sort_column == Some(column);
    let arrow = if is_sorted {
        match state.sort_direction {
            SortDirection::Ascending => " ▲",
            SortDirection::Descending => " ▼",
        }
    } else {
        ""
    };

    ui.button(format!("{}{}", text, arrow)).clicked()
}

fn toggle_sort(state: &mut ListState, column: SortColumn) {
    if state.sort_column == Some(column) {
        state.sort_direction = match state.sort_direction {
            SortDirection::Ascending => SortDirection::Descending,
            SortDirection::Descending => SortDirection::Ascending,
        };
    } else {
        state.sort_column = Some(column);
        state.sort_direction = SortDirection::Ascending;
    }
}

fn format_size(bytes: i64) -> String {
    const KB: i64 = 1024;
    const MB: i64 = KB * 1024;
    const GB: i64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
