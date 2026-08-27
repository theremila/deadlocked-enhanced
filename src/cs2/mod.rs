use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use glam::{IVec2, Mat4, Vec2, Vec3};

use crate::{
    config::{
        Config,
        aim::{AimbotConfig, RcsConfig, TriggerbotConfig},
    },
    constants::cs2::{self, TEAM_CT, TEAM_T},
    cs2::{
        binds::BindRuntime,
        bones::Bones,
        entity::{
            Entity, EntityInfo, grenade_info, planted_c4::PlantedC4, player::Player, weapon::Weapon,
        },
        features::{aimbot::Aimbot, bhop::Bunnyhop, rcs::Recoil, triggerbot::Triggerbot},
        input::Input,
        offsets::Offsets,
        target::Target,
    },
    data::{Data, PlayerData},
    math::{angles_from_vector, vec2_clamp},
    os::{mouse::Mouse, process::Process},
    parser::{bvh::Bvh, read_map, take_material_bvh},
};

mod accuracy;
mod binds;
pub mod bones;
pub mod bvh;
pub mod entity;
mod features;
mod find_offsets;
mod hitbox;
mod input;
pub mod key_codes;
mod offsets;
mod schema;
mod target;

pub struct CS2 {
    is_valid: bool,
    process: Process,
    offsets: Offsets,
    input: Input,
    bvh: Option<Bvh>,
    current_bvh: String,
    target: Target,
    players: Vec<Player>,
    dead_players: Vec<Player>,
    entities: Vec<Entity>,
    recoil: Recoil,
    aim: Aimbot,
    trigger: Triggerbot,
    bhop: Bunnyhop,
    weapon: Weapon,
    planted_c4: Option<PlantedC4>,
    last_cache: Instant,
    bind_runtime: BindRuntime,
    effective_config: Config,
}

impl CS2 {
    pub(crate) fn executable_path(&self) -> Option<PathBuf> {
        self.process.executable_path()
    }

    pub fn is_valid(&self) -> bool {
        self.is_valid && self.process.is_valid()
    }

    pub fn setup(&mut self) {
        let Some(process) = Process::open(cs2::PROCESS_NAME) else {
            self.is_valid = false;
            return;
        };
        utils::info!("process found, pid: {}", process.pid);
        self.process = process;

        self.offsets = match self.find_offsets() {
            Some(offsets) => offsets,
            None => {
                self.process = Process::new(-1);
                self.is_valid = false;
                return;
            }
        };
        utils::info!("offsets found");

        self.is_valid = true;
    }

    pub fn run(&mut self, config: &Config, mouse: &mut Mouse) {
        if !self.process.is_valid() {
            self.is_valid = false;
            utils::debug!("process is no longer valid");
            return;
        }

        self.input.update(&self.process, &self.offsets);
        self.bind_runtime.update(config, &self.input);
        let effective_config = self.bind_runtime.effective_config(config);
        let config = &effective_config;

        if self.last_cache.elapsed() > Duration::from_millis(200) {
            self.cache_entities();
            self.check_bvh();
            self.last_cache = Instant::now();
        }

        for entity in &self.entities {
            if let Entity::Smoke(smoke) = entity {
                if config.misc.no_smoke {
                    smoke.disable(self);
                }

                if config.misc.change_smoke_color {
                    smoke.color(self, &config.misc.smoke_color);
                }
            }
        }

        self.no_flash(config);
        self.fov_changer(config);
        self.bunnyhop(config, mouse);

        self.find_target(config);
        let aiming = self.aimbot(config, mouse);
        self.triggerbot(config, mouse);
        self.release_trigger_shot(mouse);
        if !aiming {
            self.rcs(config, mouse);
        }
        self.effective_config = effective_config;
    }

