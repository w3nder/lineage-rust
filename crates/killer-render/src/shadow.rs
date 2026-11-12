#[derive(Clone, Copy, Debug)]
pub struct Viewer {
    pub controller: [f32; 3],
    pub pawn: Option<[f32; 3]>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LightInfo {
    pub blob: bool,
    pub direction: [f32; 3],
    pub distance: f32,
    pub max_trace_distance: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Sun {
    pub hour: f64,
    pub direction: [f32; 3],
}

#[derive(Clone, Copy, Debug)]
pub struct LightContext {
    pub default_shadow: i32,
    pub has_viewport: bool,
    pub pawn: Option<[f32; 3]>,
    pub projector: [f32; 3],
    pub owner: Option<[f32; 3]>,
    pub clipping_range: f32,
    pub sun: Option<Sun>,
    pub has_actor: bool,
    pub collision_height: f32,
}

impl LightInfo {
    pub fn update(&mut self, context: LightContext) {
        if context.default_shadow >= 2 {
            self.blob = true;
            return;
        }
        self.blob = false;
        if context.default_shadow == 1 {
            if !context.has_viewport {
                return;
            }
            if let Some(pawn) = context.pawn {
                let location = context.owner.unwrap_or(context.projector);
                let dx = f64::from(pawn[0]) - f64::from(location[0]);
                let dy = f64::from(pawn[1]) - f64::from(location[1]);
                let range = (f64::from(context.clipping_range) * f64::from(f32::from_bits(0x3f2aaaab)) * 2048.0) as f32;
                self.blob = (dx * dx + dy * dy).sqrt() > f64::from(range);
            }
        }
        let Some(sun) = context.sun else { return };
        let [x, y, z] = sun.direction.map(f64::from);
        let length = (x * x + y * y + z * z).sqrt();
        self.direction = if (6.0..=24.0).contains(&sun.hour) && length != 0.0 {
            sun.direction
        } else {
            [0.0, 0.0, 10.0]
        };
        let [x, y, z] = self.direction.map(f64::from);
        let squared = x * x + y * y + z * z;
        if squared >= 1e-8 {
            let inverse = 1.0 / squared.sqrt();
            self.direction = [x, y, z].map(|v| (v * inverse) as f32);
        }
        let minimum_z = self.direction[0].abs() * 0.5;
        if self.direction[2].abs() <= minimum_z {
            self.direction[2] = if self.direction[2] < 0.0 { -minimum_z } else { minimum_z };
        }
        let distance = if context.has_actor { f64::from(context.collision_height) * 32.0 } else { 1500.0 };
        self.distance = distance as f32;
        self.max_trace_distance = if (-9223372036854775808.0..9223372036854775808.0).contains(&distance) {
            (distance as i64) as i32
        } else {
            0
        };
    }
}

pub fn hidden(
    previous: bool,
    has_actor: bool,
    projector: [f32; 3],
    owner: Option<[f32; 3]>,
    viewer: Option<Viewer>,
    clipping_range: f32,
) -> bool {
    if !has_actor {
        return previous;
    }
    let Some(viewer) = viewer else { return true };
    let viewer = viewer.pawn.unwrap_or(viewer.controller);
    let location = owner.unwrap_or(projector);
    let [dx, dy, dz] = std::array::from_fn(|i| f64::from(viewer[i]) - f64::from(location[i]));
    let range = (f64::from(clipping_range) * 2048.0) as f32;
    !(dz > -200.0 && dz < 200.0 && (dx * dx + dy * dy).sqrt() <= f64::from(range))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewer(position: [f32; 3]) -> Option<Viewer> {
        Some(Viewer { controller: position, pawn: None })
    }

    fn light_context() -> LightContext {
        LightContext {
            default_shadow: 0,
            has_viewport: true,
            pawn: Some([0.0; 3]),
            projector: [0.0; 3],
            owner: None,
            clipping_range: 0.2,
            sun: Some(Sun { hour: 12.0, direction: [1.0, 0.0, 0.0] }),
            has_actor: true,
            collision_height: 22.123,
        }
    }

    fn light_info() -> LightInfo {
        LightInfo { blob: false, direction: [0.1, 0.2, 0.3], distance: 1520.0, max_trace_distance: 1520 }
    }

    #[test]
    fn update_light_preserves_geometry_on_early_returns() {
        for context in [
            LightContext { default_shadow: 2, ..light_context() },
            LightContext { default_shadow: 1, has_viewport: false, ..light_context() },
            LightContext { sun: None, ..light_context() },
        ] {
            let mut light = light_info();
            light.update(context);
            assert_eq!(light.direction, light_info().direction);
            assert_eq!(light.distance, light_info().distance);
            assert_eq!(light.max_trace_distance, light_info().max_trace_distance);
            assert_eq!(light.blob, context.default_shadow >= 2);
        }
    }

    #[test]
    fn update_light_limits_sun_angle_and_truncates_projector_height() {
        let mut light = light_info();
        light.update(light_context());
        assert_eq!(light.direction, [1.0, 0.0, 0.5]);
        assert_eq!(light.distance, 22.123 * 32.0);
        assert_eq!(light.max_trace_distance, 707);
        light.update(LightContext { has_actor: false, ..light_context() });
        assert_eq!(light.distance, 1500.0);
        assert_eq!(light.max_trace_distance, 1500);
    }

    #[test]
    fn update_light_blob_threshold_does_not_use_controller_fallback() {
        let mut light = light_info();
        for (pawn, expected) in [(None, false), (Some([0.0; 3]), false), (Some([274.0, 0.0, 0.0]), true)] {
            light.update(LightContext { default_shadow: 1, pawn, ..light_context() });
            assert_eq!(light.blob, expected);
        }
    }

    #[test]
    fn check_visible_1066de80_preserves_state_without_actor() {
        for previous in [false, true] {
            assert_eq!(hidden(previous, false, [0.0; 3], None, viewer([0.0; 3]), 0.2), previous);
        }
    }

    #[test]
    fn check_visible_1066de80_uses_owner_and_strict_vertical_limits() {
        assert!(hidden(false, true, [0.0; 3], None, None, 0.2));
        for dz in [-200.0, 200.0] {
            assert!(hidden(false, true, [0.0; 3], None, viewer([0.0, 0.0, dz]), 0.2));
        }
        assert!(!hidden(true, true, [0.0; 3], None, viewer([409.6, 0.0, 199.999]), 0.2));
        assert!(hidden(false, true, [0.0; 3], None, viewer([409.601, 0.0, 0.0]), 0.2));
        assert!(!hidden(true, true, [5000.0; 3], Some([0.0; 3]), viewer([0.0; 3]), 0.2));
    }

    #[test]
    fn check_visible_1066de80_prefers_pawn_over_controller() {
        for (controller, pawn, expected) in [([5000.0; 3], [0.0; 3], false), ([0.0; 3], [5000.0; 3], true)] {
            assert_eq!(
                hidden(false, true, [0.0; 3], None, Some(Viewer { controller, pawn: Some(pawn) }), 0.2),
                expected
            );
        }
    }

    #[test]
    fn shadow_projection_uses_native_fov_margin_and_draw_scale() {
        let projection = projection(Vec3::new(10.0, 20.0, 30.0), 78.0, Vec3::new(1.0, 2.0, 5.0), 704.0);
        let expected_fov = (316.0f32).atan2(704.0).to_degrees() + 5.0;
        assert!((projection.fov_deg - expected_fov).abs() < 1e-5);
        assert!((projection.draw_scale - 704.0 * (expected_fov.to_radians() * 0.5).tan() / 128.0).abs() < 1e-5);
        assert!((projection.direction.length() - 1.0).abs() < 1e-6);
        assert_eq!(projection.camera, projection.origin - projection.direction * 704.0);
        let uv = projection.texture_matrix * projection.origin.extend(1.0);
        assert!((uv.x / uv.w - 0.5).abs() < 1e-5);
        assert!((uv.y / uv.w - 0.5).abs() < 1e-5);
    }

    #[test]
    fn shadow_target_geometry_keeps_the_c4_pool_size() {
        assert_eq!(SHADOW_TARGETS, 20);
        assert_eq!(SHADOW_SIZE, 256);
    }
}
use glam::{Mat4, Vec3, Vec4};

pub const SHADOW_TARGETS: usize = 20;
pub const SHADOW_SIZE: u32 = 256;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projection {
    pub origin: Vec3,
    pub camera: Vec3,
    pub direction: Vec3,
    pub distance: f32,
    pub fov_deg: f32,
    pub draw_scale: f32,
    pub view_proj: Mat4,
    pub texture_matrix: Mat4,
}

pub fn projection(origin: Vec3, radius: f32, direction: Vec3, distance: f32) -> Projection {
    let direction = direction.normalize_or_zero();
    let fov_deg = (2.0 * radius + 160.0).atan2(distance).to_degrees() + 5.0;
    let draw_scale = distance * (0.5 * fov_deg.to_radians()).tan() / 128.0;
    let camera = origin - direction * distance;
    let camera_model = crate::camera::Camera { eye: camera, target: origin, fov_x_deg: fov_deg, near: 1.0, far: distance * 2.0 };
    let view_proj = camera_model.view_proj(1.0);
    let texture_bias = Mat4::from_cols(
        Vec4::new(0.5, 0.0, 0.0, 0.0),
        Vec4::new(0.0, 0.5, 0.0, 0.0),
        Vec4::new(0.0, 0.0, 1.0, 0.0),
        Vec4::new(0.5, 0.5, 0.0, 1.0),
    );
    Projection { origin, camera, direction, distance, fov_deg, draw_scale, view_proj, texture_matrix: texture_bias * view_proj }
}
