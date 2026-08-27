use egui::{DragValue, Ui};

use crate::ui::{
    app::AppState,
    gui::{
        FeatureSettingsPopup,
        helpers::{
            checkbox, collapsing_open, color_picker, combo_box, drag, scroll, text_settings_button,
        },
    },
};

impl AppState {
    pub fn hud_settings(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            scroll(&mut cols[0], "hud_main_left", |ui| {
                collapsing_open(ui, "HUD", |ui| {
                    if checkbox(ui, "Watermark", &mut self.config.hud.watermark) {
                        self.send_config();
                    }
                    self.hud_text_toggle(ui, "Bomb Timer", "bomb_timer", 0);
                    if checkbox(ui, "FOV Circle", &mut self.config.hud.fov_circle) {
                        self.send_config();
                    }
                    self.hud_text_toggle(ui, "Dropped Weapons", "weapon_name", 1);
                    self.hud_text_toggle(ui, "Keybind List", "keybind_list", 2);
                    self.hud_text_toggle(ui, "Spectator List", "spectator_list", 3);
                    self.hud_text_toggle(ui, "Status Indicators", "status_text", 4);
                    if ui.button("⚙ Settings").clicked() {
                        self.feature_settings_popup = Some(FeatureSettingsPopup::Hud);
                    }
                });
            });
            scroll(&mut cols[1], "hud_main_right", |ui| {
                collapsing_open(ui, "Sniper Crosshair", |ui| {
                    if checkbox(ui, "Enabled", &mut self.config.hud.sniper_crosshair.enabled) {
                        self.send_config();
                    }
                    if ui.button("⚙ Settings").clicked() {
                        self.feature_settings_popup = Some(FeatureSettingsPopup::SniperCrosshair);
                    }
                });
                collapsing_open(ui, "Grenade Trails", |ui| {
                    if checkbox(ui, "Enabled", &mut self.config.hud.grenade_trails.enabled) {
                        self.send_config();
                    }
                    if ui.button("⚙ Settings").clicked() {
                        self.feature_settings_popup = Some(FeatureSettingsPopup::GrenadeTrails);
                    }
                });
            });
        });
        self.render_hud_popup(ui);
    }

    fn hud_text_toggle(&mut self, ui: &mut Ui, label: &str, popup: &str, field: u8) {
        ui.horizontal(|ui| {
            let changed = match field {
                0 => checkbox(ui, label, &mut self.config.hud.bomb_timer),
                1 => checkbox(ui, label, &mut self.config.hud.dropped_weapons),
                2 => checkbox(ui, label, &mut self.config.hud.keybind_list),
                3 => checkbox(ui, label, &mut self.config.hud.spectator_list),
                _ => checkbox(ui, label, &mut self.config.hud.status_indicators),
            };
            if changed {
                self.send_config();
            }
            text_settings_button(ui, &mut self.text_popup, popup);
        });
    }

    fn render_hud_popup(&mut self, ui: &mut Ui) {
        let Some(
            popup @ (FeatureSettingsPopup::Hud
            | FeatureSettingsPopup::SniperCrosshair
            | FeatureSettingsPopup::GrenadeTrails),
        ) = self.feature_settings_popup
        else {
            return;
        };
        let title = match popup {
            FeatureSettingsPopup::Hud => "HUD Settings",
            FeatureSettingsPopup::SniperCrosshair => "Sniper Crosshair Settings",
            FeatureSettingsPopup::GrenadeTrails => "Grenade Trail Settings",
            _ => unreachable!(),
        };
        let mut open = true;
        egui::Window::new(title)
            .id(egui::Id::new("hud_feature_settings"))
            .collapsible(false)
            .resizable(true)
            .default_width(320.0)
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                scroll(ui, "hud_popup_scroll", |ui| match popup {
                    FeatureSettingsPopup::Hud => self.hud_details(ui),
                    FeatureSettingsPopup::SniperCrosshair => self.crosshair_details(ui),
                    FeatureSettingsPopup::GrenadeTrails => self.trail_details(ui),
                    _ => unreachable!(),
                });
            });
        if !open {
            self.feature_settings_popup = None;
        }
    }

    fn hud_details(&mut self, ui: &mut Ui) {
        ui.heading("Appearance");
        if checkbox(ui, "Text Outline", &mut self.config.hud.text_outline) {
            self.send_config();
        }
        if drag(
            ui,
            "Line Width",
            DragValue::new(&mut self.config.hud.line_width)
                .range(0.1..=8.0)
                .speed(0.02)
                .max_decimals(1),
        ) {
            self.send_config();
        }
        if combo_box(ui, "font", "Font", &mut self.config.font) {
            self.config.font.set(ui.ctx());
            if let Some(ctx) = &self.overlay_egui {
                self.config.font.set(ctx);
            }
            self.send_config();
        }
        for (label, popup) in [
            ("Status Text", "status_text"),
            ("Grenade Name", "grenade_name"),
            ("Grenade Lineup", "grenade_lineup"),
        ] {
            ui.horizontal(|ui| {
                ui.label(label);
                text_settings_button(ui, &mut self.text_popup, popup);
            });
        }
        ui.separator();
        ui.heading("Advanced");
        if checkbox(ui, "Debug Overlay", &mut self.config.hud.debug) {
            self.send_config();
        }
        if drag(
            ui,
            "FPS",
            DragValue::new(&mut self.config.fps).range(30..=500),
        ) {
            self.send_config();
        }
    }

    fn crosshair_details(&mut self, ui: &mut Ui) {
        if drag(
            ui,
            "Line Length",
            DragValue::new(&mut self.config.hud.sniper_crosshair.line_length)
                .range(0.1..=500.0)
                .max_decimals(1)
                .speed(0.2),
        ) {
            self.send_config();
        }
        if drag(
            ui,
            "Line Width",
            DragValue::new(&mut self.config.hud.sniper_crosshair.line_width)
                .range(0.1..=10.0)
                .max_decimals(1)
                .speed(0.005),
        ) {
            self.send_config();
        }
        if drag(
            ui,
            "Gap",
            DragValue::new(&mut self.config.hud.sniper_crosshair.gap)
                .range(0.0..=200.0)
                .max_decimals(1)
                .speed(0.2),
        ) {
            self.send_config();
        }
        if color_picker(ui, "Color", &mut self.config.hud.sniper_crosshair.color) {
            self.send_config();
        }
    }

    fn trail_details(&mut self, ui: &mut Ui) {
        if checkbox(
            ui,
            "Inferno Polygon",
            &mut self.config.hud.grenade_trails.inferno_poly,
        ) {
            self.send_config();
        }
        if color_picker(ui, "Smoke", &mut self.config.hud.grenade_trails.smoke) {
            self.send_config();
        }
        if color_picker(ui, "Molotov", &mut self.config.hud.grenade_trails.molotov) {
            self.send_config();
        }
        if color_picker(
            ui,
            "Incendiary",
            &mut self.config.hud.grenade_trails.incendiary,
        ) {
            self.send_config();
        }
        if color_picker(ui, "Flash", &mut self.config.hud.grenade_trails.flash) {
            self.send_config();
        }
        if color_picker(ui, "HE Grenade", &mut self.config.hud.grenade_trails.he) {
            self.send_config();
        }
        if color_picker(ui, "Decoy", &mut self.config.hud.grenade_trails.decoy) {
            self.send_config();
        }
    }
}
