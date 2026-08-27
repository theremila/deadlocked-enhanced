use std::time::{Duration, Instant};

use rand::rng;

use crate::{
    config::Config,
    cs2::{
        CS2,
        accuracy::{WeaponAccuracy, meets_hitchance, view_basis},
        entity::{player::Player, weapon_class::WeaponClass},
        hitbox::{HitCapsule, HitSphere, capsules, spheres},
    },
    os::mouse::Mouse,
};

#[derive(Default)]
pub struct Triggerbot {
    shot_start: Option<Instant>,
    shot_end: Option<Instant>,
    pending_target: Option<usize>,
    pub active: bool,
}

struct TriggerTarget {
    pawn: usize,
    spheres: Vec<HitSphere>,
    capsules: Vec<HitCapsule>,
    score: f32,
}

impl CS2 {
    pub fn triggerbot(&mut self, config: &Config, mouse: &mut Mouse) {
        macro_rules! idle {
            () => {{
                self.trigger.shot_start = None;
                self.trigger.pending_target = None;
                if self.trigger.shot_end.take().is_some() {
                    mouse.left_release();
                }
                mouse.release_counter_strafe();
                return;
            }};
        }

        let hotkey = config.aim.triggerbot_hotkey;
        let config = self.triggerbot_config(config);

        if !config.enabled
            || !Self::check_hotkey(&self.input, config.mode, hotkey, &mut self.trigger.active)
        {
            idle!();
        }
        let firing = self.trigger.shot_end.is_some();

        let Some(local_player) = Player::local_player(self) else {
            idle!();
        };

        let weapon_class = local_player.weapon_class(self);
        if matches!(
            weapon_class,
            WeaponClass::Unknown | WeaponClass::Knife | WeaponClass::Grenade
        ) || (!firing && !local_player.weapon_ready(self))
        {
            idle!();
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
            idle!();
        }

        let eye_pos = local_player.eye_position(self);
        let view_angles = local_player.view_angles(self);
        let (view_direction, _, _) = view_basis(view_angles);
        let direct_target = local_player.crosshair_entity(self);
        let mut best: Option<TriggerTarget> = None;

        for player in &self.players {
            if (!self.is_ffa() && player.team(self) == local_player.team(self))
                || !player.is_valid(self)
                || direct_target.is_some_and(|target| target.pawn != player.pawn)
            {
                continue;
            }

            let required_damage = config.min_damage.min(player.health(self)).max(1) as f32;
            let is_direct_target = direct_target.is_some_and(|target| target.pawn == player.pawn);
            let hit_spheres = spheres(self, player, &config.bones, config.head_only);
            let hit_capsules = capsules(&hit_spheres);
            let closest_hit = hit_spheres
                .iter()
                .copied()
                .filter_map(|hit| {
                    let to_center = hit.center - eye_pos;
                    let projection = to_center.dot(view_direction);
                    if projection <= 0.0 {
                        return None;
                    }
                    let point = eye_pos + view_direction * projection;
                    let offset = point.distance(hit.center);
                    let allowed_offset = if config.prefer_center {
                        hit.radius * (config.center_tolerance / 100.0).clamp(0.01, 1.0)
                    } else {
                        hit.radius
                    };
                    (offset <= allowed_offset).then_some((hit, point, offset / hit.radius))
                })
                .min_by(|left, right| left.2.total_cmp(&right.2));
            let Some((hit, point, score)) = closest_hit else {
                continue;
            };
            if config.smoke_check && self.is_line_in_smoke(eye_pos, point) {
                continue;
            }

            let damage = if is_direct_target {
                self.calculate_direct_damage(
                    eye_pos,
                    point,
                    hit.bone,
                    player.armor(self),
                    player.has_helmet(self),
                )
            } else {
                let Some(path) = self.evaluate_shot_path(
                    &local_player,
                    player,
                    point,
                    hit.bone,
                    config.through_walls,
                    required_damage as i32,
                ) else {
                    continue;
                };
                path.damage
            };
            if damage < required_damage {
                continue;
            }

            let target = TriggerTarget {
                pawn: player.pawn,
                spheres: hit_spheres,
                capsules: hit_capsules,
                score,
            };
            if is_direct_target {
                best = Some(target);
                continue;
            }
            if best.as_ref().is_none_or(|best| target.score < best.score) {
                best = Some(target);
            }
        }

        let Some(target) = best else {
            idle!();
        };
        if firing {
            mouse.release_counter_strafe();
            return;
        }

        let now = Instant::now();
        if self.trigger.pending_target != Some(target.pawn) {
            let mean = (*config.delay.start() + *config.delay.end()) as f32 / 2.0;
            let std_dev = (*config.delay.end() - *config.delay.start()) as f32 / 2.0;
            let normal = rand_distr::Normal::new(mean, std_dev.max(f32::EPSILON)).unwrap();
            use rand_distr::Distribution as _;
            let delay = normal.sample(&mut rng()).max(0.0) as u64;
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
        if !meets_hitchance(
            eye_pos,
            view_angles,
            &target.spheres,
            &target.capsules,
            accuracy,
            config.hitchance,
        ) {
            mouse.release_counter_strafe();
            return;
        }

        if self
            .trigger
            .shot_start
            .is_some_and(|shot_time| now >= shot_time)
        {
            mouse.left_press();
            self.trigger.shot_start = None;
            self.trigger.pending_target = None;
            self.trigger.shot_end = Some(now + Duration::from_millis(config.shot_duration));
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
