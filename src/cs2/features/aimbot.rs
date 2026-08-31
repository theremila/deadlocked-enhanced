use std::time::{Duration, Instant};

use glam::{Vec2, vec2};

use crate::{
    config::Config,
    constants::timing,
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
    last_motion_update: Option<Instant>,
    pending_view_update: Option<Vec2>,
}

impl Aimbot {
    fn pause_motion(&mut self) {
        self.inertia = Vec2::ZERO;
        self.last_motion_update = None;
    }

    fn reset_tracking(&mut self) {
        self.pause_motion();
        self.current_target_pawn = None;
        self.target_acquired_time = None;
        self.initial_fov = 0.0;
    }

    fn track(&mut self, pawn: usize, current_fov: f32) {
        if self.current_target_pawn == Some(pawn) {
            return;
        }

        self.pause_motion();
        self.current_target_pawn = Some(pawn);
        self.target_acquired_time = Some(Instant::now());
        self.initial_fov = current_fov.max(0.1);
        self.curve_direction = if rand::random() { 1.0 } else { -1.0 };
        self.smooth_random_factor = rand::random();
    }

    fn smooth_velocity(&mut self, target: Vec2, alpha: f32) -> Vec2 {
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

    fn motion_timestep(&mut self, now: Instant) -> Duration {
        let elapsed = self
            .last_motion_update
            .replace(now)
            .map_or(timing::AIM_REFERENCE_INTERVAL, |last| {
                now.saturating_duration_since(last)
            });
        elapsed.min(timing::AIM_MAX_TIMESTEP)
    }

    pub(crate) fn seed_angles_applied(&mut self, current: Vec2) -> bool {
        let Some(previous) = self.pending_view_update else {
            return true;
        };
        if current == previous {
            return false;
        }
        self.pending_view_update = None;
        false
    }
}

fn time_scaled_alpha(reference_alpha: f32, step_ratio: f32) -> f32 {
    let alpha = reference_alpha.clamp(0.0, 1.0);
    if alpha == 0.0 || step_ratio <= 0.0 {
        return 0.0;
    }
    if alpha == 1.0 {
        return 1.0;
    }

    -(((-alpha).ln_1p() * step_ratio).exp_m1())
}

fn clamp_motion_to_remaining(mut motion: Vec2, remaining: Vec2) -> Vec2 {
    for axis in 0..2 {
        if motion[axis].signum() != remaining[axis].signum() {
            motion[axis] = 0.0;
        } else {
            motion[axis] = motion[axis].clamp(-remaining[axis].abs(), remaining[axis].abs());
        }
    }
    motion
}

impl CS2 {
    pub fn aimbot(&mut self, config: &Config, mouse: &mut Mouse) -> bool {
        let master_enabled = config.aim.global.aimbot.enabled;
        let trigger_wallbang = {
            let trigger = self.triggerbot_config(config);
            (
                config.aim.global.triggerbot.enabled && trigger.enabled && trigger.through_walls,
                trigger.min_damage,
                trigger.head_only,
                trigger.bones.clone(),
            )
        };
        let config = self.aimbot_config(config);

        if !master_enabled || !config.enabled {
            self.aim.active = false;
            self.aim.reset_tracking();
            return false;
        }
        self.aim.active = true;

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
            self.aim.pause_motion();
            return false;
        }

        if config.in_air_check && local_player.is_in_air(self) {
            self.aim.pause_motion();
            return false;
        }

        let eye_pos = local_player.eye_position(self);
        let target_bone = config
            .bones
            .iter()
            .copied()
            .find(|bone| bone.u64() == self.target.bone_index)
            .unwrap_or(Bones::Head);
        let target_point = self.target.position;

        if !target_point.is_finite()
            || (config.smoke_check && self.is_line_in_smoke(eye_pos, target_point))
        {
            self.aim.pause_motion();
            return false;
        }

