use egui::{Color32, Painter, Pos2, Stroke, pos2, vec2};
use glam::Vec3;

use crate::{
    config::aim::KeyMode, config::text::TextPosition, cs2::entity::weapon_class::WeaponClass,
    data::Data, math::world_to_screen, ui::app::AppState,
};

impl AppState {
    pub fn overlay_debug(&self, painter: &Painter, data: &Data) {
        if self.config.hud.debug {
            painter.line(
                vec![pos2(0.0, 0.0), pos2(data.window_size.x, data.window_size.y)],
                Stroke::new(self.config.hud.line_width, Color32::WHITE),
            );
            painter.line(
                vec![pos2(data.window_size.x, 0.0), pos2(0.0, data.window_size.y)],
                Stroke::new(self.config.hud.line_width, Color32::WHITE),
            );
        }
    }

    pub fn draw_bomb_timer(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.bomb_timer || !data.bomb.planted {
            return;
        }

        if let Some(pos) = world_to_screen(&data.bomb.position, data) {
            let cat = &self.config.hud.overlay_text.bomb_timer;
            let anchor = point_anchor(pos, cat.position, cat.font_size * 0.3);
            self.text_sized(
                painter,
                format!("{:.3}", data.bomb.timer),
                anchor,
                cat.align.to_align2(),
                cat.color,
                cat.font_size,
            );
            if data.bomb.being_defused {
                self.text_sized(
                    painter,
                    format!("defusing {:.3}", data.bomb.defuse_remain_time),
                    anchor + vec2(0.0, cat.font_size),
                    cat.align.to_align2(),
                    cat.color,
                    cat.font_size,
                );
            }
        }

        let fraction = (data.bomb.timer / 40.0).clamp(0.0, 1.0);
        let color = self.health_color((fraction * 100.0) as i32, 255);
        painter.line(
            vec![
                pos2(0.0, data.window_size.y),
                pos2(data.window_size.x * fraction, data.window_size.y),
            ],
            Stroke::new(self.config.hud.line_width * 3.0, color),
        );
    }

    pub fn draw_fov_circle(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.fov_circle || !data.in_game {
            return;
        }

        let weapon_config = self.aimbot_config(&data.weapon);

        if !weapon_config.enabled || (weapon_config.mode == KeyMode::Toggle && !data.aimbot_active)
        {
            return;
        }

        let fov_units = weapon_config.fov;
        let stroke_color = if data.aimbot_active {
            Color32::GREEN
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 180)
        };
        let stroke = Stroke::new(self.config.hud.line_width, stroke_color);

