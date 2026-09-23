use egui::Ui;

use crate::ui::app::AppState;

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl AppState {
    pub fn application_settings(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.heading("deadlocked");
            ui.label("authors: avitrano, theremila");
            ui.label(format!("Version: v{VERSION}"));
        });
    }
}

