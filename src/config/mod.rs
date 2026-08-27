use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use egui::Color32;
use serde::{Deserialize, Serialize};

use crate::{
    config::{
        aim::AimConfig,
        bind::{
            AimProfile, AimSetting, BindMode, HudSetting, MiscSetting, PlayerSetting, RcsSetting,
            SettingBind, SettingId, TriggerSetting,
        },
        hud::HudConfig,
        player::PlayerConfig,
        r#unsafe::UnsafeConfig,
    },
    font::Font,
    ui::color::Colors,
};

pub mod aim;
pub mod application;
pub mod bind;
pub mod hud;
pub mod player;
pub mod text;
pub mod r#unsafe;

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub aim: AimConfig,
    pub player: PlayerConfig,
    pub hud: HudConfig,
    pub misc: UnsafeConfig,
    pub accent_color: Color32,
    pub fps: u32,
    pub font: Font,
    #[serde(default)]
    pub binds: Vec<SettingBind>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            aim: AimConfig::default(),
            player: PlayerConfig::default(),
            hud: HudConfig::default(),
            misc: UnsafeConfig::default(),
            accent_color: Colors::PINK,
            fps: 120,
            font: Font::FiraSans,
            binds: Self::default_binds(),
        }
    }
}

impl Config {
    fn default_binds() -> Vec<SettingBind> {
        vec![
            SettingBind::single(
                SettingId::Aim(AimProfile::Global, AimSetting::Enabled),
                crate::cs2::key_codes::KeyCode::Mouse5,
                BindMode::Hold,
            ),
            SettingBind::single(
                SettingId::Trigger(AimProfile::Global, TriggerSetting::Enabled),
                crate::cs2::key_codes::KeyCode::Mouse4,
                BindMode::Hold,
            ),
            SettingBind::single(
                SettingId::Player(PlayerSetting::Enabled),
                crate::cs2::key_codes::KeyCode::X,
                BindMode::Toggle,
            ),
            SettingBind::single(
                SettingId::Misc(MiscSetting::Bunnyhop),
                crate::cs2::key_codes::KeyCode::Space,
                BindMode::Hold,
            ),
        ]
    }

    pub fn ensure_legacy_binds(&mut self) {
        if !self.binds.is_empty() {
            return;
        }
        self.binds = vec![
            SettingBind::single(
                SettingId::Aim(AimProfile::Global, AimSetting::Enabled),
                self.aim.aimbot_hotkey,
                self.aim.global.aimbot.mode.into(),
            ),
            SettingBind::single(
                SettingId::Trigger(AimProfile::Global, TriggerSetting::Enabled),
                self.aim.triggerbot_hotkey,
                self.aim.global.triggerbot.mode.into(),
            ),
            SettingBind::single(
                SettingId::Player(PlayerSetting::Enabled),
                self.player.esp_hotkey,
                BindMode::Toggle,
            ),
            SettingBind::single(
                SettingId::Misc(MiscSetting::Bunnyhop),
                self.misc.bunnyhop_hotkey,
                BindMode::Hold,
            ),
        ];
    }

