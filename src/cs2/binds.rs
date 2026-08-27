use std::collections::{HashMap, HashSet};

use crate::{
    config::{
        Config,
        bind::{BindMode, KeyChord, SettingBind, SettingId},
    },
    cs2::input::Input,
};

#[derive(Default)]
struct TargetState {
    active_chords: HashSet<usize>,
    active_holds: HashSet<usize>,
    latched: bool,
    base_value: bool,
}

#[derive(Default)]
pub struct BindRuntime {
    states: HashMap<SettingId, TargetState>,
    values: HashMap<SettingId, bool>,
    suppressed: bool,
    rebaseline_next: bool,
}

impl BindRuntime {
    pub fn rebaseline(&mut self, config: &Config) {
        let mut previous = std::mem::take(&mut self.states);
        self.values.clear();
        for binding in &config.binds {
            if !Self::has_usable_chord(binding) {
                continue;
            }
            let base_value = config.bool_value(&binding.target);
            let has_toggle = binding.chords.iter().any(|chord| {
                chord.enabled
                    && !chord.keys.is_empty()
                    && chord.mode.unwrap_or(binding.mode) == BindMode::Toggle
            });
            let mut state = previous.remove(&binding.target).unwrap_or_default();
            if state.base_value != base_value {
                state.latched = has_toggle && base_value;
            }
            state.base_value = base_value;
            state.active_chords.clear();
            state.active_holds.clear();
            self.states.insert(binding.target.clone(), state);
        }
    }

    pub fn update(&mut self, config: &Config, input: &Input) {
        self.values.clear();
        for binding in &config.binds {
            if !Self::has_usable_chord(binding) {
                self.states.remove(&binding.target);
                continue;
            }
            let matched = Self::matched_chords(binding, input);
            let state = self
                .states
                .entry(binding.target.clone())
                .or_insert_with(|| {
                    let has_toggle = binding.chords.iter().any(|chord| {
                        chord.enabled
                            && !chord.keys.is_empty()
                            && chord.mode.unwrap_or(binding.mode) == BindMode::Toggle
                    });
                    TargetState {
                        latched: has_toggle && config.bool_value(&binding.target),
                        base_value: config.bool_value(&binding.target),
                        ..TargetState::default()
                    }
                });

            if self.suppressed || self.rebaseline_next {
                state.active_chords = matched;
                state.active_holds.clear();
                self.values.insert(binding.target.clone(), state.latched);
                continue;
            }

            for index in matched.difference(&state.active_chords).copied() {
                let chord = &binding.chords[index];
                match chord.mode.unwrap_or(binding.mode) {
                    BindMode::Toggle => {
                        if chord.keys.iter().any(|key| input.key_just_pressed(*key)) {
                            state.latched = !state.latched;
                        }
                    }
                    BindMode::Hold => {
                        state.active_holds.insert(index);
                    }
                }
            }
            for index in state.active_chords.difference(&matched) {
                state.active_holds.remove(index);
            }
            state.active_chords = matched;

            self.values.insert(
                binding.target.clone(),
                state.latched || !state.active_holds.is_empty(),
            );
        }
        self.rebaseline_next = false;
    }

    pub fn effective_config(&self, base: &Config) -> Config {
        let mut effective = base.clone();
        for (target, value) in &self.values {
            effective.set_bool(target, *value);
        }
        effective
    }

    pub fn values(&self) -> &HashMap<SettingId, bool> {
        &self.values
    }

    pub fn set_suppressed(&mut self, suppressed: bool) {
        if self.suppressed == suppressed {
            return;
        }
        self.suppressed = suppressed;
        if suppressed {
            for state in self.states.values_mut() {
                state.active_holds.clear();
            }
        } else {
            self.rebaseline_next = true;
        }
    }

    fn matched_chords(binding: &SettingBind, input: &Input) -> HashSet<usize> {
        let candidates: Vec<(usize, &KeyChord)> = binding
            .chords
            .iter()
            .enumerate()
            .filter(|(_, chord)| {
                chord.enabled
                    && !chord.keys.is_empty()
                    && chord.keys.iter().all(|key| input.is_key_pressed(*key))
            })
            .collect();

        candidates
            .iter()
            .filter(|(candidate_index, candidate)| {
                !candidates.iter().any(|(other_index, other)| {
                    (*other_index < *candidate_index && candidate.keys == other.keys)
                        || !std::ptr::eq(*candidate, *other) && candidate.is_strict_subset_of(other)
                })
            })
            .map(|(index, _)| *index)
            .collect()
    }