        let trigger_allows_bone = trigger_wallbang.3.contains(&target_bone)
            && (!trigger_wallbang.2 || target_bone == Bones::Head);
        let allow_penetration = config.through_walls && trigger_wallbang.0 && trigger_allows_bone;
        let Some(path) = self.evaluate_shot_path(
            &local_player,
            target,
            target_point,
            target_bone,
            allow_penetration,
            1,
        ) else {
            self.aim.pause_motion();
            return false;
        };
        let min_damage = trigger_wallbang.1.min(target.health(self)).max(1) as f32;
        if path.penetrated && path.damage < min_damage {
            self.aim.pause_motion();
            return false;
        }

        if local_player.shots_fired(self) < config.start_bullet {
            self.aim.pause_motion();
            return false;
        }

        let target_distance = eye_pos.distance(target_point).max(1.0);
        let target_angle = self.angle_to_target(
            &local_player,
            &target_point,
            &self.target.previous_aim_punch,
        );

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
            self.aim.pause_motion();
            return false;
        }

        self.aim.track(target.pawn, current_fov);
        let tracking_time = self
            .aim
            .target_acquired_time
            .map_or(Duration::ZERO, |acquired| acquired.elapsed());
        if tracking_time < Duration::from_millis(config.reaction_time) {
            self.aim.pause_motion();
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

        let remaining_mouse_delta = vec2(
            aim_angles.y / sensitivity * 45.45,
            -aim_angles.x / sensitivity * 45.45,
        );

        let timestep = self.aim.motion_timestep(Instant::now());
        let timestep_seconds = timestep.as_secs_f32().max(f32::EPSILON);
        let step_ratio = timestep.as_secs_f32()
            / timing::AIM_REFERENCE_INTERVAL
                .as_secs_f32()
                .max(f32::EPSILON);

        let reference_fraction = 1.0 / effective_smooth;
        let movement_fraction = time_scaled_alpha(reference_fraction, step_ratio);
        let desired_velocity = remaining_mouse_delta * movement_fraction / timestep_seconds;

        let reference_inertia_alpha = 1.0 - config.inertia.clamp(0.0, 1.0) * 0.5;
        let inertia_alpha = time_scaled_alpha(reference_inertia_alpha, step_ratio);
        let motion = self.aim.smooth_velocity(desired_velocity, inertia_alpha) * timestep_seconds;
        if mouse.move_rel(clamp_motion_to_remaining(motion, remaining_mouse_delta)) {
            self.aim.pending_view_update = Some(view_angles);
        }

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
        assert_eq!(aimbot.last_motion_update, None);
    }

    #[test]
    fn inertia_never_continues_away_from_target() {
        let mut aimbot = Aimbot {
            inertia: vec2(4.0, -4.0),
            ..Default::default()
        };

        let delta = aimbot.smooth_velocity(vec2(-2.0, 2.0), 0.5);

        assert_eq!(delta, vec2(-1.0, 1.0));
    }

    #[test]
    fn inertia_cannot_overshoot_remaining_delta() {
        let mut aimbot = Aimbot {
            inertia: Vec2::splat(10.0),
            ..Default::default()
        };

        let delta = aimbot.smooth_velocity(Vec2::splat(1.0), 0.5);

        assert_eq!(delta, Vec2::ONE);
    }

    #[test]
    fn time_scaling_preserves_the_old_two_millisecond_curve() {
        let reference_fraction = 1.0 / 251.0;
        let half_step = time_scaled_alpha(reference_fraction, 0.5);
        let two_half_steps = 1.0 - (1.0 - half_step).powi(2);

        assert!((two_half_steps - reference_fraction).abs() < 1e-6);
    }

    #[test]
    fn time_scaling_is_stable_across_loop_frequencies() {
        let reference_fraction = 1.0 / 51.0;
        let quarter_step = time_scaled_alpha(reference_fraction, 0.25);
        let four_quarter_steps = 1.0 - (1.0 - quarter_step).powi(4);

        assert!((four_quarter_steps - reference_fraction).abs() < 1e-6);
    }
}
