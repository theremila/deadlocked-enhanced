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

pub fn spheres(cs2: &CS2, player: &Player, bones: &[Bones], head_only: bool) -> Vec<HitSphere> {
    bones
        .iter()
        .copied()
        .filter(|bone| !head_only || *bone == Bones::Head)
        .filter_map(|bone| {
            let center = player.bone_position(cs2, bone.u64());
            (center.is_finite() && center != Vec3::ZERO).then_some(HitSphere {
                bone,
                center,
                radius: bone_radius(bone),
            })
        })
        .collect()
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

pub fn ray_hits_sphere(origin: Vec3, direction: Vec3, hit: HitSphere) -> bool {
    let to_center = hit.center - origin;
    let projection = to_center.dot(direction);
    projection > 0.0
        && (to_center.length_squared() - projection * projection) <= hit.radius * hit.radius
}

pub fn ray_hits_capsule(origin: Vec3, direction: Vec3, capsule: HitCapsule) -> bool {
    let segment = capsule.end - capsule.start;
    let offset = origin - capsule.start;
    let segment_len_sq = segment.length_squared();
    if segment_len_sq <= f32::EPSILON {
        return ray_hits_sphere(
            origin,
            direction,
            HitSphere {
                bone: Bones::Head,
                center: capsule.start,
                radius: capsule.radius,
            },
        );
    }

    let segment_dot_ray = segment.dot(direction);
    let denominator = segment_len_sq - segment_dot_ray * segment_dot_ray;
    let segment_t = if denominator.abs() <= f32::EPSILON {
        (-offset.dot(segment) / segment_len_sq).clamp(0.0, 1.0)
    } else {
        ((offset.dot(segment) - segment_dot_ray * offset.dot(direction)) / denominator)
            .clamp(0.0, 1.0)
    };
    let closest_on_segment = capsule.start + segment * segment_t;
    let ray_t = (closest_on_segment - origin).dot(direction);
    ray_t > 0.0
        && (origin + direction * ray_t - closest_on_segment).length_squared()
            <= capsule.radius * capsule.radius
}

pub fn ray_hits_volumes(
    origin: Vec3,
    direction: Vec3,
    hit_spheres: &[HitSphere],
    hit_capsules: &[HitCapsule],
) -> bool {
    hit_spheres
        .iter()
        .copied()
        .any(|hit| ray_hits_sphere(origin, direction, hit))
        || hit_capsules
            .iter()
            .copied()
            .any(|hit| ray_hits_capsule(origin, direction, hit))
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
        } else if allow_penetration {
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
        (damage >= min_damage.max(1) as f32).then_some(ShotPath {
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
}
