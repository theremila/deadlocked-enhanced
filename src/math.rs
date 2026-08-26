use glam::{Vec2, Vec3};

use crate::data::Data;

pub fn angles_from_vector(forward: &Vec3) -> Vec2 {
    let yaw = forward.y.atan2(forward.x).to_degrees();
    let pitch = (-forward.z)
        .atan2(Vec2::new(forward.x, forward.y).length())
        .to_degrees();

    Vec2::new(pitch, yaw)
}

pub fn angles_to_fov(view_angles: &Vec2, aim_angles: &Vec2) -> f32 {
    let view_pitch = view_angles.x.to_radians();
    let view_yaw = view_angles.y.to_radians();
    let aim_pitch = aim_angles.x.to_radians();
    let aim_yaw = aim_angles.y.to_radians();

    let view = Vec3::new(
        view_pitch.cos() * view_yaw.cos(),
        view_pitch.cos() * view_yaw.sin(),
        -view_pitch.sin(),
    );
    let aim = Vec3::new(
        aim_pitch.cos() * aim_yaw.cos(),
        aim_pitch.cos() * aim_yaw.sin(),
        -aim_pitch.sin(),
    );

    view.dot(aim).clamp(-1.0, 1.0).acos().to_degrees()
}

pub fn forward_ray_offset(distance: f32, angle: f32) -> Option<f32> {
    if !distance.is_finite() || !angle.is_finite() || distance < 0.0 || angle >= 90.0 {
        return None;
    }

    Some(distance * angle.to_radians().sin())
}

pub fn vec2_clamp(vec: &mut Vec2) {
    if vec.x > 89.0 && vec.x <= 180.0 {
        vec.x = 89.0;
    }
    if vec.x > 180.0 {
        vec.x -= 360.0;
    }
    if vec.x < -89.0 {
        vec.x = -89.0;
    }
    vec.y = (vec.y + 180.0).rem_euclid(360.0) - 180.0;
}

pub fn world_to_screen(position: &Vec3, data: &Data) -> Option<egui::Pos2> {
    let vm = &data.view_matrix;
    let mut screen_position = Vec2::new(
        vm.x_axis.x * position.x
            + vm.x_axis.y * position.y
            + vm.x_axis.z * position.z
            + vm.x_axis.w,
        vm.y_axis.x * position.x
            + vm.y_axis.y * position.y
            + vm.y_axis.z * position.z
            + vm.y_axis.w,
    );

    let w = vm.w_axis.x * position.x
        + vm.w_axis.y * position.y
        + vm.w_axis.z * position.z
        + vm.w_axis.w;

    if w < 0.0001 {
        return None;
    }

    screen_position /= w;

    let half_size = Vec2::new(data.window_size.x * 0.5, data.window_size.y * 0.5);

    screen_position.x = half_size.x + 0.5 * screen_position.x * data.window_size.x + 0.5;
    screen_position.y = half_size.y - 0.5 * screen_position.y * data.window_size.y + 0.5;

    if screen_position.x < 0.0
        || screen_position.x > data.window_size.x
        || screen_position.y < 0.0
        || screen_position.y > data.window_size.y
    {
        return None;
    }

    Some(egui::pos2(screen_position.x, screen_position.y))
}

fn weighted_average_component(
    history: &std::collections::VecDeque<Vec2>,
    component: impl Fn(&Vec2) -> f32,
) -> f32 {
    if history.is_empty() {
        return 0.0;
    }

    let (sum, weight_sum) = history
        .iter()
        .enumerate()
        .fold((0.0, 0.0), |(s, w), (i, v)| {
            let weight = 1.0 + i as f32 * 0.15;
            (s + component(v) * weight, w + weight)
        });
    sum / weight_sum
}

pub fn compute_max_acceleration_component(
    history: &std::collections::VecDeque<Vec2>,
    component: impl Fn(&Vec2) -> f32,
    multiplier: f32,
    range: (f32, f32),
    fallback: f32,
) -> f32 {
    if history.len() < 3 {
        return fallback;
    }

    (weighted_average_component(history, component) * multiplier).clamp(range.0, range.1)
}

pub fn soft_clamp_acceleration(accel: f32, max_accel: f32, decay_rate: f32) -> f32 {
    if accel.abs() <= max_accel {
        return accel;
    }

    let excess = accel.abs() - max_accel;
    accel.signum() * (max_accel + excess * (-excess * decay_rate).exp())
}

pub fn record_acceleration(
    history: &mut std::collections::VecDeque<Vec2>,
    value: Vec2,
    max_size: usize,
) {
    if value.x.abs() < 25.0 && value.y.abs() < 25.0 {
        history.push_front(value.abs());
        if history.len() > max_size {
            history.pop_back();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn angular_fov_handles_wraparound() {
        let view = Vec2::new(0.0, 179.0);
        let target = Vec2::new(0.0, -179.0);

        assert!((angles_to_fov(&view, &target) - 2.0).abs() < 0.001);
    }

    #[test]
    fn forward_ray_rejects_targets_behind_view() {
        assert_eq!(forward_ray_offset(100.0, 180.0), None);
        assert_eq!(forward_ray_offset(100.0, 90.0), None);
    }

    #[test]
    fn forward_ray_reports_perpendicular_offset() {
        let offset = forward_ray_offset(100.0, 30.0).unwrap();

        assert!((offset - 50.0).abs() < 0.001);
    }
}
