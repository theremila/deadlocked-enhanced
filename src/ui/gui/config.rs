use egui::{Align, Button, Color32, Ui};

use crate::{
    config::{
        BASE_PATH, CONFIG_PATH, Config, available_configs, delete_config, parse_config,
        write_config,
    },
    ui::{
        app::AppState,
        color::Colors,
        grenades::read_grenades,
        gui::helpers::{groupbox, open_url, scroll},
    },
};

impl AppState {
    pub fn config_settings(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            scroll(left, "config_left", |left| self.config_left(left));

            let right = &mut cols[1];
            scroll(right, "config_right", |right| {
                groupbox(right, "Saved Configurations", |right| {
                    right.horizontal(|right| {
                        if right.button("🔄 Refresh").clicked() {
                            self.available_configs = available_configs();
                            self.grenades = read_grenades();
                        }
                    });

                    right.add_space(4.0);
                    right.horizontal(|right| {
                        if right.button("➕ Create").clicked() && !self.new_config_name.is_empty() {
                            if !self.new_config_name.ends_with(".toml") {
                                self.new_config_name.push_str(".toml");
                            }
                            let path = CONFIG_PATH.join(&self.new_config_name);
                            write_config(&self.config, &path);
                            self.new_config_name.clear();
                            self.current_config = path;
                            self.available_configs = available_configs();
                        }
                        right.text_edit_singleline(&mut self.new_config_name);
                    });

                    right.add_space(6.0);
                    right.separator();
                    right.add_space(6.0);

                    self.config_right(right);
                });
            });
        });
    }

    fn config_left(&mut self, ui: &mut Ui) {
        groupbox(ui, "Config Actions", |ui| {
            if ui
                .add(
                    Button::new("↺ Reset to Defaults")
                        .min_size(egui::vec2(ui.available_width(), 24.0)),
                )
                .clicked()
            {
                self.config = Config::default();
                self.send_config();
                utils::info!("loaded default config");
            }

            ui.add_space(4.0);
            if ui
                .add(
                    Button::new("📂 Open Config Folder")
                        .min_size(egui::vec2(ui.available_width(), 24.0)),
                )
                .clicked()
            {
                let url = format!("file://{}", BASE_PATH.display());
                open_url(&url);
            }
        });

        groupbox(ui, "Theme Accent Color", |ui| {
            egui::ComboBox::new("accent_color", "Accent Color")
                .selected_text(
                    Colors::ACCENT_COLORS
                        .iter()
                        .find(|c| c.1 == self.config.accent_color)
                        .unwrap_or(&Colors::ACCENT_COLORS[5])
                        .0,
                )
                .show_ui(ui, |ui| {
                    for (name, color) in Colors::ACCENT_COLORS {
                        if ui
                            .add(
                                Button::selectable(color == self.config.accent_color, name)
                                    .fill(color),
                            )
                            .clicked()
                        {
                            self.config.accent_color = color;
                            ui.ctx()
                                .global_style_mut(|style| style.visuals.selection.bg_fill = color);
                            self.send_config();
                        }
                    }
                });
        });
    }

    fn config_right(&mut self, ui: &mut Ui) {
        let mut clicked_config = None;
        let mut delete = None;

        for config in &self.available_configs {
            ui.horizontal(|ui| {
                let name = config.file_name().unwrap().to_str().unwrap();
                let is_current = *config == self.current_config;
                let accent = ui.style().visuals.selection.bg_fill;

                let btn = if is_current {
                    Button::new(egui::RichText::new(format!("● {name}")).color(Colors::TEXT))
                        .fill(Color32::from_rgba_unmultiplied(
                            accent.r(),
                            accent.g(),
                            accent.b(),
                            50,
                        ))
                        .stroke(egui::Stroke::new(1.0, accent))
                } else {
                    Button::new(name).fill(Colors::HIGHLIGHT)
                };

                if ui.add(btn).clicked() {
                    clicked_config = Some(config.clone());
                }

                ui.with_layout(egui::Layout::right_to_left(Align::Center), |ui| {
                    if ui
                        .add(
                            Button::new(
                                egui::RichText::new("🗑 Delete").color(Colors::RED).size(11.0),
                            )
                            .fill(Colors::HIGHLIGHT),
                        )
                        .clicked()
                    {
                        delete = Some(config.clone());
                    }
                });
            });
            ui.add_space(2.0);
        }

        if let Some(config_path) = clicked_config {
            self.config = parse_config(&config_path);
            self.current_config = config_path;
            self.send_config();
            ui.ctx().global_style_mut(|style| {
                style.visuals.selection.bg_fill = self.config.accent_color
            });
        }

        if let Some(config) = delete {
            delete_config(&config);
            self.available_configs = available_configs();
            if !self.available_configs.is_empty() {
                self.current_config = self.available_configs[0].clone();
                self.config = parse_config(&self.current_config);
            }
        }
    }
}
