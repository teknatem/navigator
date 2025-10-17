use eframe::egui;
use egui_dock::{DockArea, DockState, Style};
use rusqlite::Connection;

use super::panels::{AppTab, DualTabViewer};
use crate::domain::n001_project::ui::list::ProjectsListState;
use crate::domain::n002_snapshot::ui::list::SnapshotsListState;
use crate::domain::n003_snapshot_file::ui::list::ListState;
use crate::usecases::s501_create_snapshot::ScanSnapshotState;

pub struct CentralPanel;

impl CentralPanel {
    /// Shows the central panel with dock area for main content
    pub fn show(
        ctx: &egui::Context,
        dock_state: &mut DockState<AppTab>,
        db_connection: &Connection,
        projects_state: &mut ProjectsListState,
        snapshots_state: &mut SnapshotsListState,
        snapshot_files_state: &mut ListState,
        scan_snapshot_state: &mut ScanSnapshotState,
    ) {
        egui::CentralPanel::default().show(ctx, |ui| {
            DockArea::new(dock_state)
                .id(egui::Id::new("content_dock"))
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
    }
}
