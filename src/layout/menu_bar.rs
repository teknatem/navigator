use eframe::egui;

pub struct MenuBar {
    pub file_action: Option<FileAction>,
    pub edit_action: Option<EditAction>,
    pub view_action: Option<ViewAction>,
    pub settings_action: Option<SettingsAction>,
    pub help_action: Option<HelpAction>,
    pub aggregates_action: Option<AggregatesAction>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileAction {
    New,
    Open,
    Save,
    SaveAs,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditAction {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewAction {
    ToggleSidebar,
    TogglePanel,
    ToggleFullscreen,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsAction {
    ZoomIn,
    ZoomOut,
    ZoomReset,
    OpenSettingsForm,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HelpAction {
    About,
    Documentation,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AggregatesAction {
    Projects,
    Snapshots,
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuBar {
    pub fn new() -> Self {
        Self {
            file_action: None,
            edit_action: None,
            view_action: None,
            settings_action: None,
            help_action: None,
            aggregates_action: None,
        }
    }

    pub fn show(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // File menu
                ui.menu_button("Файл", |ui| {
                    if ui.button("📄 Новый").clicked() {
                        self.file_action = Some(FileAction::New);
                        ui.close_menu();
                    }
                    if ui.button("📂 Открыть...").clicked() {
                        self.file_action = Some(FileAction::Open);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("💾 Сохранить").clicked() {
                        self.file_action = Some(FileAction::Save);
                        ui.close_menu();
                    }
                    if ui.button("💾 Сохранить как...").clicked() {
                        self.file_action = Some(FileAction::SaveAs);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🚪 Выход").clicked() {
                        self.file_action = Some(FileAction::Exit);
                        ui.close_menu();
                    }
                });

                // Edit menu
                ui.menu_button("Правка", |ui| {
                    if ui.button("↶ Отменить").clicked() {
                        self.edit_action = Some(EditAction::Undo);
                        ui.close_menu();
                    }
                    if ui.button("↷ Повторить").clicked() {
                        self.edit_action = Some(EditAction::Redo);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("✂ Вырезать").clicked() {
                        self.edit_action = Some(EditAction::Cut);
                        ui.close_menu();
                    }
                    if ui.button("📋 Копировать").clicked() {
                        self.edit_action = Some(EditAction::Copy);
                        ui.close_menu();
                    }
                    if ui.button("📄 Вставить").clicked() {
                        self.edit_action = Some(EditAction::Paste);
                        ui.close_menu();
                    }
                });

                // View menu
                ui.menu_button("Вид", |ui| {
                    if ui.button("📁 Боковая панель").clicked() {
                        self.view_action = Some(ViewAction::ToggleSidebar);
                        ui.close_menu();
                    }
                    if ui.button("📊 Нижняя панель").clicked() {
                        self.view_action = Some(ViewAction::TogglePanel);
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("🖥 Полный экран").clicked() {
                        self.view_action = Some(ViewAction::ToggleFullscreen);
                        ui.close_menu();
                    }
                });

                // Settings menu
                ui.menu_button("Settings", |ui| {
                    if ui.button("⚙ Open Settings...").clicked() {
                        self.settings_action = Some(SettingsAction::OpenSettingsForm);
                        ui.close_menu();
                    }

                    ui.separator();

                    let current_zoom = ctx.zoom_factor();
                    ui.label(format!("Zoom: {:.0}%", current_zoom * 100.0));

                    if ui.button("🔍+ Zoom In (Ctrl +)").clicked() {
                        self.settings_action = Some(SettingsAction::ZoomIn);
                        ui.close_menu();
                    }
                    if ui.button("🔍- Zoom Out (Ctrl -)").clicked() {
                        self.settings_action = Some(SettingsAction::ZoomOut);
                        ui.close_menu();
                    }
                    if ui.button("🔍= Reset Zoom (Ctrl 0)").clicked() {
                        self.settings_action = Some(SettingsAction::ZoomReset);
                        ui.close_menu();
                    }
                });

                // Aggregates menu
                ui.menu_button("Aggregates", |ui| {
                    if ui.button("Projects").clicked() {
                        self.aggregates_action = Some(AggregatesAction::Projects);
                        ui.close_menu();
                    }
                    if ui.button("Snapshots").clicked() {
                        self.aggregates_action = Some(AggregatesAction::Snapshots);
                        ui.close_menu();
                    }
                });

                // Help menu
                ui.menu_button("Справка", |ui| {
                    if ui.button("📖 Документация").clicked() {
                        self.help_action = Some(HelpAction::Documentation);
                        ui.close_menu();
                    }
                    if ui.button("ℹ О программе").clicked() {
                        self.help_action = Some(HelpAction::About);
                        ui.close_menu();
                    }
                });
            });
        });
    }

    pub fn clear_actions(&mut self) {
        self.file_action = None;
        self.edit_action = None;
        self.view_action = None;
        self.settings_action = None;
        self.help_action = None;
        self.aggregates_action = None;
    }
}


