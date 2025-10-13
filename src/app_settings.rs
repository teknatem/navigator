use crate::layout::Theme;
use rusqlite::{Connection, Result as SqlResult};

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub theme: Theme,
    pub zoom: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            zoom: 1.0,
        }
    }
}

impl AppSettings {
    /// Initialize settings table in database
    pub fn init_table(conn: &Connection) -> SqlResult<()> {
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;
        Ok(())
    }

    /// Load settings from database
    pub fn load_from_db(conn: &Connection) -> SqlResult<Self> {
        let mut settings = Self::default();

        // Load theme
        if let Ok(theme_str) = Self::get_value(conn, "theme") {
            settings.theme = match theme_str.as_str() {
                "light" => Theme::Light,
                "dark" => Theme::Dark,
                _ => Theme::Dark,
            };
        }

        // Load zoom
        if let Ok(zoom_str) = Self::get_value(conn, "zoom") {
            if let Ok(zoom) = zoom_str.parse::<f32>() {
                settings.zoom = zoom.clamp(0.5, 3.0);
            }
        }

        println!(
            "Settings loaded from database: Theme={:?}, Zoom={:.0}%",
            settings.theme,
            settings.zoom * 100.0
        );
        Ok(settings)
    }

    /// Save settings to database
    pub fn save_to_db(&self, conn: &Connection) -> SqlResult<()> {
        // Save theme
        let theme_str = match self.theme {
            Theme::Light => "light",
            Theme::Dark => "dark",
        };
        Self::set_value(conn, "theme", theme_str)?;

        // Save zoom
        Self::set_value(conn, "zoom", &self.zoom.to_string())?;

        println!(
            "Settings saved to database: Theme={:?}, Zoom={:.0}%",
            self.theme,
            self.zoom * 100.0
        );
        Ok(())
    }

    /// Get a single setting value
    fn get_value(conn: &Connection, key: &str) -> SqlResult<String> {
        conn.query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
            row.get(0)
        })
    }

    /// Set a single setting value (insert or update)
    fn set_value(conn: &Connection, key: &str, value: &str) -> SqlResult<()> {
        conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            [key, value],
        )?;
        Ok(())
    }

    /// Delete a setting
    #[allow(dead_code)]
    pub fn delete_value(conn: &Connection, key: &str) -> SqlResult<()> {
        conn.execute("DELETE FROM settings WHERE key = ?1", [key])?;
        Ok(())
    }

    /// Get all settings as key-value pairs (useful for debugging)
    #[allow(dead_code)]
    pub fn get_all(conn: &Connection) -> SqlResult<Vec<(String, String)>> {
        let mut stmt = conn.prepare("SELECT key, value FROM settings ORDER BY key")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;

        let mut settings = Vec::new();
        for row in rows {
            settings.push(row?);
        }
        Ok(settings)
    }
}
