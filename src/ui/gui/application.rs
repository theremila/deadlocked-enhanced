use egui::Ui;

use crate::{
    ui::{
        app::AppState,
        color::Colors,
        gui::helpers::{groupbox, open_url, scroll},
    },
    update::UpdateStatus,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

impl AppState {
    pub fn application_settings(&mut self, ui: &mut Ui) {
        scroll(ui, "app_settings", |ui| {
            groupbox(ui, "Application Information", |ui| {
                ui.horizontal(|ui| {
                    let accent = ui.style().visuals.selection.bg_fill;
                    ui.label(
                        egui::RichText::new("DEADLOCKED")
                            .strong()
                            .size(16.0)
                            .color(accent),
                    );
                    ui.label(
                        egui::RichText::new(format!("v{VERSION}"))
                            .size(12.0)
                            .color(Colors::MUTED),
                    );
                });

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new("Advanced Counter-Strike 2 External Assistant")
                        .size(12.0)
                        .color(Colors::SUBTEXT),
                );

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                ui.horizontal(|ui| {
                    ui.label("Status:");
                    match &self.update_status {
                        UpdateStatus::UpToDate => {
                            ui.colored_label(Colors::GREEN, "● Up to date");
                        }
                        UpdateStatus::Available { version, url } => {
                            ui.colored_label(
                                Colors::YELLOW,
                                format!("● Update available: {version}"),
                            );
                            if ui.button("Download Update").clicked() {
                                open_url(url);
                            }
                        }
                        UpdateStatus::Error(err) => {
                            ui.colored_label(Colors::RED, format!("● Update check failed: {err}"));
                        }
                    }
                });

                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    if ui.button("🌐 GitHub Repository").clicked() {
                        open_url("https://github.com/avitran0/deadlocked");
                    }
                    if ui.button("🐛 Report an Issue").clicked() {
                        open_url("https://github.com/avitran0/deadlocked/issues");
                    }
                });
            });
        });
    }
}
