use egui::{DragValue, Ui};

use crate::ui::{
    app::AppState,
    drag_range::DragRange,
    gui::helpers::{
        bone_selector, checkbox, checkbox_hover, collapsing_open, combo_box, drag, keybind, scroll,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AimbotTab {
    Global,
    Weapon,
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
            let left = &mut cols[0];
            scroll(left, "aimbot_left", |ui| self.aimbot_left(ui));

            let right = &mut cols[1];
            scroll(right, "aimbot_right", |ui| self.aimbot_right(ui));
        });
    }

    fn aimbot_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Aimbot", |ui| {
            if keybind(
                ui,
                "aimbot_hotkey",
                "Hotkey",
                &mut self.config.aim.aimbot_hotkey,
            ) {
                self.send_config();
            }

            if self.aimbot_tab == AimbotTab::Weapon
                && checkbox_hover(
                    ui,
                    "Enable Override",
                    "Enable aimbot settings override for a specific weapon",
                    &mut self.weapon_config().aimbot.enable_override,
                )
            {
                self.send_config();
            }

            if checkbox(
                ui,
                "Enable Aimbot",
                &mut self.weapon_config().aimbot.enabled,
            ) {
                self.send_config();
            }

            if combo_box(
                ui,
                "aimbot_mode",
                "Mode",
                &mut self.weapon_config().aimbot.mode,
            ) {
                self.send_config();
            }
        });

        ui.collapsing("Targeting", |ui| {
            if checkbox(
                ui,
                "Target Friendlies",
                &mut self.weapon_config().aimbot.target_friendlies,
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
                    .suffix(" ms")
                    .speed(1.0),
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
                DragValue::new(&mut self.weapon_config().aimbot.start_bullet)
                    .range(0..=10)
                    .speed(0.05),
            ) {
                self.send_config();
            }

            if combo_box(
                ui,
                "targeting_mode",
                "Targeting Mode",
                &mut self.weapon_config().aimbot.targeting_mode,
            ) {
                self.send_config();
            }
        });

        ui.collapsing("Humanization", |ui| {
            if checkbox(
                ui,
                "Enable Humanization",
                &mut self.weapon_config().aimbot.humanize,
            ) {
                self.send_config();
            }

            let enabled = self.weapon_config().aimbot.humanize;
            ui.add_enabled_ui(enabled, |ui| {
                if drag(
                    ui,
                    "Curve",
                    DragValue::new(&mut self.weapon_config().aimbot.curve)
                        .range(0.0..=1.0)
                        .speed(0.005)
                        .max_decimals(2),
                ) {
                    self.send_config();
                }

                if drag(
                    ui,
                    "Tremor",
                    DragValue::new(&mut self.weapon_config().aimbot.tremor)
                        .range(0.0..=1.0)
                        .speed(0.005)
                        .max_decimals(2),
                ) {
                    self.send_config();
                }

                if drag(
                    ui,
                    "Overshoot",
                    DragValue::new(&mut self.weapon_config().aimbot.overshoot)
                        .range(0.0..=1.0)
                        .speed(0.005)
                        .max_decimals(2),
                ) {
                    self.send_config();
                }
            });
        });

        ui.collapsing("Checks", |ui| {
            if checkbox(
                ui,
                "Visibility Check",
                &mut self.weapon_config().aimbot.visibility_check,
            ) {
                self.send_config();
            }

            let visibility_check = self.weapon_config().aimbot.visibility_check;
            ui.add_enabled_ui(visibility_check, |ui| {
                if checkbox(
                    ui,
                    "Through Walls",
                    &mut self.weapon_config().aimbot.through_walls,
                ) {
                    self.send_config();
                }
            });

            if checkbox(
                ui,
                "Smoke Check",
                &mut self.weapon_config().aimbot.smoke_check,
            ) {
                self.send_config();
            }

            if checkbox(
                ui,
                "Flash Check",
                &mut self.weapon_config().aimbot.flash_check,
            ) {
                self.send_config();
            }

            if checkbox(
                ui,
                "In-Air Check",
                &mut self.weapon_config().aimbot.in_air_check,
            ) {
                self.send_config();
            }
        });

        ui.collapsing("Bones", |ui| {
            if bone_selector(ui, &mut self.weapon_config().aimbot.bones) {
                self.send_config();
            }
        });
    }

    fn aimbot_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Triggerbot", |ui| {
            if self.aimbot_tab == AimbotTab::Weapon
                && checkbox(
                    ui,
                    "Enable Override",
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
                "triggerbot_hotkey",
                "Hotkey",
                &mut self.config.aim.triggerbot_hotkey,
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

            if combo_box(
                ui,
                "triggerbot_mode",
                "Mode",
                &mut self.weapon_config().triggerbot.mode,
            ) {
                self.send_config();
            }

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

            if self.weapon_config().triggerbot.prefer_center
                && drag(
                    ui,
                    "Center Tolerance",
                    DragValue::new(&mut self.weapon_config().triggerbot.center_tolerance)
                        .range(1.0..=100.0)
                        .suffix("%")
                        .speed(0.5)
                        .max_decimals(1),
                )
            {
                self.send_config();
            }

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

            if drag(
                ui,
                "Min Damage",
                DragValue::new(&mut self.weapon_config().triggerbot.min_damage)
                    .range(1..=100)
                    .speed(1.0),
            ) {
                self.send_config();
            }

            if checkbox_hover(
                ui,
                "Auto Stop",
                "Automatically counter-strafes / waits for stop to maximize hitchance",
                &mut self.weapon_config().triggerbot.autostop,
            ) {
                self.send_config();
            }

            if drag(
                ui,
                "Hold Duration (ms)",
                DragValue::new(&mut self.weapon_config().triggerbot.shot_duration)
                    .range(0..=2000)
                    .speed(10.0),
            ) {
                self.send_config();
            }
        });

        ui.collapsing("Checks\u{200b}", |ui| {
            if checkbox(
                ui,
                "Visibility Check",
                &mut self.weapon_config().triggerbot.visibility_check,
            ) {
                self.send_config();
            }

            let visibility_check = self.weapon_config().triggerbot.visibility_check;
            ui.add_enabled_ui(visibility_check, |ui| {
                if checkbox(
                    ui,
                    "Through Walls",
                    &mut self.weapon_config().triggerbot.through_walls,
                ) {
                    self.send_config();
                }
            });

            if checkbox(
                ui,
                "Smoke Check",
                &mut self.weapon_config().triggerbot.smoke_check,
            ) {
                self.send_config();
            }

            if checkbox(
                ui,
                "Flash Check",
                &mut self.weapon_config().triggerbot.flash_check,
            ) {
                self.send_config();
            }

            if checkbox(
                ui,
                "Scope Check",
                &mut self.weapon_config().triggerbot.scope_check,
            ) {
                self.send_config();
            }

            if checkbox(
                ui,
                "In-Air Check",
                &mut self.weapon_config().triggerbot.in_air_check,
            ) {
                self.send_config();
            }

            if checkbox_hover(
                ui,
                "Velocity Check",
                "Only shoot if the player moves slower than the specified threshold",
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
                        .range(0..=5000),
                ) {
                    self.send_config();
                }
            });
        });

        ui.collapsing("Bones\u{200b}", |ui| {
            if bone_selector(ui, &mut self.weapon_config().triggerbot.bones) {
                self.send_config();
            }
        });

        collapsing_open(ui, "RCS", |ui| {
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
                            .range(0.0..=1.0)
                            .speed(0.01),
                    );
                    let y = ui.add(
                        DragValue::new(&mut rcs.strength.y)
                            .prefix("Y: ")
                            .range(0.0..=1.0)
                            .speed(0.01),
                    );
                    ui.label("Strength");
                    (x | y).changed()
                })
                .inner
            {
                self.send_config();
            }
        });
    }
}
