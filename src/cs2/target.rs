use glam::Vec2;

use crate::{
    config::{Config, aim::TargetingMode},
    constants::cs2,
    cs2::{
        CS2,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::angles_to_fov,
};

#[derive(Default)]
pub struct Target {
    pub player: Option<Player>,
    pub angle: Vec2,
    pub distance: f32,
    pub bone_index: u64,
    pub local_pawn_index: u64,
    pub previous_aim_punch: Vec2,
}

impl Target {
    pub fn reset(&mut self) {
        *self = Target::default();
    }
}

impl CS2 {
    pub fn find_target(&mut self, config: &Config) {
        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        let team = local_player.team(self);
        if team != cs2::TEAM_CT && team != cs2::TEAM_T {
            self.target.reset();
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
        let targeting_mode = &aimbot_config.targeting_mode;
        let max_fov_units = aimbot_config.fov;

        let mut best_metric = f32::MAX;
        let eye_position = local_player.eye_position(self);

        if self.target.player.is_none() {
            self.target.reset();
        }
        if let Some(player) = &self.target.player
            && !player.is_valid(self)
        {
            self.target.reset();
        }

        if self.players.is_empty() {
            self.target.reset();
            return;
        }

        let target_friendlies = aimbot_config.target_friendlies;

        // Check if existing target is still valid and within FOV
        if let Some(player) = &self.target.player {
            if player.is_valid(self) && (ffa || target_friendlies || team != player.team(self)) {
                let target_bone = if aimbot_config.bones.iter().any(|b| b.u64() == self.target.bone_index) {
                    self.target.bone_index
                } else {
                    aimbot_config.bones.first().map(|b| b.u64()).unwrap_or(7)
                };

                let bone_position = player.bone_position(self, target_bone);
                let distance = eye_position.distance(bone_position);
                let angle = self.angle_to_target(&local_player, &bone_position, &aim_punch);
                let fov_deg = angles_to_fov(&view_angles, &angle);
                let offset_units = distance * fov_deg.to_radians().sin();

                if offset_units <= max_fov_units {
                    self.target.angle = angle;
                    self.target.distance = distance;
                    self.target.bone_index = target_bone;
                    return;
                }
            }
            self.target.reset();
        }

        for player in &self.players {
            if !(ffa || target_friendlies) && team == player.team(self) {
                continue;
            }

            for bone in &aimbot_config.bones {
                let bone_position = player.bone_position(self, bone.u64());
                let distance = eye_position.distance(bone_position);
                if distance < 1.0 {
                    continue;
                }

                let angle = self.angle_to_target(&local_player, &bone_position, &aim_punch);
                let fov_deg = angles_to_fov(&view_angles, &angle);
                let offset_units = distance * fov_deg.to_radians().sin();

                if offset_units > max_fov_units {
                    continue;
                }

                let metric = match targeting_mode {
                    TargetingMode::Fov => offset_units,
                    TargetingMode::Distance => distance,
                };

                if metric < best_metric {
                    best_metric = metric;

                    self.target.player = Some(*player);
                    self.target.angle = angle;
                    self.target.distance = distance;
                    self.target.bone_index = bone.u64();
                }
            }
        }
    }
}
