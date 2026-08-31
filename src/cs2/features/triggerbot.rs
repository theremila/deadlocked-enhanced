use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

use rand::rng;

use crate::{
    config::{
        Config,
        aim::{SeedMode, TriggerTargetingMode},
    },
    cs2::{
        CS2,
        accuracy::{WeaponAccuracy, meets_hitchance, view_basis},
        entity::{player::Player, weapon_class::WeaponClass},
        hitbox::{HitCapsule, HitSphere, capsules, spheres},
        target::raycast_hitboxes,
    },
    data::TriggerStatus,
    math::{angles_to_fov, forward_ray_offset},
    os::mouse::Mouse,
};

pub struct Triggerbot {
    shot_start: Option<Instant>,
    shot_end: Option<Instant>,
    pending_target: Option<usize>,
    pending_seed_shot: Option<PendingSeedShot>,
    seed_tick_delays: VecDeque<i32>,
    pub(crate) seed_tick_offset: i32,
    seed_timing_calibrated: bool,
    stable_seed_angles: Option<(glam::Vec2, i32)>,
    pub active: bool,
    pub status: TriggerStatus,
}

#[derive(Clone, Copy)]
struct PendingSeedShot {
    marker: super::seed_sync::SeedShotMarker,
    started_at: Instant,
}

impl Default for Triggerbot {
    fn default() -> Self {
        Self {
            shot_start: None,
            shot_end: None,
            pending_target: None,
            pending_seed_shot: None,
            seed_tick_delays: VecDeque::with_capacity(9),
            seed_tick_offset: -1,
            seed_timing_calibrated: false,
            stable_seed_angles: None,
            active: false,
            status: TriggerStatus::Inactive,
        }
    }
}

