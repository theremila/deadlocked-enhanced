use glam::{Vec2, Vec3};

use crate::{
    config::{Config, aim::TargetingMode},
    constants::cs2,
    cs2::{
        CS2,
        entity::{player::Player, weapon_class::WeaponClass},
        hitbox::{HitSphere, multipoints, spheres},
    },
    math::{angles_to_fov, forward_ray_offset},
};

#[derive(Clone, Copy)]
pub(crate) enum TargetMetric {
    Fov,
    Distance,
}

#[derive(Clone, Copy)]
pub(crate) struct RaycastHit {
    pub hitbox: HitSphere,
    pub point: Vec3,
    pub normalized_offset: f32,
}

pub(crate) fn target_metric(
    mode: TargetMetric,
    view_angles: Vec2,
    target_angle: Vec2,
    distance: f32,
) -> f32 {
    match mode {
        TargetMetric::Fov => angles_to_fov(&view_angles, &target_angle),
        TargetMetric::Distance => distance,
    }
}

pub(crate) fn raycast_hitboxes(
    eye: Vec3,
    direction: Vec3,
    hitboxes: &[HitSphere],
    radius_scale: f32,
) -> Option<RaycastHit> {
    hitboxes
        .iter()
        .copied()
        .filter_map(|hitbox| {
            let to_center = hitbox.center - eye;
            let projection = to_center.dot(direction);
            if projection <= 0.0 {
                return None;
            }
            let point = eye + direction * projection;
            let offset = point.distance(hitbox.center);
            let allowed = hitbox.radius * radius_scale.clamp(0.01, 1.0);
            (offset <= allowed).then_some(RaycastHit {
                hitbox,
                point,
                normalized_offset: offset / hitbox.radius,
            })
        })
        .min_by(|left, right| left.normalized_offset.total_cmp(&right.normalized_offset))
}

#[derive(Default)]
pub struct Target {
    pub player: Option<Player>,
    pub angle: Vec2,
    pub distance: f32,
    pub bone_index: u64,
    pub position: Vec3,
    pub local_pawn_index: u64,
    pub previous_aim_punch: Vec2,
    candidates: Vec<TargetCandidate>,
}

struct TargetCandidate {
    player: Player,
    angle: Vec2,
    distance: f32,
    bone_index: u64,
    bone: crate::cs2::bones::Bones,
    position: Vec3,
    metric: f32,
    allow_penetration: bool,
    min_damage: f32,
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
        let eye_position = local_player.eye_position(self);
        let mut candidates = std::mem::take(&mut self.target.candidates);
        candidates.clear();

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

            let player_pos = player.position(self);
            let approx_dist = eye_position.distance(player_pos);
            if approx_dist < 1.0 {
                continue;
            }

            let approx_angle = self.angle_to_target(&local_player, &player_pos, &aim_punch);
            let approx_fov = angles_to_fov(&view_angles, &approx_angle);
            if let Some(approx_offset) = forward_ray_offset(approx_dist, approx_fov)
                && approx_offset > max_fov_units + 120.0
            {
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
                for point in multipoints(hit, eye_position) {
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

                    let metric_mode = match aimbot_config.targeting_mode {
                        TargetingMode::Fov => TargetMetric::Fov,
                        TargetingMode::Distance => TargetMetric::Distance,
                    };
                    let metric = target_metric(metric_mode, view_angles, angle, distance);
                    candidates.push(TargetCandidate {
                        player: *player,
                        angle,
                        distance,
                        bone_index: hit.bone.u64(),
                        bone: hit.bone,
                        position: point,
                        metric,
                        allow_penetration,
                        min_damage: wall_min_damage,
                    });
                }
            }
        }

        candidates.sort_unstable_by(|left, right| left.metric.total_cmp(&right.metric));
        let selected = candidates.iter().find(|candidate| {
            self.evaluate_shot_path(
                &local_player,
                &candidate.player,
                candidate.position,
                candidate.bone,
                candidate.allow_penetration,
                1,
            )
            .is_some_and(|path| !path.penetrated || path.damage >= candidate.min_damage)
        });

        match selected {
            Some(candidate) => {
                self.target.player = Some(candidate.player);
                self.target.angle = candidate.angle;
                self.target.distance = candidate.distance;
                self.target.bone_index = candidate.bone_index;
                self.target.position = candidate.position;
            }
            None => self.target.clear_selection(),
        }
        self.target.candidates = candidates;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cs2::bones::Bones;

    #[test]
    fn raycast_selects_the_hitbox_closest_to_its_center() {
        let hitboxes = [
            HitSphere {
                center: Vec3::new(10.0, 1.5, 0.0),
                radius: 2.0,
                bone: Bones::Head,
            },
            HitSphere {
                center: Vec3::new(20.0, 0.5, 0.0),
                radius: 2.0,
                bone: Bones::Spine3,
            },
        ];

        let selected = raycast_hitboxes(Vec3::ZERO, Vec3::X, &hitboxes, 1.0).unwrap();
        assert_eq!(selected.hitbox.bone, Bones::Spine3);
    }

    #[test]
    fn fov_metric_ignores_distance() {
        let view = Vec2::ZERO;
        let angle = Vec2::new(0.0, 5.0);
        assert_eq!(
            target_metric(TargetMetric::Fov, view, angle, 10.0),
            target_metric(TargetMetric::Fov, view, angle, 10_000.0)
        );
    }
}
