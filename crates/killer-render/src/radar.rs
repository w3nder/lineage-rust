use glam::{Mat4, Vec3, Vec4};
use killer_assets::characters::{CharacterMesh, Part, load_part};
use killer_assets::client::Client;
use killer_assets::engine_math::{cos_tab, rotator_forward, sin_tab};
use killer_assets::scene::transform::vector_rotation;
use killer_ui::radar::RadarView;

pub const VIEWPORT: (u32, u32) = (0x80, 0x60);
pub const FOV_DEG: f32 = 27.0;
pub const CAMERA_DISTANCE: f32 = 256.0;
pub const UNITS_PER_WORLD_UNIT: f32 = 0.023_333_333;
pub const RANGE: f32 = 35.0;
pub const PITCH_OFFSET: i32 = 0x800;
pub const PITCH_MIN: i32 = -0x2000;
pub const PITCH_MAX: i32 = -0x1200;
pub const ARROW_YAW_OFFSET: i32 = 0x4000;
pub const CENTER_CAMERA: [i32; 3] = [0x4000, 0, 0];
pub const CENTER_ROTATION: [i32; 3] = [0, -0x4000, 0];
pub const LIFT: f32 = 6.0;
pub const NORTH: [i32; 3] = [0, -0x4000, 0];
pub const NORTH_DISTANCE: f32 = 50.0;
pub const NORTH_PITCH: f32 = 16384.0;
pub const TIME_ROTATION: [i32; 3] = [0, 0x4000, 0];
pub const RENDER_MAP: i32 = 6;
pub const NEAR_CLIPPING_PLANE: f32 = 10.0;
pub const FAR_CLIPPING_PLANE: f32 = 65536.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RadarMesh {
    Back,
    Arrow,
    Party,
    Target,
    Tutorial1,
    Tutorial2,
    Center,
    North,
    Time,
}

impl RadarMesh {
    pub const ALL: [RadarMesh; 9] = [
        RadarMesh::Back,
        RadarMesh::Arrow,
        RadarMesh::Party,
        RadarMesh::Target,
        RadarMesh::Tutorial1,
        RadarMesh::Tutorial2,
        RadarMesh::Center,
        RadarMesh::North,
        RadarMesh::Time,
    ];

