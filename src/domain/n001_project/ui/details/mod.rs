use eframe::egui;
use rfd::FileDialog;

#[derive(Default, Clone)]
pub struct ProjectDetailsState {
    pub name: String,
    pub root_path: String,
    pub description: String,
}

impl ProjectDetailsState {
    pub fn from_values(name: &str, root_path: &str, description: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            root_path: root_path.to_string(),
            description: description.unwrap_or("").to_string(),
        }
    }
}

pub fn details_form(ui: &mut egui::Ui, state: &mut ProjectDetailsState) {
    ui.horizontal(|ui| {
        ui.label("Name:");
        ui.text_edit_singleline(&mut state.name);
    });
    ui.horizontal(|ui| {
        ui.label("Root Path:");
        ui.text_edit_singleline(&mut state.root_path);
        ui.add_space(6.0);
        if ui.button("Browse...").clicked() {
            if let Some(path) = FileDialog::new().set_directory(&state.root_path).pick_folder() {
                if let Some(s) = path.to_str() {
                    state.root_path = s.to_string();
                } else {
                    state.root_path = path.to_string_lossy().to_string();
                }
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label("Description:");
        ui.text_edit_singleline(&mut state.description);
    });
}

