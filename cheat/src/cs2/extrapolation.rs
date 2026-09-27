use glam::Vec3;

use crate::cs2::{
    CS2,
    entity::player::Player,
    hitbox::{HitCapsule, HitSphere},
};

pub const DEFAULT_TICK_INTERVAL: f32 = 1.0 / 64.0;
pub const GRAVITY: f32 = 800.0;

/// Calculates total dynamic latency window in seconds, taking local ping and engine tick interval into account.
pub fn calculate_latency_seconds(cs2: &CS2, local: &Player) -> f32 {
    let ping_ms = local.ping(cs2);
    let ping_ms = if ping_ms == 0 { 25.0 } else { ping_ms as f32 };
    let ping_seconds = ping_ms / 1000.0;
    (ping_seconds * 0.5 + DEFAULT_TICK_INTERVAL).clamp(DEFAULT_TICK_INTERVAL, 0.250)
}

/// Computes translation delta based on current velocity, airborne state, and time delta.
#[inline]
pub fn extrapolation_translation(velocity: Vec3, in_air: bool, dt: f32) -> Vec3 {
    let mut translation = velocity * dt;
    if in_air {
        translation.z -= 0.5 * GRAVITY * dt * dt;
    }
    translation
}

/// Extrapolates a single 3D position vector.
#[inline]
pub fn extrapolate_position(position: Vec3, velocity: Vec3, in_air: bool, dt: f32) -> Vec3 {
    position + extrapolation_translation(velocity, in_air, dt)
}

/// Shifts an array of HitSpheres and HitCapsules by a translation vector.
pub fn extrapolate_hitboxes(
    spheres: &[HitSphere],
    capsules: &[HitCapsule],
    translation: Vec3,
) -> (Vec<HitSphere>, Vec<HitCapsule>) {
    let ext_spheres = spheres
        .iter()
        .map(|s| HitSphere {
            center: s.center + translation,
            radius: s.radius,
            bone: s.bone,
        })
        .collect();

    let ext_capsules = capsules
        .iter()
        .map(|c| HitCapsule {
            start: c.start + translation,
            end: c.end + translation,
            radius: c.radius,
        })
        .collect();

    (ext_spheres, ext_capsules)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::Bones;

    #[test]
    fn static_target_has_zero_extrapolation() {
        let pos = Vec3::new(100.0, 200.0, 300.0);
        let vel = Vec3::ZERO;
        let ext = extrapolate_position(pos, vel, false, 0.05);
        assert_eq!(ext, pos);
    }

    #[test]
    fn ground_target_extrapolates_linearly() {
        let pos = Vec3::new(100.0, 0.0, 50.0);
        let vel = Vec3::new(250.0, 0.0, 0.0);
        let dt = 0.1;
        let ext = extrapolate_position(pos, vel, false, dt);
        assert!((ext.x - 125.0).abs() < 1e-4);
        assert!((ext.y - 0.0).abs() < 1e-4);
        assert!((ext.z - 50.0).abs() < 1e-4);
    }

    #[test]
    fn airborne_target_includes_gravity() {
        let pos = Vec3::new(0.0, 0.0, 100.0);
        let vel = Vec3::new(0.0, 0.0, 0.0);
        let dt = 0.1;
        let ext = extrapolate_position(pos, vel, true, dt);
        assert!((ext.z - 96.0).abs() < 1e-4);
    }

    #[test]
    fn hitbox_extrapolation_shifts_all_volumes() {
        let spheres = vec![HitSphere {
            center: Vec3::new(10.0, 20.0, 30.0),
            radius: 5.0,
            bone: Bones::Head,
        }];
        let capsules = vec![HitCapsule {
            start: Vec3::new(0.0, 0.0, 0.0),
            end: Vec3::new(0.0, 0.0, 20.0),
            radius: 4.0,
        }];
        let translation = Vec3::new(5.0, 10.0, 15.0);
        let (ext_spheres, ext_capsules) = extrapolate_hitboxes(&spheres, &capsules, translation);
        assert_eq!(ext_spheres[0].center, Vec3::new(15.0, 30.0, 45.0));
        assert_eq!(ext_spheres[0].radius, 5.0);
        assert_eq!(ext_capsules[0].start, Vec3::new(5.0, 10.0, 15.0));
        assert_eq!(ext_capsules[0].end, Vec3::new(5.0, 10.0, 35.0));
        assert_eq!(ext_capsules[0].radius, 4.0);
    }
}
