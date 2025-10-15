use eframe::egui;
use egui_dock::TabViewer;
use rusqlite::Connection;

use crate::domain::n001_project::ui::list::{ui_projects_list, ProjectsListState};
use crate::domain::n002_snapshot::ui::list::{ui_snapshots_list, SnapshotsListState};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum AppTab {
    Navbar,
    Projects,
    Snapshots,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetPanel {
    Navbar,
    Content,
}

impl AppTab {
    pub fn title(&self) -> &'static str {
        match self {
            AppTab::Navbar => "Navbar",
            AppTab::Projects => "Projects",
            AppTab::Snapshots => "Snapshots",
        }
    }

    pub fn target_panel(&self) -> TargetPanel {
        match self {
            AppTab::Navbar => TargetPanel::Navbar,
            AppTab::Projects | AppTab::Snapshots => TargetPanel::Content,
        }
    }
}

pub struct DualTabViewer<'a> {
    pub db_connection: &'a Connection,
    pub projects_state: &'a mut ProjectsListState,
    pub snapshots_state: &'a mut SnapshotsListState,
}

impl<'a> TabViewer for DualTabViewer<'a> {
    type Tab = AppTab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.title().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            AppTab::Navbar => {
                ui.heading("Navbar");
                ui.label("Navigation area");
            }
            AppTab::Projects => {
                ui.heading("Projects");
                ui_projects_list(ui, self.db_connection, self.projects_state);
            }
            AppTab::Snapshots => {
                ui.heading("Snapshots");
                ui_snapshots_list(ui, self.db_connection, self.snapshots_state);
            }
        }
    }
}
