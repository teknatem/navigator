use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, Style, TabViewer};
use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

mod app_settings;
mod layout;

use app_settings::AppSettings;
use layout::{MenuBar, SettingsForm, Theme};

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
    dock_state: DockState<String>,
    // Состояние для интерактивных элементов
    checkbox_state: bool,
    text_input: String,
    // База данных
    db_connection: Connection,
    db_items: Vec<(i32, String)>,
    new_item_name: String,
    db_status: String,
    // Меню и настройки
    menu_bar: MenuBar,
    settings_form: SettingsForm,
    first_frame: bool,
}

impl MyApp {
    fn new() -> Self {
        let mut dock_state = DockState::new(vec!["Tab 1".to_string()]);

        // Добавляем дополнительные вкладки
        let [_a, b] = dock_state.main_surface_mut().split_right(
            NodeIndex::root(),
            0.3,
            vec!["Tab 2".to_string()],
        );

        let [_b, c] = dock_state
            .main_surface_mut()
            .split_below(b, 0.5, vec!["Tab 3".to_string()]);

        dock_state
            .main_surface_mut()
            .split_below(c, 0.5, vec!["Database".to_string()]);

        // Инициализация базы данных
        let db_path = "navigator.db";
        let db_exists = Path::new(db_path).exists();

        if db_exists {
            println!("Database found: {}", db_path);
        } else {
            println!("Database not found. Creating new database: {}", db_path);
        }

        let db_connection = Connection::open(db_path).expect("Failed to open/create database");

        if !db_exists {
            println!("Initializing database tables...");
        }

        Self::init_database(&db_connection).expect("Failed to initialize database");

        if !db_exists {
            println!("Database created successfully with all tables");
        }

        // Загрузка настроек из базы данных
        let saved_settings = AppSettings::load_from_db(&db_connection).unwrap_or_else(|_| {
            println!("No saved settings found, using defaults");
            AppSettings::default()
        });

        let mut app = Self {
            dock_state,
            checkbox_state: false,
            text_input: String::from("Введите текст..."),
            db_connection,
            db_items: Vec::new(),
            new_item_name: String::new(),
            db_status: String::from("База данных готова"),
            menu_bar: MenuBar::new(),
            settings_form: SettingsForm::new_with_settings(&saved_settings),
            first_frame: true,
        };

        app.load_items();
        app
    }

    fn init_database(conn: &Connection) -> SqlResult<()> {
        // Create items table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS items (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL
            )",
            [],
        )?;
        println!("  ✓ Table 'items' initialized");

        // Create settings table
        AppSettings::init_table(conn)?;
        println!("  ✓ Table 'settings' initialized");

        Ok(())
    }

    fn load_items(&mut self) {
        self.db_items.clear();
        let mut stmt = self
            .db_connection
            .prepare("SELECT id, name FROM items ORDER BY id DESC")
            .expect("Не удалось подготовить запрос");

        let items_iter = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("Не удалось выполнить запрос");

        for item in items_iter {
            if let Ok(item) = item {
                self.db_items.push(item);
            }
        }
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // При первом кадре применяем сохраненные настройки
        if self.first_frame {
            self.apply_initial_settings(ctx);
            self.first_frame = false;
        }

        // Обработка горячих клавиш для масштаба
        self.handle_zoom_shortcuts(ctx);

        // Отображаем меню
        self.menu_bar.show(ctx);

        // Обработка действий из меню
        self.handle_menu_actions(ctx, frame);

        // Отображаем форму настроек и применяем изменения
        if self.settings_form.show(ctx) {
            self.apply_and_save_settings(ctx);
        }

        let MyApp {
            dock_state,
            checkbox_state,
            text_input,
            db_connection,
            db_items,
            new_item_name,
            db_status,
            ..
        } = self;

        DockArea::new(dock_state)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(
                ctx,
                &mut MyTabViewer {
                    checkbox_state,
                    text_input,
                    db_connection,
                    db_items,
                    new_item_name,
                    db_status,
                },
            );
    }
}

