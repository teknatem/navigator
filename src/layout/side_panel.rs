use eframe::egui;
use egui_dock::{DockArea, DockState, Style};
use rusqlite::Connection;

use super::panels::{AppTab, DualTabViewer};
use crate::domain::n001_project::ui::list::ProjectsListState;
use crate::domain::n002_snapshot::ui::list::SnapshotsListState;
use crate::domain::n003_snapshot_file::ui::list::ListState;
use crate::usecases::s501_create_snapshot::ScanSnapshotState;

pub struct SidePanel;

impl SidePanel {
    /// Shows the left side panel with dock area for navigation
    /// Returns the measured width of the panel after rendering
    pub fn show(
        ctx: &egui::Context,
        dock_state: &mut DockState<AppTab>,
        db_connection: &Connection,
        projects_state: &mut ProjectsListState,
        snapshots_state: &mut SnapshotsListState,
        snapshot_files_state: &mut ListState,
        scan_snapshot_state: &mut ScanSnapshotState,
        screen_width: f32,
        stored_width_fraction: f32,
        last_width: f32,
        is_resizing: bool,
    ) -> f32 {
        let mut measured_width = screen_width * stored_width_fraction;

        // Determine if pointer is currently down
        let pointer_down = ctx.input(|i| {
            i.pointer.primary_down() || i.pointer.secondary_down() || i.pointer.middle_down()
        });

        // Build the side panel with appropriate width settings
        let mut panel = egui::SidePanel::left("side_panel")
            .resizable(true)
            .min_width(60.0)
            .max_width(screen_width * 0.50);

        // Use exact width unless actively resizing
        if !(is_resizing || pointer_down) {
            panel = panel.exact_width(screen_width * stored_width_fraction);
        } else {
            let initial_width = if last_width > 0.0 {
                last_width
            } else {
                screen_width * stored_width_fraction
            };
            panel = panel.default_width(initial_width);
        }

        panel.show(ctx, |ui| {
            measured_width = ui.max_rect().width();
            DockArea::new(dock_state)
                .id(egui::Id::new("side_dock"))
                .style(Style::from_egui(ctx.style().as_ref()))
                .show_inside(
                    ui,
                    &mut DualTabViewer {
                        db_connection,
                        projects_state,
                        snapshots_state,
                        snapshot_files_state,
                        scan_snapshot_state,
                    },
                );
        });

        measured_width
    }
}
