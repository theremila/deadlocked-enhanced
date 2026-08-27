use glam::{Vec2, Vec3};

use crate::{
    config::{Config, aim::TargetingMode},
    constants::cs2,
    cs2::{
        CS2,
        entity::{player::Player, weapon_class::WeaponClass},
        hitbox::{multipoints, spheres},
    },
    math::{angles_to_fov, forward_ray_offset},
};

#[derive(Default)]
pub struct Target {
    pub player: Option<Player>,
    pub angle: Vec2,
    pub distance: f32,
    pub bone_index: u64,
    pub position: Vec3,
    pub local_pawn_index: u64,
    pub previous_aim_punch: Vec2,
}

impl Target {
    fn clear_selection(&mut self) {
        self.player = None;
        self.angle = Vec2::ZERO;
        self.distance = 0.0;
        self.bone_index = 0;
        self.position = Vec3::ZERO;
    }
}

impl CS2 {
    pub fn find_target(&mut self, config: &Config) {
        let Some(local_player) = Player::local_player(self) else {
            self.target.clear_selection();
            return;
        };

        let team = local_player.team(self);
        if team != cs2::TEAM_CT && team != cs2::TEAM_T {
            self.target.clear_selection();
            return;
        }

        let weapon_class = local_player.weapon_class(self);

        let view_angles = local_player.view_angles(self);
        let ffa = self.is_ffa();
        let shots_fired = local_player.shots_fired(self);
        let aim_punch = match (weapon_class, local_player.aim_punch(self) * 2.0) {
            (WeaponClass::Sniper, _) => Vec2::ZERO,
            (_, punch) if punch.length() == 0.0 && shots_fired > 1 => {
                self.target.previous_aim_punch
            }
            (_, punch) => punch,
        };
        self.target.previous_aim_punch = aim_punch;

        let aimbot_config = self.aimbot_config(config);
        let triggerbot_config = self.triggerbot_config(config);
        if !aimbot_config.enabled || aimbot_config.bones.is_empty() {
            self.target.clear_selection();
            return;
        }

        let max_fov_units = aimbot_config.fov;
        let mut best_metric = f32::MAX;
        let mut best_target = None;
        let eye_position = local_player.eye_position(self);

        if self.players.is_empty() {
            self.target.clear_selection();
            return;
        }

        let target_friendlies = aimbot_config.target_friendlies;

        for player in &self.players {
            if !player.is_valid(self) {
                continue;
            }

            if !(ffa || target_friendlies) && team == player.team(self) {
                continue;
            }

            for hit in spheres(self, player, &aimbot_config.bones, false) {
                let trigger_allows_bone = triggerbot_config.bones.contains(&hit.bone)
                    && (!triggerbot_config.head_only || hit.bone == crate::cs2::bones::Bones::Head);
                let allow_penetration = aimbot_config.through_walls
                    && triggerbot_config.enabled
                    && triggerbot_config.through_walls
                    && trigger_allows_bone;
                let wall_min_damage =
                    triggerbot_config.min_damage.min(player.health(self)).max(1) as f32;
                let mut bone_candidate = None;

                for (point_index, point) in multipoints(hit, eye_position).into_iter().enumerate() {
                    let distance = eye_position.distance(point);
                    if distance < 1.0
                        || (aimbot_config.smoke_check && self.is_line_in_smoke(eye_position, point))
                    {
                        continue;
                    }
                    let angle = self.angle_to_target(&local_player, &point, &aim_punch);
                    let fov_deg = angles_to_fov(&view_angles, &angle);
                    let Some(offset_units) = forward_ray_offset(distance, fov_deg) else {
                        continue;
                    };
                    if offset_units > max_fov_units {
                        continue;
                    }

                    let Some(path) = self.evaluate_shot_path(
                        &local_player,
                        player,
                        point,
                        hit.bone,
                        allow_penetration,
                        1,
                    ) else {
                        continue;
                    };
                    if path.penetrated && path.damage < wall_min_damage {
                        continue;
                    }

                    let metric = match &aimbot_config.targeting_mode {
                        TargetingMode::Fov => fov_deg,
                        TargetingMode::Distance => distance,
                    };
                    if point_index == 0 {
                        bone_candidate = Some((metric, angle, distance, point));
                        break;
                    }
                    if bone_candidate
                        .as_ref()
                        .is_none_or(|(best, _, _, _)| metric < *best)
                    {
                        bone_candidate = Some((metric, angle, distance, point));
                    }
                }

                let Some((metric, angle, distance, point)) = bone_candidate else {
                    continue;
                };
                if metric < best_metric {
                    best_metric = metric;
                    best_target = Some((*player, angle, distance, hit.bone.u64(), point));
                }
            }
        }

        if let Some((player, angle, distance, bone_index, position)) = best_target {
            self.target.player = Some(player);
            self.target.angle = angle;
            self.target.distance = distance;
            self.target.bone_index = bone_index;
            self.target.position = position;
        } else {
            self.target.clear_selection();
        }
    }
}
