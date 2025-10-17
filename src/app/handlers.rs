use eframe::egui;

use crate::{app_settings::AppSettings, layout::Theme};

impl crate::MyApp {
    pub fn handle_zoom_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::CTRL, egui::Key::Plus)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::Equals)
        }) {
            let current_zoom = ctx.zoom_factor();
            let new_zoom = (current_zoom + 0.1).min(3.0);
            ctx.set_zoom_factor(new_zoom);
            self.db_status = format!("Масштаб: {:.0}%", new_zoom * 100.0);
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Minus)) {
            let current_zoom = ctx.zoom_factor();
            let new_zoom = (current_zoom - 0.1).max(0.5);
            ctx.set_zoom_factor(new_zoom);
            self.db_status = format!("Масштаб: {:.0}%", new_zoom * 100.0);
        }

        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Num0)) {
            ctx.set_zoom_factor(1.0);
            self.db_status = "Масштаб: 100%".to_string();
        }
    }

    pub fn handle_menu_actions(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use crate::layout::menu_bar::{
            AggregatesAction, EditAction, FileAction, HelpAction, SettingsAction, UseCasesAction,
            ViewAction,
        };

        if let Some(FileAction::Exit) = self.menu_bar.file_action {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        if let Some(action) = self.menu_bar.edit_action {
            match action {
                EditAction::Undo => self.db_status = "Действие: Отменить".to_string(),
                EditAction::Redo => self.db_status = "Действие: Повторить".to_string(),
                EditAction::Cut => self.db_status = "Действие: Вырезать".to_string(),
                EditAction::Copy => self.db_status = "Действие: Копировать".to_string(),
                EditAction::Paste => self.db_status = "Действие: Вставить".to_string(),
            }
        }

        if let Some(action) = self.menu_bar.view_action {
            match action {
                ViewAction::ToggleNavbar => {
                    self.show_navbar = !self.show_navbar;
                    self.menu_bar.navbar_visible = self.show_navbar;
                }
                ViewAction::ToggleFullscreen => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    self.db_status = "Действие: Полный экран".to_string();
                }
            }
        }

        if let Some(action) = self.menu_bar.settings_action {
            let current_zoom = ctx.zoom_factor();
            match action {
                SettingsAction::OpenSettingsForm => {
                    let current_theme = if ctx.style().visuals.dark_mode {
                        Theme::Dark
                    } else {
                        Theme::Light
                    };
                    self.settings_form.open(current_zoom, current_theme);
                    self.db_status = "Settings opened".to_string();
                }
                SettingsAction::ZoomIn => {
                    let new_zoom = (current_zoom + 0.1).min(3.0);
                    ctx.set_zoom_factor(new_zoom);
                    self.db_status = format!("Масштаб: {:.0}%", new_zoom * 100.0);
                }
                SettingsAction::ZoomOut => {
                    let new_zoom = (current_zoom - 0.1).max(0.5);
                    ctx.set_zoom_factor(new_zoom);
                    self.db_status = format!("Масштаб: {:.0}%", new_zoom * 100.0);
                }
                SettingsAction::ZoomReset => {
                    ctx.set_zoom_factor(1.0);
                    self.db_status = "Масштаб: 100%".to_string();
                }
            }
        }

        if let Some(action) = self.menu_bar.aggregates_action {
            match action {
                AggregatesAction::Projects => {
                    self.open_projects_tab();
                    self.db_status = "Opened Projects tab".to_string();
                }
                AggregatesAction::Snapshots => {
                    self.open_snapshots_tab();
                    self.db_status = "Opened Snapshots tab".to_string();
                }
                AggregatesAction::SnapshotFiles => {
                    self.open_snapshot_files_tab();
                    self.db_status = "Opened Snapshot Files tab".to_string();
                }
            }
        }

        if let Some(action) = self.menu_bar.usecases_action {
            match action {
                UseCasesAction::ScanSnapshot => {
                    self.open_scan_snapshot_tab();
                    self.db_status = "Opened Scan Snapshot tab".to_string();
                }
            }
        }

        if let Some(action) = self.menu_bar.help_action {
            match action {
                HelpAction::Documentation => self.db_status = "Действие: Документация".to_string(),
                HelpAction::About => {
                    self.db_status =
                        "Navigator v0.1.0 - Rust egui + egui_dock + SQLite".to_string();
                }
            }
        }

        self.menu_bar.clear_actions();
    }

    pub fn apply_initial_settings(&mut self, ctx: &egui::Context) {
        let theme = self.settings_form.get_theme();
        let zoom = self.settings_form.get_zoom();

        match theme {
            Theme::Light => ctx.set_visuals(egui::Visuals::light()),
            Theme::Dark => ctx.set_visuals(egui::Visuals::dark()),
        }
        ctx.set_zoom_factor(zoom);
        self.db_status = format!(
            "Settings loaded: Theme={:?}, Zoom={:.0}%",
            theme,
            zoom * 100.0
        );
    }

    pub fn apply_and_save_settings(&mut self, ctx: &egui::Context) {
        let theme = self.settings_form.get_theme();
        let zoom = self.settings_form.get_zoom();

        match theme {
            Theme::Light => ctx.set_visuals(egui::Visuals::light()),
            Theme::Dark => ctx.set_visuals(egui::Visuals::dark()),
        }
        ctx.set_zoom_factor(zoom);

        // Save all current settings
        let navbar_width_frac = self.settings_form.get_navbar_width_frac();
        let app_settings = AppSettings {
            theme,
            zoom,
            navbar_width_frac,
        };
        if app_settings.save_to_db(&self.db_connection).is_ok() {
            self.db_status = format!(
                "Settings saved: Theme={:?}, Zoom={:.0}%",
                theme,
                zoom * 100.0
            );
        }
    }
}
