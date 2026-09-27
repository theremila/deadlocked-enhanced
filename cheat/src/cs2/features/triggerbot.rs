use std::time::{Duration, Instant};

use rand::rng;
use shared::{TriggerStatus, WeaponClass};

use crate::{
    config::{
        Config,
        aim::{SeedMode, TriggerTargetingMode},
    },
    cs2::{
        CS2,
        accuracy::{WeaponAccuracy, meets_hitchance_translated, view_basis},
        entity::player::Player,
        extrapolation::{
            calculate_latency_seconds, extrapolate_hitboxes, extrapolation_translation,
        },
        hitbox::{ALL_BONES, HitCapsule, HitSphere, capsules, spheres},
        target::raycast_hitboxes,
    },
    math::angles_to_fov,
    os::mouse::Mouse,
};

pub struct Triggerbot {
    shot_start: Option<Instant>,
    shot_end: Option<Instant>,
    pending_target: Option<usize>,
    pub(crate) seed_tick_offset: i32,
    stable_seed_angles: Option<(glam::Vec2, i32)>,
    pub active: bool,
    pub status: TriggerStatus,
}

impl Default for Triggerbot {
    fn default() -> Self {
        Self {
            shot_start: None,
            shot_end: None,
            pending_target: None,
            seed_tick_offset: 0,
            stable_seed_angles: None,
            active: false,
            status: TriggerStatus::Inactive,
        }
    }
}

impl Triggerbot {
    fn seed_angles_stable(&mut self, angles: glam::Vec2, tick: i32) -> bool {
        match self.stable_seed_angles {
            Some((observed, observed_tick)) => {
                let delta = (observed - angles).abs();
                let same_bucket = super::seed_sync::quantize_angle(observed.x)
                    == super::seed_sync::quantize_angle(angles.x)
                    && super::seed_sync::quantize_angle(observed.y)
                        == super::seed_sync::quantize_angle(angles.y);
                if delta.x <= 0.05 && delta.y <= 0.05 && same_bucket {
                    observed_tick != tick
                } else {
                    self.stable_seed_angles = Some((angles, tick));
                    false
                }
            }
            None => {
                self.stable_seed_angles = Some((angles, tick));
                false
            }
        }
    }
}

struct TriggerTarget {
    player: Player,
    pawn: usize,
    spheres: Vec<HitSphere>,
    capsules: Vec<HitCapsule>,
    translation: glam::Vec3,
    score: f32,
    preferred: bool,
    required_damage: i32,
    fallback_valid: bool,
}

impl CS2 {
    pub(crate) fn apply_autostop(
        &self,
        mouse: &mut Mouse,
        view_angles: glam::Vec2,
        velocity: glam::Vec3,
        threshold: f32,
    ) {
        let yaw = view_angles.y.to_radians();
        let forward_vel = velocity.x * yaw.cos() + velocity.y * yaw.sin();
        let side_vel = -velocity.x * yaw.sin() + velocity.y * yaw.cos();
        let movement_keys = crate::cs2::key_codes::MovementKeys::detect(&self.input);

        // Counter-strafe via uinput hardware simulation without writing to game memory:
        mouse.counter_strafe_dynamic(forward_vel, side_vel, threshold, movement_keys);
    }