impl MyApp {
    fn handle_zoom_shortcuts(&mut self, ctx: &egui::Context) {
        // Ctrl + Plus (увеличить масштаб)
        if ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::CTRL, egui::Key::Plus)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::Equals)
        }) {
            let current_zoom = ctx.zoom_factor();
            let new_zoom = (current_zoom + 0.1).min(3.0);
            ctx.set_zoom_factor(new_zoom);
            self.db_status = format!("Масштаб: {:.0}%", new_zoom * 100.0);
        }

        // Ctrl + Minus (уменьшить масштаб)
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Minus)) {
            let current_zoom = ctx.zoom_factor();
            let new_zoom = (current_zoom - 0.1).max(0.5);
            ctx.set_zoom_factor(new_zoom);
            self.db_status = format!("Масштаб: {:.0}%", new_zoom * 100.0);
        }

        // Ctrl + 0 (сбросить масштаб)
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::Num0)) {
            ctx.set_zoom_factor(1.0);
            self.db_status = "Масштаб: 100%".to_string();
        }
    }

    fn handle_menu_actions(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        use layout::menu_bar::{EditAction, FileAction, HelpAction, SettingsAction, ViewAction};

        // Обработка File menu
        if let Some(action) = self.menu_bar.file_action {
            match action {
                FileAction::New => {
                    println!("Создание нового файла...");
                    self.db_status = "Действие: Новый файл".to_string();
                }
                FileAction::Open => {
                    println!("Открытие файла...");
                    self.db_status = "Действие: Открыть файл".to_string();
                }
                FileAction::Save => {
                    println!("Сохранение файла...");
                    self.db_status = "Действие: Сохранить".to_string();
                }
                FileAction::SaveAs => {
                    println!("Сохранение файла как...");
                    self.db_status = "Действие: Сохранить как".to_string();
                }
                FileAction::Exit => {
                    println!("Выход из приложения...");
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }

        // Обработка Edit menu
        if let Some(action) = self.menu_bar.edit_action {
            match action {
                EditAction::Undo => {
                    println!("Отмена действия...");
                    self.db_status = "Действие: Отменить".to_string();
                }
                EditAction::Redo => {
                    println!("Повтор действия...");
                    self.db_status = "Действие: Повторить".to_string();
                }
                EditAction::Cut => {
                    println!("Вырезать...");
                    self.db_status = "Действие: Вырезать".to_string();
                }
                EditAction::Copy => {
                    println!("Копировать...");
                    self.db_status = "Действие: Копировать".to_string();
                }
                EditAction::Paste => {
                    println!("Вставить...");
                    self.db_status = "Действие: Вставить".to_string();
                }
            }
        }

        // Обработка View menu
        if let Some(action) = self.menu_bar.view_action {
            match action {
                ViewAction::ToggleSidebar => {
                    println!("Переключение боковой панели...");
                    self.db_status = "Действие: Боковая панель".to_string();
                }
                ViewAction::TogglePanel => {
                    println!("Переключение нижней панели...");
                    self.db_status = "Действие: Нижняя панель".to_string();
                }
                ViewAction::ToggleFullscreen => {
                    println!("Переключение полноэкранного режима...");
                    ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(true));
                    self.db_status = "Действие: Полный экран".to_string();
                }
            }
        }

        // Обработка Settings menu
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
                    println!("Увеличение масштаба: {:.0}%", new_zoom * 100.0);
                    self.db_status = format!("Масштаб: {:.0}%", new_zoom * 100.0);
                }
                SettingsAction::ZoomOut => {
                    let new_zoom = (current_zoom - 0.1).max(0.5);
                    ctx.set_zoom_factor(new_zoom);
                    println!("Уменьшение масштаба: {:.0}%", new_zoom * 100.0);
                    self.db_status = format!("Масштаб: {:.0}%", new_zoom * 100.0);
                }
                SettingsAction::ZoomReset => {
                    ctx.set_zoom_factor(1.0);
                    println!("Масштаб сброшен: 100%");
                    self.db_status = "Масштаб: 100%".to_string();
                }
            }
        }

        // Обработка Help menu
        if let Some(action) = self.menu_bar.help_action {
            match action {
                HelpAction::Documentation => {
                    println!("Открытие документации...");
                    self.db_status = "Действие: Документация".to_string();
                }
                HelpAction::About => {
                    println!("О программе...");
                    self.db_status =
                        "Navigator v0.1.0 - Rust egui приложение с egui_dock и SQLite".to_string();
                }
            }
        }

        // Очистка действий после обработки
        self.menu_bar.clear_actions();
    }

    fn apply_initial_settings(&mut self, ctx: &egui::Context) {
        let theme = self.settings_form.get_theme();
        let zoom = self.settings_form.get_zoom();

        // Apply theme
        match theme {
            Theme::Light => ctx.set_visuals(egui::Visuals::light()),
            Theme::Dark => ctx.set_visuals(egui::Visuals::dark()),
        }

        // Apply zoom
        ctx.set_zoom_factor(zoom);

        println!(
            "Initial settings applied: Theme={:?}, Zoom={:.0}%",
            theme,
            zoom * 100.0
        );
        self.db_status = format!(
            "Settings loaded: Theme={:?}, Zoom={:.0}%",
            theme,
            zoom * 100.0
        );
    }

    fn apply_and_save_settings(&mut self, ctx: &egui::Context) {
        let theme = self.settings_form.get_theme();
        let zoom = self.settings_form.get_zoom();

        // Apply theme
        match theme {
            Theme::Light => {
                ctx.set_visuals(egui::Visuals::light());
                println!("Theme changed to Light");
            }
            Theme::Dark => {
                ctx.set_visuals(egui::Visuals::dark());
                println!("Theme changed to Dark");
            }
        }

        // Apply zoom
        ctx.set_zoom_factor(zoom);
        println!("Zoom set to: {:.0}%", zoom * 100.0);

        // Save to database
        let app_settings = AppSettings { theme, zoom };

        match app_settings.save_to_db(&self.db_connection) {
            Ok(_) => {
                self.db_status = format!(
                    "Settings saved: Theme={:?}, Zoom={:.0}%",
                    theme,
                    zoom * 100.0
                );
            }
            Err(e) => {
                self.db_status = format!("Error saving settings: {}", e);
                eprintln!("Failed to save settings: {}", e);
            }
        }
    }
}

