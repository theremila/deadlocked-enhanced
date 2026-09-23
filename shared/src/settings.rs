use serde::{Deserialize, Serialize};

use crate::Weapon;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AimProfile {
    Global,
    Weapon(Weapon),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AimSetting {
    Override,
    Enabled,
    TargetFriendlies,
    VisibilityCheck,
    ThroughWalls,
    SmokeCheck,
    FlashCheck,
    InAirCheck,
    Humanize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggerSetting {
    Override,
    Enabled,
    PreferAimTarget,
    AutoStop,
    VisibilityCheck,
    ThroughWalls,
    SmokeCheck,
    FlashCheck,
    ScopeCheck,
    InAirCheck,
    VelocityCheck,
    HeadOnly,
    PreferCenter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RcsSetting {
    Override,
    Enabled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerSetting {
    Enabled,
    Chicken,
    ShowFriendlies,
    HeadCircle,
    HealthBar,
    ArmorBar,
    PlayerName,
    WeaponIcon,
    Tags,
    VisibleOnly,
    OofArrows,
    OofOffscreenOnly,
    SoundEsp,
    SoundShowVisible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HudSetting {
    Watermark,
    BombTimer,
    FovCircle,
    SniperCrosshair,
    DroppedWeapons,
    KeybindList,
    SpectatorList,
    StatusIndicators,
    GrenadeTrails,
    InfernoPolygon,
    TextOutline,
    Debug,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MiscSetting {
    NoFlash,
    FovChanger,
    NoSmoke,
    ChangeSmokeColor,
    Bunnyhop,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "feature", content = "setting", rename_all = "snake_case")]
pub enum SettingId {
    Aim(AimProfile, AimSetting),
    Trigger(AimProfile, TriggerSetting),
    Rcs(AimProfile, RcsSetting),
    Player(PlayerSetting),
    Hud(HudSetting),
    Misc(MiscSetting),
}

impl SettingId {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Aim(_, AimSetting::Enabled) => "Aim Assist",
            Self::Trigger(_, TriggerSetting::Enabled) => "Triggerbot",
            Self::Rcs(_, RcsSetting::Enabled) => "RCS",
            Self::Player(PlayerSetting::Enabled) => "Player ESP",
            Self::Player(PlayerSetting::OofArrows) => "OOF Arrows",
            Self::Player(PlayerSetting::SoundEsp) => "Sound ESP",
            Self::Hud(HudSetting::SniperCrosshair) => "Sniper Crosshair",
            Self::Hud(HudSetting::GrenadeTrails) => "Grenade Trails",
            Self::Misc(MiscSetting::Bunnyhop) => "Bunnyhop",
            Self::Misc(MiscSetting::NoFlash) => "No Flash",
            Self::Misc(MiscSetting::NoSmoke) => "No Smoke",
            Self::Misc(MiscSetting::FovChanger) => "FOV Changer",
            _ => "Setting",
        }
    }
}
