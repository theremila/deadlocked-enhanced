use std::time::{Duration, Instant};

use glam::{Vec2, vec2};

use crate::{
    config::Config,
    cs2::{
        CS2,
        bones::Bones,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::angles_to_fov,
    os::mouse::Mouse,
};

#[derive(Default)]
pub struct Aimbot {
    pub active: bool,
    inertia: Vec2,
    current_target_pawn: usize,
    target_acquired_time: Option<Instant>,
    initial_fov: f32,
    curve_direction: f32,
    time: f32,
}

impl CS2 {
    pub fn aimbot(&mut self, config: &Config, mouse: &mut Mouse) -> bool {
        let hotkey = config.aim.aimbot_hotkey;
        let global_config = config;
        let config = self.aimbot_config(config);

        if !config.enabled {
            self.aim.current_target_pawn = 0;
            self.aim.target_acquired_time = None;
            self.aim.initial_fov = 0.0;
            self.aim.inertia = Vec2::ZERO;
            return false;
        }

        if !Self::check_hotkey(&self.input, config.mode, hotkey, &mut self.aim.active) {
            self.aim.current_target_pawn = 0;
            self.aim.target_acquired_time = None;
            self.aim.initial_fov = 0.0;
            self.aim.inertia = Vec2::ZERO;
            return false;
        }

        let Some(target) = &self.target.player else {
            self.aim.current_target_pawn = 0;
            self.aim.target_acquired_time = None;
            self.aim.initial_fov = 0.0;
            self.aim.inertia = Vec2::ZERO;
            return false;
        };

        if !target.is_valid(self) {
            self.aim.current_target_pawn = 0;
            self.aim.target_acquired_time = None;
            self.aim.initial_fov = 0.0;
            self.aim.inertia = Vec2::ZERO;
            return false;
        }

        let Some(local_player) = Player::local_player(self) else {
            self.aim.current_target_pawn = 0;
            self.aim.target_acquired_time = None;
            self.aim.initial_fov = 0.0;
            self.aim.inertia = Vec2::ZERO;
            return false;
        };

        let weapon_class = local_player.weapon_class(self);
        let disallowed_weapons = [
            WeaponClass::Unknown,
            WeaponClass::Knife,
            WeaponClass::Grenade,
        ];
        if disallowed_weapons.contains(&weapon_class) {
            self.aim.current_target_pawn = 0;
            self.aim.target_acquired_time = None;
            self.aim.initial_fov = 0.0;
            self.aim.inertia = Vec2::ZERO;
            return false;
        }

        if config.flash_check && local_player.is_flashed(self) {
            return false;
        }

        if config.in_air_check && local_player.is_in_air(self) {
            return false;
        }

        let eye_pos = local_player.eye_position(self);
        let target_bone = if config.bones.iter().any(|b| b.u64() == self.target.bone_index) {
            self.target.bone_index
        } else {
            config.bones.first().map(|b| b.u64()).unwrap_or(Bones::Head.u64())
        };
        let bone_pos = target.bone_position(self, target_bone);

        if config.smoke_check && self.is_line_in_smoke(eye_pos, bone_pos) {
            return false;
        }

        if config.visibility_check {
            let is_visible = target.visible(self, &local_player);
            if !is_visible {
                let min_damage = self.triggerbot_config(global_config).min_damage;
                let target_bone_enum = config
                    .bones
                    .iter()
                    .find(|b| b.u64() == target_bone)
                    .cloned()
                    .unwrap_or(Bones::Head);
                let has_armor = target.armor(self) > 0;
                if !config.through_walls
                    || !self.can_penetrate_wall(
                        eye_pos,
                        bone_pos,
                        target_bone_enum,
                        has_armor,
                        min_damage,
                    )
                {
                    return false;
                }
            }
        }

        if local_player.shots_fired(self) < config.start_bullet {
            return false;
        }

        let target_distance = eye_pos.distance(bone_pos).max(1.0);
        let target_angle = self.angle_to_target(&local_player, &bone_pos, &self.target.previous_aim_punch);

        let view_angles = local_player.view_angles(self);
        let current_fov = angles_to_fov(&view_angles, &target_angle);
        let offset_units = target_distance * current_fov.to_radians().sin();

        // Check if crosshair is within FOV (in CS game units)
        if offset_units > config.fov {
            self.aim.current_target_pawn = 0;
            self.aim.target_acquired_time = None;
            self.aim.initial_fov = 0.0;
            self.aim.inertia = Vec2::ZERO;
            return false;
        }

        // Deadzone check in CS game units: if crosshair is inside bone deadzone, stop pulling
        if config.deadzone > 0.0 && offset_units <= config.deadzone {
            self.aim.inertia *= 0.5;
            return false;
        }

        // Reaction Time handling
        if self.aim.current_target_pawn != target.pawn || self.aim.target_acquired_time.is_none() {
            self.aim.target_acquired_time = Some(Instant::now());
        }

        if let Some(acquired) = self.aim.target_acquired_time {
            if config.reaction_time > 0 && acquired.elapsed() < Duration::from_millis(config.reaction_time) {
                return true;
            }
        }

        // Initialize target tracking for ABCurves
        if self.aim.current_target_pawn != target.pawn || self.aim.initial_fov <= 0.01 {
            self.aim.current_target_pawn = target.pawn;
            self.aim.initial_fov = current_fov.max(0.1);
            let rand_val: f32 = rand::random();
            self.aim.curve_direction = if rand_val > 0.5 { 1.0 } else { -1.0 };
            self.aim.time = 0.0;
        }
        self.aim.time += 0.002;

        let mut aim_angles = view_angles - target_angle;
        aim_angles.x = (aim_angles.x + 180.0).rem_euclid(360.0) - 180.0;
        aim_angles.y = (aim_angles.y + 180.0).rem_euclid(360.0) - 180.0;

        let rand_factor: f32 = rand::random();
        let randomized_smooth = (config.smooth - (rand_factor * config.smooth_random)).max(0.5);

        let effective_smooth = if config.humanize {
            let progress = (1.0 - (current_fov / self.aim.initial_fov.max(0.1))).clamp(0.0, 1.0);

            // 1. Bezier Curve / Arch trajectory (only when not directly on target)
            if config.curve > 0.0 && current_fov > 0.5 {
                let ortho = vec2(-aim_angles.y, aim_angles.x).normalize_or_zero();
                let bow = (std::f32::consts::PI * progress).sin()
                    * config.curve
                    * current_fov
                    * 0.15
                    * self.aim.curve_direction;
                aim_angles += ortho * bow;
            }

            // 2. Target Overshoot & Correction
            if config.overshoot > 0.0 && self.aim.initial_fov > 1.2 {
                let overshoot_mod = 1.0
                    + config.overshoot * 0.15 * (1.0 - progress) * (std::f32::consts::PI * progress).sin();
                aim_angles *= overshoot_mod;
            }

            // 3. Hand Tremor / Harmonic Micro-Noise
            if config.tremor > 0.0 {
                let t = self.aim.time * 25.0;
                let tremor_x = (t * 11.3).sin() * (t * 7.7).cos();
                let tremor_y = (t * 13.1).cos() * (t * 9.5).sin();
                let scale = config.tremor * 0.08 * (current_fov / 3.0).clamp(0.05, 1.0);
                aim_angles += vec2(tremor_x, tremor_y) * scale;
            }

            aim_angles.x = (aim_angles.x + 180.0).rem_euclid(360.0) - 180.0;
            aim_angles.y = (aim_angles.y + 180.0).rem_euclid(360.0) - 180.0;

            // Initial acceleration ramp-up (first 50ms) to prevent unnatural instant start
            let accel_ramp = (self.aim.time / 0.05).clamp(0.2, 1.0);

            // 4. Ease-Out Dynamic Smoothing (always active when humanize is enabled)
            let ease_out_factor = 0.85 + 0.5 * progress;
            ((randomized_smooth * 2.5 * ease_out_factor + 1.0) / accel_ramp).clamp(1.0, 500.0)
        } else {
            (randomized_smooth * 2.5 + 1.0).clamp(1.0, 500.0)
        };

        let fov_mult = local_player.fov_multiplier(self);
        let fov_mult = if fov_mult <= 0.01 || fov_mult > 2.0 { 1.0 } else { fov_mult };
        let sensitivity = (self.get_sensitivity() * fov_mult).max(0.01);

        let mouse_angles = vec2(
            aim_angles.y / sensitivity * 45.45,
            -aim_angles.x / sensitivity * 45.45,
        ) / effective_smooth;

        let alpha = 1.0 - config.inertia.clamp(0.0, 1.0) * 0.5;
        self.aim.inertia += (mouse_angles - self.aim.inertia) * alpha;
        mouse.move_rel(self.aim.inertia);

        self.recoil.previous = local_player.aim_punch(self);

        true
    }
}
