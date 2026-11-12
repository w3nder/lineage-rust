use glam::{Mat3, Mat4, Vec3, vec3};
use std::f32::consts::TAU;

const UNITS_PER_TURN: f32 = 65536.0;

pub fn sphere(sphere: [f32; 4], world: Mat4) -> (Vec3, f32) {
    let center = world.transform_point3(Vec3::new(sphere[0], sphere[1], sphere[2]));
    let scale = world.x_axis.truncate().length_squared().max(world.y_axis.truncate().length_squared()).max(world.z_axis.truncate().length_squared()).sqrt();
    (center, sphere[3] * scale)
}

fn angle(units: i32) -> f32 {
    units as f32 * TAU / UNITS_PER_TURN
}

pub fn rotator_matrix(rotator: [i32; 3]) -> Mat3 {
    let (sp, cp) = angle(rotator[0]).sin_cos();
    let (sy, cy) = angle(rotator[1]).sin_cos();
    let (sr, cr) = angle(rotator[2]).sin_cos();
    let x = vec3(cp * cy, cp * sy, sp);
    let y = vec3(sr * sp * cy - cr * sy, sr * sp * sy + cr * cy, -sr * cp);
    let z = vec3(-(cr * sp * cy + sr * sy), cy * sr - cr * sp * sy, cr * cp);
    Mat3::from_cols(x, y, z)
}

pub fn vector_rotation(v: Vec3) -> [i32; 3] {
    let units = |radians: f64| (radians * 10430.219195527361) as i32;
    let v = v.as_dvec3();
    [units(v.z.atan2((v.x * v.x + v.y * v.y).sqrt())), units(v.y.atan2(v.x)), 0]
}

pub fn ortho_rotation(axes: Mat3) -> [i32; 3] {
    let units = |radians: f64| (radians * 32768.0 / std::f64::consts::PI) as i32;
    let x = axes.x_axis.as_dvec3();
    let yaw = units(x.y.atan2(x.x));
    let pitch = units(x.z.atan2((x.x * x.x + x.y * x.y).sqrt()));
    let level = rotator_matrix([pitch, yaw, 0]).y_axis.as_dvec3();
    let roll = units(level.dot(axes.z_axis.as_dvec3()).atan2(level.dot(axes.y_axis.as_dvec3())));
    [pitch, yaw, roll]
}

pub fn placement_matrix(location: [f32; 3], rotator: [i32; 3], scale: [f32; 3]) -> Mat4 {
    let rotation = Mat4::from_mat3(rotator_matrix(rotator));
    Mat4::from_translation(Vec3::from(location)) * rotation * Mat4::from_scale(Vec3::from(scale))
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec3, vec3};

    fn close(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-3
    }

    #[test]
    fn vector_rotation_1014f310_is_pitch_and_yaw_of_the_direction() {
        assert_eq!(vector_rotation(vec3(1.0, 0.0, 0.0)), [0, 0, 0]);
        assert_eq!(vector_rotation(vec3(0.0, 2.0, 0.0)), [0, 16383, 0]);
        assert_eq!(vector_rotation(vec3(0.0, 0.0, 5.0)), [16383, 0, 0]);
        assert_eq!(vector_rotation(vec3(-83309.0, -148013.5, 0.0)), [0, -21730, 0]);
        let r = vector_rotation(vec3(-3.0, -4.0, 2.0));
        assert!(close(rotator_matrix(r).x_axis, vec3(-3.0, -4.0, 2.0).normalize()), "{r:?}");
    }

    #[test]
    fn ortho_rotation_1014f940_inverts_the_rotator_matrix() {
        for rotator in [[0, 0, 0], [0, 16384, 0], [4000, -12000, 9000], [-8000, 30000, -20000], [12000, 5000, 32000]] {
            let back = ortho_rotation(rotator_matrix(rotator));
            let (a, b) = (rotator_matrix(rotator), rotator_matrix(back));
            assert!(close(a.x_axis, b.x_axis) && close(a.y_axis, b.y_axis) && close(a.z_axis, b.z_axis), "{rotator:?} {back:?}");
        }
        assert_eq!(ortho_rotation(Mat3::IDENTITY), [0, 0, 0]);
    }

    #[test]
    fn sphere_transform_101145a0_uses_the_largest_axis() {
        let world = Mat4::from_translation(Vec3::new(10.0, 20.0, 30.0)) * Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
        assert_eq!(sphere([1.0, 2.0, 3.0, 5.0], world), (Vec3::new(12.0, 26.0, 42.0), 20.0));
    }

    #[test]
    fn yaw_quarter_turn_maps_x_to_y() {
        let m = rotator_matrix([0, 16384, 0]);
        assert!(close(m * Vec3::X, Vec3::Y));
        assert!(close(m * Vec3::Y, -Vec3::X));
        assert!(close(m * Vec3::Z, Vec3::Z));
    }

    #[test]
    fn pitch_quarter_turn_lifts_x_to_z() {
        let m = rotator_matrix([16384, 0, 0]);
        assert!(close(m * Vec3::X, Vec3::Z));
    }

    #[test]
    fn roll_quarter_turn_rotates_y_to_minus_z() {
        let m = rotator_matrix([0, 0, 16384]);
        assert!(close(m * Vec3::Y, -Vec3::Z));
        assert!(close(m * Vec3::Z, Vec3::Y));
    }

    #[test]
    fn placement_scales_rotates_then_translates() {
        let m = placement_matrix([100.0, 200.0, 300.0], [0, 16384, 0], [2.0, 2.0, 2.0]);
        let p = m.transform_point3(vec3(1.0, 0.0, 0.0));
        assert!(close(p, vec3(100.0, 202.0, 300.0)));
    }
}
