use glam::Vec3;

use crate::cs2::{CS2, bones::Bones, entity::player::Player};

const MULTIPOINT_SCALE: f32 = 0.65;

#[derive(Clone, Copy, Debug)]
pub struct HitSphere {
    pub bone: Bones,
    pub center: Vec3,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct HitCapsule {
    pub start: Vec3,
    pub end: Vec3,
    pub radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct ShotPath {
    pub damage: f32,
    pub penetrated: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct ShotPathOptions {
    pub allow_penetration: bool,
    pub min_damage: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct RayVolumeHit {
    pub point: Vec3,
    pub distance: f32,
    pub bone: Bones,
}

pub fn bone_radius(bone: Bones) -> f32 {
    match bone {
        Bones::Head => 4.5,
        Bones::Neck => 4.0,
        Bones::Spine4 | Bones::Spine3 | Bones::Spine2 | Bones::Spine1 => 8.5,
        Bones::Hip => 8.0,
        Bones::LeftShoulder
        | Bones::RightShoulder
        | Bones::LeftElbow
        | Bones::RightElbow
        | Bones::LeftHand
        | Bones::RightHand => 4.5,
        Bones::LeftHip | Bones::RightHip | Bones::LeftKnee | Bones::RightKnee => 5.0,
        Bones::LeftFoot | Bones::RightFoot => 4.0,
    }
}

pub fn spheres_from_bones(
    all_bones: &std::collections::HashMap<Bones, Vec3>,
    bones: &[Bones],
    head_only: bool,
) -> Vec<HitSphere> {
    bones
        .iter()
        .copied()
        .filter(|bone| !head_only || *bone == Bones::Head)
        .filter_map(|bone| {
            let center = all_bones.get(&bone).copied().unwrap_or(Vec3::ZERO);
            (center.is_finite() && center != Vec3::ZERO).then_some(HitSphere {
                bone,
                center,
                radius: bone_radius(bone),
            })
        })
        .collect()
}

pub fn spheres(cs2: &CS2, player: &Player, bones: &[Bones], head_only: bool) -> Vec<HitSphere> {
    let all_bones = player.all_bones(cs2);
    spheres_from_bones(&all_bones, bones, head_only)
}

pub fn capsules(hit_spheres: &[HitSphere]) -> Vec<HitCapsule> {
    Bones::CONNECTIONS
        .iter()
        .filter_map(|&(start_bone, end_bone)| {
            let start = hit_spheres.iter().find(|hit| hit.bone == start_bone)?;
            let end = hit_spheres.iter().find(|hit| hit.bone == end_bone)?;
            Some(HitCapsule {
                start: start.center,
                end: end.center,
                radius: start.radius.min(end.radius),
            })
        })
        .collect()
}

pub fn multipoints(hit: HitSphere, eye: Vec3) -> Vec<Vec3> {
    let direction = (hit.center - eye).normalize_or_zero();
    if direction == Vec3::ZERO {
        return vec![hit.center];
    }
    let reference = if direction.z.abs() > 0.9 {
        Vec3::Y
    } else {
        Vec3::Z
    };
    let right = direction.cross(reference).normalize_or_zero();
    let up = right.cross(direction).normalize_or_zero();
    let extent = hit.radius * MULTIPOINT_SCALE;

    vec![
        hit.center,
        hit.center + right * extent,
        hit.center - right * extent,
        hit.center + up * extent,
        hit.center - up * extent,
    ]
}

#[cfg(test)]
pub fn ray_hits_capsule(origin: Vec3, direction: Vec3, capsule: HitCapsule) -> bool {
    ray_capsule_distance(origin, direction.normalize_or_zero(), capsule).is_some()
}

fn ray_sphere_distance(origin: Vec3, direction: Vec3, center: Vec3, radius: f32) -> Option<f32> {
    let offset = origin - center;
    let projected = offset.dot(direction);
    let discriminant = projected * projected - (offset.length_squared() - radius * radius);
    if discriminant < 0.0 {
        return None;
    }
    let near = -projected - discriminant.sqrt();
    let far = -projected + discriminant.sqrt();
    (far > 0.0).then_some(near.max(0.0))
}

fn ray_capsule_distance(origin: Vec3, direction: Vec3, capsule: HitCapsule) -> Option<f32> {
    let axis = capsule.end - capsule.start;
    let offset = origin - capsule.start;
    let axis_sq = axis.length_squared();
    if axis_sq <= f32::EPSILON {
        return ray_sphere_distance(origin, direction, capsule.start, capsule.radius);
    }

    let axis_ray = axis.dot(direction);
    let axis_origin = axis.dot(offset);
    let ray_origin = direction.dot(offset);
    let a = axis_sq - axis_ray * axis_ray;
    let b = axis_sq * ray_origin - axis_origin * axis_ray;
    let c = axis_sq * offset.length_squared()
        - axis_origin * axis_origin
        - capsule.radius * capsule.radius * axis_sq;
    if a.abs() > f32::EPSILON {
        let discriminant = b * b - a * c;
        if discriminant >= 0.0 {
            let distance = (-b - discriminant.sqrt()) / a;
            let height = axis_origin + distance * axis_ray;
            if distance >= 0.0 && height > 0.0 && height < axis_sq {
                return Some(distance);
            }
        }
    }

    let start = ray_sphere_distance(origin, direction, capsule.start, capsule.radius);
    let end = ray_sphere_distance(origin, direction, capsule.end, capsule.radius);
    match (start, end) {
        (Some(start), Some(end)) => Some(start.min(end)),
        (Some(distance), None) | (None, Some(distance)) => Some(distance),
        (None, None) => None,
    }
}

pub fn ray_hits_volumes(
    origin: Vec3,
    direction: Vec3,
    hit_spheres: &[HitSphere],
    hit_capsules: &[HitCapsule],
) -> bool {
    ray_hit_volumes_translated(origin, direction, hit_spheres, hit_capsules, Vec3::ZERO).is_some()
}

pub fn ray_hit_volumes_translated(
    origin: Vec3,
    direction: Vec3,
    hit_spheres: &[HitSphere],
    hit_capsules: &[HitCapsule],
    translation: Vec3,
) -> Option<RayVolumeHit> {
    let direction = direction.normalize_or_zero();
    if direction == Vec3::ZERO {
        return None;
    }

    let mut nearest: Option<RayVolumeHit> = None;
    let mut consider = |candidate: RayVolumeHit| {
        if nearest.is_none_or(|hit| candidate.distance < hit.distance) {
            nearest = Some(candidate);
        }
    };

    for hit in hit_spheres {
        let center = hit.center + translation;
        let Some(distance) = ray_sphere_distance(origin, direction, center, hit.radius) else {
            continue;
        };
        consider(RayVolumeHit {
            point: origin + direction * distance,
            distance,
            bone: hit.bone,
        });
    }

    for capsule in hit_capsules {
        let translated = HitCapsule {
            start: capsule.start + translation,
            end: capsule.end + translation,
            radius: capsule.radius,
        };
        let Some(distance) = ray_capsule_distance(origin, direction, translated) else {
            continue;
        };
        let point = origin + direction * distance;
        let bone = hit_spheres
            .iter()
            .min_by(|left, right| {
                (left.center + translation)
                    .distance_squared(point)
                    .total_cmp(&(right.center + translation).distance_squared(point))
            })
            .map_or(Bones::Spine3, |hit| hit.bone);
        consider(RayVolumeHit {
            point,
            distance,
            bone,
        });
    }

    nearest
}

impl CS2 {
    pub(crate) fn evaluate_shot_path(
        &self,
        local: &Player,
        target: &Player,
        point: Vec3,
        bone: Bones,
        allow_penetration: bool,
        min_damage: i32,
    ) -> Option<ShotPath> {
        let start = local.eye_position(self);
        self.evaluate_shot_path_from(
            start,
            local,
            target,
            point,
            bone,
            ShotPathOptions {
                allow_penetration,
                min_damage,
            },
        )
    }

    pub(crate) fn evaluate_shot_path_from(
        &self,
        start: Vec3,
        local: &Player,
        target: &Player,
        point: Vec3,
        bone: Bones,
        options: ShotPathOptions,
    ) -> Option<ShotPath> {
        let visible = self.bvh.as_ref().map_or_else(
            || target.visible(self, local),
            |bvh| bvh.has_line_of_sight(start, point),
        );
        let damage = if visible {
            self.calculate_direct_damage(
                start,
                point,
                bone,
                target.armor(self),
                target.has_helmet(self),
            )
        } else if options.allow_penetration {
            self.calculate_damage(
                start,
                point,
                bone,
                target.armor(self),
                target.has_helmet(self),
            )
        } else {
            return None;
        };
        (damage >= options.min_damage.max(1) as f32).then_some(ShotPath {
            damage,
            penetrated: !visible,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_hits_capsule_between_bones() {
        let capsule = HitCapsule {
            start: Vec3::new(100.0, 0.0, -5.0),
            end: Vec3::new(100.0, 0.0, 5.0),
            radius: 3.0,
        };
        assert!(ray_hits_capsule(Vec3::ZERO, Vec3::X, capsule));
        assert!(!ray_hits_capsule(
            Vec3::new(0.0, 4.0, 0.0),
            Vec3::X,
            capsule
        ));
    }

    #[test]
    fn multipoints_stay_inside_sphere() {
        let hit = HitSphere {
            bone: Bones::Head,
            center: Vec3::new(100.0, 0.0, 0.0),
            radius: 4.5,
        };
        assert!(
            multipoints(hit, Vec3::ZERO)
                .into_iter()
                .all(|point| point.distance(hit.center) <= hit.radius)
        );
    }

    #[test]
    fn translated_ray_query_tracks_a_moving_hitbox() {
        let sphere = HitSphere {
            bone: Bones::Head,
            center: Vec3::new(100.0, 10.0, 0.0),
            radius: 2.0,
        };
        assert!(
            ray_hit_volumes_translated(
                Vec3::ZERO,
                Vec3::X,
                &[sphere],
                &[],
                Vec3::new(0.0, -10.0, 0.0),
            )
            .is_some()
        );
        assert!(
            ray_hit_volumes_translated(Vec3::ZERO, Vec3::X, &[sphere], &[], Vec3::ZERO).is_none()
        );
    }
}