    pub fn name(self) -> &'static str {
        match self {
            RadarMesh::Back => "LineageDecos.radar_back",
            RadarMesh::Arrow => "LineageDecos.radar_arrow",
            RadarMesh::Party => "LineageDecos.radar_dot_party",
            RadarMesh::Target => "LineageDecos.radar_dot_target",
            RadarMesh::Tutorial1 => "LineageDecos.radar_dot_tutorial1",
            RadarMesh::Tutorial2 => "LineageDecos.radar_dot_tutorial2",
            RadarMesh::Center => "LineageDecos.radar_center",
            RadarMesh::North => "LineageDecos.radar_n",
            RadarMesh::Time => "LineageDecos.radar_time",
        }
    }

    pub fn index(self) -> usize {
        Self::ALL.iter().position(|mesh| *mesh == self).unwrap_or(0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RadarClear {
    Depth,
    All,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadarCamera {
    pub location: Vec3,
    pub rotation: [i32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RadarStep {
    Clear(RadarClear),
    Draw { mesh: RadarMesh, location: Vec3, rotation: [i32; 3], camera: RadarCamera },
}

pub fn radar_pitch(camera_pitch: i32) -> i32 {
    camera_pitch.wrapping_sub(PITCH_OFFSET).clamp(PITCH_MIN, PITCH_MAX)
}

pub fn rotation_rows(rotation: [i32; 3]) -> [Vec3; 3] {
    let (sp, cp) = (sin_tab(rotation[0]), cos_tab(rotation[0]));
    let (sy, cy) = (sin_tab(rotation[1]), cos_tab(rotation[1]));
    let (sr, cr) = (sin_tab(rotation[2]), cos_tab(rotation[2]));
    [
        Vec3::new(cp * cy, cp * sy, sp),
        Vec3::new(sr * sp * cy - cr * sy, sr * sp * sy + cr * cy, -sr * cp),
        Vec3::new(-(cr * sp * cy + sr * sy), cy * sr - cr * sp * sy, cr * cp),
    ]
}

pub fn orbit_camera(rotation: [i32; 3]) -> RadarCamera {
    RadarCamera { location: -(rotator_forward(rotation) * CAMERA_DISTANCE), rotation }
}

pub fn radar_location(delta: [f32; 2]) -> Vec3 {
    let d = Vec3::new(delta[0], delta[1], 0.0);
    let size = d.length();
    let scaled = (size * UNITS_PER_WORLD_UNIT).min(RANGE);
    let direction = if size * size > 1e-8 { d / size } else { d };
    direction * scaled
}

pub fn arrow_yaw(delta: [f32; 3]) -> i32 {
    vector_rotation(Vec3::from(delta))[1] + ARROW_YAW_OFFSET
}

pub fn radar_steps(view: &RadarView, loaded: impl Fn(RadarMesh) -> bool, toggle: &mut bool) -> Vec<RadarStep> {
    let camera = view.camera();
    let pitch = radar_pitch(camera[0]);
    let standard = orbit_camera([pitch, camera[1], camera[2]]);
    let mut steps = Vec::new();
    let draw = |mesh, location, rotation, camera| RadarStep::Draw { mesh, location, rotation, camera };
    if [RadarMesh::Back, RadarMesh::Arrow, RadarMesh::Party, RadarMesh::Target].into_iter().all(&loaded) {
        steps.push(RadarStep::Clear(RadarClear::Depth));
        steps.push(draw(RadarMesh::Back, Vec3::ZERO, [0; 3], standard));
        steps.push(RadarStep::Clear(RadarClear::Depth));
        if let Some(to) = view.arrow_to {
            steps.push(draw(RadarMesh::Arrow, Vec3::ZERO, [0, arrow_yaw(to), 0], standard));
        }
        for member in &view.party {
            steps.push(draw(RadarMesh::Party, radar_location(*member), [0; 3], standard));
        }
        if let Some(target) = view.target {
            steps.push(draw(RadarMesh::Target, radar_location(target), [0; 3], standard));
        }
    }
    if loaded(RadarMesh::Tutorial1) && loaded(RadarMesh::Tutorial2) {
        for point in &view.points {
            steps.push(RadarStep::Clear(RadarClear::Depth));
            *toggle = !*toggle;
            let mesh = if *toggle { RadarMesh::Tutorial1 } else { RadarMesh::Tutorial2 };
            steps.push(draw(mesh, radar_location(*point), [0; 3], standard));
        }
    }
    if [RadarMesh::Center, RadarMesh::North, RadarMesh::Time].into_iter().all(&loaded) {
        steps.push(RadarStep::Clear(RadarClear::Depth));
        let center = orbit_camera(CENTER_CAMERA);
        steps.push(draw(RadarMesh::Center, rotation_rows(CENTER_CAMERA)[2] * LIFT, CENTER_ROTATION, center));
        if view.show_time {
            steps.push(RadarStep::Clear(RadarClear::All));
            steps.push(draw(RadarMesh::Time, Vec3::ZERO, TIME_ROTATION, orbit_camera([pitch, 0, 0])));
        }
        let north = rotation_rows(standard.rotation)[2] * LIFT + rotator_forward(NORTH) * NORTH_DISTANCE;
        let facing = (pitch as f32 + NORTH_PITCH) as i32;
        steps.push(draw(RadarMesh::North, north, [facing, camera[1], 0], standard));
    }
    steps
}

pub fn view_matrix(camera: &RadarCamera) -> Mat4 {
    let [forward, right, up] = rotation_rows(camera.rotation);
    let eye = camera.location;
    Mat4::from_cols(
        Vec4::new(right.x, up.x, forward.x, 0.0),
        Vec4::new(right.y, up.y, forward.y, 0.0),
        Vec4::new(right.z, up.z, forward.z, 0.0),
        Vec4::new(-right.dot(eye), -up.dot(eye), -forward.dot(eye), 1.0),
    )
}

pub fn projection(aspect: f32) -> Mat4 {
    let half = (FOV_DEG * 0.5).to_radians().tan();
    let fov_y = 2.0 * (half / aspect).atan();
    glam::camera::lh::proj::directx::perspective(fov_y, aspect, NEAR_CLIPPING_PLANE, FAR_CLIPPING_PLANE)
}

pub fn actor_matrix(location: Vec3, rotation: [i32; 3]) -> Mat4 {
    let [x, y, z] = rotation_rows(rotation);
    Mat4::from_cols(x.extend(0.0), y.extend(0.0), z.extend(0.0), location.extend(1.0))
}

pub fn viewport(screen: (u32, u32)) -> [u32; 4] {
    [screen.0.saturating_sub(VIEWPORT.0), 0, VIEWPORT.0, VIEWPORT.1]
}

pub fn load_meshes(client: &mut Client) -> Vec<(RadarMesh, CharacterMesh)> {
    RadarMesh::ALL.into_iter().filter_map(|mesh| load_part(client, &Part::new(mesh.name(), Vec::new())).map(|loaded| (mesh, loaded))).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draws(steps: &[RadarStep]) -> Vec<(RadarMesh, Vec3, [i32; 3], RadarCamera)> {
        steps.iter().filter_map(|step| match *step {
            RadarStep::Draw { mesh, location, rotation, camera } => Some((mesh, location, rotation, camera)),
            RadarStep::Clear(_) => None,
        }).collect()
    }

    fn close(a: Vec3, b: Vec3) -> bool {
        (a - b).length() < 1e-3
    }

    #[test]
    fn the_pitch_is_clamped_with_a_signed_compare_after_the_offset() {
        assert_eq!(radar_pitch(0), PITCH_MAX);
        assert_eq!(radar_pitch(-0x3000), PITCH_MIN);
        assert_eq!(radar_pitch(-0x1000), -0x1800);
        assert_eq!(radar_pitch(-0xa00), -0x1200);
        assert_eq!(radar_pitch(-0xa01), -0x1201);
        assert_eq!(radar_pitch(-0x1800), PITCH_MIN);
        assert_eq!(radar_pitch(0xf000), PITCH_MAX);
    }

    #[test]
    fn world_offsets_scale_by_the_engine_factor_and_stop_at_35_units() {
        assert_eq!(radar_location([0.0, 0.0]), Vec3::ZERO);
        assert!(close(radar_location([300.0, 0.0]), Vec3::new(7.0, 0.0, 0.0)));
        assert!(close(radar_location([0.0, -9000.0]), Vec3::new(0.0, -35.0, 0.0)));
        assert!(close(radar_location([1500.0, 1500.0]), Vec3::new(1.0, 1.0, 0.0).normalize() * 35.0));
        assert!(close(radar_location([1500.0 / 2.0f32.sqrt(), 1500.0 / 2.0f32.sqrt()]), Vec3::new(24.7487, 24.7487, 0.0)));
    }

    #[test]
    fn the_camera_orbits_the_origin_at_256_units_and_looks_at_it() {
        let camera = orbit_camera([-0x2000, 0x4000, 0]);
        assert!((camera.location.length() - 256.0).abs() < 1e-2);
        assert!(camera.location.z > 0.0 && camera.location.y < 0.0);
        let view = view_matrix(&camera);
        let origin = view.transform_point3(Vec3::ZERO);
        assert!(origin.x.abs() < 1e-3 && origin.y.abs() < 1e-3 && (origin.z - 256.0).abs() < 1e-2);
        let center = orbit_camera(CENTER_CAMERA);
        assert!(close(center.location, Vec3::new(0.0, 0.0, -256.0)));
        let ahead = view_matrix(&center).transform_point3(Vec3::new(0.0, 0.0, 10.0));
        assert!(ahead.z > 256.0);
    }

    #[test]
    fn the_projection_keeps_27_degrees_across_and_takes_the_vertical_from_the_screen_aspect() {
        let p = projection(1024.0 / 768.0);
        let half = 13.5f32.to_radians().tan();
        assert!((p.x_axis.x - 1.0 / half).abs() < 1e-4);
        assert!((p.y_axis.y - (1024.0 / 768.0) / half).abs() < 1e-3);
        let wide = projection(16.0 / 9.0);
        assert!((wide.x_axis.x - 1.0 / half).abs() < 1e-4);
        assert!((wide.y_axis.y - (16.0 / 9.0) / half).abs() < 1e-3);
        let clip = p * Vec4::new(0.0, 0.0, NEAR_CLIPPING_PLANE, 1.0);
        assert!((clip.z / clip.w).abs() < 1e-5);
        let far = p * Vec4::new(0.0, 0.0, FAR_CLIPPING_PLANE, 1.0);
        assert!((far.z / far.w - 1.0).abs() < 1e-5);
        assert_eq!(viewport((1024, 768)), [896, 0, 128, 96]);
    }

    #[test]
    fn the_back_dots_and_arrow_follow_the_c4_order_and_clears() {
        let view = RadarView {
            camera_pitch: -0x1000,
            camera_yaw: 0x2000,
            camera_roll: 0,
            party: vec![[300.0, 0.0]],
            target: Some([0.0, 300.0]),
            arrow_to: Some([0.0, 300.0, 50.0]),
            points: Vec::new(),
            show_time: false,
        };
        let mut toggle = false;
        let steps = radar_steps(&view, |_| true, &mut toggle);
        let order: Vec<_> = steps.iter().map(|step| match step {
            RadarStep::Clear(clear) => format!("{clear:?}"),
            RadarStep::Draw { mesh, .. } => format!("{mesh:?}"),
        }).collect();
        assert_eq!(order, ["Depth", "Back", "Depth", "Arrow", "Party", "Target", "Depth", "Center", "North"]);
        let draws = draws(&steps);
        let standard = orbit_camera([-0x1800, 0x2000, 0]);
        assert_eq!(draws[0], (RadarMesh::Back, Vec3::ZERO, [0; 3], standard));
        assert_eq!(draws[1].2, [0, 0x4000 + 16383, 0]);
        assert!(close(draws[2].1, Vec3::new(7.0, 0.0, 0.0)));
        assert!(close(draws[3].1, Vec3::new(0.0, 7.0, 0.0)));
        assert_eq!(draws[4].0, RadarMesh::Center);
        assert!(close(draws[4].1, Vec3::new(-6.0, 0.0, 0.0)));
        assert_eq!((draws[4].2, draws[4].3), (CENTER_ROTATION, orbit_camera(CENTER_CAMERA)));
        let up = rotation_rows(standard.rotation)[2];
        assert!(close(draws[5].1, up * 6.0 + Vec3::new(0.0, -50.0, 0.0)));
        assert_eq!(draws[5].2, [-0x1800 + 0x4000, 0x2000, 0]);
        assert_eq!(draws[5].3, standard);
        let none = radar_steps(&RadarView { arrow_to: None, ..view.clone() }, |_| true, &mut toggle);
        assert!(draws_of(&none, RadarMesh::Arrow).is_empty());
    }

    fn draws_of(steps: &[RadarStep], wanted: RadarMesh) -> Vec<Vec3> {
        draws(steps).into_iter().filter(|draw| draw.0 == wanted).map(|draw| draw.1).collect()
    }

    #[test]
    fn the_arrow_turns_to_the_target_with_a_quarter_turn_added() {
        assert_eq!(arrow_yaw([1.0, 0.0, 0.0]), 0x4000);
        assert_eq!(arrow_yaw([0.0, 1.0, 0.0]), 0x4000 + 16383);
        assert_eq!(arrow_yaw([-1.0, 0.0, 99.0]), 0x4000 + 32767);
        assert_eq!(arrow_yaw([0.0, -1.0, 0.0]), 0x4000 - 16383);
    }

    #[test]
    fn tutorial_dots_toggle_on_every_call_without_a_clock() {
        let view = RadarView { points: vec![[100.0, 0.0]], ..Default::default() };
        let mut toggle = false;
        let first = radar_steps(&view, |_| true, &mut toggle);
        assert_eq!(draws_of(&first, RadarMesh::Tutorial1).len(), 1);
        let second = radar_steps(&view, |_| true, &mut toggle);
        assert_eq!(draws_of(&second, RadarMesh::Tutorial2).len(), 1);
        let pair = RadarView { points: vec![[100.0, 0.0], [0.0, 100.0]], ..Default::default() };
        let mut toggle = false;
        for _ in 0..3 {
            let steps = radar_steps(&pair, |_| true, &mut toggle);
            assert_eq!(draws_of(&steps, RadarMesh::Tutorial1).len(), 1);
            assert_eq!(draws_of(&steps, RadarMesh::Tutorial2).len(), 1);
            assert!(close(draws_of(&steps, RadarMesh::Tutorial1)[0], Vec3::new(2.3333333, 0.0, 0.0)));
        }
        let clears = radar_steps(&pair, |_| true, &mut toggle).iter().filter(|step| matches!(step, RadarStep::Clear(RadarClear::Depth))).count();
        assert_eq!(clears, 2 + 2 + 1);
    }

    #[test]
    fn the_time_mode_clears_the_viewport_and_missing_meshes_drop_their_group() {
        let view = RadarView { show_time: true, camera_pitch: -0x1000, ..Default::default() };
        let mut toggle = false;
        let steps = radar_steps(&view, |_| true, &mut toggle);
        let at = steps.iter().position(|step| *step == RadarStep::Clear(RadarClear::All)).unwrap();
        assert!(matches!(steps[at - 1], RadarStep::Draw { mesh: RadarMesh::Center, .. }));
        assert_eq!(steps[at + 1], RadarStep::Draw { mesh: RadarMesh::Time, location: Vec3::ZERO, rotation: TIME_ROTATION, camera: orbit_camera([-0x1800, 0, 0]) });
        assert!(matches!(steps[at + 2], RadarStep::Draw { mesh: RadarMesh::North, .. }));
        let partial = radar_steps(&RadarView { points: vec![[1.0, 1.0]], ..view.clone() }, |mesh| mesh != RadarMesh::Party && mesh != RadarMesh::Time, &mut toggle);
        let meshes: Vec<_> = draws(&partial).into_iter().map(|draw| draw.0).collect();
        assert_eq!(meshes, [RadarMesh::Tutorial1]);
        assert_eq!(RadarMesh::ALL.map(RadarMesh::index), [0, 1, 2, 3, 4, 5, 6, 7, 8]);
    }

    #[test]
    fn the_actor_matrix_places_and_turns_the_mesh() {
        let m = actor_matrix(Vec3::new(1.0, 2.0, 3.0), [0, 0x4000, 0]);
        assert!(close(m.transform_point3(Vec3::X), Vec3::new(1.0, 3.0, 3.0)));
        assert!(close(m.transform_point3(Vec3::Y), Vec3::new(0.0, 2.0, 3.0)));
    }
}
