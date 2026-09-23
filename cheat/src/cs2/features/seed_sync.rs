use std::f32::consts::TAU;

use glam::Vec2;

use crate::cs2::{
    CS2,
    accuracy::{WeaponAccuracy, view_basis},
    entity::player::Player,
    hitbox::{HitCapsule, HitSphere, ShotPathOptions, ray_hit_volumes_translated},
};

pub(crate) const PREDICTION_TICKS: i32 = 2;
const TICK_INTERVAL: f32 = 1.0 / 64.0;

#[derive(Clone, Copy, Debug)]
pub(crate) struct SeedSnapshot {
    command_angles: Vec2,
    tick: i32,
}

impl SeedSnapshot {
    pub(crate) fn is_current(self, command_angles: Vec2, tick: i32) -> bool {
        self.tick == tick && self.command_angles == command_angles
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum SeedPrediction {
    Ready(SeedSnapshot),
    Miss,
    Unavailable,
}

pub(crate) struct SeedTarget<'a> {
    pub player: &'a Player,
    pub spheres: &'a [HitSphere],
    pub capsules: &'a [HitCapsule],
    pub min_damage: i32,
}

#[derive(Clone, Copy)]
pub(crate) struct SeedPredictionOptions {
    pub allow_penetration: bool,
    pub smoke_check: bool,
    pub tick_offset: i32,
    pub prediction_ticks: i32,
}

#[derive(Clone, Copy)]
pub(crate) struct SeedShotMarker {
    pub weapon: usize,
    pub tick: i32,
    pub clip_ammo: i32,
    pub recoil_index: f32,
}

struct ValveRng {
    state: i32,
    index: i32,
    table: [i32; 32],
    seeded: bool,
}

impl ValveRng {
    fn new(seed: i32) -> Self {
        Self {
            state: seed.wrapping_abs().wrapping_neg(),
            index: 0,
            table: [0; 32],
            seeded: false,
        }
    }

    fn generate(&mut self) -> i32 {
        if !self.seeded {
            let mut value = (-self.state).max(1);
            for index in (0..=39_i32).rev() {
                value = Self::lcg(value);
                if index < 32 {
                    self.table[index as usize] = value;
                }
            }
            self.state = value;
            self.index = self.table[0];
            self.seeded = true;
        }

        self.state = Self::lcg(self.state);
        let index = (self.index / 0x4000000) as usize;
        self.index = self.table[index];
        self.table[index] = self.state;
        self.index
    }

    #[allow(clippy::excessive_precision)]
    fn random_float(&mut self, min: f32, max: f32) -> f32 {
        let normalized = 0.999_999_88_f32.min(self.generate() as f32 * 4.656_613e-10);
        min + normalized * (max - min)
    }

    fn lcg(state: i32) -> i32 {
        let quotient = state / 127_773;
        let mut result = 16_807_i32
            .wrapping_mul(state - quotient * 127_773)
            .wrapping_sub(2_836_i32.wrapping_mul(quotient));
        if result < 0 {
            result += 2_147_483_647;
        }
        result
    }
}

fn sha1_first_u32(data: &[u8]) -> u32 {
    let bit_len = (data.len() as u64) * 8;
    let mut padded = Vec::with_capacity((data.len() + 72) & !63);
    padded.extend_from_slice(data);
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&bit_len.to_be_bytes());

    let mut state = [
        0x6745_2301_u32,
        0xEFCD_AB89,
        0x98BA_DCFE,
        0x1032_5476,
        0xC3D2_E1F0,
    ];
    for block in padded.chunks_exact(64) {
        let mut words = [0_u32; 80];
        for (index, word) in words.iter_mut().take(16).enumerate() {
            let start = index * 4;
            *word = u32::from_be_bytes(block[start..start + 4].try_into().unwrap());
        }
        for index in 16..80 {
            words[index] =
                (words[index - 3] ^ words[index - 8] ^ words[index - 14] ^ words[index - 16])
                    .rotate_left(1);
        }

        let [mut a, mut b, mut c, mut d, mut e] = state;
        for (index, word) in words.iter().enumerate() {
            let (function, constant) = match index {
                0..=19 => ((b & c) | (!b & d), 0x5A82_7999),
                20..=39 => (b ^ c ^ d, 0x6ED9_EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1B_BCDC),
                _ => (b ^ c ^ d, 0xCA62_C1D6),
            };
            let next = a
                .rotate_left(5)
                .wrapping_add(function)
                .wrapping_add(e)
                .wrapping_add(constant)
                .wrapping_add(*word);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = next;
        }
        state[0] = state[0].wrapping_add(a);
        state[1] = state[1].wrapping_add(b);
        state[2] = state[2].wrapping_add(c);
        state[3] = state[3].wrapping_add(d);
        state[4] = state[4].wrapping_add(e);
    }

