use eframe::egui;
use std::time::{Duration, Instant};
use egui_dock::{DockArea, DockState, Style, SurfaceIndex};
use rusqlite::Connection;

mod app_settings;
mod domain;
mod layout;
mod app;
mod shared;

use app_settings::AppSettings;
use domain::n001_project::ui::list::ProjectsListState;
use domain::n002_snapshot::ui::list::SnapshotsListState;
use layout::{AppTab, DualTabViewer, MenuBar, SettingsForm, TargetPanel};

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Navigator"),
        ..Default::default()
    };

    eframe::run_native("Navigator", options, Box::new(|_cc| Ok(Box::new(MyApp::new()))))
}

struct MyApp {
    // Two independent dock areas: left (navbar) and right (content)
    dock_nav: DockState<AppTab>,
    dock_content: DockState<AppTab>,
    // Database and status
    db_connection: Connection,
    db_items: Vec<(i32, String)>,
    db_status: String,
    // Domain UI states
    projects_state: ProjectsListState,
    snapshots_state: SnapshotsListState,
    // Menu & settings
    menu_bar: MenuBar,
    settings_form: SettingsForm,
    first_frame: bool,
    saved_navbar_width_frac: f32,
    // Debounce autosave of navbar width
    pending_nav_frac: Option<f32>,
    nav_save_deadline: Option<Instant>,
    last_nav_w: f32,
    resizing_active: bool,
    show_navbar: bool,
}

impl MyApp {
    fn open_projects_tab(&mut self) {
        self.open_or_focus(AppTab::Projects);
    }
    fn open_snapshots_tab(&mut self) {
        self.open_or_focus(AppTab::Snapshots);
    }

    fn open_or_focus(&mut self, tab: AppTab) {
        match tab.target_panel() {
            TargetPanel::Navbar => {
                let ds = &mut self.dock_nav;
                if let Some((node_idx, tab_idx)) = ds.find_main_surface_tab(&tab) {
                    ds.set_active_tab((SurfaceIndex::main(), node_idx, tab_idx));
                    ds.set_focused_node_and_surface((SurfaceIndex::main(), node_idx));
                } else {
                    ds.main_surface_mut().push_to_focused_leaf(tab);
                }
            }
            TargetPanel::Content => {
                let ds = &mut self.dock_content;
                if let Some((node_idx, tab_idx)) = ds.find_main_surface_tab(&tab) {
                    ds.set_active_tab((SurfaceIndex::main(), node_idx, tab_idx));
                    ds.set_focused_node_and_surface((SurfaceIndex::main(), node_idx));
                } else {
                    ds.main_surface_mut().push_to_focused_leaf(tab);
                }
            }
        }
    }

    fn new() -> Self {
        let dock_nav = DockState::new(vec![AppTab::Navbar]);
        let dock_content = DockState::new(vec![AppTab::Projects]);

        // Open or create database and ensure schema
        let db_connection =
            crate::shared::db::open_or_create(crate::shared::db::DB_PATH).expect("Failed to open/create database");

        // Load saved settings
        let saved_settings = AppSettings::load_from_db(&db_connection).unwrap_or_else(|_| {
            println!("No saved settings found, using defaults");
            AppSettings::default()
        });

        let mut app = Self {
            dock_nav,
            dock_content,
            db_connection,
            db_items: Vec::new(),
            db_status: String::from("Ready"),
            menu_bar: MenuBar::new(),
            settings_form: SettingsForm::new_with_settings(&saved_settings),
            first_frame: true,
            projects_state: ProjectsListState::default(),
            snapshots_state: SnapshotsListState::default(),
            saved_navbar_width_frac: saved_settings.navbar_width_frac,
            pending_nav_frac: None,
            nav_save_deadline: None,
            last_nav_w: 0.0,
            resizing_active: false,
            show_navbar: true,
        };

        app.load_items();
        app
    }