    pub fn data(&self, data: &mut Data) {
        let config = &self.effective_config;
        data.bound_values.clone_from(self.bind_runtime.values());
        data.players.clear();
        data.friendlies.clear();
        data.spectators.clear();
        data.entities.clear();

        let sdl_window: usize = self.process.read(self.offsets.direct.sdl_window);
        if sdl_window == 0 {
            data.window_position = Vec2::ZERO;
            data.window_size = Vec2::ONE;
        } else {
            data.window_position = self.process.read::<IVec2>(sdl_window + 0x18).as_vec2();
            data.window_size = self
                .process
                .read::<IVec2>(sdl_window + 0x18 + 0x08)
                .as_vec2();
        }

        let Some(local_player) = Player::local_player(self) else {
            data.weapon = Weapon::default();
            data.in_game = false;
            return;
        };
        let local_team = local_player.team(self);
        if local_team != TEAM_T && local_team != TEAM_CT {
            data.weapon = Weapon::default();
            data.in_game = false;
            return;
        }
        let is_ffa = self.is_ffa();
        let spectator_target = local_player.spectator_target(self);
        let active_pawn = if let Some(target) = spectator_target {
            target.pawn
        } else {
            local_player.pawn
        };

        for player in &self.players {
            if spectator_target.is_some() && player.pawn == active_pawn {
                continue;
            }

            let player_data = PlayerData {
                steam_id: player.steam_id(self),
                health: player.health(self),
                armor: player.armor(self),
                position: player.position(self),
                head: player.bone_position(self, Bones::Head.u64()),
                name: player.name(self),
                weapon: player.weapon(self),
                ammo: (player.clip_ammo(self), player.reserve_ammo(self)),
                bones: player.all_bones(self),
                has_defuser: player.has_defuser(self),
                has_helmet: player.has_helmet(self),
                has_bomb: player.has_bomb(self),
                visible: player.visible(self, &local_player),
                color: player.color(self),
                rotation: player.rotation(self),
                sound: player.is_making_sound(self),
            };

            if !is_ffa && player.team(self) == local_team {
                data.friendlies.push(player_data);
            } else {
                data.players.push(player_data);
            }
        }

        for player in &self.dead_players {
            if let Some(target) = player.spectator_target(self)
                && target.pawn == active_pawn
            {
                data.spectators.push(player.name(self));
            }
        }

        data.local_player = PlayerData {
            steam_id: local_player.steam_id(self),
            health: local_player.health(self),
            armor: local_player.armor(self),
            position: local_player.position(self),
            head: local_player.eye_position(self),
            name: local_player.name(self),
            weapon: local_player.weapon(self),
            ammo: (
                local_player.clip_ammo(self),
                local_player.reserve_ammo(self),
            ),
            bones: local_player.all_bones(self),
            has_defuser: local_player.has_defuser(self),
            has_helmet: local_player.has_helmet(self),
            has_bomb: local_player.has_bomb(self),
            visible: true,
            color: local_player.color(self),
            rotation: local_player.rotation(self),
            sound: None,
        };

        data.entities.clear();
        for entity in &self.entities {
            data.entities.push(match entity {
                Entity::Weapon { weapon, entity } => EntityInfo::Weapon {
                    weapon: weapon.clone(),
                    position: Player::entity(*entity).position(self),
                    ammo: (
                        Weapon::clip_ammo(*entity, self),
                        Weapon::reserve_ammo(*entity, self),
                    ),
                },
                Entity::Inferno(inferno) => EntityInfo::Inferno(inferno.info(self)),
                Entity::Smoke(smoke) => EntityInfo::Smoke(smoke.info(self)),
                Entity::Molotov(molotov) => EntityInfo::Molotov(molotov.info(self)),
                Entity::Flashbang(entity) => {
                    EntityInfo::Flashbang(grenade_info(*entity, "Flashbang", self))
                }
                Entity::HeGrenade(entity) => {
                    EntityInfo::HeGrenade(grenade_info(*entity, "HE Grenade", self))
                }
                Entity::Decoy(entity) => EntityInfo::Decoy(grenade_info(*entity, "Decoy", self)),
                Entity::Chicken(chicken) => EntityInfo::ChickenInfo(chicken.info(self)),
            });
        }

        data.weapon = local_player.weapon(self);
        data.in_game = true;
        data.is_ffa = is_ffa;
        data.map_name = self.current_map();
        data.aimbot_active = self.aim.active;
        data.aim_target_position = self.target.player.as_ref().and(
            self.target
                .position
                .is_finite()
                .then_some(self.target.position),
        );
        data.triggerbot_active = self.trigger.active;
        data.esp_active = config.player.enabled;
        data.view_matrix = self.process.read::<Mat4>(self.offsets.direct.view_matrix);
        data.view_angles = local_player.view_angles(self);

        if let Some(bomb) = &self.planted_c4 {
            data.bomb.planted = bomb.is_planted(self);
            data.bomb.timer = bomb.time_to_explosion(self);
            data.bomb.position = bomb.position(self);
            data.bomb.being_defused = bomb.is_being_defused(self);
            data.bomb.defuse_remain_time = bomb.time_to_defuse(self);
        } else {
            data.bomb.planted = false;
        }
    }