    state[0].swap_bytes()
}

fn quantize_angle(angle: f32) -> f32 {
    let normalized = angle - (angle / 360.0 + 0.5).floor() * 360.0;
    (normalized * 2.0).round() * 0.5
}

fn spread_seed(angles: Vec2, tick: i32) -> u32 {
    let mut input = [0_u8; 12];
    input[..4].copy_from_slice(&quantize_angle(angles.x).to_le_bytes());
    input[4..8].copy_from_slice(&quantize_angle(angles.y).to_le_bytes());
    input[8..].copy_from_slice(&tick.to_le_bytes());
    sha1_first_u32(&input)
}

fn spread_offset(
    seed: i32,
    accuracy: WeaponAccuracy,
    recoil_index: f32,
    item_definition: u16,
    num_bullets: i32,
) -> Vec2 {
    const REVOLVER: u16 = 64;
    const NEGEV: u16 = 28;

    let mut rng = ValveRng::new(seed);
    let mut inaccuracy_radius = rng.random_float(0.0, 1.0);
    let inaccuracy_angle = rng.random_float(0.0, TAU);
    if item_definition == REVOLVER && num_bullets == 1 {
        inaccuracy_radius = 1.0 - inaccuracy_radius * inaccuracy_radius;
    } else if item_definition == NEGEV && recoil_index < 3.0 {
        let mut value = inaccuracy_radius;
        let mut count = 3;
        loop {
            count -= 1;
            value *= value;
            if count as f32 <= recoil_index {
                break;
            }
        }
        inaccuracy_radius = 1.0 - value;
    }

    let mut spread_radius = rng.random_float(0.0, 1.0);
    let spread_angle = rng.random_float(0.0, TAU);
    if item_definition == REVOLVER && num_bullets == 1 {
        spread_radius = 1.0 - spread_radius * spread_radius;
    } else if item_definition == NEGEV && recoil_index < 3.0 {
        let mut value = spread_radius;
        let mut count = 3;
        loop {
            count -= 1;
            value *= value;
            if count as f32 <= recoil_index {
                break;
            }
        }
        spread_radius = 1.0 - value;
    }

    inaccuracy_radius *= accuracy.inaccuracy;
    spread_radius *= accuracy.spread;
    Vec2::new(
        spread_angle.cos() * spread_radius + inaccuracy_angle.cos() * inaccuracy_radius,
        spread_angle.sin() * spread_radius + inaccuracy_angle.sin() * inaccuracy_radius,
    )
}

fn prediction_window_hits(hits: impl IntoIterator<Item = bool>) -> bool {
    hits.into_iter().all(|hit| hit)
}

impl CS2 {
    pub(crate) fn seed_shot_marker(&self, local: &Player) -> Option<SeedShotMarker> {
        let weapon = local.weapon_address(self)?;
        let tick = local.tick_base(self)?;
        let recoil_offset = self.offsets.weapon_accuracy.recoil_index?;
        Some(SeedShotMarker {
            weapon,
            tick,
            clip_ammo: self.process.read(weapon + self.offsets.weapon.clip_primary),
            recoil_index: self.process.read(weapon + recoil_offset),
        })
    }

