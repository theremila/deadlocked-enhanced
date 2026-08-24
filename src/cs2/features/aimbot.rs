use glam::{Vec2, vec2};

use crate::{
    config::Config,
    cs2::{
        CS2,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::{angles_to_fov, vec2_clamp},
    os::mouse::Mouse,
};

#[derive(Default)]
pub struct Aimbot {
    pub active: bool,
    inertia: Vec2,
    current_target_pawn: usize,
    initial_fov: f32,
    curve_direction: f32,
    time: f32,
}

impl CS2 {
    pub fn aimbot(&mut self, config: &Config, mouse: &mut Mouse) -> bool {
        let hotkey = config.aim.aimbot_hotkey;
        let config = self.aimbot_config(config);

        if !config.enabled {
            self.aim.current_target_pawn = 0;
            self.aim.initial_fov = 0.0;
            return false;
        }

        if !Self::check_hotkey(&self.input, config.mode, hotkey, &mut self.aim.active) {
            self.aim.current_target_pawn = 0;
            self.aim.initial_fov = 0.0;
            return false;
        }

        let Some(target) = &self.target.player else {
            self.aim.current_target_pawn = 0;
            self.aim.initial_fov = 0.0;
            return false;
        };

        if !target.is_valid(self) {
            self.aim.current_target_pawn = 0;
            self.aim.initial_fov = 0.0;
            return false;
        }

        let Some(local_player) = Player::local_player(self) else {
            self.aim.current_target_pawn = 0;
            self.aim.initial_fov = 0.0;
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
            self.aim.initial_fov = 0.0;
            return false;
        }

        if config.flash_check && local_player.is_flashed(self) {
            return false;
        }

        if config.visibility_check && !target.visible(self, &local_player) {
            return false;
        }

        if local_player.shots_fired(self) < config.start_bullet {
            return false;
        }

        let target_angle = {
            let mut smallest_fov = 360.0;
            let mut smallest_angle = glam::Vec2::ZERO;
            for bone in &config.bones {
                let bone_pos = target.bone_position(self, bone.u64());
                let angle =
                    self.angle_to_target(&local_player, &bone_pos, &self.target.previous_aim_punch);
                let fov = angles_to_fov(&local_player.view_angles(self), &angle);
                if fov < smallest_fov {
                    smallest_fov = fov;
                    smallest_angle = angle;
                }
            }

            smallest_angle
        };

        let view_angles = local_player.view_angles(self);
        let current_fov = angles_to_fov(&view_angles, &target_angle);
        if current_fov
            > (config.fov
                * if config.distance_adjusted_fov {
                    self.distance_scale(self.target.distance)
                } else {
                    1.0
                })
        {
            self.aim.current_target_pawn = 0;
            self.aim.initial_fov = 0.0;
            return false;
        }

        // Deadzone check: if crosshair is already within the deadzone of the bone, stop pulling
        if config.deadzone > 0.0 && current_fov <= config.deadzone {
            self.aim.inertia = Vec2::ZERO;
            return false;
        }

        // Initialize target tracking for ABCurves
        if self.aim.current_target_pawn != target.pawn || self.aim.initial_fov <= 0.01 {
            self.aim.current_target_pawn = target.pawn;
            self.aim.initial_fov = current_fov.max(0.1);
            self.aim.curve_direction = if ((self.aim.time * 100.0) as i32 % 2) == 0 {
                1.0
            } else {
                -1.0
            };
            self.aim.time = 0.0;
        }
        self.aim.time += 0.002;

        let mut aim_angles = view_angles - target_angle;
        if aim_angles.y < -180.0 {
            aim_angles.y += 360.0
        }
        vec2_clamp(&mut aim_angles);

        let rand_factor: f32 = rand::random();
        let randomized_smooth = (config.smooth - (rand_factor * config.smooth_random)).max(0.5);

        let effective_smooth = if config.humanize {
            let progress = (1.0 - (current_fov / self.aim.initial_fov.max(0.1))).clamp(0.0, 1.0);

            // 1. Bezier Curve / Arch trajectory
            if config.curve > 0.0 {
                let ortho = vec2(-aim_angles.y, aim_angles.x).normalize_or_zero();
                let bow = (std::f32::consts::PI * progress).sin()
                    * config.curve
                    * current_fov
                    * 0.25
                    * self.aim.curve_direction;
                aim_angles += ortho * bow;
            }

            // 2. Target Overshoot & Correction
            if config.overshoot > 0.0 && self.aim.initial_fov > 0.8 {
                let overshoot_mod = 1.0
                    + config.overshoot * 0.2 * (1.0 - progress) * (std::f32::consts::PI * progress).sin();
                aim_angles *= overshoot_mod;
            }

            // 3. Hand Tremor / Harmonic Micro-Noise
            if config.tremor > 0.0 {
                let t = self.aim.time * 25.0;
                let tremor_x = (t * 11.3).sin() * (t * 7.7).cos();
                let tremor_y = (t * 13.1).cos() * (t * 9.5).sin();
                let scale = config.tremor * 0.12 * (current_fov / 3.0).clamp(0.05, 1.0);
                aim_angles += vec2(tremor_x, tremor_y) * scale;
            }

            vec2_clamp(&mut aim_angles);

            // Initial acceleration ramp-up (first 50ms) to prevent unnatural instant start
            let accel_ramp = (self.aim.time / 0.05).clamp(0.2, 1.0);

            // 4. Ease-Out Dynamic Smoothing (always active when humanize is enabled)
            let ease_out_factor = 0.8 + 0.6 * progress;
            ((randomized_smooth * 2.5 * ease_out_factor + 1.0) / accel_ramp).clamp(1.0, 500.0)
        } else {
            (randomized_smooth * 2.5 + 1.0).clamp(1.0, 500.0)
        };

        let sensitivity = (self.get_sensitivity() * local_player.fov_multiplier(self)).max(0.001);

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
