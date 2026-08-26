use egui::{Align2, Color32, Painter, Pos2, Stroke, pos2, vec2};
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

        let stroke_color = if data.aimbot_active {
            Color32::GREEN
        } else {
            Color32::from_rgba_unmultiplied(255, 255, 255, 180)
        };
        let stroke = Stroke::new(self.config.hud.line_width, stroke_color);

        let Some(position) = data.aim_target_position else {
            return;
        };
        let Some(screen_position) = world_to_screen(&position, data) else {
            return;
        };
        let Some(radius) = self.calculate_unit_radius_px(data, &position, weapon_config.fov) else {
            return;
        };

        painter.circle_stroke(screen_position, radius, stroke);
    }

    fn calculate_unit_radius_px(
        &self,
        data: &Data,
        position: &Vec3,
        radius_units: f32,
    ) -> Option<f32> {
        let matrix = &data.view_matrix;
        let row_x = matrix.x_axis.truncate();
        let row_y = matrix.y_axis.truncate();
        let row_w = matrix.w_axis.truncate();
        let clip_x = row_x.dot(*position) + matrix.x_axis.w;
        let clip_y = row_y.dot(*position) + matrix.y_axis.w;
        let w = row_w.dot(*position) + matrix.w_axis.w;

        if w < 0.01 {
            return None;
        }

        let ndc_x_per_unit = (row_x * w - row_w * clip_x).length() / w.powi(2);
        let ndc_y_per_unit = (row_y * w - row_w * clip_y).length() / w.powi(2);
        let radius_x = radius_units * ndc_x_per_unit * data.window_size.x * 0.5;
        let radius_y = radius_units * ndc_y_per_unit * data.window_size.y * 0.5;
        let radius = (radius_x + radius_y) * 0.5;

        radius.is_finite().then_some(radius)
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

    pub fn draw_watermark(&self, painter: &Painter, data: &Data) {
        if !self.config.hud.watermark {
            return;
        }

        const FONT_SIZE: f32 = 13.0;
        const PADDING: egui::Vec2 = egui::vec2(10.0, 5.0);
        const SCREEN_MARGIN: f32 = 12.0;
        const CORNER_RADIUS: f32 = 4.0;
        const BG_COLOR: Color32 = Color32::from_black_alpha(180);

        let text = format!("deadlocked.enhanced / {} fps", self.fps());
        let font = egui::FontId::proportional(FONT_SIZE);
        let galley = painter.layout_no_wrap(text.clone(), font, Color32::WHITE);

        let size = galley.size() + PADDING * 2.0;
        let min_pos = pos2(
            (data.window_size.x - size.x - SCREEN_MARGIN).round(),
            SCREEN_MARGIN,
        );
        let rect = egui::Rect::from_min_size(min_pos, size);

        painter.rect_filled(rect, CORNER_RADIUS, BG_COLOR);
        self.text_sized(
            painter,
            text,
            rect.center(),
            Align2::CENTER_CENTER,
            Color32::WHITE,
            FONT_SIZE,
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
