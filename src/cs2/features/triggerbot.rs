use std::time::{Duration, Instant};

use glam::{Vec2, Vec3};
use rand::rng;

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
pub struct Triggerbot {
    shot_start: Option<Instant>,
    shot_end: Option<Instant>,
    pub active: bool,
}

fn bone_hitbox_radius(bone: Bones) -> f32 {
    match bone {
        Bones::Head => 4.5,
        Bones::Neck => 4.0,
        Bones::Spine4 | Bones::Spine3 | Bones::Spine2 | Bones::Spine1 => 8.5,
        Bones::Hip => 8.0,
        Bones::LeftShoulder
        | Bones::RightShoulder
        | Bones::LeftElbow
        | Bones::RightElbow
        | Bones::LeftHand
        | Bones::RightHand => 4.5,
        Bones::LeftHip
        | Bones::RightHip
        | Bones::LeftKnee
        | Bones::RightKnee => 5.0,
        Bones::LeftFoot | Bones::RightFoot => 4.0,
    }
}

impl CS2 {
    pub fn triggerbot(&mut self, config: &Config, mouse: &mut Mouse) {
        let hotkey = config.aim.triggerbot_hotkey;
        let config = self.triggerbot_config(config);

        if !config.enabled {
            return;
        }

        if !Self::check_hotkey(&self.input, config.mode, hotkey, &mut self.trigger.active) {
            return;
        }

        if self.trigger.shot_start.is_some() || self.trigger.shot_end.is_some() {
            return;
        }

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        if config.flash_check && local_player.is_flashed(self) {
            return;
        }

        if config.in_air_check && local_player.is_in_air(self) {
            return;
        }

        if config.scope_check
            && local_player.weapon_class(self) == WeaponClass::Sniper
            && !local_player.is_scoped(self)
        {
            return;
        }

        if config.velocity_check && local_player.velocity(self).length() > config.velocity_threshold
        {
            return;
        }

        let eye_pos = local_player.eye_position(self);
        let view_angles = local_player.view_angles(self);

        let mut target_found = false;
        let mut best_bone_pos = Vec3::ZERO;
        let mut best_target_bone = Bones::Head;
        let mut best_dist = 1.0;

        // 1. Direct crosshair entity check
        if let Some(crosshair_player) = local_player.crosshair_entity(self) {
            if self.is_ffa() || crosshair_player.team(self) != local_player.team(self) {
                let has_armor = crosshair_player.armor(self) > 0;
                for &bone in &config.bones {
                    if config.head_only && bone != Bones::Head {
                        continue;
                    }

                    let bone_pos = crosshair_player.bone_position(self, bone.u64());
                    let dist = eye_pos.distance(bone_pos).max(1.0);
                    let target_angle = self.angle_to_target(&local_player, &bone_pos, &Vec2::ZERO);
                    let fov = angles_to_fov(&view_angles, &target_angle);
                    let offset_units = dist * fov.to_radians().sin();
                    let bone_radius = bone_hitbox_radius(bone);

                    if offset_units <= bone_radius {
                        if config.smoke_check && self.is_line_in_smoke(eye_pos, bone_pos) {
                            continue;
                        }
                        if config.visibility_check {
                            let is_visible = crosshair_player.visible(self, &local_player);
                            if !is_visible {
                                if !config.through_walls
                                    || !self.can_penetrate_wall(
                                        eye_pos,
                                        bone_pos,
                                        bone,
                                        has_armor,
                                        config.min_damage,
                                    )
                                {
                                    continue;
                                }
                            }
                        }
                        target_found = true;
                        best_bone_pos = bone_pos;
                        best_target_bone = bone;
                        best_dist = dist;
                        break;
                    }
                }
            }
        }

        // 2. Full ray-trace scan across all entities and configured bones (handles Through Walls, AutoWall & exact hitbox rays)
        if !target_found {
            let mut smallest_offset = f32::MAX;

            for player in &self.players {
                if !self.is_ffa() && player.team(self) == local_player.team(self) {
                    continue;
                }
                if !player.is_valid(self) {
                    continue;
                }

                let has_armor = player.armor(self) > 0;
                let is_visible = player.visible(self, &local_player);

                if config.visibility_check && !is_visible && !config.through_walls {
                    continue;
                }

                for &bone in &config.bones {
                    if config.head_only && bone != Bones::Head {
                        continue;
                    }

                    let bone_pos = player.bone_position(self, bone.u64());
                    let dist = eye_pos.distance(bone_pos);
                    if dist < 1.0 {
                        continue;
                    }

                    let angle = self.angle_to_target(&local_player, &bone_pos, &Vec2::ZERO);
                    let fov = angles_to_fov(&view_angles, &angle);
                    let offset_units = dist * fov.to_radians().sin();
                    let bone_radius = bone_hitbox_radius(bone);

                    if offset_units <= bone_radius && offset_units < smallest_offset {
                        if config.smoke_check && self.is_line_in_smoke(eye_pos, bone_pos) {
                            continue;
                        }

                        if config.visibility_check && !is_visible {
                            if !self.can_penetrate_wall(
                                eye_pos,
                                bone_pos,
                                bone,
                                has_armor,
                                config.min_damage,
                            ) {
                                continue;
                            }
                        }

                        smallest_offset = offset_units;
                        target_found = true;
                        best_bone_pos = bone_pos;
                        best_target_bone = bone;
                        best_dist = dist;
                    }
                }
            }
        }

        if !target_found {
            return;
        }

        if config.prefer_center {
            let target_angle = self.angle_to_target(&local_player, &best_bone_pos, &Vec2::ZERO);
            let fov = angles_to_fov(&view_angles, &target_angle);
            let offset_units = best_dist * fov.to_radians().sin();
            let base_radius = bone_hitbox_radius(best_target_bone);
            let max_allowed_offset = base_radius * (config.center_tolerance / 100.0).clamp(0.01, 1.0);
            if offset_units > max_allowed_offset {
                return;
            }
        }

        let speed = local_player.velocity(self).length();
        let is_in_air = local_player.is_in_air(self);
        let is_scoped = local_player.is_scoped(self);

        // AutoStop: Apply active counter-strafe and hold fire until stopped
        if config.autostop && speed > 20.0 {
            let yaw = view_angles.y.to_radians();
            let vel = local_player.velocity(self);
            let forward_vel = vel.x * yaw.cos() + vel.y * yaw.sin();
            let side_vel = -vel.x * yaw.sin() + vel.y * yaw.cos();
            mouse.counter_strafe(forward_vel, side_vel);
            return;
        }

        // Hitchance calculation (spread cone probability)
        if config.hitchance > 0.0 {
            let base_inaccuracy = self.weapon.base_inaccuracy(is_scoped);
            let move_spread = if is_in_air {
                0.08
            } else {
                (speed / 250.0).clamp(0.0, 2.0) * 0.035
            };
            let total_inaccuracy = base_inaccuracy + move_spread;
            let spread_radius = best_dist * total_inaccuracy;
            let target_radius = bone_hitbox_radius(best_target_bone);

            let calculated_hitchance = (target_radius / (spread_radius + target_radius)).powi(2) * 100.0;
            if calculated_hitchance < config.hitchance {
                return;
            }
        }

        let mean = (*config.delay.start() + *config.delay.end()) as f32 / 2.0;
        let std_dev = (*config.delay.end() - *config.delay.start()) as f32 / 2.0;

        let normal = rand_distr::Normal::new(mean, std_dev).unwrap();
        use rand_distr::Distribution as _;
        let delay = normal.sample(&mut rng()).max(0.0) as u64;

        let now = Instant::now();
        let delay = Duration::from_millis(delay);
        self.trigger.shot_start = Some(now + delay);
        self.trigger.shot_end = Some(now + delay + Duration::from_millis(config.shot_duration));
    }

    pub fn triggerbot_shoot(&mut self, mouse: &mut Mouse) {
        let now = Instant::now();

        if let Some(shot_time) = self.trigger.shot_start
            && now >= shot_time
        {
            mouse.left_press();
            self.trigger.shot_start = None;
        }

        if let Some(shot_end) = self.trigger.shot_end
            && now >= shot_end
        {
            mouse.left_release();
            self.trigger.shot_end = None;
        }
    }
}
