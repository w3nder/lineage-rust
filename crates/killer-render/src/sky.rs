use crate::camera::Camera;
use crate::environment::SkyState;
use glam::{Mat4, Vec2, Vec3, Vec4};
use killer_assets::scene::Vertex;
use killer_assets::scene::sky::{BodyKind, Flare, SkyBodyDraw};

/// `RenderLensFlare`: `base = ftol(SizeX · 0.3)`, `maxSize = ftol(SizeX · 0.4)`.
const FLARE_BASE_FRACTION: f32 = 0.3;
const FLARE_MAX_FRACTION: f32 = 0.4;
/// `_ftol` on the engine's extended-precision intermediates: a hair below an integer still truncates to it.
const FTOL_EPSILON: f32 = 1e-3;

fn ftol(value: f32) -> f32 {
    (value + FTOL_EPSILON.copysign(value)).trunc()
}
/// The weather the killer runs in: `UL2NEnvManager` EnvType 0, no transition.
pub const CURRENT_ENV_TYPE: u8 = 0;

pub fn sky_camera(camera: &Camera, sky_location: Vec3) -> Camera {
    Camera { eye: sky_location, target: sky_location + (camera.target - camera.eye), ..*camera }
}

/// `CameraX`/`CameraY` of the scene node: the world directions along which screen x grows and screen y (downwards) grows.
pub fn screen_axes(view_proj: Mat4) -> (Vec3, Vec3) {
    let inverse = view_proj.inverse();
    let unproject = |x: f32, y: f32| {
        let p = inverse * Vec4::new(x, y, 0.5, 1.0);
        p.truncate() / p.w
    };
    let origin = unproject(0.0, 0.0);
    ((unproject(1.0, 0.0) - origin).normalize_or(Vec3::X), (unproject(0.0, -1.0) - origin).normalize_or(-Vec3::Z))
}

/// `RenderSun`/`RenderMoon`: `X = CameraX̂ · half`, `Y = −CameraŶ · half`, quad `Loc − X − Y, + X − Y, + X + Y, − X + Y` with uv (0,0) (1,0) (1,1) (0,1).
pub fn body_quad(center: Vec3, half: f32, view_proj: Mat4, color: [f32; 4]) -> [Vertex; 4] {
    let (camera_x, camera_y) = screen_axes(view_proj);
    let x = camera_x * half;
    let y = -camera_y * half;
    let normal = camera_x.cross(y).normalize_or(Vec3::Z);
    let corner = |p: Vec3, uv: [f32; 2]| Vertex { position: p.to_array(), normal: normal.to_array(), uv, mask_uv: uv, color };
    [corner(center - x - y, [0.0, 0.0]), corner(center + x - y, [1.0, 0.0]), corner(center + x + y, [1.0, 1.0]), corner(center - x + y, [0.0, 1.0])]
}

pub const QUAD_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

pub struct BodyPlacement {
    pub center: Vec3,
    pub half: f32,
    pub color: [f32; 4],
}

/// Where and how large a sun or moon is this hour, or `None` when `RenderSun`/`RenderMoon` skip it.
pub fn place_body(body: &SkyBodyDraw, sky: &SkyState) -> Option<BodyPlacement> {
    let origin = Vec3::from(body.origin);
    match body.kind {
        BodyKind::Sun => sky.sun_visible.then(|| BodyPlacement {
            center: origin + sky.sun_body * body.radius,
            half: body.half_width_per_scale * sky.sun_scale,
            color: [sky.sun_sprite[0], sky.sun_sprite[1], sky.sun_sprite[2], 1.0],
        }),
        BodyKind::Moon => (sky.moon_visible && body.env_type == CURRENT_ENV_TYPE).then(|| BodyPlacement {
            center: origin + sky.moon_body * body.radius,
            half: body.half_width_per_scale * sky.moon_scale,
            color: [1.0; 4],
        }),
    }
}

