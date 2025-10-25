use eframe::egui;
use egui_dock::TabViewer;
use rusqlite::Connection;

use crate::domain::n001_project::ui::list::{ui_projects_list, ProjectsListState};
use crate::domain::n002_snapshot::ui::list::{ui_snapshots_list, SnapshotsListState};
use crate::domain::n003_snapshot_file::ui::list::{ui_list, ListState};
use crate::domain::n004_snapshot_aggregate::ui::list::{ui_list as ui_aggregates_list, ListState as AggregatesListState};
use crate::usecases::s501_create_snapshot::{ui_scan_snapshot, ScanSnapshotState};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AppTab {
    Projects,
    Snapshots,
    SnapshotFiles,
    SnapshotAggregates,
    ScanSnapshot,
}

impl AppTab {
    pub fn title(&self) -> &'static str {
        match self {
            AppTab::Projects => "Projects",
            AppTab::Snapshots => "Snapshots",
            AppTab::SnapshotFiles => "Snapshot Files",
            AppTab::SnapshotAggregates => "Snapshot Aggregates",
            AppTab::ScanSnapshot => "Scan Snapshot",
        }
    }
}

pub struct DualTabViewer<'a> {
    pub db_connection: &'a Connection,
    pub projects_state: &'a mut ProjectsListState,
    pub snapshots_state: &'a mut SnapshotsListState,
    pub snapshot_files_state: &'a mut ListState,
    pub snapshot_aggregates_state: &'a mut AggregatesListState,
    pub scan_snapshot_state: &'a mut ScanSnapshotState,
}

impl<'a> TabViewer for DualTabViewer<'a> {
    type Tab = AppTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            AppTab::Projects => {
                ui.heading("Projects");
                ui_projects_list(ui, self.db_connection, self.projects_state);
            }
            AppTab::Snapshots => {
                ui.heading("Snapshots");
                ui_snapshots_list(ui, self.db_connection, self.snapshots_state);
            }
            AppTab::SnapshotFiles => {
                ui.heading("Snapshot Files");
                ui_list(ui, self.db_connection, self.snapshot_files_state);
            }
            AppTab::SnapshotAggregates => {
                ui.heading("Snapshot Aggregates");
                ui_aggregates_list(ui, self.db_connection, self.snapshot_aggregates_state);
            }
            AppTab::ScanSnapshot => {
                ui_scan_snapshot(ui, self.db_connection, self.scan_snapshot_state);
            }
        }
    }
}
