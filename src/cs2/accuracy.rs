use glam::{Vec2, Vec3};
use std::f32::consts::TAU;

use crate::cs2::{
    CS2,
    entity::player::Player,
    hitbox::{HitCapsule, HitSphere, ray_hits_volumes},
};

const HITCHANCE_SEEDS: usize = 256;

#[derive(Clone, Copy, Debug)]
pub struct WeaponAccuracy {
    pub inaccuracy: f32,
    pub spread: f32,
    pub max_speed: f32,
}

pub fn view_basis(angles: Vec2) -> (Vec3, Vec3, Vec3) {
    let pitch = angles.x.to_radians();
    let yaw = angles.y.to_radians();
    let forward = Vec3::new(
        pitch.cos() * yaw.cos(),
        pitch.cos() * yaw.sin(),
        -pitch.sin(),
    );
    let right = Vec3::new(-yaw.sin(), yaw.cos(), 0.0);
    let up = forward.cross(right).normalize();
    (forward, right, up)
}

fn random_unit(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
    ((*state >> 8) as f32) / ((1_u32 << 24) - 1) as f32
}

fn hash_seed(mut seed: u32) -> u32 {
    seed = (seed ^ 61) ^ (seed >> 16);
    seed = seed.wrapping_add(seed << 3);
    seed ^= seed >> 4;
    seed = seed.wrapping_mul(0x27D4_EB2D);
    seed ^ (seed >> 15)
}

fn spread_offset(seed: usize, accuracy: WeaponAccuracy) -> Vec2 {
    let mut state = hash_seed(seed as u32 + 1);
    let inaccuracy_radius = random_unit(&mut state) * accuracy.inaccuracy;
    let inaccuracy_angle = random_unit(&mut state) * TAU;
    let spread_radius = random_unit(&mut state) * accuracy.spread;
    let spread_angle = random_unit(&mut state) * TAU;
    Vec2::new(
        inaccuracy_angle.cos() * inaccuracy_radius + spread_angle.cos() * spread_radius,
        inaccuracy_angle.sin() * inaccuracy_radius + spread_angle.sin() * spread_radius,
    )
}

pub fn meets_hitchance(
    eye: Vec3,
    view_angles: Vec2,
    spheres: &[HitSphere],
    capsules: &[HitCapsule],
    accuracy: WeaponAccuracy,
    required: f32,
) -> bool {
    if required <= 0.0 {
        return true;
    }

    let required_hits =
        ((required.clamp(0.0, 100.0) / 100.0) * HITCHANCE_SEEDS as f32).ceil() as usize;
    let (forward, right, up) = view_basis(view_angles);
    let mut hits = 0;

    for seed in 0..HITCHANCE_SEEDS {
        let offset = spread_offset(seed, accuracy);
        let direction = (forward + right * offset.x + up * offset.y).normalize();

        if ray_hits_volumes(eye, direction, spheres, capsules) {
            hits += 1;
            if hits >= required_hits {
                return true;
            }
        }
        if hits + HITCHANCE_SEEDS - seed - 1 < required_hits {
            return false;
        }
    }
    false
}

impl CS2 {
    pub(crate) fn live_weapon_accuracy(&self, local: &Player) -> Option<WeaponAccuracy> {
        let offsets = &self.offsets.weapon_accuracy;
        let weapon = local.weapon_address(self)?;
        let vdata: usize = self.process.read(weapon + offsets.vdata?);
        if vdata == 0 {
            return None;
        }

        let mode = offsets
            .weapon_mode
            .map(|offset| self.process.read::<i32>(weapon + offset))
            .unwrap_or_default();
        let firing_mode = |values: [f32; 2]| if mode != 0 { values[1] } else { values[0] };
        let stand = firing_mode(self.process.read(vdata + offsets.inaccuracy_stand?));
        let crouch = offsets
            .inaccuracy_crouch
            .map(|offset| firing_mode(self.process.read(vdata + offset)))
            .unwrap_or(stand);
        let move_inaccuracy = firing_mode(self.process.read(vdata + offsets.inaccuracy_move?));
        let max_speed = firing_mode(self.process.read(vdata + offsets.max_speed?));
        let spread = firing_mode(self.process.read(vdata + offsets.spread?));
        let velocity = local.velocity(self);
        let speed = velocity.truncate().length();
        let movement = ((speed - max_speed * 0.34) / (max_speed * 0.61))
            .clamp(0.0, 1.0)
            .powf(0.25)
            * move_inaccuracy;
        let turning = offsets
            .turning_inaccuracy
            .map(|offset| self.process.read::<f32>(weapon + offset))
            .unwrap_or_default();
        let penalty = offsets
            .accuracy_penalty
            .map(|offset| self.process.read::<f32>(weapon + offset))
            .unwrap_or_default();
        let flags: i32 = self.process.read(local.pawn + self.offsets.pawn.flags);
        let base = if flags & 2 != 0 { crouch } else { stand };
        let air = if flags & 1 == 0 {
            let initial = offsets
                .inaccuracy_jump_initial
                .map(|offset| self.process.read::<f32>(vdata + offset))
                .unwrap_or_default();
            let apex = offsets
                .inaccuracy_jump_apex
                .map(|offset| self.process.read::<f32>(vdata + offset))
                .unwrap_or(initial);
            let vertical = (velocity.z.abs() / 300.0).clamp(0.0, 1.0);
            apex + (initial - apex) * vertical
        } else {
            0.0
        };
        let inaccuracy = base + movement + air.max(0.0) + turning.max(0.0) + penalty.max(0.0);

        (inaccuracy.is_finite()
            && spread.is_finite()
            && max_speed > 0.0
            && (0.0..=1.0).contains(&inaccuracy)
            && (0.0..=1.0).contains(&spread))
        .then_some(WeaponAccuracy {
            inaccuracy,
            spread,
            max_speed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spread_samples_stay_inside_combined_radius() {
        let accuracy = WeaponAccuracy {
            inaccuracy: 0.02,
            spread: 0.01,
            max_speed: 250.0,
        };
        assert!(
            (0..HITCHANCE_SEEDS)
                .map(|seed| spread_offset(seed, accuracy))
                .all(|offset| offset.length() <= accuracy.inaccuracy + accuracy.spread + 1e-6)
        );
    }

    #[test]
    fn inaccuracy_radius_is_linear_not_edge_weighted() {
        let accuracy = WeaponAccuracy {
            inaccuracy: 1.0,
            spread: 0.0,
            max_speed: 250.0,
        };
        let average = (0..HITCHANCE_SEEDS)
            .map(|seed| spread_offset(seed, accuracy).length())
            .sum::<f32>()
            / HITCHANCE_SEEDS as f32;
        assert!((0.42..=0.58).contains(&average));
    }
}
