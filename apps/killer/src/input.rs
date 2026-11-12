use glam::{Vec3, Vec4};
use killer_render::camera::Camera;
use killer_world::{World,NetCommand};
use killer_world::anim::WaitType;

pub const CLICK_DRAG_THRESHOLD_PX: f32 = 4.0;

pub fn pixel_ray(camera: &Camera, size: (u32, u32), pixel: (f32, f32)) -> Option<(Vec3, Vec3)> {
    if size.0 == 0 || size.1 == 0 {
        return None;
    }
    let aspect = size.0 as f32 / size.1 as f32;
    let inverse = camera.view_proj(aspect).inverse();
    let x = pixel.0 / size.0 as f32 * 2.0 - 1.0;
    let y = 1.0 - pixel.1 / size.1 as f32 * 2.0;
    let unproject = |depth: f32| {
        let p = inverse * Vec4::new(x, y, depth, 1.0);
        p.truncate() / p.w
    };
    let near = unproject(0.0);
    let dir = (near - camera.eye).try_normalize()?;
    (dir.is_finite() && camera.eye.is_finite()).then_some((camera.eye, dir))
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct ClickTracker {
    pressed_at: Option<(f32, f32)>,
    travelled: f32,
}

impl ClickTracker {
    pub fn press(&mut self, at: Option<(f32, f32)>) {
        self.pressed_at = at;
        self.travelled = 0.0;
    }

    pub fn moved(&mut self, to: (f32, f32)) {
        if let Some(from) = self.pressed_at {
            let distance = ((to.0 - from.0).powi(2) + (to.1 - from.1).powi(2)).sqrt();
            self.travelled = self.travelled.max(distance);
        }
    }

    pub fn release(&mut self) -> Option<(f32, f32)> {
        let at = self.pressed_at.take();
        let click = self.travelled <= CLICK_DRAG_THRESHOLD_PX;
        self.travelled = 0.0;
        at.filter(|_| click)
    }

    pub fn cancel(&mut self) {
        self.pressed_at = None;
        self.travelled = 0.0;
    }
}

pub const PICK_FLAGS: u32 = 0x200bf;
pub const PICK_FLAGS_SHIFT: u32 = 0x300bf;
pub const PICK_SHIFT_START: f32 = 30.0;
pub const PICK_RANGE: f32 = 10000.0;

pub fn pick_query(origin: Vec3, dir: Vec3, shift: bool) -> killer_assets::pickup::TraceQuery {
    let (start, flags) = if shift { (origin + dir * PICK_SHIFT_START, PICK_FLAGS_SHIFT) } else { (origin, PICK_FLAGS) };
    killer_assets::pickup::TraceQuery { start, end: origin + dir * PICK_RANGE, extent: Vec3::splat(0.1), flags }
}

pub fn target_action(world: &mut World, object_id: i32, ctrl: bool, shift: bool) -> Option<NetCommand> {
    let me = world.me.as_ref()?;
    if me.dead {
        return None;
    }
    if world.camera_fixed {
        world.pending_click_target = world.entity(object_id).map(|e| e.object_id).or_else(|| world.static_target(object_id).map(|t| t.object_id));
        return None;
    }
    if ctrl && me.wait_type() != WaitType::Stand {
        return None;
    }
    if !world.select(object_id) {
        return None;
    }
    let me = world.me.as_ref()?;
    let origin = me.position.to_array().map(|v|v as i32);
    Some(if ctrl { NetCommand::AttackRequest { object_id, origin, shift } } else { NetCommand::Action { object_id, origin, shift } })
}

pub fn ground_click_allowed(world: &World, keyboard_moving: bool, shift: bool) -> bool {
    let Some(me) = world.me.as_ref() else { return false };
    !shift && !me.dead && !world.camera_fixed && !keyboard_moving && me.wait_type() == WaitType::Stand
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::{CameraSettings, OrbitCamera};
    use crate::characters::{demo_world, project};
    use killer_world::EntityKind;

    fn camera() -> Camera {
        Camera { eye: Vec3::new(100.0, -900.0, 300.0), target: Vec3::new(100.0, 0.0, 0.0), fov_x_deg: 60.0, near: 10.0, far: 60000.0 }
    }

    #[test]
    fn shift_moves_the_pick_start_30_units_along_the_ray_and_selects_the_alternative_flags() {
        let (origin, dir) = (Vec3::new(1.0, 2.0, 3.0), Vec3::X);
        let plain = pick_query(origin, dir, false);
        assert_eq!((plain.start, plain.flags), (origin, 0x200bf));
        let shifted = pick_query(origin, dir, true);
        assert_eq!((shifted.start, shifted.flags), (origin + Vec3::X * 30.0, 0x300bf));
        assert_eq!(shifted.end, plain.end);
        assert_eq!(plain.end, origin + Vec3::X * 10000.0);
    }

    #[test]
    fn the_central_pixel_ray_follows_the_camera_direction() {
        let camera = camera();
        let (origin, dir) = pixel_ray(&camera, (1280, 720), (640.0, 360.0)).unwrap();
        let forward = (camera.target - camera.eye).normalize();
        assert!((dir - forward).length() < 1e-4, "{dir:?}");
        let along = (origin - camera.eye).dot(forward);
        assert!((origin - camera.eye).length() < camera.near * 1.01 + 1e-2, "{origin:?}");
        assert!(along >= 0.0);
    }

    #[test]
    fn pixels_to_the_right_and_top_tilt_the_ray_accordingly() {
        let camera = camera();
        let forward = (camera.target - camera.eye).normalize();
        let right = Vec3::Z.cross(forward).normalize();
        let (_, dir) = pixel_ray(&camera, (1280, 720), (1280.0, 360.0)).unwrap();
        assert!(dir.dot(right) > 0.0);
        assert!((dir.dot(right) - 30f32.to_radians().sin()).abs() < 1e-3, "{}", dir.dot(right));
        let (_, up) = pixel_ray(&camera, (1280, 720), (640.0, 0.0)).unwrap();
        assert!(up.z > forward.z);
        assert_eq!(pixel_ray(&camera, (0, 720), (0.0, 0.0)), None);
    }

    #[test]
    fn the_demo_lector_projects_and_unprojects_onto_the_same_ray() {
        let anchor = Vec3::new(83400.0, 148000.0, -3400.0);
        let orbit = OrbitCamera::behind(anchor, 16384, CameraSettings::default());
        let world = demo_world(orbit.target, orbit.facing(), 0);
        let lector = world.entities.values().find(|e| matches!(e.kind, EntityKind::Npc { npc_id: 7001, .. })).unwrap();
        let camera = orbit.camera();
        for size in [(1280u32, 720u32), (2560, 1440), (1000, 1000)] {
            let pixel = project(&camera, (size.0 as f32, size.1 as f32), lector.position).unwrap();
            let (origin, dir) = pixel_ray(&camera, size, (pixel[0], pixel[1])).unwrap();
            let to = lector.position - origin;
            let off_ray = (to - dir * to.dot(dir)).length();
            assert!(off_ray < 1.0, "{size:?}: {off_ray}");

        }
    }

    #[test]
    fn clicking_the_own_pawn_seen_from_the_orbit_camera_picks_it() {
        let anchor = Vec3::new(83400.0, 148000.0, -3400.0);
        let orbit = OrbitCamera::behind(anchor, 16384, CameraSettings::default());
        let world = demo_world(orbit.target, orbit.facing(), 0);
        let me = world.me.as_ref().unwrap();
        let camera = orbit.camera();
        let size = (1280u32, 720u32);
        for point in [me.position, me.position + Vec3::Z * me.collision_height * 0.9, me.position - Vec3::Z * me.collision_height * 0.9] {
            let pixel = project(&camera, (size.0 as f32, size.1 as f32), point).unwrap();
            let (origin, dir) = pixel_ray(&camera, size, (pixel[0], pixel[1])).unwrap();
            assert!(killer_assets::collision::cylinder::trace(me.position.to_array(),me.collision_radius,me.collision_height,origin.to_array(),(origin+dir*10000.0).to_array(),[0.1;3]).is_some(), "{point:?} -> {pixel:?}");
        }
    }

    #[test]
    fn a_press_and_release_without_travel_is_a_click() {
        let mut click = ClickTracker::default();
        click.press(Some((10.0, 20.0)));
        click.moved((12.0, 22.0));
        click.moved((10.0, 20.0));
        assert_eq!(click.release(), Some((10.0, 20.0)));
        assert_eq!(click.release(), None);
    }

    #[test]
    fn travelling_past_the_threshold_turns_the_press_into_a_drag() {
        let mut click = ClickTracker::default();
        click.press(Some((10.0, 20.0)));
        click.moved((10.0, 20.0 + CLICK_DRAG_THRESHOLD_PX + 1.0));
        click.moved((10.0, 20.0));
        assert_eq!(click.release(), None);
        click.press(None);
        assert_eq!(click.release(), None);
        click.press(Some((0.0, 0.0)));
        click.cancel();
        assert_eq!(click.release(), None);
    }
}