struct MyTabViewer<'a> {
    checkbox_state: &'a mut bool,
    text_input: &'a mut String,
    db_connection: &'a Connection,
    db_items: &'a mut Vec<(i32, String)>,
    new_item_name: &'a mut String,
    db_status: &'a mut String,
}

impl<'a> TabViewer for MyTabViewer<'a> {
    type Tab = String;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.as_str().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        ui.heading(format!("Содержимое: {}", tab));
        ui.separator();

        match tab.as_str() {
            "Tab 1" => {
                ui.label("Это первая вкладка");
                ui.add_space(10.0);
                if ui.button("Кнопка 1").clicked() {
                    println!("Нажата кнопка на Tab 1");
                }
            }
            "Tab 2" => {
                ui.label("Это вторая вкладка");
                ui.add_space(10.0);
                if ui.checkbox(self.checkbox_state, "Чекбокс").changed() {
                    println!("Чекбокс изменён на: {}", self.checkbox_state);
                }
                ui.label(format!(
                    "Состояние: {}",
                    if *self.checkbox_state {
                        "✓ Включен"
                    } else {
                        "✗ Выключен"
                    }
                ));
            }
            "Tab 3" => {
                ui.label("Это третья вкладка");
                ui.add_space(10.0);
                ui.text_edit_singleline(self.text_input);
                ui.add_space(5.0);
                ui.label(format!("Введённый текст: {}", self.text_input));
            }
            "Database" => {
                ui.label("Работа с базой данных SQLite");
                ui.separator();

                ui.horizontal(|ui| {
                    ui.label("Новая запись:");
                    ui.text_edit_singleline(self.new_item_name);
                    if ui.button("➕ Добавить").clicked() {
                        self.add_database_item();
                    }
                });

                ui.add_space(5.0);
                ui.label(&*self.db_status);
                ui.separator();

                ui.label(format!("Записей в базе: {}", self.db_items.len()));
                ui.add_space(5.0);

                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        let mut delete_id = None;
                        for (id, name) in self.db_items.iter() {
                            ui.horizontal(|ui| {
                                ui.label(format!("ID: {}", id));
                                ui.label(format!("Имя: {}", name));
                                if ui.button("🗑 Удалить").clicked() {
                                    delete_id = Some(*id);
                                }
                            });
                            ui.separator();
                        }
                        if let Some(id) = delete_id {
                            self.delete_database_item(id);
                        }
                    });

                ui.add_space(10.0);
                if ui.button("🔄 Обновить список").clicked() {
                    self.reload_database_items();
                }
            }
            _ => {
                ui.label("Неизвестная вкладка");
            }
        }
    }
}

impl<'a> MyTabViewer<'a> {
    fn add_database_item(&mut self) {
        let name = self.new_item_name.trim();
        if name.is_empty() {
            *self.db_status = "Ошибка: имя не может быть пустым".to_string();
            return;
        }

        match self
            .db_connection
            .execute("INSERT INTO items (name) VALUES (?1)", [name])
        {
            Ok(_) => {
                *self.db_status = format!("Добавлено: {}", name);
                self.reload_database_items();
                self.new_item_name.clear();
            }
            Err(e) => {
                *self.db_status = format!("Ошибка: {}", e);
            }
        }
    }

    fn delete_database_item(&mut self, id: i32) {
        match self
            .db_connection
            .execute("DELETE FROM items WHERE id = ?1", [id])
        {
            Ok(_) => {
                *self.db_status = format!("Удалено: ID {}", id);
                self.reload_database_items();
            }
            Err(e) => {
                *self.db_status = format!("Ошибка удаления: {}", e);
            }
        }
    }

    fn reload_database_items(&mut self) {
        self.db_items.clear();
        let mut stmt = self
            .db_connection
            .prepare("SELECT id, name FROM items ORDER BY id DESC")
            .expect("Не удалось подготовить запрос");

        let items_iter = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .expect("Не удалось выполнить запрос");

        for item in items_iter {
            if let Ok(item) = item {
                self.db_items.push(item);
            }
        }
        *self.db_status = "Список обновлён".to_string();
    }
}