    pub fn bool_value(&self, id: &SettingId) -> bool {
        match id {
            SettingId::Aim(profile, setting) => {
                let value = &self.weapon_profile(profile).aimbot;
                match setting {
                    AimSetting::Override => value.enable_override,
                    AimSetting::Enabled => value.enabled,
                    AimSetting::TargetFriendlies => value.target_friendlies,
                    AimSetting::DistanceAdjustedFov => value.distance_adjusted_fov,
                    AimSetting::VisibilityCheck => value.visibility_check,
                    AimSetting::ThroughWalls => value.through_walls,
                    AimSetting::SmokeCheck => value.smoke_check,
                    AimSetting::FlashCheck => value.flash_check,
                    AimSetting::InAirCheck => value.in_air_check,
                    AimSetting::Humanize => value.humanize,
                }
            }
            SettingId::Trigger(profile, setting) => {
                let value = &self.weapon_profile(profile).triggerbot;
                match setting {
                    TriggerSetting::Override => value.enable_override,
                    TriggerSetting::Enabled => value.enabled,
                    TriggerSetting::AutoStop => value.autostop,
                    TriggerSetting::VisibilityCheck => value.visibility_check,
                    TriggerSetting::ThroughWalls => value.through_walls,
                    TriggerSetting::SmokeCheck => value.smoke_check,
                    TriggerSetting::FlashCheck => value.flash_check,
                    TriggerSetting::ScopeCheck => value.scope_check,
                    TriggerSetting::InAirCheck => value.in_air_check,
                    TriggerSetting::VelocityCheck => value.velocity_check,
                    TriggerSetting::HeadOnly => value.head_only,
                    TriggerSetting::PreferCenter => value.prefer_center,
                }
            }
            SettingId::Rcs(profile, setting) => {
                let value = &self.weapon_profile(profile).rcs;
                match setting {
                    RcsSetting::Override => value.enable_override,
                    RcsSetting::Enabled => value.enabled,
                }
            }
            SettingId::Player(setting) => match setting {
                PlayerSetting::Enabled => self.player.enabled,
                PlayerSetting::Chicken => self.player.chicken,
                PlayerSetting::ShowFriendlies => self.player.show_friendlies,
                PlayerSetting::HeadCircle => self.player.head_circle,
                PlayerSetting::HealthBar => self.player.health_bar,
                PlayerSetting::ArmorBar => self.player.armor_bar,
                PlayerSetting::PlayerName => self.player.player_name,
                PlayerSetting::WeaponIcon => self.player.weapon_icon,
                PlayerSetting::Tags => self.player.tags,
                PlayerSetting::VisibleOnly => self.player.visible_only,
                PlayerSetting::OofArrows => self.player.oof_arrows,
                PlayerSetting::OofOffscreenOnly => self.player.oof_offscreen_only,
                PlayerSetting::SoundEsp => self.player.sound.enabled,
                PlayerSetting::SoundShowVisible => self.player.sound.show_visible,
            },
            SettingId::Hud(setting) => match setting {
                HudSetting::Watermark => self.hud.watermark,
                HudSetting::BombTimer => self.hud.bomb_timer,
                HudSetting::FovCircle => self.hud.fov_circle,
                HudSetting::SniperCrosshair => self.hud.sniper_crosshair.enabled,
                HudSetting::DroppedWeapons => self.hud.dropped_weapons,
                HudSetting::KeybindList => self.hud.keybind_list,
                HudSetting::SpectatorList => self.hud.spectator_list,
                HudSetting::StatusIndicators => self.hud.status_indicators,
                HudSetting::GrenadeTrails => self.hud.grenade_trails.enabled,
                HudSetting::InfernoPolygon => self.hud.grenade_trails.inferno_poly,
                HudSetting::TextOutline => self.hud.text_outline,
                HudSetting::Debug => self.hud.debug,
            },
            SettingId::Misc(setting) => match setting {
                MiscSetting::NoFlash => self.misc.no_flash,
                MiscSetting::FovChanger => self.misc.fov_changer,
                MiscSetting::NoSmoke => self.misc.no_smoke,
                MiscSetting::ChangeSmokeColor => self.misc.change_smoke_color,
                MiscSetting::Bunnyhop => self.misc.bunnyhop,
            },
        }
    }

    pub fn set_bool(&mut self, id: &SettingId, enabled: bool) {
        match id {
            SettingId::Aim(profile, setting) => {
                let value = &mut self.weapon_profile_mut(profile).aimbot;
                match setting {
                    AimSetting::Override => value.enable_override = enabled,
                    AimSetting::Enabled => value.enabled = enabled,
                    AimSetting::TargetFriendlies => value.target_friendlies = enabled,
                    AimSetting::DistanceAdjustedFov => value.distance_adjusted_fov = enabled,
                    AimSetting::VisibilityCheck => value.visibility_check = enabled,
                    AimSetting::ThroughWalls => value.through_walls = enabled,
                    AimSetting::SmokeCheck => value.smoke_check = enabled,
                    AimSetting::FlashCheck => value.flash_check = enabled,
                    AimSetting::InAirCheck => value.in_air_check = enabled,
                    AimSetting::Humanize => value.humanize = enabled,
                }
            }
            SettingId::Trigger(profile, setting) => {
                let value = &mut self.weapon_profile_mut(profile).triggerbot;
                match setting {
                    TriggerSetting::Override => value.enable_override = enabled,
                    TriggerSetting::Enabled => value.enabled = enabled,
                    TriggerSetting::AutoStop => value.autostop = enabled,
                    TriggerSetting::VisibilityCheck => value.visibility_check = enabled,
                    TriggerSetting::ThroughWalls => value.through_walls = enabled,
                    TriggerSetting::SmokeCheck => value.smoke_check = enabled,
                    TriggerSetting::FlashCheck => value.flash_check = enabled,
                    TriggerSetting::ScopeCheck => value.scope_check = enabled,
                    TriggerSetting::InAirCheck => value.in_air_check = enabled,
                    TriggerSetting::VelocityCheck => value.velocity_check = enabled,
                    TriggerSetting::HeadOnly => value.head_only = enabled,
                    TriggerSetting::PreferCenter => value.prefer_center = enabled,
                }
            }
            SettingId::Rcs(profile, setting) => {
                let value = &mut self.weapon_profile_mut(profile).rcs;
                match setting {
                    RcsSetting::Override => value.enable_override = enabled,
                    RcsSetting::Enabled => value.enabled = enabled,
                }
            }
            SettingId::Player(setting) => match setting {
                PlayerSetting::Enabled => self.player.enabled = enabled,
                PlayerSetting::Chicken => self.player.chicken = enabled,
                PlayerSetting::ShowFriendlies => self.player.show_friendlies = enabled,
                PlayerSetting::HeadCircle => self.player.head_circle = enabled,
                PlayerSetting::HealthBar => self.player.health_bar = enabled,
                PlayerSetting::ArmorBar => self.player.armor_bar = enabled,
                PlayerSetting::PlayerName => self.player.player_name = enabled,
                PlayerSetting::WeaponIcon => self.player.weapon_icon = enabled,
                PlayerSetting::Tags => self.player.tags = enabled,
                PlayerSetting::VisibleOnly => self.player.visible_only = enabled,
                PlayerSetting::OofArrows => self.player.oof_arrows = enabled,
                PlayerSetting::OofOffscreenOnly => self.player.oof_offscreen_only = enabled,
                PlayerSetting::SoundEsp => self.player.sound.enabled = enabled,
                PlayerSetting::SoundShowVisible => self.player.sound.show_visible = enabled,
            },
            SettingId::Hud(setting) => match setting {
                HudSetting::Watermark => self.hud.watermark = enabled,
                HudSetting::BombTimer => self.hud.bomb_timer = enabled,
                HudSetting::FovCircle => self.hud.fov_circle = enabled,
                HudSetting::SniperCrosshair => self.hud.sniper_crosshair.enabled = enabled,
                HudSetting::DroppedWeapons => self.hud.dropped_weapons = enabled,
                HudSetting::KeybindList => self.hud.keybind_list = enabled,
                HudSetting::SpectatorList => self.hud.spectator_list = enabled,
                HudSetting::StatusIndicators => self.hud.status_indicators = enabled,
                HudSetting::GrenadeTrails => self.hud.grenade_trails.enabled = enabled,
                HudSetting::InfernoPolygon => self.hud.grenade_trails.inferno_poly = enabled,
                HudSetting::TextOutline => self.hud.text_outline = enabled,
                HudSetting::Debug => self.hud.debug = enabled,
            },
            SettingId::Misc(setting) => match setting {
                MiscSetting::NoFlash => self.misc.no_flash = enabled,
                MiscSetting::FovChanger => self.misc.fov_changer = enabled,
                MiscSetting::NoSmoke => self.misc.no_smoke = enabled,
                MiscSetting::ChangeSmokeColor => self.misc.change_smoke_color = enabled,
                MiscSetting::Bunnyhop => self.misc.bunnyhop = enabled,
            },
        }
    }

