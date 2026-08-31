use egui::{DragValue, Ui};

use crate::{
    config::aim::{SeedMode, TriggerTargetingMode},
    config::bind::{AimProfile, AimSetting, RcsSetting, SettingId, TriggerSetting},
    ui::{
        app::AppState,
        drag_range::DragRange,
        gui::helpers::{bone_selector, collapsing_open, combo_box, drag, drag_hover, scroll},
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AimbotTab {
    Global,
    Weapon,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AimSettingsPopup {
    Aim,
    Trigger,
}

impl AppState {
    fn aim_profile(&self) -> AimProfile {
        match self.aimbot_tab {
            AimbotTab::Global => AimProfile::Global,
            AimbotTab::Weapon => AimProfile::Weapon(self.aimbot_weapon.clone()),
        }
    }

    fn aim_id(&self, setting: AimSetting) -> SettingId {
        SettingId::Aim(self.aim_profile(), setting)
    }

    fn trigger_id(&self, setting: TriggerSetting) -> SettingId {
        SettingId::Trigger(self.aim_profile(), setting)
    }

    fn rcs_id(&self, setting: RcsSetting) -> SettingId {
        SettingId::Rcs(self.aim_profile(), setting)
    }

    pub fn aimbot_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.aimbot_tab, AimbotTab::Global, "Global");
            ui.selectable_value(&mut self.aimbot_tab, AimbotTab::Weapon, "Weapon");
            if self.aimbot_tab == AimbotTab::Weapon {
                combo_box(ui, "aimbot_weapon", "Weapon", &mut self.aimbot_weapon);
            }
            ui.separator();
            if ui
                .button("Copy")
                .on_hover_text("Copy this Aim, Triggerbot and RCS profile")
                .clicked()
            {
                self.aim_profile_clipboard = Some(self.weapon_config_ref().clone());
            }
            let can_paste = self.aim_profile_clipboard.is_some();
            if ui
                .add_enabled(can_paste, egui::Button::new("Paste"))
                .on_hover_text("Paste the copied profile into the current weapon")
                .clicked()
            {
                let mut profile = self.aim_profile_clipboard.clone().unwrap();
                let override_enabled = self.aimbot_tab == AimbotTab::Weapon;
                profile.aimbot.enable_override = override_enabled;
                profile.triggerbot.enable_override = override_enabled;
                profile.rcs.enable_override = override_enabled;
                *self.weapon_config() = profile;
                self.send_config();
            }
        });
        ui.separator();

        ui.columns(2, |cols| {
            scroll(&mut cols[0], "aim_main", |ui| self.aim_main(ui));
            scroll(&mut cols[1], "trigger_main", |ui| self.trigger_main(ui));
        });
        self.render_aim_settings_popup(ui);
    }

    fn aim_main(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Aim", |ui| {
            if self.aimbot_tab == AimbotTab::Weapon
                && self.bool_setting_hover(
                    ui,
                    "Enable Override",
                    Some("Enable Aim settings override for this weapon"),
                    self.aim_id(AimSetting::Override),
                )
            {
                self.send_config();
            }
            if self.bool_setting(ui, "Enable Aim", self.aim_id(AimSetting::Enabled)) {
                self.send_config();
            }
            if combo_box(
                ui,
                "aim_targeting_mode",
                "Targeting Mode",
                &mut self.weapon_config().aimbot.targeting_mode,
            ) {
                self.send_config();
            }
            if drag(
                ui,
                "FOV",
                DragValue::new(&mut self.weapon_config().aimbot.fov)
                    .range(1.0..=300.0)
                    .suffix(" u")
                    .speed(0.5)
                    .max_decimals(1),
            ) {
                self.send_config();
            }
            if drag(
                ui,
                "Smooth",
                DragValue::new(&mut self.weapon_config().aimbot.smooth)
                    .range(1.0..=100.0)
                    .speed(0.1)
                    .max_decimals(1),
            ) {
                self.send_config();
            }
            if ui.button("⚙ Settings").clicked() {
                self.aim_settings_popup = Some(AimSettingsPopup::Aim);
            }
        });
    }

    fn trigger_main(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Triggerbot", |ui| {
            if self.aimbot_tab == AimbotTab::Weapon
                && self.bool_setting_hover(
                    ui,
                    "Enable Override",
                    Some("Enable Triggerbot settings override for this weapon"),
                    self.trigger_id(TriggerSetting::Override),
                )
            {
                self.send_config();
            }
            if self.bool_setting(
                ui,
                "Enable Triggerbot",
                self.trigger_id(TriggerSetting::Enabled),
            ) {
                self.send_config();
            }
            let seed_enabled = self.weapon_config().triggerbot.seed_mode != SeedMode::Off;
            let delay = ui
                .add_enabled_ui(!seed_enabled, |ui| {
                    ui.add(DragRange::new(
                        "Delay (ms)",
                        &mut self.weapon_config().triggerbot.delay,
                        0..=999,
                    ))
                })
                .inner
                .on_disabled_hover_text(
                    "Seed prediction fires immediately; delay is used when seed mode is off",
                );
            if delay.changed() {
                self.send_config();
            }
            if ui.button("⚙ Settings").clicked() {
                self.aim_settings_popup = Some(AimSettingsPopup::Trigger);
            }
        });
    }

    fn render_aim_settings_popup(&mut self, ui: &mut Ui) {
        let Some(popup) = self.aim_settings_popup else {
            return;
        };
        let mut open = true;
        let title = match popup {
            AimSettingsPopup::Aim => "Aim Settings",
            AimSettingsPopup::Trigger => "Triggerbot Settings",
        };
        egui::Window::new(title)
            .id(egui::Id::new("aim_feature_settings"))
            .collapsible(false)
            .resizable(true)
            .default_width(300.0)
            .open(&mut open)
            .show(ui.ctx(), |ui| match popup {
                AimSettingsPopup::Aim => scroll(ui, "aim_popup_scroll", |ui| self.aim_advanced(ui)),
                AimSettingsPopup::Trigger => {
                    scroll(ui, "trigger_popup_scroll", |ui| self.trigger_advanced(ui))
                }
            });
        if !open {
            self.aim_settings_popup = None;
        }
    }

    fn aim_advanced(&mut self, ui: &mut Ui) {
        ui.heading("Targeting");
        if self.bool_setting(
            ui,
            "Target Friendlies",
            self.aim_id(AimSetting::TargetFriendlies),
        ) {
            self.send_config();
        }
        if drag(
            ui,
            "Randomize Smooth",
            DragValue::new(&mut self.weapon_config().aimbot.smooth_random)
                .range(0.0..=20.0)
                .speed(0.05)
                .max_decimals(1),
        ) {
            self.send_config();
        }
        if drag(
            ui,
            "Deadzone",
            DragValue::new(&mut self.weapon_config().aimbot.deadzone)
                .range(0.0..=50.0)
                .suffix(" u")
                .speed(0.1)
                .max_decimals(1),
        ) {
            self.send_config();
        }
        if drag(
            ui,
            "Reaction Time",
            DragValue::new(&mut self.weapon_config().aimbot.reaction_time)
                .range(0..=500)
                .suffix(" ms"),
        ) {
            self.send_config();
        }
        if drag(
            ui,
            "Inertia",
            DragValue::new(&mut self.weapon_config().aimbot.inertia)
                .range(0.0..=1.0)
                .speed(0.005)
                .max_decimals(2),
        ) {
            self.send_config();
        }
        if drag(
            ui,
            "Start Bullet",
            DragValue::new(&mut self.weapon_config().aimbot.start_bullet).range(0..=10),
        ) {
            self.send_config();
        }

        ui.separator();
        ui.heading("Humanization");
        if self.bool_setting(ui, "Enable Humanization", self.aim_id(AimSetting::Humanize)) {
            self.send_config();
        }
        let enabled = self.weapon_config().aimbot.humanize;
        ui.add_enabled_ui(enabled, |ui| {
            for (label, field) in [("Curve", 0_u8), ("Tremor", 1_u8), ("Overshoot", 2_u8)] {
                let value = match field {
                    0 => &mut self.weapon_config().aimbot.curve,
                    1 => &mut self.weapon_config().aimbot.tremor,
                    _ => &mut self.weapon_config().aimbot.overshoot,
                };
                if drag(
                    ui,
                    label,
                    DragValue::new(value)
                        .range(0.0..=1.0)
                        .speed(0.005)
                        .max_decimals(2),
                ) {
                    self.send_config();
                }
            }
        });

        ui.separator();
        ui.heading("Checks");
        if self.bool_setting(
            ui,
            "Visibility Check",
            self.aim_id(AimSetting::VisibilityCheck),
        ) {
            self.send_config();
        }
        let visibility = self.weapon_config().aimbot.visibility_check;
        ui.add_enabled_ui(visibility, |ui| {
            if self.bool_setting_hover(
                ui,
                "Through Walls",
                Some("Uses Triggerbot autowall rules and requires Triggerbot Through Walls"),
                self.aim_id(AimSetting::ThroughWalls),
            ) {
                self.send_config();
            }
        });
        for (label, setting) in [
            ("Smoke Check", AimSetting::SmokeCheck),
            ("Flash Check", AimSetting::FlashCheck),
            ("In-Air Check", AimSetting::InAirCheck),
        ] {
            if self.bool_setting(ui, label, self.aim_id(setting)) {
                self.send_config();
            }
        }

        ui.separator();
        ui.heading("Bones");
        if bone_selector(ui, &mut self.weapon_config().aimbot.bones) {
            self.send_config();
        }

        ui.separator();
        ui.heading("RCS");
        if self.aimbot_tab == AimbotTab::Weapon
            && self.bool_setting(ui, "Enable Override", self.rcs_id(RcsSetting::Override))
        {
            self.send_config();
        }
        if self.bool_setting(ui, "Enable RCS", self.rcs_id(RcsSetting::Enabled)) {
            self.send_config();
        }
        if ui
            .horizontal(|ui| {
                let rcs = &mut self.weapon_config().rcs;
                let x = ui.add(
                    DragValue::new(&mut rcs.strength.x)
                        .prefix("X: ")
                        .range(0.0..=1.0),
                );
                let y = ui.add(
                    DragValue::new(&mut rcs.strength.y)
                        .prefix("Y: ")
                        .range(0.0..=1.0),
                );
                ui.label("Strength");
                (x | y).changed()
            })
            .inner
        {
            self.send_config();
        }
    }

    fn trigger_advanced(&mut self, ui: &mut Ui) {
        ui.heading("Accuracy");
        let mut seed_changed = false;
        let current_seed_mode = self.weapon_config().triggerbot.seed_mode;
        egui::ComboBox::new("trigger_seed_mode", "Seed")
            .selected_text(current_seed_mode.label())
            .show_ui(ui, |ui| {
                for mode in [SeedMode::Off, SeedMode::Always, SeedMode::WhenAvailable] {
                    if ui
                        .selectable_value(
                            &mut self.weapon_config().triggerbot.seed_mode,
                            mode,
                            mode.label(),
                        )
                        .clicked()
                    {
                        seed_changed = true;
                    }
                }
            })
            .response
            .on_hover_text(
                "Off: use Hitchance. Always: require a validated seed. When Available: use a validated seed and fall back to Hitchance only when prediction data cannot be read",
            );
        if seed_changed {
            self.send_config();
        }
        if combo_box(
            ui,
            "trigger_targeting_mode",
            "Targeting Mode",
            &mut self.weapon_config().triggerbot.targeting_mode,
        ) {
            self.send_config();
        }
        let fov_targeting =
            self.weapon_config().triggerbot.targeting_mode == TriggerTargetingMode::Fov;
        ui.add_enabled_ui(fov_targeting, |ui| {
            if drag_hover(
                ui,
                "FOV",
                "Maximum world-space offset from the center ray; the closest angular target wins",
                DragValue::new(&mut self.weapon_config().triggerbot.fov)
                    .range(1.0..=300.0)
                    .suffix(" u")
                    .speed(0.5)
                    .max_decimals(1),
            ) {
                self.send_config();
            }
        });
        if self.bool_setting_hover(
            ui,
            "Prefer Aim Target",
            Some(
                "Prioritize the active Aim target when it passes Triggerbot's own bones and checks; falls back to the selected targeting mode",
            ),
            self.trigger_id(TriggerSetting::PreferAimTarget),
        ) {
            self.send_config();
        }
        if self.bool_setting(ui, "Head Only", self.trigger_id(TriggerSetting::HeadOnly)) {
            self.send_config();
        }
        if self.bool_setting(
            ui,
            "Prefer Center",
            self.trigger_id(TriggerSetting::PreferCenter),
        ) {
            self.send_config();
        }
        let prefer_center = self.weapon_config().triggerbot.prefer_center;
        ui.add_enabled_ui(prefer_center, |ui| {
            if drag(
                ui,
                "Center Tolerance",
                DragValue::new(&mut self.weapon_config().triggerbot.center_tolerance)
                    .range(1.0..=100.0)
                    .suffix("%")
                    .speed(0.5)
                    .max_decimals(1),
            ) {
                self.send_config();
            }
        });
        if drag_hover(
            ui,
            "Fallback Hitchance",
            "Used with Seed Off, or with When Available when prediction data cannot be read",
            DragValue::new(&mut self.weapon_config().triggerbot.hitchance)
                .range(0.0..=100.0)
                .suffix("%")
                .speed(0.5)
                .max_decimals(1),
        ) {
            self.send_config();
        }
        if drag_hover(
            ui,
            "Min Damage",
            "Minimum estimated health damage for visible and wall-penetrating shots",
            DragValue::new(&mut self.weapon_config().triggerbot.min_damage).range(1..=100),
        ) {
            self.send_config();
        }
        if self.bool_setting_hover(
            ui,
            "Auto Stop",
            Some("Experimental: counter-strafe until the weapon reaches accurate speed"),
            self.trigger_id(TriggerSetting::AutoStop),
        ) {
            self.send_config();
        }
        if drag(
            ui,
            "Hold Duration",
            DragValue::new(&mut self.weapon_config().triggerbot.shot_duration)
                .range(0..=2000)
                .suffix(" ms"),
        ) {
            self.send_config();
        }

        ui.separator();
        ui.heading("Checks");
        if self.bool_setting(
            ui,
            "Visibility Check",
            self.trigger_id(TriggerSetting::VisibilityCheck),
        ) {
            self.send_config();
        }
        let visibility = self.weapon_config().triggerbot.visibility_check;
        ui.add_enabled_ui(visibility, |ui| {
            if self.bool_setting_hover(
                ui,
                "Through Walls",
                Some("Estimate penetration damage from the map collision geometry"),
                self.trigger_id(TriggerSetting::ThroughWalls),
            ) {
                self.send_config();
            }
        });
        for (label, setting) in [
            ("Smoke Check", TriggerSetting::SmokeCheck),
            ("Flash Check", TriggerSetting::FlashCheck),
            ("Scope Check", TriggerSetting::ScopeCheck),
            ("In-Air Check", TriggerSetting::InAirCheck),
        ] {
            if self.bool_setting(ui, label, self.trigger_id(setting)) {
                self.send_config();
            }
        }
        if self.bool_setting_hover(
            ui,
            "Velocity Check",
            Some("Only shoot below the configured movement speed"),
            self.trigger_id(TriggerSetting::VelocityCheck),
        ) {
            self.send_config();
        }
        let velocity_check = self.weapon_config().triggerbot.velocity_check;
        ui.add_enabled_ui(velocity_check, |ui| {
            if drag(
                ui,
                "Velocity Threshold",
                DragValue::new(&mut self.weapon_config().triggerbot.velocity_threshold)
                    .range(0.0..=5000.0),
            ) {
                self.send_config();
            }
        });

        ui.separator();
        ui.heading("Bones");
        if bone_selector(ui, &mut self.weapon_config().triggerbot.bones) {
            self.send_config();
        }
    }
}
