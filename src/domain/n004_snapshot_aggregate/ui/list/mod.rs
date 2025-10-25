use eframe::egui;
use rusqlite::Connection;

use crate::domain::n002_snapshot::repository as snapshot_repo;
use crate::domain::n002_snapshot::ui::picker::{ui_snapshot_picker, SnapshotPickerState};
use crate::domain::n004_snapshot_aggregate::model::SnapshotAggregate;
use crate::domain::n004_snapshot_aggregate::repository as repo;

#[derive(Debug, Clone, Copy, PartialEq)]
enum SortColumn {
    Code,
    Name,
    Description,
    FileCount,
    TotalSize,
    CreatedAt,
    UpdatedAt,
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

    // Aggregate list display
    if let Some(snapshot_id) = state.snapshot_id {
        match repo::list_by_snapshot(conn, snapshot_id) {
            Ok(mut aggregates) => {
                if aggregates.is_empty() {
                    ui.label(egui::RichText::new("No aggregates found. Create some to populate.").weak());
                } else {
                    // Apply filter
                    if !state.filter.is_empty() {
                        let filter_lower = state.filter.to_lowercase();
                        aggregates.retain(|a| {
                            a.code.to_lowercase().contains(&filter_lower)
                                || a.name.to_lowercase().contains(&filter_lower)
                                || a.description
                                    .as_ref()
                                    .map_or(false, |d| d.to_lowercase().contains(&filter_lower))
                        });
                    }

                    // Apply sorting
                    if let Some(sort_col) = state.sort_column {
                        aggregates.sort_by(|a, b| {
                            let cmp = match sort_col {
                                SortColumn::Code => a.code.cmp(&b.code),
                                SortColumn::Name => a.name.cmp(&b.name),
                                SortColumn::Description => a
                                    .description
                                    .as_ref()
                                    .map(|s| s.as_str())
                                    .unwrap_or("")
                                    .cmp(
                                        b.description.as_ref().map(|s| s.as_str()).unwrap_or(""),
                                    ),
                                SortColumn::FileCount => a.file_count.cmp(&b.file_count),
                                SortColumn::TotalSize => a.total_size_bytes.cmp(&b.total_size_bytes),
                                SortColumn::CreatedAt => a.created_at.cmp(&b.created_at),
                                SortColumn::UpdatedAt => a.updated_at.cmp(&b.updated_at),
                            };

                            match state.sort_direction {
                                SortDirection::Ascending => cmp,
                                SortDirection::Descending => cmp.reverse(),
                            }
                        });
                    }

                    // Show statistics
                    if let Ok((aggregate_count, total_files, total_size)) = repo::get_stats(conn, snapshot_id) {
                        ui.horizontal(|ui| {
                            ui.label(format!("Aggregates: {}", aggregate_count));
                            ui.label(format!("Total Files: {}", total_files));
                            ui.label(format!("Total Size: {}", format_size(total_size)));
                        });
                        ui.add_space(6.0);
                    }

                    ui.label(format!(
                        "Displaying {} aggregates",
                        aggregates.len()
                    ));
                    ui.add_space(6.0);

                    // Table with scrolling
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            render_table(ui, &aggregates, state);
                        });
                }
            }
            Err(e) => {
                ui.colored_label(
                    egui::Color32::LIGHT_RED,
                    format!("Error loading aggregates: {}", e),
                );
            }
        }
    } else {
        ui.label(egui::RichText::new("Select a snapshot to view its aggregates").weak());
    }

    if !state.status.is_empty() {
        ui.add_space(12.0);
        ui.colored_label(egui::Color32::LIGHT_BLUE, &state.status);
    }
}

fn render_table(ui: &mut egui::Ui, aggregates: &[SnapshotAggregate], state: &mut ListState) {
    use egui_extras::{Column, TableBuilder};

    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
        .column(Column::auto().at_least(100.0)) // Code
        .column(Column::auto().at_least(200.0)) // Name
        .column(Column::auto().at_least(300.0)) // Description
        .column(Column::auto().at_least(80.0)) // File Count
        .column(Column::auto().at_least(100.0)) // Total Size
        .column(Column::auto().at_least(150.0)) // Created At
        .column(Column::auto().at_least(150.0)) // Updated At
        .header(20.0, |mut header| {
            header.col(|ui| {
                if sortable_header(ui, "Code", state, SortColumn::Code) {
                    toggle_sort(state, SortColumn::Code);
                }
            });

            header.col(|ui| {
                if sortable_header(ui, "Name", state, SortColumn::Name) {
                    toggle_sort(state, SortColumn::Name);
                }
            });

            header.col(|ui| {
                if sortable_header(ui, "Description", state, SortColumn::Description) {
                    toggle_sort(state, SortColumn::Description);
                }
            });

            header.col(|ui| {
                if sortable_header(ui, "Files", state, SortColumn::FileCount) {
                    toggle_sort(state, SortColumn::FileCount);
                }
            });

            header.col(|ui| {
                if sortable_header(ui, "Size", state, SortColumn::TotalSize) {
                    toggle_sort(state, SortColumn::TotalSize);
                }
            });

            header.col(|ui| {
                if sortable_header(ui, "Created", state, SortColumn::CreatedAt) {
                    toggle_sort(state, SortColumn::CreatedAt);
                }
            });

            header.col(|ui| {
                if sortable_header(ui, "Updated", state, SortColumn::UpdatedAt) {
                    toggle_sort(state, SortColumn::UpdatedAt);
                }
            });
        })
        .body(|mut body| {
            for aggregate in aggregates {
                body.row(18.0, |mut row| {
                    // Code
                    row.col(|ui| {
                        ui.label(&aggregate.code);
                    });

                    // Name
                    row.col(|ui| {
                        ui.label(&aggregate.name);
                    });

                    // Description
                    row.col(|ui| {
                        ui.label(aggregate.description.as_deref().unwrap_or(""));
                    });

                    // File Count
                    row.col(|ui| {
                        ui.label(format!("{}", aggregate.file_count));
                    });

                    // Total Size
                    row.col(|ui| {
                        ui.label(format_size(aggregate.total_size_bytes));
                    });

                    // Created At
                    row.col(|ui| {
                        ui.label(format_timestamp(&aggregate.created_at));
                    });

                    // Updated At
                    row.col(|ui| {
                        ui.label(format_timestamp(&aggregate.updated_at));
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

fn format_timestamp(timestamp: &str) -> String {
    // Try to parse and format the timestamp for better display
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(timestamp) {
        dt.format("%Y-%m-%d %H:%M").to_string()
    } else {
        timestamp.to_string()
    }
}
