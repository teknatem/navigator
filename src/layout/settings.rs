use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Light,
    Dark,
}

pub struct SettingsForm {
    pub is_open: bool,
    // Temporary values (before Apply)
    temp_theme: Theme,
    temp_zoom: f32,
    temp_navbar_width_frac: f32,
    // Applied values
    pub current_theme: Theme,
    pub current_zoom: f32,
    pub current_navbar_width_frac: f32,
}

impl Default for SettingsForm {
    fn default() -> Self {
        Self::new()
    }
}

impl SettingsForm {
    pub fn new() -> Self {
        Self {
            is_open: false,
            temp_theme: Theme::Dark,
            temp_zoom: 1.0,
            temp_navbar_width_frac: 0.20,
            current_theme: Theme::Dark,
            current_zoom: 1.0,
            current_navbar_width_frac: 0.20,
        }
    }

    pub fn new_with_settings(settings: &crate::app_settings::AppSettings) -> Self {
        Self {
            is_open: false,
            temp_theme: settings.theme,
            temp_zoom: settings.zoom,
            temp_navbar_width_frac: settings.navbar_width_frac,
            current_theme: settings.theme,
            current_zoom: settings.zoom,
            current_navbar_width_frac: settings.navbar_width_frac,
        }
    }

    pub fn open(&mut self, current_zoom: f32, current_theme: Theme) {
        self.is_open = true;
        self.temp_zoom = current_zoom;
        self.temp_theme = current_theme;
    }

    pub fn show(&mut self, ctx: &egui::Context) -> bool {
        if !self.is_open {
            return false;
        }

        let mut settings_changed = false;
        let mut should_close = false;

        egui::Window::new("⚙ Settings")
            .open(&mut self.is_open)
            .resizable(true)
            .default_width(400.0)
            .default_height(300.0)
            .show(ctx, |ui| {
                ui.heading("Application Settings");
                ui.add_space(10.0);

                // Theme selection
                ui.group(|ui| {
                    ui.label("Theme:");
                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.selectable_value(&mut self.temp_theme, Theme::Light, "☀ Light");
                        ui.selectable_value(&mut self.temp_theme, Theme::Dark, "🌙 Dark");
                    });
                });

                ui.add_space(15.0);

                // Zoom control
                ui.group(|ui| {
                    ui.label("Zoom Level:");
                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label(format!("{:.0}%", self.temp_zoom * 100.0));
                        ui.add_space(10.0);

                        if ui.button("➖").clicked() {
                            self.temp_zoom = (self.temp_zoom - 0.1).max(0.5);
                        }

                        ui.add(egui::Slider::new(&mut self.temp_zoom, 0.5..=3.0).show_value(false));

                        if ui.button("➕").clicked() {
                            self.temp_zoom = (self.temp_zoom + 0.1).min(3.0);
                        }
                    });

                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        ui.label("Range: 50% - 300%");
                        if ui.button("Reset to 100%").clicked() {
                            self.temp_zoom = 1.0;
                        }
                    });
                });

                ui.add_space(20.0);

                // Navbar width
                ui.group(|ui| {
                    ui.label("Navbar Width:");
                    ui.add_space(5.0);

                    let mut pct = (self.temp_navbar_width_frac * 100.0).round();
                    ui.horizontal(|ui| {
                        ui.label(format!("{:.0}%", pct));
                        ui.add_space(10.0);
                        let range = 10.0..=50.0;
                        if ui
                            .add(egui::Slider::new(&mut pct, range).show_value(false))
                            .changed()
                        {
                            self.temp_navbar_width_frac = (pct / 100.0) as f32;
                        }
                    });
                });

                ui.add_space(20.0);

                // Action buttons
                ui.separator();
                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("✓ Apply").clicked() {
                        self.current_theme = self.temp_theme;
                        self.current_zoom = self.temp_zoom;
                        self.current_navbar_width_frac = self.temp_navbar_width_frac;
                        settings_changed = true;
                        should_close = true;
                    }

                    ui.add_space(10.0);

                    if ui.button("Cancel").clicked() {
                        // Revert temporary changes
                        self.temp_theme = self.current_theme;
                        self.temp_zoom = self.current_zoom;
                        self.temp_navbar_width_frac = self.current_navbar_width_frac;
                        should_close = true;
                    }
                });

                ui.add_space(5.0);
                ui.label("💡 Settings will be applied after clicking 'Apply'");
            });

        if should_close {
            self.is_open = false;
        }

        settings_changed
    }

    pub fn get_theme(&self) -> Theme {
        self.current_theme
    }

    pub fn get_zoom(&self) -> f32 {
        self.current_zoom
    }
}

impl SettingsForm {
    pub fn get_navbar_width_frac(&self) -> f32 { self.current_navbar_width_frac }
    pub fn set_current_navbar_width_frac(&mut self, frac: f32) {
        self.current_navbar_width_frac = frac.clamp(0.10, 0.50);
        self.temp_navbar_width_frac = self.current_navbar_width_frac;
    }
}