    fn has_usable_chord(binding: &SettingBind) -> bool {
        binding
            .chords
            .iter()
            .any(|chord| chord.enabled && !chord.keys.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::bind::{
            AimProfile, AimSetting, BindMode, KeyChord, PlayerSetting, SettingBind, SettingId,
        },
        cs2::{input::Input, key_codes::KeyCode},
    };

    #[test]
    fn canonical_chord_deduplicates_and_sorts_keys() {
        let chord = KeyChord::new([KeyCode::B, KeyCode::LeftControl, KeyCode::B]);
        assert_eq!(chord.keys, vec![KeyCode::B, KeyCode::LeftControl]);
    }

    #[test]
    fn detects_strict_subset() {
        let key = KeyChord::new([KeyCode::B]);
        let chord = KeyChord::new([KeyCode::LeftControl, KeyCode::B]);
        assert!(key.is_strict_subset_of(&chord));
        assert!(!chord.is_strict_subset_of(&key));
    }

    #[test]
    fn default_bind_targets_stay_stable() {
        let config = Config::default();
        assert!(config.binds.iter().any(|binding| {
            binding.target == SettingId::Aim(AimProfile::Global, AimSetting::Enabled)
                && binding.mode == BindMode::Hold
        }));
    }

    #[test]
    fn toggle_only_flips_on_a_rising_edge() {
        let target = SettingId::Player(PlayerSetting::Chicken);
        let mut config = Config::default();
        config.player.chicken = false;
        config.binds = vec![SettingBind::single(
            target.clone(),
            KeyCode::B,
            BindMode::Toggle,
        )];
        let mut runtime = BindRuntime::default();
        runtime.rebaseline(&config);
        let mut input = Input::new();

        input.set_test_keys(&[KeyCode::B]);
        runtime.update(&config, &input);
        assert_eq!(runtime.values().get(&target), Some(&true));

        input.set_test_keys(&[KeyCode::B]);
        runtime.update(&config, &input);
        assert_eq!(runtime.values().get(&target), Some(&true));

        input.set_test_keys(&[]);
        runtime.update(&config, &input);
        assert_eq!(runtime.values().get(&target), Some(&true));
    }

    #[test]
    fn simultaneous_hold_chords_release_independently() {
        let target = SettingId::Player(PlayerSetting::Chicken);
        let mut config = Config::default();
        config.player.chicken = false;
        config.binds = vec![SettingBind {
            target: target.clone(),
            mode: BindMode::Hold,
            chords: vec![KeyChord::new([KeyCode::B]), KeyChord::new([KeyCode::C])],
        }];
        let mut runtime = BindRuntime::default();
        runtime.rebaseline(&config);
        let mut input = Input::new();

        for keys in [
            vec![KeyCode::B],
            vec![KeyCode::B, KeyCode::C],
            vec![KeyCode::C],
        ] {
            input.set_test_keys(&keys);
            runtime.update(&config, &input);
            assert_eq!(runtime.values().get(&target), Some(&true));
        }
        input.set_test_keys(&[]);
        runtime.update(&config, &input);
        assert_eq!(runtime.values().get(&target), Some(&false));
    }

    #[test]
    fn specific_chord_suppresses_its_subset() {
        let target = SettingId::Player(PlayerSetting::Chicken);
        let mut config = Config::default();
        config.player.chicken = false;
        config.binds = vec![SettingBind {
            target: target.clone(),
            mode: BindMode::Toggle,
            chords: vec![
                KeyChord::new([KeyCode::B]),
                KeyChord::new([KeyCode::LeftControl, KeyCode::B]),
            ],
        }];
        let mut runtime = BindRuntime::default();
        runtime.rebaseline(&config);
        let mut input = Input::new();

        input.set_test_keys(&[KeyCode::LeftControl]);
        runtime.update(&config, &input);
        input.set_test_keys(&[KeyCode::LeftControl, KeyCode::B]);
        runtime.update(&config, &input);
        assert_eq!(runtime.values().get(&target), Some(&true));
    }

    #[test]
    fn capture_rebaseline_does_not_trigger_the_captured_key() {
        let target = SettingId::Player(PlayerSetting::Chicken);
        let mut config = Config::default();
        config.player.chicken = false;
        config.binds = vec![SettingBind::single(
            target.clone(),
            KeyCode::B,
            BindMode::Hold,
        )];
        let mut runtime = BindRuntime::default();
        runtime.rebaseline(&config);
        let mut input = Input::new();

        runtime.set_suppressed(true);
        input.set_test_keys(&[KeyCode::B]);
        runtime.update(&config, &input);
        assert_eq!(runtime.values().get(&target), Some(&false));

        runtime.set_suppressed(false);
        input.set_test_keys(&[KeyCode::B]);
        runtime.update(&config, &input);
        assert_eq!(runtime.values().get(&target), Some(&false));

        input.set_test_keys(&[]);
        runtime.update(&config, &input);
        input.set_test_keys(&[KeyCode::B]);
        runtime.update(&config, &input);
        assert_eq!(runtime.values().get(&target), Some(&true));
    }
}
