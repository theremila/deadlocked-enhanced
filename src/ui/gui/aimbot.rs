use egui::{DragValue, Ui};

use crate::ui::{
    app::AppState,
    drag_range::DragRange,
    gui::helpers::{
        bone_selector, checkbox, checkbox_hover, collapsing_open, combo_box, drag, drag_hover,
        keybind, scroll,
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
    pub fn aimbot_settings(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.aimbot_tab, AimbotTab::Global, "Global");
            ui.selectable_value(&mut self.aimbot_tab, AimbotTab::Weapon, "Weapon");
            if self.aimbot_tab == AimbotTab::Weapon {
                combo_box(ui, "aimbot_weapon", "Weapon", &mut self.aimbot_weapon);
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
                && checkbox_hover(
                    ui,
                    "Enable Override",
                    "Enable Aim settings override for this weapon",
                    &mut self.weapon_config().aimbot.enable_override,
                )
            {
                self.send_config();
            }
            if checkbox(ui, "Enable Aim", &mut self.weapon_config().aimbot.enabled) {
                self.send_config();
            }
            if keybind(
                ui,
                "aim_hotkey",
                "Hotkey",
                &mut self.config.aim.aimbot_hotkey,
            ) {
                self.send_config();
            }
            if combo_box(
                ui,
                "aim_mode",
                "Mode",
                &mut self.weapon_config().aimbot.mode,
            ) {
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
                && checkbox_hover(
                    ui,
                    "Enable Override",
                    "Enable Triggerbot settings override for this weapon",
                    &mut self.weapon_config().triggerbot.enable_override,
                )
            {
                self.send_config();
            }
            if checkbox(
                ui,
                "Enable Triggerbot",
                &mut self.weapon_config().triggerbot.enabled,
            ) {
                self.send_config();
            }
            if keybind(
                ui,
                "trigger_hotkey",
                "Hotkey",
                &mut self.config.aim.triggerbot_hotkey,
            ) {
                self.send_config();
            }
            if combo_box(
                ui,
                "trigger_mode",
                "Mode",
                &mut self.weapon_config().triggerbot.mode,
            ) {
                self.send_config();
            }
            if ui
                .add(DragRange::new(
                    "Delay (ms)",
                    &mut self.weapon_config().triggerbot.delay,
                    0..=999,
                ))
                .changed()
            {
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
        if checkbox(
            ui,
            "Target Friendlies",
            &mut self.weapon_config().aimbot.target_friendlies,
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
        if checkbox(
            ui,
            "Enable Humanization",
            &mut self.weapon_config().aimbot.humanize,
        ) {
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
        if checkbox(
            ui,
            "Visibility Check",
            &mut self.weapon_config().aimbot.visibility_check,
        ) {
            self.send_config();
        }
        let visibility = self.weapon_config().aimbot.visibility_check;
        ui.add_enabled_ui(visibility, |ui| {
            if checkbox_hover(
                ui,
                "Through Walls",
                "Uses Triggerbot autowall rules and requires Triggerbot Through Walls",
                &mut self.weapon_config().aimbot.through_walls,
            ) {
                self.send_config();
            }
        });
        for (label, field) in [
            ("Smoke Check", 0_u8),
            ("Flash Check", 1),
            ("In-Air Check", 2),
        ] {
            let value = match field {
                0 => &mut self.weapon_config().aimbot.smoke_check,
                1 => &mut self.weapon_config().aimbot.flash_check,
                _ => &mut self.weapon_config().aimbot.in_air_check,
            };
            if checkbox(ui, label, value) {
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
            && checkbox(
                ui,
                "Enable Override",
                &mut self.weapon_config().rcs.enable_override,
            )
        {
            self.send_config();
        }
        if checkbox(ui, "Enable RCS", &mut self.weapon_config().rcs.enabled) {
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
        if checkbox(
            ui,
            "Head Only",
            &mut self.weapon_config().triggerbot.head_only,
        ) {
            self.send_config();
        }
        if checkbox(
            ui,
            "Prefer Center",
            &mut self.weapon_config().triggerbot.prefer_center,
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
        if drag(
            ui,
            "Hitchance",
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
        if checkbox_hover(
            ui,
            "Auto Stop",
            "Counter-strafe until the weapon reaches accurate speed",
            &mut self.weapon_config().triggerbot.autostop,
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
        if checkbox(
            ui,
            "Visibility Check",
            &mut self.weapon_config().triggerbot.visibility_check,
        ) {
            self.send_config();
        }
        let visibility = self.weapon_config().triggerbot.visibility_check;
        ui.add_enabled_ui(visibility, |ui| {
            if checkbox_hover(
                ui,
                "Through Walls",
                "Estimate penetration damage from the map collision geometry",
                &mut self.weapon_config().triggerbot.through_walls,
            ) {
                self.send_config();
            }
        });
        for (label, field) in [
            ("Smoke Check", 0_u8),
            ("Flash Check", 1),
            ("Scope Check", 2),
            ("In-Air Check", 3),
        ] {
            let value = match field {
                0 => &mut self.weapon_config().triggerbot.smoke_check,
                1 => &mut self.weapon_config().triggerbot.flash_check,
                2 => &mut self.weapon_config().triggerbot.scope_check,
                _ => &mut self.weapon_config().triggerbot.in_air_check,
            };
            if checkbox(ui, label, value) {
                self.send_config();
            }
        }
        if checkbox_hover(
            ui,
            "Velocity Check",
            "Only shoot below the configured movement speed",
            &mut self.weapon_config().triggerbot.velocity_check,
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
