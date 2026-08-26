use std::time::{Duration, Instant};

use glam::{Vec2, vec2};

use crate::{
    config::Config,
    cs2::{
        CS2,
        bones::Bones,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::{angles_to_fov, forward_ray_offset, vec2_clamp},
    os::mouse::Mouse,
};

#[derive(Default)]
pub struct Aimbot {
    pub active: bool,
    inertia: Vec2,
    current_target_pawn: Option<usize>,
    target_acquired_time: Option<Instant>,
    initial_fov: f32,
    curve_direction: f32,
    smooth_random_factor: f32,
}

impl Aimbot {
    fn reset_tracking(&mut self) {
        self.inertia = Vec2::ZERO;
        self.current_target_pawn = None;
        self.target_acquired_time = None;
        self.initial_fov = 0.0;
    }

    fn track(&mut self, pawn: usize, current_fov: f32) {
        if self.current_target_pawn == Some(pawn) {
            return;
        }

        self.current_target_pawn = Some(pawn);
        self.target_acquired_time = Some(Instant::now());
        self.initial_fov = current_fov.max(0.1);
        self.curve_direction = if rand::random() { 1.0 } else { -1.0 };
        self.smooth_random_factor = rand::random();
    }

    fn smooth_delta(&mut self, target: Vec2, alpha: f32) -> Vec2 {
        for axis in 0..2 {
            let target_axis = target[axis];
            let inertia_axis = &mut self.inertia[axis];

            if target_axis == 0.0 || target_axis.signum() != inertia_axis.signum() {
                *inertia_axis = 0.0;
            }

            *inertia_axis += (target_axis - *inertia_axis) * alpha;
            *inertia_axis = inertia_axis.clamp(-target_axis.abs(), target_axis.abs());
        }

        self.inertia
    }
}

impl CS2 {
    pub fn aimbot(&mut self, config: &Config, mouse: &mut Mouse) -> bool {
        let hotkey = config.aim.aimbot_hotkey;
        let config = self.aimbot_config(config);

        if !config.enabled {
            self.aim.reset_tracking();
            return false;
        }

        if !Self::check_hotkey(&self.input, config.mode, hotkey, &mut self.aim.active) {
            self.aim.reset_tracking();
            return false;
        }

        let Some(target) = &self.target.player else {
            self.aim.reset_tracking();
            return false;
        };

        if !target.is_valid(self) {
            self.aim.reset_tracking();
            return false;
        }

        let Some(local_player) = Player::local_player(self) else {
            self.aim.reset_tracking();
            return false;
        };

        let weapon_class = local_player.weapon_class(self);
        if matches!(
            weapon_class,
            WeaponClass::Unknown | WeaponClass::Knife | WeaponClass::Grenade
        ) {
            self.aim.reset_tracking();
            return false;
        }

        if config.flash_check && local_player.is_flashed(self) {
            return false;
        }

        if config.in_air_check && local_player.is_in_air(self) {
            return false;
        }

        let eye_pos = local_player.eye_position(self);
        let target_bone = if config
            .bones
            .iter()
            .any(|b| b.u64() == self.target.bone_index)
        {
            self.target.bone_index
        } else {
            config
                .bones
                .first()
                .map(|b| b.u64())
                .unwrap_or(Bones::Head.u64())
        };
        let bone_pos = target.bone_position(self, target_bone);

        if config.smoke_check && self.is_line_in_smoke(eye_pos, bone_pos) {
            return false;
        }

        if config.visibility_check && !target.visible(self, &local_player) {
            let target_bone_enum = config
                .bones
                .iter()
                .find(|b| b.u64() == target_bone)
                .cloned()
                .unwrap_or(Bones::Head);
            let has_armor = target.armor(self) > 0;
            if !config.through_walls
                || !self.can_penetrate_wall(eye_pos, bone_pos, target_bone_enum, has_armor, 1)
            {
                return false;
            }
        }

        if local_player.shots_fired(self) < config.start_bullet {
            return false;
        }

        let target_distance = eye_pos.distance(bone_pos).max(1.0);
        let target_angle =
            self.angle_to_target(&local_player, &bone_pos, &self.target.previous_aim_punch);

        let view_angles = local_player.view_angles(self);
        let current_fov = angles_to_fov(&view_angles, &target_angle);
        let Some(offset_units) = forward_ray_offset(target_distance, current_fov) else {
            self.aim.reset_tracking();
            return false;
        };

        if offset_units > config.fov {
            self.aim.reset_tracking();
            return false;
        }

        if config.deadzone > 0.0 && offset_units <= config.deadzone {
            self.aim.inertia *= 0.5;
            return false;
        }

        self.aim.track(target.pawn, current_fov);
        let tracking_time = self
            .aim
            .target_acquired_time
            .map_or(Duration::ZERO, |acquired| acquired.elapsed());
        if tracking_time < Duration::from_millis(config.reaction_time) {
            return true;
        }

        let mut aim_angles = view_angles - target_angle;
        vec2_clamp(&mut aim_angles);

        let randomized_smooth =
            (config.smooth - self.aim.smooth_random_factor * config.smooth_random).max(0.5);

        let effective_smooth = if config.humanize {
            let progress = (1.0 - (current_fov / self.aim.initial_fov.max(0.1))).clamp(0.0, 1.0);

            if config.curve > 0.0 && current_fov > 0.5 {
                let ortho = vec2(-aim_angles.y, aim_angles.x).normalize_or_zero();
                let bow = (std::f32::consts::PI * progress).sin()
                    * config.curve
                    * current_fov
                    * 0.15
                    * self.aim.curve_direction;
                aim_angles += ortho * bow;
            }

            if config.overshoot > 0.0 && self.aim.initial_fov > 1.2 {
                let overshoot_mod = 1.0
                    + config.overshoot
                        * 0.15
                        * (1.0 - progress)
                        * (std::f32::consts::PI * progress).sin();
                aim_angles *= overshoot_mod;
            }

            if config.tremor > 0.0 {
                let t = tracking_time.as_secs_f32() * 25.0;
                let tremor_x = (t * 11.3).sin() * (t * 7.7).cos();
                let tremor_y = (t * 13.1).cos() * (t * 9.5).sin();
                let scale = config.tremor * 0.08 * (current_fov / 3.0).clamp(0.05, 1.0);
                aim_angles += vec2(tremor_x, tremor_y) * scale;
            }

            vec2_clamp(&mut aim_angles);

            let accel_ramp = (tracking_time.as_secs_f32() / 0.05).clamp(0.2, 1.0);

            let ease_out_factor = 0.85 + 0.5 * progress;
            ((randomized_smooth * 2.5 * ease_out_factor + 1.0) / accel_ramp).clamp(1.0, 500.0)
        } else {
            (randomized_smooth * 2.5 + 1.0).clamp(1.0, 500.0)
        };

        let fov_mult = local_player.fov_multiplier(self);
        let fov_mult = if fov_mult <= 0.01 || fov_mult > 2.0 {
            1.0
        } else {
            fov_mult
        };
        let sensitivity = (self.get_sensitivity() * fov_mult).max(0.01);

        let mouse_angles = vec2(
            aim_angles.y / sensitivity * 45.45,
            -aim_angles.x / sensitivity * 45.45,
        ) / effective_smooth;

        let alpha = 1.0 - config.inertia.clamp(0.0, 1.0) * 0.5;
        mouse.move_rel(self.aim.smooth_delta(mouse_angles, alpha));

        self.recoil.previous = local_player.aim_punch(self);

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracking_same_target_does_not_restart_acquisition() {
        let mut aimbot = Aimbot::default();

        aimbot.track(10, 4.0);
        let acquired_at = aimbot.target_acquired_time;
        aimbot.track(10, 2.0);

        assert_eq!(aimbot.target_acquired_time, acquired_at);
        assert_eq!(aimbot.initial_fov, 4.0);
    }

    #[test]
    fn tracking_new_target_starts_fresh() {
        let mut aimbot = Aimbot::default();

        aimbot.track(10, 4.0);
        aimbot.track(20, 2.0);

        assert_eq!(aimbot.current_target_pawn, Some(20));
        assert_eq!(aimbot.initial_fov, 2.0);
        assert!(aimbot.target_acquired_time.is_some());
    }

    #[test]
    fn reset_clears_tracking_but_keeps_toggle_state() {
        let mut aimbot = Aimbot {
            active: true,
            ..Default::default()
        };
        aimbot.track(10, 4.0);

        aimbot.reset_tracking();

        assert!(aimbot.active);
        assert_eq!(aimbot.current_target_pawn, None);
        assert_eq!(aimbot.target_acquired_time, None);
        assert_eq!(aimbot.inertia, Vec2::ZERO);
    }

    #[test]
    fn inertia_never_continues_away_from_target() {
        let mut aimbot = Aimbot {
            inertia: vec2(4.0, -4.0),
            ..Default::default()
        };

        let delta = aimbot.smooth_delta(vec2(-2.0, 2.0), 0.5);

        assert_eq!(delta, vec2(-1.0, 1.0));
    }

    #[test]
    fn inertia_cannot_overshoot_remaining_delta() {
        let mut aimbot = Aimbot {
            inertia: Vec2::splat(10.0),
            ..Default::default()
        };

        let delta = aimbot.smooth_delta(Vec2::splat(1.0), 0.5);

        assert_eq!(delta, Vec2::ONE);
    }
}