/// The sun's pixel position on a `width × height` viewport, or `None` when it is behind the camera.
pub fn project(view_proj: Mat4, point: Vec3, width: f32, height: f32) -> Option<Vec2> {
    let clip = view_proj * point.extend(1.0);
    if clip.w <= 0.0 {
        return None;
    }
    let ndc = clip / clip.w;
    Some(Vec2::new((ndc.x + 1.0) * 0.5 * width, (1.0 - ndc.y) * 0.5 * height))
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlareQuad {
    pub center: Vec2,
    pub size: f32,
}

/// `RenderLensFlare` geometry: `center_i = lerp(sun, mirror, Offset_i)`, `size_i = min(ftol(SizeX·0.3)·closeness·Scale_i/halfDiag, ftol(SizeX·0.4))`.
pub fn lens_flare(sun: Vec2, width: f32, height: f32, flares: &[Flare]) -> Vec<FlareQuad> {
    let hx = width / 2.0;
    let hy = height / 2.0;
    let half_diag = ftol((hx * hx + hy * hy).sqrt());
    let closeness = half_diag - ftol(((sun.y - hy).powi(2) + (sun.x - hx).powi(2)).sqrt());
    let max_size = ftol(width * FLARE_MAX_FRACTION);
    let base = ftol(width * FLARE_BASE_FRACTION);
    let mirror = Vec2::new(ftol(2.0 * hx - sun.x), ftol(2.0 * hy - sun.y));
    flares
        .iter()
        .map(|flare| {
            let center = Vec2::new(ftol(mirror.x * flare.offset + (1.0 - flare.offset) * sun.x), ftol(mirror.y * flare.offset + (1.0 - flare.offset) * sun.y));
            let size = ftol(base * closeness * flare.scale / half_diag).min(max_size);
            FlareQuad { center, size }
        })
        .collect()
}

/// Screen quad in pixels for the 2D flare pass, whose camera maps pixels straight to clip space.
pub fn flare_quad(quad: FlareQuad) -> [Vertex; 4] {
    let corner = |x: f32, y: f32, uv: [f32; 2]| Vertex { position: [x, y, 0.0], normal: [0.0, 0.0, 1.0], uv, mask_uv: uv, color: [1.0; 4] };
    let (c, h) = (quad.center, ftol(quad.size / 2.0));
    [corner(c.x - h, c.y - h, [0.0, 0.0]), corner(c.x + h, c.y - h, [1.0, 0.0]), corner(c.x + h, c.y + h, [1.0, 1.0]), corner(c.x - h, c.y + h, [0.0, 1.0])]
}

/// Pixel coordinates (x right, y down) to clip space at depth 0.5.
pub fn pixel_projection(width: f32, height: f32) -> Mat4 {
    Mat4::from_cols(
        Vec4::new(2.0 / width, 0.0, 0.0, 0.0),
        Vec4::new(0.0, -2.0 / height, 0.0, 0.0),
        Vec4::new(0.0, 0.0, 1.0, 0.0),
        Vec4::new(-1.0, 1.0, 0.5, 1.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Vec4, vec3};

    #[test]
    fn sky_camera_keeps_the_rotation_and_moves_to_the_sky_location() {
        let camera = Camera { eye: vec3(1000.0, -1000.0, 50.0), target: vec3(1000.0, 0.0, 50.0), fov_x_deg: 60.0, near: 1.0, far: 5000.0 };
        let sky = sky_camera(&camera, vec3(100_000.0, 0.0, 0.0));
        assert_eq!(sky.eye, vec3(100_000.0, 0.0, 0.0));
        assert_eq!(sky.target - sky.eye, camera.target - camera.eye);
        let ahead = sky.view_proj(1.0) * Vec4::new(100_000.0, 500.0, 0.0, 1.0);
        assert!((ahead.x / ahead.w).abs() < 1e-4 && (ahead.y / ahead.w).abs() < 1e-4);
    }

    fn giran_flares() -> Vec<Flare> {
        let textures = ["Flare02", "Flare02", "Flare02", "Flare02", "Flare01", "Flare03", "Flare04", "Flare06", "Flare06"];
        let offsets = [-0.12, -0.10, 0.10, 0.12, 0.17, 0.20, 0.40, 0.50, 0.60];
        let scales = [0.5, 0.38, 0.30, 0.40, 1.0, 1.0, 1.0, 2.0, 3.0];
        (0..9).map(|i| Flare { texture: textures[i].to_owned(), offset: offsets[i], scale: scales[i] }).collect()
    }

    #[test]
    fn the_lens_flare_sizes_and_centers_follow_render_lens_flare() {
        let quads = lens_flare(Vec2::new(512.0, 384.0), 1024.0, 768.0, &giran_flares());
        let sizes: Vec<f32> = quads.iter().map(|q| q.size).collect();
        assert_eq!(sizes, [153.0, 116.0, 92.0, 122.0, 307.0, 307.0, 307.0, 409.0, 409.0]);
        assert!(quads.iter().all(|q| q.center == Vec2::new(512.0, 384.0)));
        let corners = flare_quad(quads[0]);
        assert_eq!((corners[0].position[0], corners[2].position[0]), (512.0 - 76.0, 512.0 + 76.0));
        let half_way = lens_flare(Vec2::new(768.0, 576.0), 1024.0, 768.0, &giran_flares());
        assert_eq!(half_way[4].size, 153.0);
        assert_eq!(half_way[0].center, Vec2::new(829.0, 622.0));
        assert_eq!(half_way[8].center, Vec2::new(460.0, 345.0));
    }

    #[test]
    fn the_body_quad_faces_the_camera_and_projects_to_pixels() {
        let camera = Camera { eye: vec3(0.0, -300.0, 0.0), target: vec3(0.0, 0.0, 0.0), fov_x_deg: 90.0, near: 1.0, far: 5000.0 };
        let view_proj = camera.view_proj(1.0);
        let quad = body_quad(Vec3::ZERO, 10.0, view_proj, [1.0; 4]);
        let (right, down) = screen_axes(view_proj);
        assert!((down - -Vec3::Z).length() < 1e-3, "{down:?}");
        let clip_of = |p: [f32; 3]| view_proj * Vec3::from(p).extend(1.0);
        let (c0, c1, c3) = (clip_of(quad[0].position), clip_of(quad[1].position), clip_of(quad[3].position));
        assert!(c1.x / c1.w > c0.x / c0.w && (c1.y / c1.w - c0.y / c0.w).abs() < 1e-4, "{c0:?} {c1:?}");
        assert!(c3.y / c3.w > c0.y / c0.w, "{c0:?} {c3:?}");
        assert!((Vec3::from(quad[1].position) - Vec3::from(quad[0].position) - right * 20.0).length() < 1e-3);
        let px = project(camera.view_proj(1.0), Vec3::ZERO, 1024.0, 1024.0).unwrap();
        assert!((px - Vec2::new(512.0, 512.0)).length() < 1e-2, "{px:?}");
        assert!(project(camera.view_proj(1.0), vec3(0.0, -600.0, 0.0), 1024.0, 1024.0).is_none());
        let clip = pixel_projection(1024.0, 768.0) * Vec4::new(1024.0, 768.0, 0.0, 1.0);
        assert_eq!((clip.x, clip.y, clip.z), (1.0, -1.0, 0.5));
    }
}
