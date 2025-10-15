#![allow(dead_code)]
use eframe::egui;
use rusqlite::Connection;

use crate::domain::n002_snapshot::model::Snapshot;
use crate::domain::n002_snapshot::repository as repo;

#[derive(Default)]
pub struct SnapshotPickerState {
    pub open: bool,
    pub search: String,
    pub selected_id: Option<i64>,
    pub status: String,
}

impl SnapshotPickerState {
    pub fn open(&mut self) {
        self.open = true;
        self.selected_id = None;
        self.status.clear();
    }
}

pub fn ui_snapshot_picker(
    ui: &mut egui::Ui,
    conn: &Connection,
    state: &mut SnapshotPickerState,
) -> Option<i64> {
    if !state.open { return None; }
    let mut result: Option<i64> = None;
    let mut open = true;

    egui::Window::new("Pick Snapshot")
        .collapsible(false)
        .resizable(true)
        .open(&mut open)
        .show(ui.ctx(), |ui| {
            ui.horizontal(|ui| {
                ui.label("Search:");
                let _ = ui.text_edit_singleline(&mut state.search);
            });
            ui.add_space(6.0);

            let list: Vec<Snapshot> = match repo::list_all(conn) {
                Ok(list) => list,
                Err(e) => { state.status = format!("Error: {}", e); Vec::new() }
            };
            let q = state.search.to_lowercase();
            let filtered = if q.is_empty() { list } else { list.into_iter().filter(|s|
                s.name.to_lowercase().contains(&q)
                || s.comment.as_deref().unwrap_or("").to_lowercase().contains(&q)
            ).collect() };

            ui.group(|ui| {
                ui.set_min_height(200.0);
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for s in filtered.iter() {
                        let is_selected = state.selected_id == Some(s.id);
                        let mut label = egui::RichText::new(format!("{}", s.name));
                        if is_selected { label = label.strong(); }
                        let response = ui.selectable_label(is_selected, label);
                        if response.clicked() { state.selected_id = Some(s.id); }
                        if response.double_clicked() { state.selected_id = Some(s.id); result = state.selected_id; }
                    }
                });
            });

            ui.add_space(6.0);
            if !state.status.is_empty() { ui.colored_label(egui::Color32::LIGHT_RED, &state.status); }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                let can = state.selected_id.is_some();
                if ui.add_enabled(can, egui::Button::new("Select")).clicked() { result = state.selected_id; }
                if ui.button("Cancel").clicked() { state.open = false; state.selected_id = None; }
            });
        });

    if !open { state.open = false; }
    if let Some(id) = result { state.open = false; return Some(id); }
    None
}