        match weapon_config.fov_mode {
            crate::config::aim::FovMode::TargetBone => {
                let targets = if weapon_config.target_friendlies {
                    data.players.iter().chain(data.friendlies.iter())
                } else {
                    data.players.iter().chain([].iter())
                };

                let screen_center = pos2(data.window_size.x / 2.0, data.window_size.y / 2.0);

                for player in targets {
                    if player.health <= 0 {
                        continue;
                    }

                    if weapon_config.visibility_check && !player.visible {
                        continue;
                    }

                    let head_pos = player.head;
                    let Some(screen_pos) = world_to_screen(&head_pos, data) else {
                        continue;
                    };

                    let Some(radius) = self.calculate_unit_radius_px(data, &head_pos, fov_units) else {
                        continue;
                    };

                    // Check if crosshair is inside this bone FOV circle to give visual feedback
                    let dist_to_crosshair = screen_center.distance(screen_pos);
                    let target_stroke = if dist_to_crosshair <= radius {
                        Stroke::new(self.config.hud.line_width * 1.5, Color32::GREEN)
                    } else {
                        stroke
                    };

                    painter.circle_stroke(screen_pos, radius, target_stroke);
                }
            }
            crate::config::aim::FovMode::Crosshair => {
                let fov_rad = (self.get_current_fov().to_radians() / 2.0).tan();
                let radius = if fov_rad > 0.001 {
                    let focal_length_px = (data.window_size.x * 0.5) / fov_rad;
                    (fov_units / 500.0) * focal_length_px
                } else {
                    50.0
                };
                let center = pos2(data.window_size.x / 2.0, data.window_size.y / 2.0);
                painter.circle_stroke(center, radius, stroke);
            }
        }
    }

    fn calculate_unit_radius_px(
        &self,
        data: &Data,
        position: &Vec3,
        radius_units: f32,
    ) -> Option<f32> {
        let vm = &data.view_matrix;
        let w = vm.w_axis.x * position.x
            + vm.w_axis.y * position.y
            + vm.w_axis.z * position.z
            + vm.w_axis.w;

        if w < 0.01 {
            return None;
        }

        let fov_rad = (self.get_current_fov().to_radians() / 2.0).tan();
        if fov_rad <= 0.001 {
            return None;
        }
        let focal_length_px = (data.window_size.x * 0.5) / fov_rad;
        let radius_px = (radius_units / w) * focal_length_px;
        Some(radius_px)
    }

    pub fn draw_keybind_list(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.keybind_list {
            return;
        }

        let cat = &self.config.hud.overlay_text.keybind_list;
        let position = screen_anchor(
            [data.window_size.x, data.window_size.y],
            cat.position,
            10.0,
            0.0,
        );
        let aimbot_color = if data.aimbot_active {
            Color32::GREEN
        } else {
            cat.color
        };
        self.text_sized(
            painter,
            format!("Aimbot: {:?}", self.config.aim.aimbot_hotkey),
            position,
            cat.align.to_align2(),
            aimbot_color,
            cat.font_size,
        );

        let triggerbot_color = if data.triggerbot_active {
            Color32::GREEN
        } else {
            cat.color
        };
        self.text_sized(
            painter,
            format!("Triggerbot: {:?}", self.config.aim.triggerbot_hotkey),
            position + vec2(0.0, cat.font_size),
            cat.align.to_align2(),
            triggerbot_color,
            cat.font_size,
        );
    }

    pub fn draw_spectator_list(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.spectator_list {
            return;
        }

        let cat = &self.config.hud.overlay_text.spectator_list;
        let position = screen_anchor(
            [data.window_size.x, data.window_size.y],
            cat.position,
            10.0,
            cat.font_size * 3.0,
        );
        self.text_sized(
            painter,
            "Spectators:",
            position,
            cat.align.to_align2(),
            cat.color,
            cat.font_size,
        );

        for (i, name) in data.spectators.iter().enumerate() {
            self.text_sized(
                painter,
                format!("> {name}"),
                position + vec2(0.0, cat.font_size * (i as f32 + 1.0)),
                cat.align.to_align2(),
                cat.color,
                cat.font_size,
            );
        }
    }

    #[allow(dead_code)]
    fn get_current_fov(&self) -> f32 {
        (if self.config.misc.fov_changer {
            self.config.misc.desired_fov
        } else {
            crate::constants::cs2::DEFAULT_FOV
        }) as f32
    }

    #[allow(dead_code)]
    fn calculate_fov_radius(&self, data: &Data, target_fov: f32) -> f32 {
        let current_fov = self.get_current_fov();
        let screen_width = data.window_size.x;

        let current_fov_tan = (current_fov.to_radians() / 2.0).tan();
        if current_fov_tan == 0.0 {
            return 0.0;
        }

        let target_fov_tan = (target_fov.to_radians() / 2.0).tan();
        (target_fov_tan / current_fov_tan) * (screen_width / 2.0)
    }



    pub fn draw_sniper_crosshair(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.sniper_crosshair.enabled
            || WeaponClass::from_string(data.weapon.as_ref()) != WeaponClass::Sniper
        {
            return;
        }

        let length = self.config.hud.sniper_crosshair.line_length;
        let gap = self.config.hud.sniper_crosshair.gap / 2.0;
        let center = data.window_size / 2.0;

        let stroke = Stroke::new(
            self.config.hud.sniper_crosshair.line_width,
            self.config.hud.sniper_crosshair.color,
        );

        painter.line_segment(
            [
                pos2(center.x + gap, center.y),
                pos2(center.x + gap + length, center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x, center.y + gap),
                pos2(center.x, center.y + gap + length),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x - gap, center.y),
                pos2(center.x - gap - length, center.y),
            ],
            stroke,
        );
        painter.line_segment(
            [
                pos2(center.x, center.y - gap),
                pos2(center.x, center.y - gap - length),
            ],
            stroke,
        );
    }
}

pub fn point_anchor(point: Pos2, position: TextPosition, offset: f32) -> Pos2 {
    match position {
        TextPosition::TopLeft => point + vec2(-offset, -offset),
        TextPosition::TopCenter => point + vec2(0.0, -offset),
        TextPosition::TopRight => point + vec2(offset, -offset),
        TextPosition::CenterLeft => point + vec2(-offset, 0.0),
        TextPosition::Center => point,
        TextPosition::CenterRight => point + vec2(offset, 0.0),
        TextPosition::BottomLeft => point + vec2(-offset, offset),
        TextPosition::BottomCenter => point + vec2(0.0, offset),
        TextPosition::BottomRight => point + vec2(offset, offset),
    }
}

pub fn screen_anchor(size: [f32; 2], position: TextPosition, pad_x: f32, offset_y: f32) -> Pos2 {
    let [w, h] = size;
    match position {
        TextPosition::TopLeft => pos2(pad_x, offset_y),
        TextPosition::TopCenter => pos2(w / 2.0, offset_y),
        TextPosition::TopRight => pos2(w - pad_x, offset_y),
        TextPosition::CenterLeft => pos2(pad_x, h / 2.0 + offset_y),
        TextPosition::Center => pos2(w / 2.0, h / 2.0 + offset_y),
        TextPosition::CenterRight => pos2(w - pad_x, h / 2.0 + offset_y),
        TextPosition::BottomLeft => pos2(pad_x, h + offset_y),
        TextPosition::BottomCenter => pos2(w / 2.0, h + offset_y),
        TextPosition::BottomRight => pos2(w - pad_x, h + offset_y),
    }
}
