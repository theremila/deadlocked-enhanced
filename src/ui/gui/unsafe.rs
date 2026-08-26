use egui::{DragValue, Ui};

use crate::ui::{
    app::AppState,
    gui::helpers::{checkbox, color_picker, combo_box, drag, groupbox, keybind, scroll},
};

impl AppState {
    pub fn unsafe_settings(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            scroll(left, "unsafe_left", |left| {
                self.unsafe_left(left);
            });

            let right = &mut cols[1];
            scroll(right, "unsafe_right", |right| {
                self.unsafe_right(right);
            });
        });
    }

    fn unsafe_left(&mut self, ui: &mut Ui) {
        groupbox(ui, "Bunnyhop / Movement", |ui| {
            if checkbox(ui, "Enable Bunnyhop", &mut self.config.misc.bunnyhop) {
                self.send_config();
            }

            if combo_box(
                ui,
                "bunnyhop_mode",
                "Mode",
                &mut self.config.misc.bunnyhop_mode,
            ) {
                self.send_config();
            }

            if keybind(
                ui,
                "bunnyhop_hotkey",
                "Hotkey",
                &mut self.config.misc.bunnyhop_hotkey,
            ) {
                self.send_config();
            }
        });

        groupbox(ui, "Flashbang (No Flash)", |ui| {
            if checkbox(ui, "Enable No Flash", &mut self.config.misc.no_flash) {
                self.send_config();
            }

            if drag(
                ui,
                "Max Flash Alpha",
                DragValue::new(&mut self.config.misc.max_flash_alpha)
                    .range(0.0..=255.0)
                    .speed(0.5)
                    .max_decimals(0),
            ) {
                self.send_config();
            }
        });
    }

    fn unsafe_right(&mut self, ui: &mut Ui) {
        groupbox(ui, "Field of View (FOV)", |ui| {
            if checkbox(ui, "Enable FOV Changer", &mut self.config.misc.fov_changer) {
                self.send_config();
            }

            ui.horizontal(|ui| {
                if ui
                    .add(
                        DragValue::new(&mut self.config.misc.desired_fov)
                            .speed(0.2)
                            .range(1..=179),
                    )
                    .changed()
                {
                    self.send_config();
                }
                ui.label("Desired FOV");

                if ui.button("↺").on_hover_text("Reset to 90").clicked() {
                    self.config.misc.desired_fov = crate::constants::cs2::DEFAULT_FOV;
                    self.send_config();
                }
            });
        });

        groupbox(ui, "Smoke Removal & Color", |ui| {
            if checkbox(ui, "No Smoke", &mut self.config.misc.no_smoke) {
                self.send_config();
            }

            if checkbox(
                ui,
                "Change Smoke Color",
                &mut self.config.misc.change_smoke_color,
            ) {
                self.send_config();
            }

            if color_picker(ui, "Smoke Color", &mut self.config.misc.smoke_color) {
                self.send_config();
            }
        });
    }
}
