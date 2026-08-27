use serde::{Deserialize, Serialize};

use crate::{
    config::aim::KeyMode,
    cs2::{entity::weapon::Weapon, key_codes::KeyCode},
};

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
    DistanceAdjustedFov,
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

/// Stable identity shared by the config, GUI and game-thread bind evaluator.
/// Labels deliberately do not participate, so renaming UI text cannot break a bind.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindMode {
    Toggle,
    Hold,
}

impl From<KeyMode> for BindMode {
    fn from(value: KeyMode) -> Self {
        match value {
            KeyMode::Toggle => Self::Toggle,
            KeyMode::Hold => Self::Hold,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyChord {
    pub keys: Vec<KeyCode>,
    pub mode: Option<BindMode>,
    pub enabled: bool,
}

impl KeyChord {
    pub fn new(keys: impl IntoIterator<Item = KeyCode>) -> Self {
        let mut chord = Self {
            keys: keys
                .into_iter()
                .filter(|key| *key != KeyCode::None)
                .collect(),
            mode: None,
            enabled: true,
        };
        chord.canonicalize();
        chord
    }

    pub fn canonicalize(&mut self) {
        self.keys.sort_unstable_by_key(|key| *key as usize);
        self.keys.dedup();
        self.keys.retain(|key| *key != KeyCode::None);
    }

    pub fn is_strict_subset_of(&self, other: &Self) -> bool {
        self.keys.len() < other.keys.len() && self.keys.iter().all(|key| other.keys.contains(key))
    }
}

impl Default for KeyChord {
    fn default() -> Self {
        Self::new([])
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct SettingBind {
    pub target: SettingId,
    pub mode: BindMode,
    pub chords: Vec<KeyChord>,
}

impl SettingBind {
    pub fn single(target: SettingId, key: KeyCode, mode: BindMode) -> Self {
        Self {
            target,
            mode,
            chords: vec![KeyChord::new([key])],
        }
    }

    pub fn chord_text(&self) -> String {
        self.chords
            .iter()
            .filter(|chord| chord.enabled && !chord.keys.is_empty())
            .map(|chord| {
                chord
                    .keys
                    .iter()
                    .map(|key| format!("{key:?}"))
                    .collect::<Vec<_>>()
                    .join("+")
            })
            .collect::<Vec<_>>()
            .join(" / ")
    }

    pub fn has_visible_chord(&self) -> bool {
        self.chords
            .iter()
            .any(|chord| chord.enabled && !chord.keys.is_empty())
    }
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

impl Default for SettingBind {
    fn default() -> Self {
        Self {
            target: SettingId::Player(PlayerSetting::Enabled),
            mode: BindMode::Toggle,
            chords: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::Config;

    #[test]
    fn config_bindings_round_trip_through_toml() {
        let config = Config::default();
        let serialized = toml::to_string(&config).expect("serialize config");
        let parsed: Config = toml::from_str(&serialized).expect("deserialize config");
        assert_eq!(parsed.binds, config.binds);
    }

    #[test]
    fn missing_bindings_are_available_for_legacy_migration() {
        let config = Config::default();
        let mut value = toml::Value::try_from(config).expect("serialize config value");
        value.as_table_mut().expect("config table").remove("binds");
        let mut parsed: Config = value.try_into().expect("deserialize legacy config");
        assert!(parsed.binds.is_empty());

        parsed.ensure_legacy_binds();
        assert_eq!(parsed.binds.len(), 4);
    }
}
