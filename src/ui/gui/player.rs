use egui::{DragValue, Ui};

use crate::ui::{
    app::AppState,
    gui::{
        FeatureSettingsPopup,
        helpers::{
            checkbox, checkbox_hover, collapsing_open, color_picker, combo_box, drag, keybind,
            scroll, text_settings_button,
        },
    },
};

impl AppState {
    pub fn player_settings(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            scroll(&mut cols[0], "player_main_left", |ui| {
                collapsing_open(ui, "Players", |ui| {
                    if checkbox(ui, "Enable Player ESP", &mut self.config.player.enabled) {
                        self.send_config();
                    }
                    if keybind(
                        ui,
                        "esp_hotkey",
                        "Hotkey",
                        &mut self.config.player.esp_hotkey,
                    ) {
                        self.send_config();
                    }
                    if ui.button("⚙ Settings").clicked() {
                        self.feature_settings_popup = Some(FeatureSettingsPopup::Player);
                    }
                });
                collapsing_open(ui, "Out Of Field Arrows", |ui| {
                    if checkbox(ui, "Enable OOF Arrows", &mut self.config.player.oof_arrows) {
                        self.send_config();
                    }
                    if ui.button("⚙ Settings").clicked() {
                        self.feature_settings_popup = Some(FeatureSettingsPopup::OofArrows);
                    }
                });
            });
            scroll(&mut cols[1], "player_main_right", |ui| {
                collapsing_open(ui, "Sound ESP", |ui| {
                    if checkbox_hover(
                        ui,
                        "Enable Sound ESP",
                        "Show a circle under players when they make sound",
                        &mut self.config.player.sound.enabled,
                    ) {
                        self.send_config();
                    }
                    if ui.button("⚙ Settings").clicked() {
                        self.feature_settings_popup = Some(FeatureSettingsPopup::SoundEsp);
                    }
                });
            });
        });
        self.render_player_popup(ui);
    }

    fn render_player_popup(&mut self, ui: &mut Ui) {
        let Some(
            popup @ (FeatureSettingsPopup::Player
            | FeatureSettingsPopup::OofArrows
            | FeatureSettingsPopup::SoundEsp),
        ) = self.feature_settings_popup
        else {
            return;
        };
        let title = match popup {
            FeatureSettingsPopup::Player => "Player ESP Settings",
            FeatureSettingsPopup::OofArrows => "OOF Arrows Settings",
            FeatureSettingsPopup::SoundEsp => "Sound ESP Settings",
            _ => unreachable!(),
        };
        let mut open = true;
        egui::Window::new(title)
            .id(egui::Id::new("player_feature_settings"))
            .collapsible(false)
            .resizable(true)
            .default_width(320.0)
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                scroll(ui, "player_popup_scroll", |ui| match popup {
                    FeatureSettingsPopup::Player => self.player_details(ui),
                    FeatureSettingsPopup::OofArrows => self.oof_details(ui),
                    FeatureSettingsPopup::SoundEsp => self.sound_details(ui),
                    _ => unreachable!(),
                });
            });
        if !open {
            self.feature_settings_popup = None;
        }
    }

    fn player_details(&mut self, ui: &mut Ui) {
        ui.heading("Rendering");
        if checkbox(ui, "Chicken", &mut self.config.player.chicken) {
            self.send_config();
        }
        if checkbox(
            ui,
            "Show Friendlies",
            &mut self.config.player.show_friendlies,
        ) {
            self.send_config();
        }
        if checkbox_hover(
            ui,
            "Visible Only",
            "Only show visible players",
            &mut self.config.player.visible_only,
        ) {
            self.send_config();
        }
        if combo_box(ui, "draw_box", "Box", &mut self.config.player.draw_box) {
            self.send_config();
        }
        if combo_box(ui, "box_mode", "Box Mode", &mut self.config.player.box_mode) {
            self.send_config();
        }
        if combo_box(
            ui,
            "draw_skeleton",
            "Skeleton",
            &mut self.config.player.draw_skeleton,
        ) {
            self.send_config();
        }
        if checkbox(ui, "Head Circle", &mut self.config.player.head_circle) {
            self.send_config();
        }

        ui.separator();
        ui.heading("Info");
        if checkbox(ui, "Health Bar", &mut self.config.player.health_bar) {
            self.send_config();
        }
        if checkbox(ui, "Armor Bar", &mut self.config.player.armor_bar) {
            self.send_config();
        }
        self.player_text_toggle(ui, "Player Name", "player_name", 0);
        self.player_text_toggle(ui, "Weapon Icon", "weapon_icon", 1);
        ui.horizontal(|ui| {
            ui.label("Ammo");
            text_settings_button(ui, &mut self.text_popup, "ammo_text");
        });
        self.player_text_toggle(ui, "Show Tags", "player_tags", 2);

        ui.separator();
        ui.heading("Colors");
        if color_picker(
            ui,
            "Box (visible)",
            &mut self.config.player.box_visible_color,
        ) {
            self.send_config();
        }
        if color_picker(
            ui,
            "Box (invisible)",
            &mut self.config.player.box_invisible_color,
        ) {
            self.send_config();
        }
        if color_picker(ui, "Skeleton", &mut self.config.player.skeleton_color) {
            self.send_config();
        }
    }

    fn player_text_toggle(&mut self, ui: &mut Ui, label: &str, popup: &str, field: u8) {
        ui.horizontal(|ui| {
            let changed = match field {
                0 => checkbox(ui, label, &mut self.config.player.player_name),
                1 => checkbox(ui, label, &mut self.config.player.weapon_icon),
                _ => checkbox(ui, label, &mut self.config.player.tags),
            };
            if changed {
                self.send_config();
            }
            text_settings_button(ui, &mut self.text_popup, popup);
        });
    }

    fn oof_details(&mut self, ui: &mut Ui) {
        if checkbox_hover(
            ui,
            "Offscreen Only",
            "Only show arrows for players outside the screen FOV",
            &mut self.config.player.oof_offscreen_only,
        ) {
            self.send_config();
        }
        if drag(
            ui,
            "Radius",
            DragValue::new(&mut self.config.player.oof_radius)
                .range(50.0..=500.0)
                .speed(1.0)
                .max_decimals(0),
        ) {
            self.send_config();
        }
        if drag(
            ui,
            "Size",
            DragValue::new(&mut self.config.player.oof_size)
                .range(6.0..=30.0)
                .speed(0.5)
                .max_decimals(1),
        ) {
            self.send_config();
        }
        if color_picker(ui, "Color", &mut self.config.player.oof_color) {
            self.send_config();
        }
    }

    fn sound_details(&mut self, ui: &mut Ui) {
        if drag(
            ui,
            "Fadeout Time",
            DragValue::new(&mut self.config.player.sound.fadeout_duration)
                .range(0.0..=10.0)
                .suffix(" s")
                .speed(0.01),
        ) {
            self.send_config();
        }
        if checkbox(
            ui,
            "Show Visible",
            &mut self.config.player.sound.show_visible,
        ) {
            self.send_config();
        }
        ui.separator();
        ui.heading("Ranges");
        self.sound_range(ui, "Footstep", 0, 200.0..=6000.0);
        self.sound_range(ui, "Gunshot", 1, 200.0..=10000.0);
        self.sound_range(ui, "Weapon", 2, 200.0..=6000.0);
    }

    fn sound_range(
        &mut self,
        ui: &mut Ui,
        label: &str,
        field: u8,
        range: std::ops::RangeInclusive<f32>,
    ) {
        ui.horizontal(|ui| {
            let value = match field {
                0 => &mut self.config.player.sound.footstep_diameter,
                1 => &mut self.config.player.sound.gunshot_diameter,
                _ => &mut self.config.player.sound.weapon_diameter,
            };
            let changed = ui
                .add(DragValue::new(value).speed(10.0).range(range))
                .changed();
            ui.label(label);
            let reset = ui.button("↺").on_hover_text("Reset").clicked();
            if reset {
                match field {
                    0 => {
                        self.config.player.sound.footstep_diameter =
                            crate::constants::cs2::SOUND_ESP_FOOTSTEP_DIAMETER_DEFAULT
                    }
                    1 => {
                        self.config.player.sound.gunshot_diameter =
                            crate::constants::cs2::SOUND_ESP_GUNSHOT_DIAMETER_DEFAULT
                    }
                    _ => {
                        self.config.player.sound.weapon_diameter =
                            crate::constants::cs2::SOUND_ESP_WEAPON_DIAMETER_DEFAULT
                    }
                }
            }
            if changed || reset {
                self.send_config();
            }
        });
    }
}
