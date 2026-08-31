use std::{collections::HashMap, ops::RangeInclusive};

use glam::Vec2;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::cs2::{bones::Bones, entity::weapon::Weapon};

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WeaponConfig {
    pub aimbot: AimbotConfig,
    pub rcs: RcsConfig,
    pub triggerbot: TriggerbotConfig,
}

impl WeaponConfig {
    pub fn enabled(enabled: bool) -> Self {
        let aimbot = AimbotConfig {
            enable_override: enabled,
            ..Default::default()
        };
        Self {
            aimbot,
            rcs: RcsConfig::default(),
            triggerbot: TriggerbotConfig::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AimbotConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub target_friendlies: bool,
    pub distance_adjusted_fov: bool,
    pub start_bullet: i32,
    pub visibility_check: bool,
    pub through_walls: bool,
    pub smoke_check: bool,
    pub flash_check: bool,
    pub in_air_check: bool,
    pub fov: f32,
    pub smooth: f32,
    pub smooth_random: f32,
    pub deadzone: f32,
    pub reaction_time: u64,
    pub inertia: f32,
    pub bones: Vec<Bones>,
    pub targeting_mode: TargetingMode,
    pub humanize: bool,
    pub curve: f32,
    pub tremor: f32,
    pub overshoot: f32,
}

impl Default for AimbotConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: true,
            target_friendlies: false,
            distance_adjusted_fov: true,
            start_bullet: 0,
            visibility_check: true,
            through_walls: false,
            smoke_check: true,
            flash_check: true,
            in_air_check: false,
            fov: 25.0,
            smooth: 15.0,
            smooth_random: 2.0,
            deadzone: 0.0,
            reaction_time: 0,
            inertia: 1.0,
            bones: vec![
                Bones::Head,
                Bones::Neck,
                Bones::Spine4,
                Bones::Spine3,
                Bones::Spine2,
                Bones::Spine1,
                Bones::Hip,
            ],
            targeting_mode: TargetingMode::Fov,
            humanize: true,
            curve: 0.35,
            tremor: 0.15,
            overshoot: 0.1,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct RcsConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub strength: Vec2,
}

impl Default for RcsConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: false,
            strength: Vec2::splat(0.5),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum TargetingMode {
    Fov,
    Distance,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum TriggerTargetingMode {
    Fov,
    Raycast,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, EnumIter)]
pub enum SeedMode {
    #[default]
    Off,
    Always,
    WhenAvailable,
}

impl SeedMode {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::Always => "Always",
            Self::WhenAvailable => "When Available",
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TriggerbotConfig {
    pub enable_override: bool,
    pub enabled: bool,
    pub delay: RangeInclusive<u64>,
    pub shot_duration: u64,
    pub hitchance: f32,
    pub seed_mode: SeedMode,
    pub targeting_mode: TriggerTargetingMode,
    pub fov: f32,
    pub prefer_aim_target: bool,
    pub min_damage: i32,
    pub autostop: bool,
    pub visibility_check: bool,
    pub through_walls: bool,
    pub smoke_check: bool,
    pub flash_check: bool,
    pub scope_check: bool,
    pub in_air_check: bool,
    pub velocity_check: bool,
    pub velocity_threshold: f32,
    pub head_only: bool,
    pub prefer_center: bool,
    pub center_tolerance: f32,
    pub bones: Vec<Bones>,
}

impl Default for TriggerbotConfig {
    fn default() -> Self {
        Self {
            enable_override: false,
            enabled: false,
            delay: 100..=200,
            shot_duration: 200,
            hitchance: 50.0,
            seed_mode: SeedMode::Off,
            targeting_mode: TriggerTargetingMode::Raycast,
            fov: 25.0,
            prefer_aim_target: true,
            min_damage: 20,
            autostop: false,
            visibility_check: true,
            through_walls: false,
            smoke_check: true,
            flash_check: true,
            scope_check: true,
            in_air_check: false,
            velocity_check: true,
            velocity_threshold: 100.0,
            head_only: false,
            prefer_center: false,
            center_tolerance: 50.0,
            bones: vec![
                Bones::Head,
                Bones::Neck,
                Bones::Spine4,
                Bones::Spine3,
                Bones::Spine2,
                Bones::Spine1,
                Bones::Hip,
            ],
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AimConfig {
    pub global: WeaponConfig,
    pub weapons: HashMap<Weapon, WeaponConfig>,
}

impl Default for AimConfig {
    fn default() -> Self {
        let mut weapons = HashMap::new();
        for weapon in Weapon::iter() {
            weapons.insert(weapon, WeaponConfig::default());
        }

        Self {
            global: WeaponConfig::enabled(true),
            weapons,
        }
    }
}