    pub fn triggerbot(&mut self, config: &Config, mouse: &mut Mouse) {
        macro_rules! idle {
            ($status:expr) => {{
                self.trigger.status = $status;
                self.trigger.shot_start = None;
                self.trigger.pending_target = None;
                if self.trigger.shot_end.take().is_some() {
                    mouse.left_release();
                }
                mouse.release_counter_strafe();
                return;
            }};
        }

        let master_enabled = config.aim.global.triggerbot.enabled;
        let config = self.triggerbot_config(config);

        self.trigger.active = master_enabled && config.enabled;
        if !self.trigger.active {
            idle!(TriggerStatus::Inactive);
        }
        let firing = self.trigger.shot_end.is_some();

        let Some(local_player) = Player::local_player(self) else {
            idle!(TriggerStatus::NoTarget);
        };
        if !local_player.is_valid(self) || local_player.health(self) <= 0 {
            idle!(TriggerStatus::Inactive);
        }

        let weapon_class = local_player.weapon_class(self);
        if matches!(
            weapon_class,
            WeaponClass::Unknown | WeaponClass::Knife | WeaponClass::Grenade
        ) {
            idle!(TriggerStatus::ChecksBlocked);
        }

        let weapon_ready = local_player.weapon_ready(self);
        if !firing && !weapon_ready {
            if config.autostop
                && config.autostop_between_shots
                && self.trigger.pending_target.is_some()
            {
                let velocity = local_player.velocity(self);
                let speed = velocity.length();
                let scoped = local_player.is_scoped(self);
                let live_accuracy = self.live_weapon_accuracy(&local_player);
                let max_speed = live_accuracy
                    .map(|accuracy| accuracy.max_speed)
                    .unwrap_or_else(|| self.weapon.max_speed(scoped));
                let stop_speed = max_speed * 0.34;
                let in_air = local_player.is_in_air(self);
                let allow_stop = !in_air || config.autostop_in_air;
                if allow_stop {
                    self.trigger.status = TriggerStatus::AutoStop;
                    let view_angles = local_player.view_angles(self);
                    if speed > stop_speed {
                        self.apply_autostop(mouse, view_angles, velocity, 2.0);
                    } else {
                        mouse.release_counter_strafe();
                    }
                    return;
                }
            } else {
                mouse.release_counter_strafe();
            }
            idle!(TriggerStatus::ChecksBlocked);
        }

        let seed_active = config.seed_mode != SeedMode::Off;
        if (config.flash_check && local_player.is_flashed(self))
            || (!seed_active && config.in_air_check && local_player.is_in_air(self))
            || (config.scope_check
                && weapon_class == WeaponClass::Sniper
                && !local_player.is_scoped(self))
            || (!seed_active
                && config.velocity_check
                && !config.autostop
                && local_player.velocity(self).length() > config.velocity_threshold)
        {
            idle!(TriggerStatus::ChecksBlocked);
        }

        let eye_pos = local_player.eye_position(self);
        let view_angles = local_player.view_angles(self);
        let (view_direction, _, _) = view_basis(view_angles);
        let direct_target = local_player.crosshair_entity(self);
        let preferred_pawn = (config.prefer_aim_target && self.aim.active)
            .then(|| self.target.player.map(|target| target.pawn))
            .flatten();
        let local_team = local_player.team(self);
        let is_ffa = self.is_ffa();
        let mut best: Option<TriggerTarget> = None;

        let latency = if config.auto_extrapolation {
            calculate_latency_seconds(self, &local_player)
        } else {
            0.0
        };

        for player in &self.players {
            let is_preferred = preferred_pawn == Some(player.pawn);
            if (config.targeting_mode == TriggerTargetingMode::Raycast
                && !is_preferred
                && direct_target.is_some_and(|target| target.pawn != player.pawn))
                || (!is_ffa && player.team(self) == local_team)
                || !player.is_valid(self)
            {
                continue;
            }

            let required_damage = config.min_damage.min(player.health(self)).max(1) as f32;
            let is_direct_target = config.targeting_mode == TriggerTargetingMode::Raycast
                && direct_target.is_some_and(|target| target.pawn == player.pawn);

            let bones_to_scan = if seed_active {
                &ALL_BONES[..]
            } else {
                &config.bones[..]
            };
            let head_only = if seed_active { false } else { config.head_only };
            let mut hit_spheres = spheres(self, player, bones_to_scan, head_only);
            if config.prefer_center {
                let radius_scale = (config.center_tolerance / 100.0).clamp(0.01, 1.0);
                for hitbox in &mut hit_spheres {
                    hitbox.radius *= radius_scale;
                }
            }
            let hit_capsules = capsules(&hit_spheres);

            let target_translation = if config.auto_extrapolation {
                extrapolation_translation(
                    player.velocity(self),
                    player.is_in_air(self),
                    latency,
                )
            } else {
                glam::Vec3::ZERO
            };

            let (eval_spheres, _eval_capsules) = if target_translation != glam::Vec3::ZERO {
                extrapolate_hitboxes(&hit_spheres, &hit_capsules, target_translation)
            } else {
                (hit_spheres.clone(), hit_capsules.clone())
            };

            let closest_fov_hit = |enforce_limit: bool| {
                eval_spheres
                    .iter()
                    .copied()
                    .filter_map(|hit| {
                        let distance = eye_pos.distance(hit.center);
                        if distance < 0.1 {
                            return Some((hit, eye_pos, 0.0));
                        }
                        let angle =
                            self.angle_to_target(&local_player, &hit.center, &glam::Vec2::ZERO);
                        let fov_degrees = angles_to_fov(&view_angles, &angle);
                        (!enforce_limit || fov_degrees <= config.fov).then_some((
                            hit,
                            hit.center,
                            fov_degrees,
                        ))
                    })
                    .min_by(|left, right| left.2.total_cmp(&right.2))
            };
            let closest_hit = if is_preferred {
                closest_fov_hit(false)
            } else {
                match config.targeting_mode {
                    TriggerTargetingMode::Raycast => {
                        raycast_hitboxes(eye_pos, view_direction, &hit_spheres, 1.0)
                            .or_else(|| raycast_hitboxes(eye_pos, view_direction, &eval_spheres, 1.0))
                            .map(|raycast| (raycast.hitbox, raycast.point, raycast.normalized_offset))
                    }
                    TriggerTargetingMode::Fov => closest_fov_hit(true),
                }
            };
            let Some((hit, point, score)) = closest_hit else {
                continue;
            };
            if config.smoke_check && self.is_line_in_smoke(eye_pos, point) {
                continue;
            }

            let fallback_damage = if is_direct_target && target_translation == glam::Vec3::ZERO {
                Some(self.calculate_direct_damage(
                    eye_pos,
                    point,
                    hit.bone,
                    player.armor(self),
                    player.has_helmet(self),
                ))
            } else {
                self.evaluate_shot_path(
                    &local_player,
                    player,
                    point,
                    hit.bone,
                    config.through_walls,
                    required_damage as i32,
                )
                .map(|path| path.damage)
            };
            let fallback_valid = fallback_damage.is_some_and(|damage| damage >= required_damage);
            if !fallback_valid && !seed_active {
                continue;
            }

            let target = TriggerTarget {
                player: *player,
                pawn: *player.pawn,
                spheres: hit_spheres,
                capsules: hit_capsules,
                translation: target_translation,
                score,
                preferred: is_preferred,
                required_damage: required_damage as i32,
                fallback_valid,
            };
            if best.as_ref().is_none_or(|best| {
                (target.preferred && !best.preferred)
                    || (target.preferred == best.preferred && target.score < best.score)
            }) {
                best = Some(target);
            }
        }

        let Some(target) = best else {
            idle!(TriggerStatus::NoTarget);
        };
        if firing {
            self.trigger.status = TriggerStatus::Firing;
            mouse.release_counter_strafe();
            return;
        }

        let now = Instant::now();
        if self.trigger.pending_target != Some(target.pawn) {
            let delay = if config.seed_mode != SeedMode::Off {
                0
            } else {
                let mean = (*config.delay.start() + *config.delay.end()) as f32 / 2.0;
                let std_dev = (*config.delay.end() - *config.delay.start()) as f32 / 2.0;
                let normal = rand_distr::Normal::new(mean, std_dev.max(f32::EPSILON)).unwrap();
                use rand_distr::Distribution as _;
                normal.sample(&mut rng()).max(0.0) as u64
            };
            self.trigger.pending_target = Some(target.pawn);
            self.trigger.shot_start = Some(now + Duration::from_millis(delay));
        }

        let velocity = local_player.velocity(self);
        let speed = velocity.length();
        let scoped = local_player.is_scoped(self);
        let live_accuracy = self.live_weapon_accuracy(&local_player);
        let max_speed = live_accuracy
            .map(|accuracy| accuracy.max_speed)
            .unwrap_or_else(|| self.weapon.max_speed(scoped));
        let stop_speed = max_speed * 0.34;
        let in_air = local_player.is_in_air(self);
        let allow_autostop = config.autostop && (!in_air || config.autostop_in_air);

        let stationary_accuracy = WeaponAccuracy {
            inaccuracy: self.weapon.base_inaccuracy(scoped),
            spread: self.weapon.base_spread(),
            max_speed,
        };

        let stationary_viable = if config.seed_mode == SeedMode::Off {
            meets_hitchance_translated(
                eye_pos,
                view_angles,
                &target.spheres,
                &target.capsules,
                target.translation,
                stationary_accuracy,
                config.hitchance,
            )
        } else {
            matches!(
                self.seed_prediction(
                    &local_player,
                    stationary_accuracy,
                    super::seed_sync::SeedTarget {
                        player: &target.player,
                        spheres: &target.spheres,
                        capsules: &target.capsules,
                        min_damage: target.required_damage,
                    },
                    super::seed_sync::SeedPredictionOptions {
                        allow_penetration: config.through_walls,
                        smoke_check: config.smoke_check,
                        tick_offset: self.trigger.seed_tick_offset,
                        prediction_ticks: super::seed_sync::PREDICTION_TICKS,
                    },
                ),
                super::seed_sync::SeedPrediction::Ready(_)
            )
        };

        let delay_active = self.trigger.shot_start.is_some_and(|shot_time| now < shot_time);
        let shot_imminent = if delay_active {
            let time_until_shot = self
                .trigger
                .shot_start
                .map(|t| t.saturating_duration_since(now))
                .unwrap_or_default();
            config.autostop_early && time_until_shot <= Duration::from_millis(35)
        } else {
            true
        };

        if allow_autostop && stationary_viable && shot_imminent && speed > stop_speed {
            self.apply_autostop(mouse, view_angles, velocity, 2.0);
            self.trigger.status = TriggerStatus::AutoStop;
            return;
        } else {
            mouse.release_counter_strafe();
        }

        let accuracy = live_accuracy.unwrap_or_else(|| {
            let movement = if in_air {
                0.08
            } else {
                (speed / max_speed).clamp(0.0, 2.0) * 0.035
            };
            WeaponAccuracy {
                inaccuracy: self.weapon.base_inaccuracy(scoped) + movement,
                spread: self.weapon.base_spread(),
                max_speed,
            }
        });
        let hitchance = || {
            let meets_hitchance = meets_hitchance_translated(
                eye_pos,
                view_angles,
                &target.spheres,
                &target.capsules,
                target.translation,
                accuracy,
                config.hitchance,
            );
            (meets_hitchance, TriggerStatus::HitchanceMiss)
        };
        let mut seed_snapshot = None;
        let seed_angles_ready = config.seed_mode != SeedMode::Off
            && (self.aim.seed_angles_applied(view_angles) || !self.aim.active)
            && local_player
                .tick_base(self)
                .is_some_and(|tick| self.trigger.seed_angles_stable(view_angles, tick));
        let (accurate, status) = if config.seed_mode == SeedMode::Off {
            self.trigger.stable_seed_angles = None;
            hitchance()
        } else if !seed_angles_ready {
            (false, TriggerStatus::SeedUnstable)
        } else {
            let prediction_ticks = super::seed_sync::PREDICTION_TICKS;
            match self.seed_prediction(
                &local_player,
                accuracy,
                super::seed_sync::SeedTarget {
                    player: &target.player,
                    spheres: &target.spheres,
                    capsules: &target.capsules,
                    min_damage: target.required_damage,
                },
                super::seed_sync::SeedPredictionOptions {
                    allow_penetration: config.through_walls,
                    smoke_check: config.smoke_check,
                    tick_offset: self.trigger.seed_tick_offset,
                    prediction_ticks,
                },
            ) {
                super::seed_sync::SeedPrediction::Ready(success) => {
                    seed_snapshot = Some(success.snapshot);
                    (true, TriggerStatus::SeedReady)
                }
                super::seed_sync::SeedPrediction::Miss => (false, TriggerStatus::SeedMiss),
                super::seed_sync::SeedPrediction::Unavailable
                    if config.seed_mode == SeedMode::WhenAvailable && target.fallback_valid =>
                {
                    let (accurate, _) = hitchance();
                    (accurate, TriggerStatus::FallbackHitchance)
                }
                super::seed_sync::SeedPrediction::Unavailable => {
                    (false, TriggerStatus::SeedUnavailable)
                }
            }
        };
        self.trigger.status = status;
        if !accurate {
            mouse.release_counter_strafe();
            return;
        }

        if self
            .trigger
            .shot_start
            .is_some_and(|shot_time| now >= shot_time)
        {
            if let Some(snapshot) = seed_snapshot {
                let current_angles = local_player.view_angles(self);
                let Some(current_tick) = local_player.tick_base(self) else {
                    mouse.release_counter_strafe();
                    self.trigger.status = TriggerStatus::SeedUnavailable;
                    return;
                };
                if !snapshot.is_current(current_angles, current_tick) {
                    mouse.release_counter_strafe();
                    self.trigger.status = TriggerStatus::SeedUnstable;
                    return;
                }
            }
            mouse.left_press();
            self.trigger.status = TriggerStatus::Firing;
            self.trigger.shot_start = None;
            self.trigger.pending_target = None;
            self.trigger.shot_end = Some(now + Duration::from_millis(config.shot_duration));
        } else if self.trigger.shot_start.is_some() {
            self.trigger.status = TriggerStatus::Delay;
        }
    }