    pub(crate) fn seed_prediction(
        &self,
        local: &Player,
        accuracy: WeaponAccuracy,
        target: SeedTarget<'_>,
        options: SeedPredictionOptions,
    ) -> SeedPrediction {
        let Some(weapon) = local.weapon_address(self) else {
            return SeedPrediction::Unavailable;
        };
        let Some(tick) = local.tick_base(self) else {
            return SeedPrediction::Unavailable;
        };
        let Some(recoil_offset) = self.offsets.weapon_accuracy.recoil_index else {
            return SeedPrediction::Unavailable;
        };
        let recoil_index: f32 = self.process.read(weapon + recoil_offset);
        if !recoil_index.is_finite() {
            return SeedPrediction::Unavailable;
        }
        let Some(vdata_offset) = self.offsets.weapon_accuracy.vdata else {
            return SeedPrediction::Unavailable;
        };
        let vdata: usize = self.process.read(weapon + vdata_offset);
        if vdata == 0 {
            return SeedPrediction::Unavailable;
        }
        let num_bullets = self
            .offsets
            .weapon_accuracy
            .num_bullets
            .map(|offset| self.process.read::<i32>(vdata + offset))
            .filter(|bullets| (1..=64).contains(bullets))
            .unwrap_or(1);
        let item_definition: u16 = self.process.read(
            weapon
                + self.offsets.weapon.attribute_manager
                + self.offsets.weapon.item
                + self.offsets.econ_item_view.item_definition_index,
        );
        let command_angles = local.view_angles(self);
        if !command_angles.is_finite()
            || command_angles.x.abs() > 89.0
            || command_angles.y.abs() > 360.0
        {
            return SeedPrediction::Unavailable;
        }
        let recoil_scale = self
            .offsets
            .convar
            .recoil_scale
            .map(|address| self.process.read::<f32>(address + 0x58))
            .filter(|scale| scale.is_finite() && (0.0..=10.0).contains(scale))
            .unwrap_or(2.0);
        let shot_angles = command_angles + local.aim_punch(self) * recoil_scale;
        let (forward, right, up) = view_basis(shot_angles);
        let eye = local.eye_position(self);
        let local_velocity = local.velocity(self);
        let target_velocity = target.player.velocity(self);

        // Every plausible server tick must produce a valid shot.
        let hits =
            prediction_window_hits((0..options.prediction_ticks.max(1)).map(|prediction_tick| {
                let candidate_offset = options.tick_offset + prediction_tick;
                let prediction_time = candidate_offset as f32 * TICK_INTERVAL;
                let candidate_eye = eye + local_velocity * prediction_time;
                let target_translation = target_velocity * prediction_time;
                let seed = spread_seed(command_angles, tick + candidate_offset);
                let spread = spread_offset(
                    seed.wrapping_add(1) as i32,
                    accuracy,
                    recoil_index,
                    item_definition,
                    num_bullets,
                );
                let direction = (forward + right * spread.x + up * spread.y).normalize();
                let Some(hit) = ray_hit_volumes_translated(
                    candidate_eye,
                    direction,
                    target.spheres,
                    target.capsules,
                    target_translation,
                ) else {
                    return false;
                };
                if options.smoke_check && self.is_line_in_smoke(candidate_eye, hit.point) {
                    return false;
                }
                self.evaluate_shot_path_from(
                    candidate_eye,
                    local,
                    target.player,
                    hit.point,
                    hit.bone,
                    ShotPathOptions {
                        allow_penetration: options.allow_penetration,
                        min_damage: target.min_damage,
                    },
                )
                .is_some()
            }));
        if hits {
            SeedPrediction::Ready(SeedSnapshot {
                command_angles,
                tick,
            })
        } else {
            SeedPrediction::Miss
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_matches_expected_byte_order() {
        assert_eq!(sha1_first_u32(b"abc"), 0x363E_99A9);
    }

    #[test]
    fn seed_changes_with_tick() {
        let angles = Vec2::new(12.5, -90.0);
        assert_ne!(spread_seed(angles, 100), spread_seed(angles, 101));
    }

    #[test]
    fn every_uncertain_prediction_tick_must_hit() {
        assert!(prediction_window_hits([true, true]));
        assert!(!prediction_window_hits([true, false]));
        assert!(!prediction_window_hits([false, true]));
    }

    #[test]
    fn snapshot_rejects_a_new_tick_or_seed_angle_bucket() {
        let snapshot = SeedSnapshot {
            command_angles: Vec2::new(10.0, 20.0),
            tick: 100,
        };
        assert!(snapshot.is_current(Vec2::new(10.0, 20.0), 100));
        assert!(!snapshot.is_current(Vec2::new(10.1, 20.0), 100));
        assert!(!snapshot.is_current(Vec2::new(10.0, 20.0), 101));
    }
}
