use eframe::egui;
use egui_dock::{DockState, SurfaceIndex};
use rusqlite::Connection;
use std::time::{Duration, Instant};

mod app;
mod app_settings;
mod domain;
mod layout;
mod shared;
mod usecases;

use app_settings::AppSettings;
use domain::n001_project::ui::list::ProjectsListState;
use domain::n002_snapshot::ui::list::SnapshotsListState;
use domain::n003_snapshot_file::ui::tree_view::TreeViewState;
use layout::{AppTab, CentralPanel, MenuBar, SettingsForm, SidePanel};
use usecases::s501_create_snapshot::ScanSnapshotState;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Navigator"),
        ..Default::default()
    };

    eframe::run_native(
        "Navigator",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::new()))),
    )
}

struct MyApp {
    // Two independent dock areas: left (side panel) and right (content)
    dock_nav: DockState<AppTab>,
    dock_content: DockState<AppTab>,
    // Database and status
    db_connection: Connection,
    db_items: Vec<(i32, String)>,
    db_status: String,
    // Domain UI states
    projects_state: ProjectsListState,
    snapshots_state: SnapshotsListState,
    snapshot_files_state: TreeViewState,
    // Usecase UI states
    scan_snapshot_state: ScanSnapshotState,
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
    fn open_snapshot_files_tab(&mut self) {
        self.open_or_focus(AppTab::SnapshotFiles);
    }
    fn open_scan_snapshot_tab(&mut self) {
        self.open_or_focus(AppTab::ScanSnapshot);
    }

    fn open_or_focus(&mut self, tab: AppTab) {
        let ds = &mut self.dock_content;
        if let Some((node_idx, tab_idx)) = ds.find_main_surface_tab(&tab) {
            ds.set_active_tab((SurfaceIndex::main(), node_idx, tab_idx));
            ds.set_focused_node_and_surface((SurfaceIndex::main(), node_idx));
        } else {
            ds.main_surface_mut().push_to_focused_leaf(tab);
        }
    }

    fn new() -> Self {
        let dock_nav = DockState::new(vec![]);
        let dock_content = DockState::new(vec![AppTab::Projects]);

        // Open or create database and ensure schema
        let db_connection = crate::shared::db::open_or_create(crate::shared::db::DB_PATH)
            .expect("Failed to open/create database");

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
            snapshot_files_state: TreeViewState::default(),
            scan_snapshot_state: ScanSnapshotState::default(),
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
        // 1. Apply persisted theme/zoom on first frame
        if self.first_frame {
            self.apply_initial_settings(ctx);
            self.first_frame = false;
        }

        // 2. Global shortcuts
        self.handle_zoom_shortcuts(ctx);

        // 3. TopBottomPanel (menu)
        self.menu_bar.navbar_visible = self.show_navbar;
        self.menu_bar.show(ctx);
        self.handle_menu_actions(ctx, frame);

        // 4. Settings Window
        if self.settings_form.show(ctx) {
            self.apply_and_save_settings(ctx);
        }

        // 5. SidePanel (left side panel)
        let screen_w = ctx.input(|i| i.screen_rect().width());
        let stored_frac = self.settings_form.get_navbar_width_frac();
        let mut measured_nav_w = screen_w * stored_frac;

        if self.show_navbar {
            measured_nav_w = SidePanel::show(
                ctx,
                &mut self.dock_nav,
                &self.db_connection,
                &mut self.projects_state,
                &mut self.snapshots_state,
                &mut self.snapshot_files_state,
                &mut self.scan_snapshot_state,
                screen_w,
                stored_frac,
                self.last_nav_w,
                self.resizing_active,
            );
        }

        // 6. Handle SidePanel width changes (debounced save)
        let pointer_down = ctx.input(|i| {
            i.pointer.primary_down() || i.pointer.secondary_down() || i.pointer.middle_down()
        });
        let changed = (measured_nav_w - self.last_nav_w).abs() > 0.5;
        if pointer_down && changed {
            self.resizing_active = true;
        }
        if !pointer_down && self.resizing_active {
            let new_frac = (measured_nav_w / screen_w).clamp(0.10, 0.50);
            self.pending_nav_frac = Some(new_frac);
            self.settings_form.set_current_navbar_width_frac(new_frac);
            self.nav_save_deadline = Some(Instant::now() + Duration::from_millis(250));
            self.resizing_active = false;
        }
        self.last_nav_w = measured_nav_w;

        if let (Some(target_frac), Some(deadline)) = (self.pending_nav_frac, self.nav_save_deadline)
        {
            if Instant::now() >= deadline {
                let theme = self.settings_form.get_theme();
                let zoom = self.settings_form.get_zoom();
                let app_settings = AppSettings {
                    theme,
                    zoom,
                    navbar_width_frac: target_frac,
                };
                if app_settings.save_to_db(&self.db_connection).is_ok() {
                    self.saved_navbar_width_frac = target_frac;
                }
                self.pending_nav_frac = None;
                self.nav_save_deadline = None;
            }
        }

        // 7. CentralPanel (main content)
        CentralPanel::show(
            ctx,
            &mut self.dock_content,
            &self.db_connection,
            &mut self.projects_state,
            &mut self.snapshots_state,
            &mut self.snapshot_files_state,
            &mut self.scan_snapshot_state,
        );
    }
}