    pub fn new() -> Self {
        Self {
            is_valid: false,
            process: Process::new(-1),
            offsets: Offsets::default(),
            input: Input::new(),
            bvh: None,
            current_bvh: String::new(),
            target: Target::default(),
            players: Vec::with_capacity(64),
            dead_players: Vec::with_capacity(12),
            entities: Vec::with_capacity(128),
            recoil: Recoil::default(),
            aim: Aimbot::default(),
            trigger: Triggerbot::default(),
            bhop: Bunnyhop::default(),
            weapon: Weapon::default(),
            planted_c4: None,
            last_cache: Instant::now(),
            bind_runtime: BindRuntime::default(),
            effective_config: Config::default(),
        }
    }

    pub fn rebaseline_binds(&mut self, config: &Config) {
        self.bind_runtime.rebaseline(config);
    }

    pub fn set_bind_capture(&mut self, capturing: bool) {
        self.bind_runtime.set_suppressed(capturing);
    }

    fn aimbot_config<'a>(&self, config: &'a Config) -> &'a AimbotConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.aimbot.enable_override
        {
            return &weapon_config.aimbot;
        }
        &config.aim.global.aimbot
    }

    fn rcs_config<'a>(&self, config: &'a Config) -> &'a RcsConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.rcs.enable_override
        {
            return &weapon_config.rcs;
        }
        &config.aim.global.rcs
    }

    fn triggerbot_config<'a>(&self, config: &'a Config) -> &'a TriggerbotConfig {
        if let Some(weapon_config) = config.aim.weapons.get(&self.weapon)
            && weapon_config.triggerbot.enable_override
        {
            return &weapon_config.triggerbot;
        }
        &config.aim.global.triggerbot
    }

    fn angle_to_target(&self, local_player: &Player, position: &Vec3, aim_punch: &Vec2) -> Vec2 {
        let eye_position = local_player.eye_position(self);
        let forward = (position - eye_position).normalize();

        let mut angles = angles_from_vector(&forward) - aim_punch;
        vec2_clamp(&mut angles);

        angles
    }

    fn entity_has_owner(&self, entity: usize) -> bool {
        self.process
            .read::<i32>(entity + self.offsets.controller.owner_entity)
            != -1
    }

    // convars
    fn get_sensitivity(&self) -> f32 {
        self.process.read(self.offsets.convar.sensitivity + 0x58)
    }

    fn is_ffa(&self) -> bool {
        self.process.read::<u8>(self.offsets.convar.ffa + 0x58) == 1
    }

    pub fn is_line_in_smoke(&self, start: Vec3, end: Vec3) -> bool {
        let dir = end - start;
        let len = dir.length();
        if len < 0.001 {
            return false;
        }
        let norm_dir = dir / len;

        for entity in &self.entities {
            if let entity::Entity::Smoke(smoke) = entity {
                let smoke_pos = smoke.info(self).position;
                let t = (smoke_pos - start).dot(norm_dir).clamp(0.0, len);
                let closest = start + norm_dir * t;
                if (smoke_pos - closest).length() <= 145.0 {
                    return true;
                }
            }
        }
        false
    }

    pub fn calculate_damage(
        &self,
        start: Vec3,
        end: Vec3,
        target_bone: Bones,
        armor: i32,
        has_helmet: bool,
    ) -> f32 {
        let mut damage = self.base_hit_damage(start.distance(end), target_bone);

        if let Some(bvh) = &self.bvh {
            let intersections = bvh.segment_intersections(start, end);
            if !intersections.len().is_multiple_of(2) {
                return 0.0;
            }
            let penetration_count = intersections.len().div_ceil(2);
            if penetration_count > 4 {
                return 0.0;
            }

            if penetration_count > 0 {
                let penetration = self.weapon.penetration_power().max(0.1);
                let mut effective_thickness = 0.0;
                let mut material_retention = 1.0;
                for pair in intersections.chunks_exact(2) {
                    if !pair[0].2 || pair[1].2 {
                        return 0.0;
                    }
                    let thickness = (pair[1].0 - pair[0].0).max(0.0);
                    if thickness < 0.5 {
                        return 0.0;
                    }
                    let material_modifier =
                        (pair[0].1.penetration_modifier() + pair[1].1.penetration_modifier()) * 0.5;
                    if thickness > penetration * 24.0 * material_modifier {
                        return 0.0;
                    }
                    effective_thickness += thickness / material_modifier;
                    material_retention *= (0.72 + material_modifier * 0.06).clamp(0.7, 0.95);
                }
                let thickness_modifier = (-effective_thickness / (penetration * 64.0)).exp();
                let surface_modifier = (0.78 + penetration * 0.04)
                    .clamp(0.25, 0.9)
                    .powi(penetration_count as i32);
                damage = damage * thickness_modifier * surface_modifier * material_retention
                    - penetration_count as f32 * (3.0 / penetration);
            }
        }

        self.apply_armor(damage, target_bone, armor, has_helmet)
    }

    fn base_hit_damage(&self, distance: f32, target_bone: Bones) -> f32 {
        let damage =
            self.weapon.base_damage() * self.weapon.range_modifier().powf(distance / 500.0);
        damage
            * match target_bone {
                Bones::Head => self.weapon.headshot_multiplier(),
                Bones::Hip => 1.25,
                Bones::LeftHip
                | Bones::RightHip
                | Bones::LeftKnee
                | Bones::RightKnee
                | Bones::LeftFoot
                | Bones::RightFoot => 0.75,
                _ => 1.0,
            }
    }

    fn apply_armor(
        &self,
        mut damage: f32,
        target_bone: Bones,
        armor: i32,
        has_helmet: bool,
    ) -> f32 {
        let armored_hitgroup = match target_bone {
            Bones::Head => has_helmet,
            Bones::LeftHip
            | Bones::RightHip
            | Bones::LeftKnee
            | Bones::RightKnee
            | Bones::LeftFoot
            | Bones::RightFoot => false,
            _ => armor > 0,
        };
        if armored_hitgroup {
            let damage_to_health = damage * (self.weapon.armor_ratio() * 0.5);
            let damage_to_armor = (damage - damage_to_health) * 0.5;
            damage = if damage_to_armor > armor as f32 {
                damage - armor as f32 * 2.0
            } else {
                damage_to_health
            };
        }
        damage.max(0.0)
    }

    pub fn calculate_direct_damage(
        &self,
        start: Vec3,
        end: Vec3,
        target_bone: Bones,
        armor: i32,
        has_helmet: bool,
    ) -> f32 {
        let damage = self.base_hit_damage(start.distance(end), target_bone);
        self.apply_armor(damage, target_bone, armor, has_helmet)
    }

    fn current_time(&self) -> f32 {
        let global_vars: usize = self.process.read(self.offsets.direct.global_vars);
        self.process.read(global_vars + 0x30)
    }

    fn current_map(&self) -> String {
        let global_vars: usize = self.process.read(self.offsets.direct.global_vars);
        self.process
            .read_string(self.process.read(global_vars + 0x198))
    }

    #[allow(dead_code)]
    fn distance_scale(&self, distance: f32) -> f32 {
        if distance > 500.0 {
            1.0
        } else {
            5.0 - (distance / 125.0)
        }
    }

    fn check_bvh(&mut self) {
        let current_map = self.current_map();
        if let Some(material_bvh) = take_material_bvh(&current_map) {
            self.bvh = Some(material_bvh);
            utils::info!("activated material-aware BVH for {current_map}");
        }
        if current_map != self.current_bvh {
            self.bvh = read_map(self, &current_map);
            if self.bvh.is_some() {
                utils::info!("loaded bvh for {current_map}");
                self.current_bvh = current_map;
            }
        }
    }
}
