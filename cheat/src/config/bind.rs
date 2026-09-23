use serde::{Deserialize, Serialize};
pub use shared::{
    AimProfile, AimSetting, HudSetting, MiscSetting, PlayerSetting, RcsSetting, SettingId,
    TriggerSetting,
};

use crate::cs2::key_codes::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BindMode {
    Toggle,
    Hold,
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
    fn missing_bindings_use_current_defaults() {
        let config = Config::default();
        let mut value = toml::Value::try_from(config).expect("serialize config value");
        value.as_table_mut().expect("config table").remove("binds");
        let parsed: Config = value.try_into().expect("deserialize config");

        assert_eq!(parsed.binds.len(), 4);
    }
}
