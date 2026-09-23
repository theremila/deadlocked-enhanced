use egui::{DragValue, Ui};

use crate::{
    config::bind::{MiscSetting, SettingId},
    ui::{
        app::AppState,
        gui::{
            FeatureSettingsPopup,
            helpers::{collapsing_open, color_picker, combo_box, scroll},
        },
    },
};

impl AppState {
    pub fn unsafe_settings(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            scroll(&mut cols[0], "unsafe_main_left", |ui| {
                self.misc_card(
                    ui,
                    "Bunnyhop",
                    "Enable Bunnyhop",
                    FeatureSettingsPopup::Bunnyhop,
                    0,
                );
                self.misc_card(
                    ui,
                    "No Flash",
                    "Enable No Flash",
                    FeatureSettingsPopup::NoFlash,
                    1,
                );
            });
            scroll(&mut cols[1], "unsafe_main_right", |ui| {
                self.misc_card(
                    ui,
                    "Smokes",
                    "Enable No Smoke",
                    FeatureSettingsPopup::Smokes,
                    2,
                );
                self.misc_card(
                    ui,
                    "FOV Changer",
                    "Enable FOV Changer",
                    FeatureSettingsPopup::FovChanger,
                    3,
                );
            });
        });
        self.render_unsafe_popup(ui);
    }

    fn misc_card(
        &mut self,
        ui: &mut Ui,
        title: &str,
        label: &str,
        popup: FeatureSettingsPopup,
        field: u8,
    ) {
        collapsing_open(ui, title, |ui| {
            let setting = match field {
                0 => MiscSetting::Bunnyhop,
                1 => MiscSetting::NoFlash,
                2 => MiscSetting::NoSmoke,
                _ => MiscSetting::FovChanger,
            };
            let changed = self.bool_setting(ui, label, SettingId::Misc(setting));
            if changed {
                self.send_config();
            }
            if ui.button("⚙ Settings").clicked() {
                self.feature_settings_popup = Some(popup);
            }
        });
    }

    fn render_unsafe_popup(&mut self, ui: &mut Ui) {
        let Some(
            popup @ (FeatureSettingsPopup::Bunnyhop
            | FeatureSettingsPopup::NoFlash
            | FeatureSettingsPopup::Smokes
            | FeatureSettingsPopup::FovChanger),
        ) = self.feature_settings_popup
        else {
            return;
        };
        let title = match popup {
            FeatureSettingsPopup::Bunnyhop => "Bunnyhop Settings",
            FeatureSettingsPopup::NoFlash => "No Flash Settings",
            FeatureSettingsPopup::Smokes => "Smoke Settings",
            FeatureSettingsPopup::FovChanger => "FOV Changer Settings",
            _ => unreachable!(),
        };
        let mut open = true;
        egui::Window::new(title)
            .id(egui::Id::new("unsafe_feature_settings"))
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ui.ctx(), |ui| match popup {
                FeatureSettingsPopup::Bunnyhop => self.bunnyhop_details(ui),
                FeatureSettingsPopup::NoFlash => self.flash_details(ui),
                FeatureSettingsPopup::Smokes => self.smoke_details(ui),
                FeatureSettingsPopup::FovChanger => self.fov_details(ui),
                _ => unreachable!(),
            });
        if !open {
            self.feature_settings_popup = None;
        }
    }

    fn bunnyhop_details(&mut self, ui: &mut Ui) {
        if combo_box(
            ui,
            "bunnyhop_mode",
            "Mode",
            &mut self.config.misc.bunnyhop_mode,
        ) {
            self.send_config();
        }
    }

    fn flash_details(&mut self, ui: &mut Ui) {
        if ui
            .horizontal(|ui| {
                let changed = ui
                    .add(
                        DragValue::new(&mut self.config.misc.max_flash_alpha)
                            .range(0.0..=255.0)
                            .speed(0.5)
                            .max_decimals(0),
                    )
                    .changed();
                ui.label("Max Flash Alpha");
                changed
            })
            .inner
        {
            self.send_config();
        }
    }

    fn smoke_details(&mut self, ui: &mut Ui) {
        if self.bool_setting(
            ui,
            "Change Smoke Color",
            SettingId::Misc(MiscSetting::ChangeSmokeColor),
        ) {
            self.send_config();
        }
        if color_picker(ui, "Smoke Color", &mut self.config.misc.smoke_color) {
            self.send_config();
        }
    }

    fn fov_details(&mut self, ui: &mut Ui) {
        let changed = ui
            .horizontal(|ui| {
                let changed = ui
                    .add(
                        DragValue::new(&mut self.config.misc.desired_fov)
                            .speed(0.1)
                            .range(1..=179),
                    )
                    .changed();
                ui.label("Desired FOV");
                let reset = ui.button("Reset").clicked();
                if reset {
                    self.config.misc.desired_fov = crate::constants::cs2::DEFAULT_FOV;
                }
                changed || reset
            })
            .inner;
        if changed {
            self.send_config();
        }
    }
}