    pub fn release_trigger_shot(&mut self, mouse: &mut Mouse) {
        let now = Instant::now();

        if let Some(shot_end) = self.trigger.shot_end
            && now >= shot_end
        {
            mouse.left_release();
            self.trigger.shot_end = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_angle_must_remain_unchanged_across_a_tick() {
        let mut trigger = Triggerbot::default();
        let angles = glam::Vec2::new(10.0, 20.0);

        assert!(!trigger.seed_angles_stable(angles, 100));
        assert!(!trigger.seed_angles_stable(angles, 100));
        assert!(trigger.seed_angles_stable(angles, 101));
        assert!(!trigger.seed_angles_stable(angles + glam::Vec2::X, 101));
        assert!(trigger.seed_angles_stable(angles + glam::Vec2::X, 102));
    }

    #[test]
    fn seed_angle_tolerates_micro_jitter_in_same_bucket() {
        let mut trigger = Triggerbot::default();
        let angles = glam::Vec2::new(10.1, 20.1);

        assert!(!trigger.seed_angles_stable(angles, 100));
        let jittered = glam::Vec2::new(10.12, 20.11);
        assert!(trigger.seed_angles_stable(jittered, 101));
    }

    #[test]
    fn seed_angle_rejects_jitter_crossing_quantization_boundary() {
        let mut trigger = Triggerbot::default();
        let angles = glam::Vec2::new(10.49, 20.0);

        assert!(!trigger.seed_angles_stable(angles, 100));
        let crossed = glam::Vec2::new(10.51, 20.0);
        assert!(!trigger.seed_angles_stable(crossed, 101));
    }

    #[test]
    fn autostop_counter_strafe_direction_vectors() {
        let yaw = 0.0_f32.to_radians();
        let vel = glam::Vec3::new(100.0, 0.0, 0.0);
        let forward_vel = vel.x * yaw.cos() + vel.y * yaw.sin();
        let side_vel = -vel.x * yaw.sin() + vel.y * yaw.cos();
        assert!(forward_vel > 5.0);
        assert_eq!(side_vel.abs(), 0.0);

        // Velocity in +Y (North / Left in Source 2):
        let vel_left = glam::Vec3::new(0.0, 100.0, 0.0);
        let forward_vel = vel_left.x * yaw.cos() + vel_left.y * yaw.sin();
        let side_vel = -vel_left.x * yaw.sin() + vel_left.y * yaw.cos();
        assert_eq!(forward_vel.abs(), 0.0);
        assert!(side_vel > 5.0, "positive side_vel indicates movement to the left");

        // Velocity in -Y (South / Right in Source 2):
        let vel_right = glam::Vec3::new(0.0, -100.0, 0.0);
        let forward_vel = vel_right.x * yaw.cos() + vel_right.y * yaw.sin();
        let side_vel = -vel_right.x * yaw.sin() + vel_right.y * yaw.cos();
        assert_eq!(forward_vel.abs(), 0.0);
        assert!(side_vel < -5.0, "negative side_vel indicates movement to the right");
    }
}
