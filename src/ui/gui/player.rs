use egui::{DragValue, Ui};

use crate::{
    config::bind::{PlayerSetting, SettingId},
    ui::{
        app::AppState,
        gui::{
            FeatureSettingsPopup,
            helpers::{
                collapsing_open, color_picker, combo_box, drag, scroll, text_settings_button,
            },
        },
    },
};

impl AppState {
    pub fn player_settings(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            scroll(&mut cols[0], "player_main_left", |ui| {
                collapsing_open(ui, "Players", |ui| {
                    if self.bool_setting(
                        ui,
                        "Enable Player ESP",
                        SettingId::Player(PlayerSetting::Enabled),
                    ) {
                        self.send_config();
                    }
                    if ui.button("⚙ Settings").clicked() {
                        self.feature_settings_popup = Some(FeatureSettingsPopup::Player);
                    }
                });
                collapsing_open(ui, "Out Of Field Arrows", |ui| {
                    if self.bool_setting(
                        ui,
                        "Enable OOF Arrows",
                        SettingId::Player(PlayerSetting::OofArrows),
                    ) {
                        self.send_config();
                    }
                    if ui.button("⚙ Settings").clicked() {
                        self.feature_settings_popup = Some(FeatureSettingsPopup::OofArrows);
                    }
                });
            });
            scroll(&mut cols[1], "player_main_right", |ui| {
                collapsing_open(ui, "Sound ESP", |ui| {
                    if self.bool_setting_hover(
                        ui,
                        "Enable Sound ESP",
                        Some("Show a circle under players when they make sound"),
                        SettingId::Player(PlayerSetting::SoundEsp),
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
        if self.bool_setting(ui, "Chicken", SettingId::Player(PlayerSetting::Chicken)) {
            self.send_config();
        }
        if self.bool_setting(
            ui,
            "Show Friendlies",
            SettingId::Player(PlayerSetting::ShowFriendlies),
        ) {
            self.send_config();
        }
        if self.bool_setting_hover(
            ui,
            "Visible Only",
            Some("Only show visible players"),
            SettingId::Player(PlayerSetting::VisibleOnly),
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
        if self.bool_setting(
            ui,
            "Head Circle",
            SettingId::Player(PlayerSetting::HeadCircle),
        ) {
            self.send_config();
        }

        ui.separator();
        ui.heading("Info");
        if self.bool_setting(
            ui,
            "Health Bar",
            SettingId::Player(PlayerSetting::HealthBar),
        ) {
            self.send_config();
        }
        if self.bool_setting(ui, "Armor Bar", SettingId::Player(PlayerSetting::ArmorBar)) {
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
            let setting = match field {
                0 => PlayerSetting::PlayerName,
                1 => PlayerSetting::WeaponIcon,
                _ => PlayerSetting::Tags,
            };
            let changed = self.bool_setting(ui, label, SettingId::Player(setting));
            if changed {
                self.send_config();
            }
            text_settings_button(ui, &mut self.text_popup, popup);
        });
    }

    fn oof_details(&mut self, ui: &mut Ui) {
        if self.bool_setting_hover(
            ui,
            "Offscreen Only",
            Some("Only show arrows for players outside the screen FOV"),
            SettingId::Player(PlayerSetting::OofOffscreenOnly),
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
        if self.bool_setting(
            ui,
            "Show Visible",
            SettingId::Player(PlayerSetting::SoundShowVisible),
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