    fn weapon_profile(&self, profile: &AimProfile) -> &aim::WeaponConfig {
        match profile {
            AimProfile::Global => &self.aim.global,
            AimProfile::Weapon(weapon) => self.aim.weapons.get(weapon).unwrap_or(&self.aim.global),
        }
    }

    fn weapon_profile_mut(&mut self, profile: &AimProfile) -> &mut aim::WeaponConfig {
        match profile {
            AimProfile::Global => &mut self.aim.global,
            AimProfile::Weapon(weapon) => self.aim.weapons.entry(weapon.clone()).or_default(),
        }
    }
}

pub const DEFAULT_CONFIG_NAME: &str = "deadlocked.toml";

pub static BASE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = std::env::var_os("XDG_CONFIG_HOME")
        .and_then(|p| {
            if p.is_empty() {
                None
            } else {
                Some(PathBuf::from(p))
            }
        })
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .map(|base| base.join("deadlocked"))
        .unwrap_or_else(|| {
            std::env::current_exe()
                .ok()
                .and_then(|exe| exe.parent().map(|p| p.to_path_buf()))
                .unwrap_or_else(|| PathBuf::from("."))
        });
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path
});

pub static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    let path = BASE_PATH.join("configs");
    if !path.exists() {
        let _ = std::fs::create_dir_all(&path);
    }
    path
});

pub fn parse_config(path: &Path) -> Config {
    if !path.exists() || path.is_dir() {
        return Config::default();
    }

    let Ok(config_string) = std::fs::read_to_string(path) else {
        return Config::default();
    };

    let config = toml::from_str::<Config>(&config_string);
    if config.is_err() {
        utils::warn!("config file invalid");
    } else if let Some(file_name) = path.file_name() {
        utils::info!("loaded config {:?}", file_name);
    }
    let mut config = config.unwrap_or_default();
    config.ensure_legacy_binds();
    config
}

pub fn write_config(config: &Config, path: &Path) {
    let out = toml::to_string(&config).unwrap();
    let _ = std::fs::write(path, out);
}

pub fn delete_config(path: &Path) {
    if !path.exists() {
        return;
    }

    if std::fs::remove_file(path).is_ok()
        && let Some(file_name) = path.file_name()
    {
        utils::info!("deleted config {:?}", file_name);
    }
}

pub fn available_configs() -> Vec<PathBuf> {
    let mut files = Vec::with_capacity(8);
    let Ok(dir) = std::fs::read_dir::<&Path>(CONFIG_PATH.as_ref()) else {
        return files;
    };

    for path in dir {
        let Ok(file) = path else {
            continue;
        };
        let Ok(file_type) = file.file_type() else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }
        let file_name = file.file_name();
        let Some(file_name) = file_name.to_str() else {
            continue;
        };
        if !file_name.ends_with(".toml") {
            continue;
        }
        files.push(file.path())
    }
    if files.is_empty() {
        let path = CONFIG_PATH.join(DEFAULT_CONFIG_NAME);
        write_config(&Config::default(), &path);
        files.push(path);
    }
    files
}