impl Triggerbot {
    fn seed_angles_stable(&mut self, angles: glam::Vec2, tick: i32) -> bool {
        match self.stable_seed_angles {
            Some((observed, observed_tick)) if observed == angles => observed_tick != tick,
            _ => {
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
    score: f32,
    preferred: bool,
    required_damage: i32,
    fallback_valid: bool,
}

impl CS2 {
    fn update_seed_calibration(&mut self) {
        let Some(pending) = self.trigger.pending_seed_shot else {
            return;
        };
        if pending.started_at.elapsed() > Duration::from_millis(500) {
            self.trigger.pending_seed_shot = None;
            return;
        }

        let Some(local) = Player::local_player(self) else {
            return;
        };
        let Some(current) = self.seed_shot_marker(&local) else {
            return;
        };
        if current.weapon != pending.marker.weapon {
            self.trigger.pending_seed_shot = None;
            return;
        }
        if current.clip_ammo >= pending.marker.clip_ammo
            && current.recoil_index <= pending.marker.recoil_index
        {
            return;
        }

        let delay = (current.tick - pending.marker.tick).clamp(0, 6);
        if self.trigger.seed_tick_delays.len() == 9 {
            self.trigger.seed_tick_delays.pop_front();
        }
        self.trigger.seed_tick_delays.push_back(delay);
        self.trigger.pending_seed_shot = None;

        if self.trigger.seed_tick_delays.len() < 3 {
            return;
        }
        let mut delays: Vec<_> = self.trigger.seed_tick_delays.iter().copied().collect();
        delays.sort_unstable();
        let offset = (delays[delays.len() / 2] - 1).clamp(-1, 3);
        if !self.trigger.seed_timing_calibrated || offset != self.trigger.seed_tick_offset {
            utils::info!(
                "seed trigger timing calibrated: window {}..{} ({} samples)",
                offset,
                offset + super::seed_sync::PREDICTION_TICKS - 1,
                delays.len()
            );
        }
        self.trigger.seed_tick_offset = offset;
        self.trigger.seed_timing_calibrated = true;
    }

    pub fn triggerbot(&mut self, config: &Config, mouse: &mut Mouse) {
        self.update_seed_calibration();

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

        let weapon_class = local_player.weapon_class(self);
        if matches!(
            weapon_class,
            WeaponClass::Unknown | WeaponClass::Knife | WeaponClass::Grenade
        ) || (!firing && !local_player.weapon_ready(self))
        {
            idle!(TriggerStatus::ChecksBlocked);
        }

        if (config.flash_check && local_player.is_flashed(self))
            || (config.in_air_check && local_player.is_in_air(self))
            || (config.scope_check
                && weapon_class == WeaponClass::Sniper
                && !local_player.is_scoped(self))
            || (config.velocity_check
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

            let mut hit_spheres = spheres(self, player, &config.bones, config.head_only);
            if config.prefer_center {
                let radius_scale = (config.center_tolerance / 100.0).clamp(0.01, 1.0);
                for hitbox in &mut hit_spheres {
                    hitbox.radius *= radius_scale;
                }
            }
            let hit_capsules = capsules(&hit_spheres);
            let closest_fov_hit = |enforce_limit: bool| {
                hit_spheres
                    .iter()
                    .copied()
                    .filter_map(|hit| {
                        let distance = eye_pos.distance(hit.center);
                        if distance < 1.0 {
                            return None;
                        }
                        let angle =
                            self.angle_to_target(&local_player, &hit.center, &glam::Vec2::ZERO);
                        let fov_degrees = angles_to_fov(&view_angles, &angle);
                        let offset = forward_ray_offset(distance, fov_degrees)?;
                        (!enforce_limit || offset <= config.fov).then_some((
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
                        raycast_hitboxes(eye_pos, view_direction, &hit_spheres, 1.0).map(
                            |raycast| (raycast.hitbox, raycast.point, raycast.normalized_offset),
                        )
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

            let fallback_damage = if is_direct_target {
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
            if !fallback_valid && config.seed_mode == SeedMode::Off {
                continue;
            }

            let target = TriggerTarget {
                player: *player,
                pawn: player.pawn,
                spheres: hit_spheres,
                capsules: hit_capsules,
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
        if config.autostop && speed > stop_speed {
            self.trigger.status = TriggerStatus::AutoStop;
            let yaw = view_angles.y.to_radians();
            let forward_vel = velocity.x * yaw.cos() + velocity.y * yaw.sin();
            let side_vel = -velocity.x * yaw.sin() + velocity.y * yaw.cos();
            mouse.counter_strafe(forward_vel, side_vel, 5.0);
            return;
        }
        mouse.release_counter_strafe();

        let accuracy = live_accuracy.unwrap_or_else(|| {
            let movement = if local_player.is_in_air(self) {
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
            let meets_hitchance = meets_hitchance(
                eye_pos,
                view_angles,
                &target.spheres,
                &target.capsules,
                accuracy,
                config.hitchance,
            );
            (meets_hitchance, TriggerStatus::HitchanceMiss)
        };
        let mut seed_snapshot = None;
        let seed_angles_ready = config.seed_mode != SeedMode::Off
            && self.aim.seed_angles_applied(view_angles)
            && local_player
                .tick_base(self)
                .is_some_and(|tick| self.trigger.seed_angles_stable(view_angles, tick));
        let (accurate, status) = if config.seed_mode == SeedMode::Off {
            self.trigger.stable_seed_angles = None;
            hitchance()
        } else if !seed_angles_ready {
            (false, TriggerStatus::SeedUnstable)
        } else {
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
                    prediction_ticks: super::seed_sync::PREDICTION_TICKS,
                },
            ) {
                super::seed_sync::SeedPrediction::Ready(snapshot) => {
                    seed_snapshot = Some(snapshot);
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
                    self.trigger.status = TriggerStatus::SeedUnavailable;
                    return;
                };
                if !snapshot.is_current(current_angles, current_tick) {
                    self.trigger.status = TriggerStatus::SeedUnstable;
                    return;
                }
                self.trigger.pending_seed_shot =
                    self.seed_shot_marker(&local_player)
                        .map(|marker| PendingSeedShot {
                            marker,
                            started_at: now,
                        });
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
}