    fn load_items(&mut self) {
        self.db_items.clear();
        let mut stmt = self
            .db_connection
            .prepare("SELECT id, name FROM items ORDER BY id DESC")
            .expect("prepare failed");

        let items_iter = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("query failed");

        for item in items_iter {
            if let Ok(item) = item {
                self.db_items.push(item);
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Apply persisted theme/zoom on first frame
        if self.first_frame {
            self.apply_initial_settings(ctx);
            self.first_frame = false;
        }

        // Global shortcuts
        self.handle_zoom_shortcuts(ctx);

        // Menubar + actions
        self.menu_bar.navbar_visible = self.show_navbar;
        self.menu_bar.show(ctx);
        self.handle_menu_actions(ctx, frame);

        // Settings dialog
        if self.settings_form.show(ctx) {
            self.apply_and_save_settings(ctx);
        }

        // Two-panel layout: left navbar (resizable), right content
        let screen_w = ctx.input(|i| i.screen_rect().width());
        let stored_frac = self.settings_form.get_navbar_width_frac();
        let mut measured_nav_w: f32 = screen_w * stored_frac;

        // Use exact width from stored fraction unless user is actively resizing
        let pointer_down_pre = ctx.input(|i| i.pointer.primary_down() || i.pointer.secondary_down() || i.pointer.middle_down());
        if self.show_navbar {
        let mut panel = egui::SidePanel::left("navbar_panel")
            .resizable(true)
            .min_width(60.0)
            .max_width(screen_w * 0.50);
        if !(self.resizing_active || pointer_down_pre) {
            panel = panel.exact_width(screen_w * stored_frac);
        } else {
            let initial_w = if self.last_nav_w > 0.0 { self.last_nav_w } else { screen_w * stored_frac };
            panel = panel.default_width(initial_w);
        }
        panel.show(ctx, |ui| {
                measured_nav_w = ui.max_rect().width();
                DockArea::new(&mut self.dock_nav)
                    .id(egui::Id::new("nav_dock"))
                    .style(Style::from_egui(ctx.style().as_ref()))
                    .show_inside(
                        ui,
                        &mut DualTabViewer {
                            db_connection: &self.db_connection,
                            projects_state: &mut self.projects_state,
                            snapshots_state: &mut self.snapshots_state,
                        },
                    );
            });
        }

        // Persist current width into settings (as fraction) and debounce auto-save if changed
        let pointer_down = ctx.input(|i| i.pointer.primary_down() || i.pointer.secondary_down() || i.pointer.middle_down());
        let changed = (measured_nav_w - self.last_nav_w).abs() > 0.5;
        if pointer_down && changed {
            self.resizing_active = true;
        }
        if !pointer_down && self.resizing_active {
            // User finished resizing: schedule save by fraction
            let new_frac = (measured_nav_w / screen_w).clamp(0.10, 0.50);
            self.pending_nav_frac = Some(new_frac);
            // Reflect immediately in Settings UI
            self.settings_form.set_current_navbar_width_frac(new_frac);
            self.nav_save_deadline = Some(Instant::now() + Duration::from_millis(250));
            self.resizing_active = false;
        }
        self.last_nav_w = measured_nav_w;

        // Debounced save
        if let (Some(target_frac), Some(deadline)) = (self.pending_nav_frac, self.nav_save_deadline) {
            if Instant::now() >= deadline {
                let theme = self.settings_form.get_theme();
                let zoom = self.settings_form.get_zoom();
                let app_settings = AppSettings { theme, zoom, navbar_width_frac: target_frac };
                if app_settings.save_to_db(&self.db_connection).is_ok() {
                    self.saved_navbar_width_frac = target_frac;
                }
                self.pending_nav_frac = None;
                self.nav_save_deadline = None;
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            DockArea::new(&mut self.dock_content)
                .id(egui::Id::new("content_dock"))
                .style(Style::from_egui(ctx.style().as_ref()))
                .show_inside(
                    ui,
                    &mut DualTabViewer {
                        db_connection: &self.db_connection,
                        projects_state: &mut self.projects_state,
                        snapshots_state: &mut self.snapshots_state,
                    },
                );
        });
    }
}
