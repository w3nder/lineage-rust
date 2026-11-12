use crate::trace::Hit;
use glam::Vec3;
use killer_assets::engine_math::{Coords, coords_mul_rotator, rotator_forward, transform_vector_by};
use killer_render::camera::Camera;
use killer_ui::ini::IniFile;
use killer_world::engine_input::{InputAction, parse_command, parse_float};
use killer_world::special_camera::{Request, Rest, SpecialCameraState};
use killer_world::{CameraCommand, EntityKind, World};
use std::f32::consts::{PI, TAU};

pub const ENGINE_BASE_DISTANCE: f32 = 250.0;
pub const ENGINE_DEFAULT_ZOOM: f32 = -20.0;
pub const ENGINE_CLASS_MIN_ZOOM: f32 = -200.0;
pub const ENGINE_CLASS_MAX_ZOOM: f32 = 1000.0;
pub const ENGINE_WHEEL_DEFAULT_SPEED: f32 = 1.0;
pub const ENGINE_WHEEL_FINE_FACTOR: f32 = 0.125;
pub const ENGINE_ZOOMED_IN_LIFT_FACTOR: f32 = 0.8;
pub const ENGINE_DEFAULT_PITCH_UNITS: f32 = 2700.0;
pub const UE_UNITS_PER_TURN: f32 = 65536.0;
pub const ENGINE_CAMERA_HIT_MARGIN: f32 = 30.0;
pub const ENGINE_TRACE_END_DROP: f32 = 10.0;
const ENGINE_MIN_PITCH_UNITS: f32 = -15000.0;
const ENGINE_MAX_PITCH_UNITS: f32 = 16384.0;
const DRAG_SENSITIVITY: f32 = 0.005;
pub const ENGINE_HOLD_MIN_ZOOM: f32 = -200.0;
pub const ENGINE_HOLD_MAX_ZOOM: f32 = 1000.0;
const ENGINE_HOLD_DEFAULT_SPEED: f32 = 1.0;
const ENGINE_DEFAULT_DIST_NEAR: f32 = 6.0;
const ENGINE_DEFAULT_DIST_SPEED: f32 = 50.0;
const ENGINE_VOLUME_RADIUS_MARGIN: i32 = 10;
const ENGINE_VOLUME_RADIUS_FLOOR: i32 = 10;
const ENGINE_VOLUME_HALF: f64 = 0.5;
const ENGINE_VOLUME_TRIES: usize = 2;

pub const ENGINE_DEFAULT_FOV_X_DEG: f32 = 60.0;
const NEAR: f32 = 10.0;
pub const ENGINE_FAR_CLIPPING_PLANE: f32 = 65536.0;
const FAR: f32 = ENGINE_FAR_CLIPPING_PLANE;
const FREE_SPEED: f32 = 800.0;
const FREE_FAST_SPEED: f32 = 4000.0;
const CONTROLLER_SECTION: &str = "Engine.LineagePlayerController";
const INPUT_SECTION: &str = "Engine.Input";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraSettings {
    pub mouse_speed: [f32; 2],
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub view_height_adjust: f32,
    pub hit_check: bool,
    pub hit_check_min_dist: i32,
    pub volume_camera: bool,
    pub volume_radius: i32,
    pub fixed_default_dist: f32,
    pub fixed_default_pitch: i32,
    pub wheel_down: f32,
    pub wheel_up: f32,
    pub fixed_views: [FixedView; FIXED_VIEW_SLOTS],
    pub fixed_view_count: usize,
    pub fixed_on_right_release_ms: Option<f32>,
    pub turn_back_on_middle: bool,
    pub auto_tracking: bool,
    pub auto_tracking_speed: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FixedView {
    pub yaw: i32,
    pub pitch: i32,
    pub dist: f32,
    pub view_height: f32,
    pub hide_player: bool,
    pub disable_zoom: bool,
}

pub const FIXED_VIEW_SLOTS: usize = 10;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PawnCameraExec {
    Ignored,
    FixedView,
    TurnBack,
}

pub const TURN_BACK_FRONT: std::ops::RangeInclusive<i32> = 0x7f39..=0x80c7;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DefaultMove {
    pub moving: bool,
    pub use_yaw: bool,
    pub use_pitch: bool,
    pub use_dist: bool,
    pub yaw: i32,
    pub pitch: i32,
    pub dist: f32,
    pub saved_target_yaw: i32,
    pub saved_target_pitch: i32,
    pub fixed_index: usize,
    pub disable_zoom: bool,
    pub hide_player: bool,
    pub fixed_mode: bool,
}

impl Default for CameraSettings {
    fn default() -> Self {
        Self {
            mouse_speed: [0.0; 2],
            min_zoom: ENGINE_CLASS_MIN_ZOOM,
            max_zoom: ENGINE_CLASS_MAX_ZOOM,
            view_height_adjust: 0.0,
            hit_check: false,
            hit_check_min_dist: 0,
            volume_camera: false,
            volume_radius: 0,
            fixed_default_dist: 0.0,
            fixed_default_pitch: 0,
            wheel_down: 0.0,
            wheel_up: 0.0,
            fixed_views: [FixedView::default(); FIXED_VIEW_SLOTS],
            fixed_view_count: 0,
            fixed_on_right_release_ms: None,
            turn_back_on_middle: false,
            auto_tracking: false,
            auto_tracking_speed: 0.0,
        }
    }
}

impl CameraSettings {
    pub fn drag_radians(&self, dx: f32, dy: f32) -> (f32, f32) {
        let yaw = (dx * self.mouse_speed[0] * 0.01).trunc();
        let pitch = (dy * self.mouse_speed[1] * 0.01).trunc().clamp(-15000.0, 15000.0);
        (ue_angle(yaw), ue_angle(pitch))
    }

    pub fn from_ini(ini: &IniFile) -> Self {
        let defaults = Self::default();
        let float = |key: &str, fallback: f32| ini.get(CONTROLLER_SECTION, key).and_then(|v| v.parse().ok()).unwrap_or(fallback);
        Self {
            mouse_speed: [axis_bind(ini.get(INPUT_SECTION, "MouseX"), "CameraYaw"), axis_bind(ini.get(INPUT_SECTION, "MouseY"), "CameraPitch")],
            min_zoom: float("MinZoomingDist", defaults.min_zoom),
            max_zoom: float("MaxZoomingDist", defaults.max_zoom),
            view_height_adjust: float("CameraViewHeightAdjust", defaults.view_height_adjust),
            hit_check: ini.bool(CONTROLLER_SECTION, "bUseHitCheckCamera").unwrap_or(defaults.hit_check),
            hit_check_min_dist: ini.int(CONTROLLER_SECTION, "HitCheckCameraMinDist").unwrap_or(defaults.hit_check_min_dist),
            volume_camera: ini.bool(CONTROLLER_SECTION, "bUseVolumeCamera").unwrap_or(defaults.volume_camera),
            volume_radius: ini.int(CONTROLLER_SECTION, "VolumeCameraRadius").unwrap_or(defaults.volume_radius),
            fixed_default_dist: float("FixedDefaultCameraDist[0]", defaults.fixed_default_dist),
            fixed_default_pitch: float("FixedDefaultCameraPitch[0]", defaults.fixed_default_pitch as f32) as i32,
            wheel_down: zoom_bind(ini.get(INPUT_SECTION, "MouseWheelDown")),
            wheel_up: zoom_bind(ini.get(INPUT_SECTION, "MouseWheelUp")),
            fixed_view_count: ini.int(CONTROLLER_SECTION, "FixedDefaultViewNum").unwrap_or(0).clamp(0, FIXED_VIEW_SLOTS as i32) as usize,
            fixed_views: {
                std::array::from_fn(|i| FixedView {
                    yaw: float(&format!("FixedDefaultCameraYaw[{i}]"), 0.0) as i32,
                    pitch: float(&format!("FixedDefaultCameraPitch[{i}]"), 0.0) as i32,
                    dist: float(&format!("FixedDefaultCameraDist[{i}]"), 0.0),
                    view_height: float(&format!("FixedDefaultCameraViewHeight[{i}]"), 0.0),
                    hide_player: ini.int(CONTROLLER_SECTION, &format!("FixedDefaultCameraHidePlayer[{i}]")).unwrap_or(0) != 0,
                    disable_zoom: ini.int(CONTROLLER_SECTION, &format!("FixedDefaultCameraDisableZoom[{i}]")).unwrap_or(0) != 0,
                })
            },
            fixed_on_right_release_ms: release_bind(ini.get(INPUT_SECTION, "RightMouse"), "fixeddefaultcamera"),
            turn_back_on_middle: has_command(ini.get(INPUT_SECTION, "MiddleMouse"), "turnback"),
            auto_tracking: defaults.auto_tracking,
            auto_tracking_speed: float("AutoTrackingPawnSpeed", defaults.auto_tracking_speed),
        }
    }
}

fn axis_bind(value: Option<&str>, name: &str) -> f32 {
    let Some(command) = commands(value).find(|command| command.split_whitespace().next().is_some_and(|word| word.eq_ignore_ascii_case(name))) else { return 0.0; };
    command.split_whitespace().find_map(|word| {
        let (key, value) = word.split_once('=')?;
        key.eq_ignore_ascii_case("Speed").then(|| value.parse::<f32>().ok()).flatten()
    }).filter(|value| value.is_finite()).unwrap_or(1.0)
}

fn commands(value: Option<&str>) -> impl Iterator<Item = &str> {
    value.into_iter().flat_map(|bind| bind.split('|')).map(str::trim)
}

fn has_command(value: Option<&str>, name: &str) -> bool {
    commands(value).any(|command| command.split_whitespace().next().is_some_and(|word| word.eq_ignore_ascii_case(name)))
}

fn release_bind(value: Option<&str>, name: &str) -> Option<f32> {
    let command = commands(value).find(|command| command.split_whitespace().next().is_some_and(|word| word.eq_ignore_ascii_case(name)))?;
    let words: Vec<&str> = command.split_whitespace().collect();
    if !words.iter().any(|word| word.eq_ignore_ascii_case("onrelease")) {
        return Some(f32::INFINITY);
    }
    Some(words.iter().filter_map(|word| word.split_once('=')).find(|(key, _)| key.eq_ignore_ascii_case("maxpressedtime")).and_then(|(_, v)| v.parse().ok()).unwrap_or(f32::INFINITY))
}

fn wrap_units(mut value: i32) -> i32 {
    while value > 32768 { value -= 65536; }
    while value < -32767 { value += 65536; }
    value
}

fn approach_units(delta_time: f32, temp: i32) -> i32 {
    let mut step = if (-600..0).contains(&temp) {
        (-5000.0 * delta_time) as i32
    } else if temp > 0 && temp <= 600 {
        (5000.0 * delta_time) as i32
    } else {
        (temp as f32 / 6.0 * 50.0 * delta_time) as i32
    };
    if temp > 0 && step > temp { step = temp; }
    if temp < 0 && step < temp { step = temp; }
    step
}

fn approach_distance(delta_time: f32, temp: f32) -> f32 {
    let mut step = if (-ENGINE_DEFAULT_DIST_NEAR..0.0).contains(&temp) {
        -ENGINE_DEFAULT_DIST_SPEED * delta_time
    } else if temp > 0.0 && temp <= ENGINE_DEFAULT_DIST_NEAR {
        ENGINE_DEFAULT_DIST_SPEED * delta_time
    } else {
        temp / ENGINE_DEFAULT_DIST_NEAR * ENGINE_DEFAULT_DIST_SPEED * delta_time
    };
    if temp > 0.0 && step > temp { step = temp; }
    if temp < 0.0 && step < temp { step = temp; }
    step
}

fn rotation_to_location_with_dist(rotation: [i32; 3], location: Vec3, dist: f32) -> Vec3 {
    location - rotator_forward(rotation) * dist
}

fn view_axis(rotation: [i32; 3]) -> Vec3 {
    Vec3::from(transform_vector_by([1.0, 0.0, 0.0], &coords_mul_rotator(&Coords::UNIT, rotation)))
}

fn hit_location(from: Vec3, to: Vec3, hit: Option<Hit>) -> Vec3 {
    match hit {
        Some(hit) => from + (to - from).normalize_or_zero() * hit.distance,
        None => Vec3::ZERO,
    }
}

fn is_block_rotation(rotation: [i32; 3], location: Vec3, dist: f32, radius: i32, trace: &impl Fn(Vec3, Vec3) -> Option<Hit>) -> bool {
    let [pitch, yaw, roll] = rotation;
    [rotation, [pitch, yaw.wrapping_add(radius), roll], [pitch, yaw.wrapping_sub(radius), roll], [pitch.wrapping_add(radius), yaw, roll]]
        .into_iter()
        .any(|probe| trace(location, rotation_to_location_with_dist(probe, location, dist)).is_some())
}

fn script_is_block_rotation(rotation: [i32; 3], location: Vec3, dist: f32, radius: i32, trace: &impl Fn(Vec3, Vec3) -> Option<Hit>) -> bool {
    let [pitch, yaw, roll] = rotation;
    [[pitch, yaw.wrapping_add(radius), roll], [pitch, yaw.wrapping_sub(radius), roll], [pitch.wrapping_add(radius), yaw, roll], [pitch.wrapping_sub(radius), yaw, roll]]
        .into_iter()
        .any(|probe| trace(location, location - rotator_forward(probe) * dist).is_some())
}

fn zoom_bind(value: Option<&str>) -> f32 {
    value
        .into_iter()
        .flat_map(|bind| bind.split('|'))
        .filter_map(|command| {
            let mut words = command.split_whitespace();
            let sign = match words.next()?.to_ascii_lowercase().as_str() {
                "zoominpress" => -1.0,
                "zoomoutpress" => 1.0,
                _ => return None,
            };
            let speed = words
                .filter_map(|word| word.split_once('='))
                .find(|(key, _)| key.eq_ignore_ascii_case("speed"))
                .and_then(|(_, speed)| speed.parse().ok())
                .unwrap_or(ENGINE_WHEEL_DEFAULT_SPEED);
            Some(sign * speed)
        })
        .sum()
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct VolumeState {
    pub old_eye: Vec3,
    pub old_rotation: [i32; 3],
    pub radius: i32,
    pub view_target: Vec3,
    pub zoom_tick: Option<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrbitCamera {
    pub default_move: DefaultMove,
    pub target: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub zoom: f32,
    pub desired_zoom: f32,
    pub distance: f32,
    pub lift: f32,
    pub settings: CameraSettings,
    pub special: SpecialCameraState,
    pub volume: VolumeState,
    pub manual_yaw: bool,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self::behind(Vec3::ZERO, 0, CameraSettings::default())
    }
}

pub fn ue_angle(units: f32) -> f32 {
    units / UE_UNITS_PER_TURN * TAU
}

impl OrbitCamera {
    pub fn behind(target: Vec3, yaw_units: i32, settings: CameraSettings) -> Self {
        let mut camera = Self {
            default_move: DefaultMove::default(),
            target,
            yaw: ue_angle(yaw_units as f32) + PI,
            pitch: ue_angle(ENGINE_DEFAULT_PITCH_UNITS),
            zoom: ENGINE_DEFAULT_ZOOM,
            desired_zoom: ENGINE_DEFAULT_ZOOM,
            distance: 0.0,
            lift: 0.0,
            settings,
            special: SpecialCameraState::default(),
            volume: VolumeState { view_target: target, ..VolumeState::default() },
            manual_yaw: false,
        };
        camera.collide(|_, _| None);
        camera
    }

    pub fn view_distance(&self) -> f32 {
        ENGINE_BASE_DISTANCE + self.zoom
    }

    pub fn camera(&self) -> Camera {
        if self.special.active() {
            return Camera { eye: self.special.eye(), target: self.special.look_at(), fov_x_deg: ENGINE_DEFAULT_FOV_X_DEG, near: NEAR, far: FAR };
        }
        let eye = self.target + self.distance * direction(self.yaw, self.pitch);
        Camera { eye, target: self.target, fov_x_deg: ENGINE_DEFAULT_FOV_X_DEG, near: NEAR, far: FAR }
    }

    pub fn facing(&self) -> i32 {
        ((self.yaw - PI) / TAU * UE_UNITS_PER_TURN).round().rem_euclid(UE_UNITS_PER_TURN) as i32
    }

    pub fn follow(&mut self, position: Vec3) {
        let target = position + Vec3::Z * (self.settings.view_height_adjust + self.lift);
        let manual = std::mem::take(&mut self.manual_yaw);
        if self.settings.auto_tracking
            && self.settings.auto_tracking_speed != 0.0
            && !manual
            && !self.default_move.moving
            && !self.special.active()
            && target != self.target
        {
            let toward = target - self.volume.old_eye;
            let wanted = ((toward.y.atan2(toward.x) / TAU * UE_UNITS_PER_TURN).round() as i32) & 0xffff;
            let old = self.ue_yaw() & 0xffff;
            let fraction = self.settings.auto_tracking_speed.min(1.0);
            let ahead = wanted.wrapping_sub(old) & 0xffff;
            let yaw = if ahead < 0x8000 {
                old + (fraction * ahead as f32) as i32
            } else {
                old - (fraction * (old.wrapping_sub(wanted) & 0xffff) as f32) as i32
            };
            self.set_ue_yaw(wrap_units(yaw));
        }
        self.target = target;
        self.volume.view_target = position;
    }

    pub fn ue_yaw(&self) -> i32 {
        wrap_units(((self.yaw - PI) / TAU * UE_UNITS_PER_TURN).round() as i32)
    }

    pub fn ue_pitch(&self) -> i32 {
        wrap_units((-self.pitch / TAU * UE_UNITS_PER_TURN).round() as i32)
    }

    fn set_ue_yaw(&mut self, units: i32) {
        self.yaw = ue_angle(units as f32) + PI;
    }

    fn set_ue_pitch(&mut self, units: i32) {
        self.pitch = -ue_angle(units as f32);
    }

    fn engine_rotation(&self) -> [i32; 3] {
        [self.ue_pitch(), self.ue_yaw(), 0]
    }

    fn set_engine_rotation(&mut self, rotation: [i32; 3]) {
        self.set_ue_pitch(rotation[0]);
        self.set_ue_yaw(rotation[1]);
    }

    pub fn observer_default_camera(&mut self) {
        let views = &self.settings.fixed_views[..self.settings.fixed_view_count];
        let Some(view) = views.first() else { return };
        let m = &mut self.default_move;
        m.use_yaw = true;
        m.use_pitch = true;
        m.use_dist = true;
        m.yaw = view.yaw;
        m.pitch = view.pitch;
        m.dist = view.dist;
        m.moving = true;
    }

    pub fn fixed_default_camera(&mut self, pawn_yaw: i32, pawn_pitch: i32) -> bool {
        self.fixed_default_step(0, pawn_yaw, pawn_pitch)
    }

    pub fn fixed_default_step(&mut self, step: i32, pawn_yaw: i32, pawn_pitch: i32) -> bool {
        let count = self.settings.fixed_view_count as i32;
        if count == 0 {
            return false;
        }
        let mut index = self.default_move.fixed_index as i32 + step;
        if index < 0 {
            index = count - 1;
        } else if index >= count {
            index = 0;
        }
        self.default_move.fixed_index = index as usize;
        self.fixed_view_camera(index as usize, pawn_yaw, pawn_pitch)
    }

    pub fn exec_pawn_camera(&mut self, command: &str, action: InputAction, held_ms: f32, pawn: Option<(i32, i32)>, camera_fixed: bool) -> PawnCameraExec {
        let mut cmd = command;
        if parse_command(&mut cmd, "FIXEDDEFAULTCAMERA") {
            let on_release = parse_command(&mut cmd, "OnRelease");
            let expected = if on_release { InputAction::Release } else { InputAction::Press };
            if action != expected {
                return PawnCameraExec::Ignored;
            }
            let Some((pawn_yaw, pawn_pitch)) = pawn else { return PawnCameraExec::Ignored };
            if self.special.active() || camera_fixed {
                return PawnCameraExec::Ignored;
            }
            if action == InputAction::Release && parse_float(cmd, "MAXPRESSEDTIME=").is_some_and(|max| max < held_ms) {
                return PawnCameraExec::Ignored;
            }
            let step = if parse_command(&mut cmd, "UP") {
                -1
            } else if parse_command(&mut cmd, "DOWN") {
                1
            } else {
                0
            };
            return if self.fixed_default_step(step, pawn_yaw, pawn_pitch) { PawnCameraExec::FixedView } else { PawnCameraExec::Ignored };
        }
        let mut cmd = command;
        if parse_command(&mut cmd, "TURNBACK") && action == InputAction::Press {
            let Some((pawn_yaw, _)) = pawn else { return PawnCameraExec::Ignored };
            if self.special.active() {
                return PawnCameraExec::Ignored;
            }
            return if self.turn_back(pawn_yaw) { PawnCameraExec::TurnBack } else { PawnCameraExec::Ignored };
        }
        PawnCameraExec::Ignored
    }

    pub fn camera_mode(&mut self, mode: i32, pawn_yaw: i32, pawn_pitch: i32) -> bool {
        if mode == 1 {
            self.default_move.fixed_mode = true;
            return self.fixed_view_camera(1, pawn_yaw, pawn_pitch);
        }
        if !self.default_move.fixed_mode {
            return false;
        }
        self.default_move.fixed_mode = false;
        self.fixed_view_camera(0, pawn_yaw, pawn_pitch)
    }

    fn fixed_view_camera(&mut self, index: usize, pawn_yaw: i32, pawn_pitch: i32) -> bool {
        let views = &self.settings.fixed_views[..self.settings.fixed_view_count];
        let Some(&view) = views.get(index) else { return false };
        let m = &mut self.default_move;
        m.use_yaw = true;
        m.use_pitch = true;
        m.use_dist = true;
        m.pitch = view.pitch;
        m.yaw = view.yaw;
        m.dist = view.dist;
        self.settings.view_height_adjust = view.view_height;
        m.disable_zoom = view.disable_zoom;
        m.hide_player = view.hide_player;
        m.saved_target_yaw = pawn_yaw;
        m.saved_target_pitch = pawn_pitch;
        m.moving = true;
        true
    }

    pub fn special_rest(&self) -> Rest {
        Rest { yaw: self.ue_yaw(), pitch: self.ue_pitch(), dist: self.view_distance() }
    }

    pub fn camera_commands(&mut self, world: &mut World, dt: f32) {
        let pawn = world.me.as_ref().map(|me| (me.object_id, me.position, me.heading));
        for command in world.effects.take_camera_commands() {
            match command {
                CameraCommand::Special(packet) => {
                    let player = world.me.as_ref().filter(|me| me.object_id == packet.target_id).or_else(|| world.entity(packet.target_id));
                    if player.is_some_and(|entity| matches!(entity.kind, EntityKind::Player { .. })) {
                        self.special.request(packet.target_id, &Request::from_packet(&packet));
                    }
                }
                CameraCommand::Normal => {
                    let rest = self.special_rest();
                    self.special.normal_camera(rest);
                }
                CameraCommand::Mode(mode) => {
                    if let Some((_, _, heading)) = pawn {
                        self.camera_mode(mode, heading, 0);
                    }
                    world.camera_fixed = self.default_move.fixed_mode;
                    if mode == 1 {
                        world.pending_click_target = None;
                    }
                }
            }
        }
        if !self.special.active() {
            return;
        }
        let target = match self.special.view_target {
            Some(id) => world.me.as_ref().filter(|me| me.object_id == id).or_else(|| world.entity(id)).map(|entity| entity.position),
            None => pawn.map(|(_, position, _)| position),
        };
        let rest = self.special_rest();
        self.special.tick(dt, target, rest);
    }

    pub fn turn_back(&mut self, pawn_yaw: i32) -> bool {
        let current = if self.default_move.moving { self.default_move.saved_target_yaw } else { self.ue_yaw() };
        let diff = current.wrapping_sub(pawn_yaw) & 0xffff;
        let m = &mut self.default_move;
        m.saved_target_yaw = if TURN_BACK_FRONT.contains(&diff) { pawn_yaw } else { pawn_yaw.wrapping_add(0x8000) };
        m.moving = true;
        m.use_yaw = true;
        true
    }

    pub fn player_tick(&mut self, delta_time: f32) {
        if !self.default_move.moving {
            if self.settings.volume_camera {
                self.volume.zoom_tick = Some(delta_time);
            }
            return;
        }
        let m = self.default_move;
        if m.use_yaw {
            let target = m.saved_target_yaw.wrapping_add(m.yaw).wrapping_sub(self.ue_yaw());
            if target.rem_euclid(65536) == 0 {
                self.default_move.use_yaw = false;
            } else {
                let step = approach_units(delta_time, wrap_units(target));
                let yaw = wrap_units(self.ue_yaw() + step);
                self.set_ue_yaw(yaw);
                self.volume.old_rotation[1] = yaw;
            }
        }
        if m.use_pitch {
            let target = m.saved_target_pitch.wrapping_add(m.pitch).wrapping_sub(self.ue_pitch());
            if target.rem_euclid(65536) == 0 {
                self.default_move.use_pitch = false;
            } else {
                let step = approach_units(delta_time, wrap_units(target));
                let pitch = wrap_units(self.ue_pitch() + step);
                self.set_ue_pitch(pitch);
                self.volume.old_rotation[0] = pitch;
            }
        }
        if m.use_dist {
            if self.zoom + ENGINE_BASE_DISTANCE == m.dist {
                self.default_move.use_dist = false;
            } else {
                let temp = m.dist - (self.zoom + ENGINE_BASE_DISTANCE);
                self.zoom += approach_distance(delta_time, temp);
                self.desired_zoom = self.zoom;
            }
        }
        let m = &mut self.default_move;
        if !m.use_yaw && !m.use_pitch && !m.use_dist {
            m.moving = false;
        }
    }

    pub fn drag(&mut self, dx: f32, dy: f32) {
        let (yaw, pitch) = self.settings.drag_radians(dx, dy);
        if yaw != 0.0 {
            self.default_move.moving = false;
            self.manual_yaw = true;
        }
        self.yaw -= yaw;
        self.pitch = (self.pitch + pitch).clamp(-ue_angle(ENGINE_MAX_PITCH_UNITS), -ue_angle(ENGINE_MIN_PITCH_UNITS));
    }

    pub fn wheel(&mut self, notches: f32, pawn_radius: Option<f32>) {
        if self.special.active() || self.default_move.disable_zoom {
            return;
        }
        let bind = if notches > 0.0 { self.settings.wheel_up } else { self.settings.wheel_down };
        let amount = bind * notches.abs();
        if amount == 0.0 {
            return;
        }
        let fine = pawn_radius.is_some_and(|radius| self.desired_zoom < 2.0 * radius + self.settings.min_zoom);
        let step = amount.abs() * if fine { ENGINE_WHEEL_FINE_FACTOR } else { 1.0 };
        if amount > 0.0 {
            self.desired_zoom = (self.desired_zoom + step).min(self.settings.max_zoom);
            self.zoom = (self.zoom + step).min(self.settings.max_zoom);
            self.lift = 0.0;
        } else {
            self.desired_zoom = (self.desired_zoom - step).max(self.settings.min_zoom);
            self.zoom -= step;
            if self.zoom < self.settings.min_zoom {
                self.zoom = self.settings.min_zoom;
                if let Some(radius) = pawn_radius
                    && self.lift == 0.0
                {
                    self.lift = radius * ENGINE_ZOOMED_IN_LIFT_FACTOR;
                }
            }
        }
        self.distance = self.view_distance() - ENGINE_CAMERA_HIT_MARGIN;
    }

    pub fn exec_input(&mut self, command: &str, action: InputAction, delta: f32) -> bool {
        let mut cmd = command;
        if parse_command(&mut cmd, "ZOOMINHOLD") && action == InputAction::Hold {
            return self.zoom_hold(cmd, delta, true);
        }
        if parse_command(&mut cmd, "ZOOMOUTHOLD") && action == InputAction::Hold {
            return self.zoom_hold(cmd, delta, false);
        }
        false
    }

    fn zoom_hold(&mut self, command: &str, delta: f32, zoom_in: bool) -> bool {
        if self.special.active() || self.default_move.disable_zoom {
            return true;
        }
        let speed = parse_float(command, "SPEED=").unwrap_or(ENGINE_HOLD_DEFAULT_SPEED);
        let step = f64::from(speed) * f64::from(delta);
        if zoom_in {
            self.desired_zoom = (f64::from(self.desired_zoom) - step) as f32;
            self.zoom = (f64::from(self.zoom) - step) as f32;
            if self.desired_zoom < ENGINE_HOLD_MIN_ZOOM {
                self.desired_zoom = ENGINE_HOLD_MIN_ZOOM;
            }
            if self.zoom < ENGINE_HOLD_MIN_ZOOM {
                self.zoom = ENGINE_HOLD_MIN_ZOOM;
            }
        } else {
            self.desired_zoom = (f64::from(self.desired_zoom) + step) as f32;
            self.zoom = (f64::from(self.zoom) + step) as f32;
            if self.desired_zoom > ENGINE_HOLD_MAX_ZOOM {
                self.desired_zoom = ENGINE_HOLD_MAX_ZOOM;
            }
            if self.zoom > ENGINE_HOLD_MAX_ZOOM {
                self.zoom = ENGINE_HOLD_MAX_ZOOM;
            }
        }
        true
    }

    pub fn trace_end(&self) -> Vec3 {
        self.target + direction(self.yaw, self.pitch) * (self.view_distance() + ENGINE_CAMERA_HIT_MARGIN) - Vec3::Z * ENGINE_TRACE_END_DROP
    }

    pub fn collide(&mut self, trace: impl Fn(Vec3, Vec3) -> Option<Hit>) {
        if let Some(delta_time) = self.volume.zoom_tick.take() {
            self.volume_zooming(delta_time, &trace);
        }
        if !self.settings.volume_camera {
            self.zoom = self.desired_zoom;
        }
        let eye = if self.settings.volume_camera {
            let (eye, distance) = self.volume_camera(&trace);
            self.distance = distance.max(NEAR);
            eye
        } else {
            let view = if self.settings.hit_check { self.hit_check(&trace) } else { self.hit_loop(&trace) };
            self.distance = view.max(NEAR);
            self.target + self.distance * direction(self.yaw, self.pitch)
        };
        self.volume.old_eye = eye;
        self.volume.old_rotation = self.engine_rotation();
    }

    fn volume_zooming(&mut self, delta_time: f32, trace: &impl Fn(Vec3, Vec3) -> Option<Hit>) {
        if self.zoom == self.desired_zoom {
            return;
        }
        let step = approach_distance(delta_time, self.desired_zoom - self.zoom);
        self.zoom += step;
        if script_is_block_rotation(self.volume.old_rotation, self.volume.view_target, self.zoom + ENGINE_BASE_DISTANCE, self.settings.volume_radius, trace) {
            self.zoom -= step;
            return;
        }
        self.volume.radius = self.settings.volume_radius;
        let start = self.target;
        let end = start - rotator_forward(self.volume.old_rotation) * (self.zoom + ENGINE_BASE_DISTANCE);
        if let Some(hit) = trace(start, end) {
            self.zoom = hit.distance - ENGINE_BASE_DISTANCE;
        }
    }

    fn volume_camera(&mut self, trace: &impl Fn(Vec3, Vec3) -> Option<Hit>) -> (Vec3, f32) {
        let target = self.target;
        let view_dist = self.zoom + ENGINE_BASE_DISTANCE;
        let rotation = self.engine_rotation();
        let view = view_axis(rotation);
        let eye = rotation_to_location_with_dist(rotation, target, view_dist);
        let radius = self.volume.radius.wrapping_sub(ENGINE_VOLUME_RADIUS_MARGIN);
        if !is_block_rotation(rotation, target, view_dist, radius, trace) {
            return (target - view * view_dist, view_dist);
        }
        let end = target - rotation_to_location_with_dist(rotation, target, view_dist);
        let hit = hit_location(target, end, trace(target, end));
        let old_eye = self.volume.old_eye;
        if (old_eye - eye).length() > (hit - eye).length() {
            let projected = (target - hit).dot(view);
            let dist = if projected <= view_dist { projected } else { view_dist };
            return (target - view * dist, dist);
        }
        let old = self.volume.old_rotation;
        let shrunk = view_dist - (eye - old_eye).length();
        let mut current = rotation;
        for _ in 0..ENGINE_VOLUME_TRIES {
            let diff = [wrap_units(old[0].wrapping_sub(current[0])), wrap_units(old[1].wrapping_sub(current[1])), old[2].wrapping_sub(current[2])];
            let half = diff.map(|units| (f64::from(units) * ENGINE_VOLUME_HALF) as i32);
            current = [current[0].wrapping_add(half[0]), current[1].wrapping_add(half[1]), current[2].wrapping_add(half[2])];
            if !is_block_rotation(current, target, shrunk, radius, trace) {
                self.set_engine_rotation(current);
                self.zoom = shrunk - ENGINE_BASE_DISTANCE;
                let turned = wrap_units(old[0].wrapping_sub(current[0])).abs().max(wrap_units(old[1].wrapping_sub(current[1])).abs());
                self.volume.radius = self.volume.radius.wrapping_sub(turned);
                if self.volume.radius < ENGINE_VOLUME_RADIUS_FLOOR {
                    self.volume.radius = ENGINE_VOLUME_RADIUS_FLOOR;
                }
                return (rotation_to_location_with_dist(current, target, shrunk), shrunk);
            }
        }
        self.set_engine_rotation(old);
        self.zoom = shrunk - ENGINE_BASE_DISTANCE;
        (rotation_to_location_with_dist(old, target, shrunk), shrunk)
    }

    fn hit_check(&self, trace: &impl Fn(Vec3, Vec3) -> Option<Hit>) -> f32 {
        let end = self.trace_end();
        let back = direction(self.yaw, self.pitch);
        let along = (end - self.target).normalize_or_zero();
        let view = match trace(self.target, end) {
            Some(hit) => (hit.distance * along.dot(back)).min(self.view_distance()),
            None => self.view_distance(),
        };
        view - ENGINE_CAMERA_HIT_MARGIN
    }

    fn hit_loop(&self, trace: &impl Fn(Vec3, Vec3) -> Option<Hit>) -> f32 {
        let back = direction(self.yaw, self.pitch);
        let mut view = self.view_distance();
        let mut eye = view - ENGINE_CAMERA_HIT_MARGIN;
        while view >= ENGINE_CAMERA_HIT_MARGIN {
            let far = self.target + back * view;
            let Some(hit) = trace(far, self.target) else { break };
            let projected = view - hit.distance;
            if hit.terrain {
                eye = projected.min(view) - ENGINE_CAMERA_HIT_MARGIN;
                break;
            }
            if (projected - eye).abs() >= self.settings.hit_check_min_dist as f32 {
                break;
            }
            view = (projected - ENGINE_CAMERA_HIT_MARGIN).min(view);
            eye = view - ENGINE_CAMERA_HIT_MARGIN;
        }
        let far = self.target + back * view;
        match trace(self.target, far) {
            Some(hit) if hit.terrain => hit.distance.min(view) - ENGINE_CAMERA_HIT_MARGIN,
            _ => eye,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FreeCamera {
    pub position: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl FreeCamera {
    pub fn from_orbit(orbit: &OrbitCamera) -> Self {
        let camera = orbit.camera();
        Self { position: camera.eye, yaw: orbit.yaw + PI, pitch: -orbit.pitch }
    }

    pub fn camera(&self) -> Camera {
        let forward = direction(self.yaw, self.pitch);
        Camera { eye: self.position, target: self.position + forward, fov_x_deg: ENGINE_DEFAULT_FOV_X_DEG, near: NEAR, far: FAR }
    }

    pub fn look(&mut self, dx: f32, dy: f32) {
        self.yaw -= dx * DRAG_SENSITIVITY;
        self.pitch = (self.pitch - dy * DRAG_SENSITIVITY).clamp(-89f32.to_radians(), 89f32.to_radians());
    }

    pub fn advance(&mut self, moves: &Movement, dt: f32) {
        let forward = direction(self.yaw, self.pitch);
        let right = forward.cross(Vec3::Z).normalize_or(Vec3::X);
        let mut step = Vec3::ZERO;
        if moves.forward {
            step += forward;
        }
        if moves.back {
            step -= forward;
        }
        if moves.right {
            step += right;
        }
        if moves.left {
            step -= right;
        }
        let speed = if moves.fast { FREE_FAST_SPEED } else { FREE_SPEED };
        self.position += step.normalize_or_zero() * speed * dt;
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Movement {
    pub forward: bool,
    pub back: bool,
    pub left: bool,
    pub right: bool,
    pub fast: bool,
}

fn direction(yaw: f32, pitch: f32) -> Vec3 {
    Vec3::new(pitch.cos() * yaw.cos(), pitch.cos() * yaw.sin(), pitch.sin())
}

pub fn apply_shake(camera: Camera, offset: [i32; 3]) -> Camera {
    if offset == [0; 3] {
        return camera;
    }
    let look = camera.target - camera.eye;
    let distance = look.length();
    if distance <= f32::EPSILON {
        return camera;
    }
    let mut rotation = killer_assets::scene::transform::vector_rotation(look);
    for axis in 0..3 {
        rotation[axis] = rotation[axis].wrapping_add(offset[axis]);
    }
    let forward = killer_assets::engine_math::rotator_forward(rotation);
    Camera { target: camera.eye + forward * distance, ..camera }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_ui::ini::parse_ini;

    const USER_INI: &str = include_str!("../../../crates/killer-ui/tests/fixtures/user.ini");

    fn ini_settings() -> CameraSettings {
        CameraSettings::from_ini(&parse_ini(USER_INI))
    }

    fn orbit(target: Vec3, yaw: f32, pitch: f32, distance: f32) -> OrbitCamera {
        let zoom = distance + ENGINE_CAMERA_HIT_MARGIN - ENGINE_BASE_DISTANCE;
        OrbitCamera { special: SpecialCameraState::default(), default_move: DefaultMove::default(), target, yaw, pitch, zoom, desired_zoom: zoom, distance, lift: 0.0, settings: ini_settings(), volume: VolumeState::default(), manual_yaw: false }
    }

    fn wall_at(x: f32, terrain: bool) -> impl Fn(Vec3, Vec3) -> Option<Hit> {
        move |from: Vec3, to: Vec3| {
            let dir = to - from;
            let t = (x - from.x) / dir.x * dir.length();
            (dir.x != 0.0 && t >= 0.0 && t <= dir.length()).then_some(Hit { distance: t, terrain })
        }
    }

    #[test]
    fn the_user_ini_gives_the_zoom_limits_the_hit_check_and_the_wheel_binds() {
        let s = ini_settings();
        assert_eq!(s.min_zoom, -200.0);
        assert_eq!(s.max_zoom, 65500.0);
        assert_eq!(s.view_height_adjust, 12.0);
        assert!(s.hit_check);
        assert_eq!(s.hit_check_min_dist, 300);
        assert_eq!(s.fixed_default_dist, 230.0);
        assert_eq!(s.fixed_default_pitch, -2700);
        assert_eq!((s.wheel_down, s.wheel_up), (-20.0, 20.0));
    }

    #[test]
    fn without_the_ini_the_class_defaults_apply_and_the_wheel_is_unbound() {
        let s = CameraSettings::from_ini(&parse_ini(""));
        assert_eq!(s, CameraSettings::default());
        assert_eq!((s.min_zoom, s.max_zoom), (-200.0, 1000.0));
        assert_eq!(s.view_height_adjust, 0.0);
        assert!(!s.hit_check);
        assert_eq!(s.hit_check_min_dist, 0);
        assert_eq!((s.wheel_down, s.wheel_up), (0.0, 0.0));
        let partial = parse_ini("[Engine.LineagePlayerController]\nMaxZoomingDist=500\n[Engine.Input]\nMouseWheelUp=zoomoutpress\nMouseWheelDown=Jump | ZoomInPress Speed=5.0 | Other");
        let s = CameraSettings::from_ini(&partial);
        assert_eq!((s.min_zoom, s.max_zoom), (-200.0, 500.0));
        assert_eq!((s.wheel_down, s.wheel_up), (-5.0, 1.0));
        let mut o = OrbitCamera::default();
        o.wheel(1.0, None);
        o.wheel(-1.0, None);
        assert_eq!(o.zoom, ENGINE_DEFAULT_ZOOM);
    }

    #[test]
    fn orbit_eye_sits_behind_and_above_the_target() {
        let o = orbit(Vec3::new(100.0, 200.0, -3400.0), 0.0, 30f32.to_radians(), 700.0);
        let c = o.camera();
        assert!((c.target - Vec3::new(100.0, 200.0, -3400.0)).length() < 1e-3);
        assert!((c.eye.z - (-3400.0 + 350.0)).abs() < 1.0);
        assert!(((c.eye - c.target).length() - 700.0).abs() < 1e-2);
    }

    #[test]
    fn initial_orbit_from_player_start_views_230_and_places_the_eye_30_short() {
        let location = Vec3::new(100.0, 200.0, -3400.0);
        for settings in [ini_settings(), CameraSettings::default()] {
            let o = OrbitCamera::behind(location, 16384, settings);
            let c = o.camera();
            let pitch = 14.83f32.to_radians();
            assert_eq!(o.target, location);
            assert_eq!(c.target, location);
            assert_eq!(o.zoom, -20.0);
            assert_eq!(o.view_distance(), 230.0);
            assert_eq!(o.distance, 200.0);
            assert!((c.eye.x - 100.0).abs() < 1e-2);
            assert!((c.eye.y - (200.0 - 200.0 * pitch.cos())).abs() < 1e-1);
            assert!((c.eye.z - (-3400.0 + 200.0 * pitch.sin())).abs() < 1e-1);
            assert!(((c.eye - c.target).length() - 200.0).abs() < 1e-2);
        }
        let o = OrbitCamera::behind(location, 16384, ini_settings());
        let end = o.trace_end();
        assert!(((end + Vec3::Z * ENGINE_TRACE_END_DROP - location).length() - 260.0).abs() < 1e-2);
        assert!(end.y < o.camera().eye.y);
    }

    #[test]
    fn facing_is_the_heading_the_orbit_was_placed_behind() {
        assert_eq!(OrbitCamera::behind(Vec3::ZERO, 16384, ini_settings()).facing(), 16384);
        assert_eq!(OrbitCamera::behind(Vec3::ZERO, 0, ini_settings()).facing(), 0);
        assert_eq!(OrbitCamera::behind(Vec3::ZERO, 49152, ini_settings()).facing(), 49152);
    }

    #[test]
    fn follow_adds_the_ini_view_height_and_keeps_yaw_pitch_and_distance() {
        let mut o = OrbitCamera::behind(Vec3::ZERO, 16384, ini_settings());
        let before = o;
        o.follow(Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(o.target, Vec3::new(10.0, 20.0, 42.0));
        assert_eq!((o.yaw, o.pitch, o.distance), (before.yaw, before.pitch, before.distance));
        assert!((o.camera().eye - before.camera().eye - Vec3::new(10.0, 20.0, 42.0)).length() < 1e-3);
        let mut o = OrbitCamera::default();
        o.follow(Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(o.target, Vec3::new(10.0, 20.0, 30.0));
    }

    #[test]
    fn camera_axis_1058043a_uses_bind_speed_and_integer_ue_units() {
        let settings = ini_settings();
        assert_eq!(settings.mouse_speed, [3600.0; 2]);
        for (delta, units) in [(1.0, 36.0), (-1.0, -36.0), (0.04, 1.0), (-0.04, -1.0), (0.01, 0.0)] {
            let (yaw, pitch) = settings.drag_radians(delta, delta);
            assert_eq!((yaw, pitch), (ue_angle(units), ue_angle(units)));
        }
        let settings = CameraSettings::from_ini(&parse_ini("[Engine.Input]\nMouseX=Other | camerayaw speed=-7200\nMouseY=CameraPitch\n"));
        assert_eq!(settings.mouse_speed, [-7200.0, 1.0]);
        assert_eq!(settings.drag_radians(1.0, 100.0), (ue_angle(-72.0), ue_angle(1.0)));
        let settings = CameraSettings::from_ini(&parse_ini("[Engine.Input]\nMouseX=Other\n"));
        assert_eq!(settings.drag_radians(500.0, -500.0), (0.0, 0.0));
    }

    #[test]
    fn pitch_clamp_matches_engine_units() {
        let mut o = OrbitCamera::behind(Vec3::ZERO, 0, ini_settings());
        o.drag(0.0, 1e6);
        assert!((o.pitch - ue_angle(15000.0)).abs() < 1e-6);
        o.drag(0.0, -1e6);
        o.drag(0.0, -1e6);
        o.drag(0.0, -1e6);
        assert!((o.pitch + ue_angle(16384.0)).abs() < 1e-6);
        assert!(o.camera().view_proj(1.0).is_finite());
        o.drag(0.0, 10_000.0);
        assert!(o.pitch <= ue_angle(15000.0) + 1e-6);
    }

    #[test]
    fn the_wheel_is_ignored_during_a_special_camera_like_zoominpress_1057ad65() {
        let mut o = OrbitCamera::behind(Vec3::ZERO, 0, ini_settings());
        o.special.request(1, &killer_world::special_camera::Request { dist: 300.0, yaw: 0, pitch: 0, time: 1.0, duration: 2.0, turn: 0, rise: 0, wide: false, flag: false });
        assert!(o.special.active());
        o.wheel(1.0, None);
        o.wheel(-1.0, None);
        assert_eq!((o.zoom, o.desired_zoom), (ENGINE_DEFAULT_ZOOM, ENGINE_DEFAULT_ZOOM));
    }

    #[test]
    fn the_far_plane_is_the_core_far_clipping_plane_so_the_max_zoom_stays_in_view() {
        assert_eq!(ENGINE_FAR_CLIPPING_PLANE, 65536.0);
        let o = OrbitCamera::behind(Vec3::ZERO, 0, ini_settings());
        assert_eq!((o.camera().near, o.camera().far, o.camera().fov_x_deg), (10.0, 65536.0, 60.0));
    }

    #[test]
    fn the_wheel_moves_the_zoom_by_the_bind_speed_between_min_and_max_zooming_dist() {
        let mut o = OrbitCamera::behind(Vec3::ZERO, 0, ini_settings());
        o.wheel(1.0, None);
        assert_eq!(o.zoom, 0.0);
        assert_eq!(o.view_distance(), 250.0);
        assert_eq!(o.distance, 220.0);
        o.wheel(-2.0, None);
        assert_eq!(o.zoom, -40.0);
        for _ in 0..4000 {
            o.wheel(1.0, None);
        }
        assert_eq!(o.zoom, 65500.0);
        assert_eq!(o.view_distance(), 65750.0);
        for _ in 0..4000 {
            o.wheel(-1.0, None);
        }
        assert_eq!(o.zoom, -200.0);
        assert_eq!(o.view_distance(), 50.0);
        assert_eq!(o.distance, 20.0);
        assert_eq!(o.lift, 0.0);
        o.wheel(0.0, None);
        assert_eq!(o.zoom, -200.0);
        let mut o = OrbitCamera::behind(Vec3::ZERO, 0, CameraSettings { wheel_up: 20.0, wheel_down: -20.0, ..Default::default() });
        for _ in 0..100 {
            o.wheel(1.0, None);
        }
        assert_eq!((o.zoom, o.view_distance(), o.distance), (1000.0, 1250.0, 1220.0));
    }

    #[test]
    fn near_the_minimum_a_pawn_makes_the_step_an_eighth_and_bottoming_out_lifts_the_target() {
        let mut o = OrbitCamera { zoom: -170.0, desired_zoom: -170.0, ..OrbitCamera::behind(Vec3::ZERO, 0, ini_settings()) };
        o.wheel(-1.0, Some(8.0));
        assert_eq!(o.zoom, -190.0);
        o.wheel(-1.0, Some(8.0));
        assert_eq!(o.zoom, -192.5);
        assert_eq!(o.lift, 0.0);
        for _ in 0..10 {
            o.wheel(-1.0, Some(8.0));
        }
        assert_eq!(o.zoom, -200.0);
        assert_eq!(o.lift, 6.4);
        o.follow(Vec3::ZERO);
        assert_eq!(o.target, Vec3::Z * (12.0 + 6.4));
        o.wheel(1.0, Some(8.0));
        assert_eq!(o.zoom, -197.5);
        assert_eq!(o.lift, 0.0);
        o.follow(Vec3::ZERO);
        assert_eq!(o.target, Vec3::Z * 12.0);
    }

    #[test]
    fn with_hit_check_a_wall_places_the_eye_30_in_front_of_the_hit_and_the_distance_returns_when_it_clears() {
        let mut o = OrbitCamera::behind(Vec3::ZERO, 16384, ini_settings());
        let wall = wall_at(120.0, false);
        o.yaw = 0.0;
        o.pitch = 0.0;
        o.collide(&wall);
        assert!((o.distance - 90.0).abs() < 1e-3, "{}", o.distance);
        assert!(o.camera().eye.x < 120.0);
        assert_eq!(o.view_distance(), 230.0);
        o.collide(|_, _| None);
        assert!((o.distance - 200.0).abs() < 1e-3);
        o.pitch = 30f32.to_radians();
        o.collide(&wall);
        let eye = o.camera().eye;
        let margin = 120.0 - eye.x;
        assert!(margin > 25.0 && margin < 35.0, "{eye:?}");
        assert!(o.distance < 138.0 && o.distance > 100.0, "{}", o.distance);
        o.collide(|_, _| Some(Hit { distance: 5.0, terrain: false }));
        assert_eq!(o.distance, NEAR);
        assert!(o.camera().view_proj(1.0).is_finite());
    }

    #[test]
    fn without_hit_check_the_loop_traces_from_the_far_end_and_only_pulls_in_hits_closer_than_the_min_dist() {
        let settings = CameraSettings { hit_check_min_dist: 300, ..Default::default() };
        let mut o = OrbitCamera::behind(Vec3::ZERO, 16384, settings);
        o.yaw = 0.0;
        o.pitch = 0.0;
        assert_eq!(o.distance, 200.0);
        o.collide(wall_at(120.0, false));
        assert!((o.distance - 60.0).abs() < 1e-3, "{}", o.distance);
        o.collide(wall_at(120.0, true));
        assert!((o.distance - 90.0).abs() < 1e-3, "{}", o.distance);
        o.collide(|_, _| None);
        assert_eq!(o.distance, 200.0);
        let mut far = OrbitCamera::behind(Vec3::ZERO, 16384, settings);
        far.yaw = 0.0;
        far.pitch = 0.0;
        far.zoom = 1000.0;
        far.desired_zoom = 1000.0;
        far.collide(wall_at(120.0, false));
        assert_eq!(far.distance, 1220.0);
        far.collide(wall_at(120.0, true));
        assert!((far.distance - 90.0).abs() < 1e-3, "{}", far.distance);
        let mut o = OrbitCamera::behind(Vec3::ZERO, 16384, CameraSettings::default());
        o.yaw = 0.0;
        o.pitch = 0.0;
        o.collide(wall_at(120.0, false));
        assert_eq!(o.distance, 200.0);
        o.collide(wall_at(120.0, true));
        assert!((o.distance - 90.0).abs() < 1e-3, "{}", o.distance);
    }

    #[test]
    fn without_hit_check_the_outermost_wall_is_found_first_and_the_loop_walks_inward() {
        let settings = CameraSettings { hit_check_min_dist: 300, ..Default::default() };
        let mut o = OrbitCamera::behind(Vec3::ZERO, 16384, settings);
        o.yaw = 0.0;
        o.pitch = 0.0;
        let walls = |from: Vec3, to: Vec3| [180.0, 100.0].into_iter().filter_map(|x| wall_at(x, false)(from, to)).min_by(|a, b| a.distance.total_cmp(&b.distance));
        o.collide(walls);
        assert!((o.distance - 40.0).abs() < 1e-3, "{}", o.distance);
    }

    #[test]
    fn free_camera_starts_where_the_orbit_eye_is_and_looks_at_the_target() {
        let o = orbit(Vec3::new(0.0, 0.0, 0.0), 0.3, 0.4, 500.0);
        let f = FreeCamera::from_orbit(&o);
        let c = f.camera();
        let toward_target = (o.camera().target - c.eye).normalize();
        assert!((c.eye - o.camera().eye).length() < 1e-3);
        assert!(((c.target - c.eye).normalize() - toward_target).length() < 1e-4);
    }

    #[test]
    fn auto_tracking_turns_the_view_toward_the_new_target_from_the_old_eye_unless_dragged_or_moving_to_default() {
        let mut settings = ini_settings();
        settings.auto_tracking = true;
        assert_eq!(settings.auto_tracking_speed, 0.4);
        let start = OrbitCamera::behind(Vec3::ZERO, 0, settings);
        let old_eye = start.volume.old_eye;
        let mut tracked = start;
        tracked.follow(Vec3::new(0.0, 300.0, 0.0));
        let toward = tracked.target - old_eye;
        let wanted = ((toward.y.atan2(toward.x) / TAU * UE_UNITS_PER_TURN).round() as i32) & 0xffff;
        let old = start.ue_yaw() & 0xffff;
        let step = (0.4 * (wanted.wrapping_sub(old) & 0xffff) as f32) as i32;
        assert_eq!(tracked.ue_yaw(), wrap_units(old + step));
        assert_ne!(tracked.yaw, start.yaw);
        let mut still = start;
        still.follow(Vec3::ZERO + Vec3::Z * (still.settings.view_height_adjust + still.lift));
        assert_eq!(still.yaw, start.yaw);
        let mut dragged = start;
        dragged.drag(10.0, 0.0);
        let yaw = dragged.yaw;
        dragged.follow(Vec3::new(0.0, 300.0, 0.0));
        assert_eq!(dragged.yaw, yaw);
        dragged.follow(Vec3::new(0.0, 600.0, 0.0));
        assert_ne!(dragged.yaw, yaw);
        let mut moving = start;
        moving.default_move.moving = true;
        moving.follow(Vec3::new(0.0, 300.0, 0.0));
        assert_eq!(moving.yaw, start.yaw);
        let mut off = start;
        off.settings.auto_tracking = false;
        off.follow(Vec3::new(0.0, 300.0, 0.0));
        assert_eq!(off.yaw, start.yaw);
    }
}

#[cfg(test)]
mod default_camera_tests {
    use super::*;

    fn settings() -> CameraSettings {
        CameraSettings::from_ini(&killer_ui::ini::parse_ini(include_str!("../../../crates/killer-ui/tests/fixtures/user.ini")))
    }

    #[test]
    fn the_user_ini_binds_fixed_default_camera_on_quick_right_release_and_turn_back_on_middle() {
        let settings = settings();
        assert_eq!(settings.fixed_on_right_release_ms, Some(200.0));
        assert!(settings.turn_back_on_middle);
        assert_eq!(settings.fixed_view_count, 3);
        assert_eq!(settings.fixed_views[0], FixedView { yaw: 0, pitch: -2700, dist: 230.0, view_height: 0.0, hide_player: false, disable_zoom: false });
        assert_eq!(settings.fixed_views[1].pitch, 560);
        assert!(settings.fixed_views[1].hide_player);
    }

    #[test]
    fn camera_mode_one_selects_fixed_view_one_and_other_modes_restore_view_zero() {
        let mut camera = OrbitCamera::behind(Vec3::ZERO, 0, settings());
        assert!(!camera.camera_mode(0, 0, 0));
        assert!(camera.camera_mode(1, 500, 0));
        let m = camera.default_move;
        assert!(m.fixed_mode && m.moving && m.use_yaw && m.use_pitch && m.use_dist);
        assert_eq!((m.pitch, m.dist, m.hide_player, m.saved_target_yaw), (560, 80.0, true, 500));
        assert_eq!(camera.settings.view_height_adjust, 14.0);
        assert!(camera.camera_mode(0, 500, 0));
        assert!(!camera.default_move.fixed_mode);
        assert_eq!((camera.default_move.pitch, camera.default_move.dist), (-2700, 230.0));
        assert!(!camera.camera_mode(0, 500, 0));
    }

    #[test]
    fn the_fixed_default_camera_eases_yaw_pitch_and_distance_like_calc_camera_moving_to_default() {
        let mut camera = OrbitCamera::behind(Vec3::ZERO, 0, settings());
        camera.set_ue_yaw(4000);
        camera.set_ue_pitch(-1000);
        assert!(camera.fixed_default_camera(1000, 0));
        assert_eq!(camera.settings.view_height_adjust, 0.0);
        camera.player_tick(0.1);
        assert_eq!(camera.ue_yaw(), 4000 + ((1000 - 4000) as f32 / 6.0 * 50.0 * 0.1) as i32);
        for _ in 0..2000 { camera.player_tick(0.016); }
        assert_eq!((camera.ue_yaw(), camera.ue_pitch()), (1000, -2700));
        assert!((camera.zoom + ENGINE_BASE_DISTANCE - 230.0).abs() < 1e-3);
        assert!(!camera.default_move.moving);
    }

    fn binds() -> killer_world::engine_input::KeyBinds {
        let ini = killer_ui::ini::parse_ini("[Engine.Input]\nPageUp=FixedDefaultCamera Down\nPageDown=FixedDefaultCamera Up\nHome=FixedDefaultCamera\nEnd=TurnBack\nEscape=CameraModeSelectingCancel\n");
        killer_world::engine_input::KeyBinds::from_ini(|name| ini.get(killer_world::engine_input::INPUT_SECTION, name))
    }

    fn press(camera: &mut OrbitCamera, binds: &mut killer_world::engine_input::KeyBinds, vk: u32, camera_fixed: bool) -> Vec<PawnCameraExec> {
        binds.press(vk).iter().map(|command| camera.exec_pawn_camera(command, InputAction::Press, 0.0, Some((1000, 0)), camera_fixed)).collect()
    }

    #[test]
    fn pageup_pagedown_home_and_end_run_the_user_ini_camera_binds() {
        let mut binds = binds();
        let mut camera = OrbitCamera::behind(Vec3::ZERO, 0, settings());
        let mut step = |camera: &mut OrbitCamera, vk: u32| {
            let fired = press(camera, &mut binds, vk, false);
            binds.release(vk);
            fired
        };
        assert_eq!(step(&mut camera, 0x22), [PawnCameraExec::FixedView]);
        assert_eq!((camera.default_move.fixed_index, camera.default_move.pitch), (2, -3320));
        assert_eq!(step(&mut camera, 0x21), [PawnCameraExec::FixedView]);
        assert_eq!(camera.default_move.fixed_index, 0);
        assert_eq!(step(&mut camera, 0x21), [PawnCameraExec::FixedView]);
        assert_eq!(camera.default_move.fixed_index, 1);
        assert!(camera.default_move.hide_player);
        assert_eq!(step(&mut camera, 0x21), [PawnCameraExec::FixedView]);
        assert_eq!(camera.default_move.fixed_index, 2);
        assert_eq!(step(&mut camera, 0x22), [PawnCameraExec::FixedView]);
        assert_eq!(camera.default_move.fixed_index, 1);
        assert_eq!(step(&mut camera, 0x24), [PawnCameraExec::FixedView]);
        assert_eq!(camera.default_move.fixed_index, 1);
        camera.default_move.moving = false;
        camera.set_ue_yaw(0);
        assert_eq!(step(&mut camera, 0x23), [PawnCameraExec::TurnBack]);
        assert_eq!(camera.default_move.saved_target_yaw, 1000 + 0x8000);
        assert_eq!(press(&mut camera, &mut binds, 0x24, true), [PawnCameraExec::Ignored]);
    }

    #[test]
    fn fixed_default_camera_obeys_the_event_the_pawn_and_the_max_pressed_time() {
        let mut camera = OrbitCamera::behind(Vec3::ZERO, 0, settings());
        let release = "FixedDefaultCamera OnRelease MaxPressedTime=200.0 ";
        assert_eq!(camera.exec_pawn_camera(release, InputAction::Press, 0.0, Some((0, 0)), false), PawnCameraExec::Ignored);
        assert_eq!(camera.exec_pawn_camera(release, InputAction::Release, 250.0, Some((0, 0)), false), PawnCameraExec::Ignored);
        assert_eq!(camera.exec_pawn_camera(release, InputAction::Release, 150.0, None, false), PawnCameraExec::Ignored);
        assert_eq!(camera.exec_pawn_camera(release, InputAction::Release, 150.0, Some((0, 0)), true), PawnCameraExec::Ignored);
        assert_eq!(camera.exec_pawn_camera(release, InputAction::Release, 150.0, Some((0, 0)), false), PawnCameraExec::FixedView);
        assert_eq!(camera.exec_pawn_camera("TurnBack", InputAction::Release, 0.0, Some((0, 0)), false), PawnCameraExec::Ignored);
        assert_eq!(camera.exec_pawn_camera("TurnBack", InputAction::Press, 0.0, None, false), PawnCameraExec::Ignored);
    }

    #[test]
    fn turn_back_faces_the_pawn_or_returns_behind_and_a_yaw_drag_cancels() {
        let mut camera = OrbitCamera::behind(Vec3::ZERO, 0, settings());
        camera.set_ue_yaw(0);
        assert!(camera.turn_back(0));
        assert_eq!(camera.default_move.saved_target_yaw, 0x8000);
        camera.default_move.moving = false;
        camera.set_ue_yaw(-32768 + 10);
        camera.turn_back(0);
        assert_eq!(camera.default_move.saved_target_yaw, 0);
        camera.drag(3.0, 0.0);
        assert!(!camera.default_move.moving);
    }

    #[test]
    fn apply_shake_rotates_the_view_about_the_eye_keeping_the_distance() {
        let camera = Camera { eye: Vec3::new(10.0, 20.0, 30.0), target: Vec3::new(110.0, 20.0, 30.0), fov_x_deg: 60.0, near: 1.0, far: 100.0 };
        assert_eq!(apply_shake(camera, [0; 3]), camera);
        let shaken = apply_shake(camera, [0, 16384, 0]);
        assert_eq!(shaken.eye, camera.eye);
        assert!(((shaken.target - shaken.eye).length() - 100.0).abs() < 1e-3);
        assert!((shaken.target - Vec3::new(10.0, 120.0, 30.0)).length() < 0.05, "{:?}", shaken.target);
        let pitched = apply_shake(camera, [16384, 0, 0]);
        assert!((pitched.target - Vec3::new(10.0, 20.0, 130.0)).length() < 0.05, "{:?}", pitched.target);
    }
}

#[cfg(test)]
mod volume_camera_tests {
    use super::*;
    use std::cell::{Cell, RefCell};

    const TARGET: Vec3 = Vec3::new(1000.0, 0.0, 0.0);

    fn volume_settings(radius: i32) -> CameraSettings {
        CameraSettings { volume_camera: true, volume_radius: radius, ..CameraSettings::default() }
    }

    fn facing_x(target: Vec3, settings: CameraSettings) -> OrbitCamera {
        let mut o = OrbitCamera::behind(target, 0, settings);
        o.yaw = PI;
        o.pitch = 0.0;
        o
    }

    fn along_x(degrees: f32, distance: f32) -> impl Fn(Vec3, Vec3) -> Option<Hit> {
        move |from: Vec3, to: Vec3| ((from - to).normalize_or_zero().x > degrees.to_radians().cos()).then_some(Hit { distance, terrain: false })
    }

    #[test]
    fn the_user_ini_turns_the_volume_camera_off_with_radius_1000() {
        let ini = killer_ui::ini::parse_ini(include_str!("../../../crates/killer-ui/tests/fixtures/user.ini"));
        let s = CameraSettings::from_ini(&ini);
        assert!(!s.volume_camera);
        assert_eq!(s.volume_radius, 1000);
        let s = CameraSettings::from_ini(&killer_ui::ini::parse_ini("[Engine.LineagePlayerController]\nbUseVolumeCamera=true\nVolumeCameraRadius=500\n"));
        assert!(s.volume_camera);
        assert_eq!(s.volume_radius, 500);
    }

    #[test]
    fn rotation_to_location_with_dist_10531890_and_the_view_axis_use_the_gmath_tables() {
        assert_eq!(rotation_to_location_with_dist([0, 0, 0], TARGET, 230.0), Vec3::new(770.0, 0.0, 0.0));
        assert_eq!(view_axis([0, 0, 0]), Vec3::X);
        let half = std::f32::consts::FRAC_1_SQRT_2;
        assert!((rotation_to_location_with_dist([0, 8192, 0], Vec3::ZERO, 100.0) - Vec3::new(-100.0 * half, -100.0 * half, 0.0)).length() < 1e-3);
        assert!((view_axis([-2700, 16384, 0]) - rotator_forward([-2700, 16384, 0])).length() < 1e-5);
    }

    #[test]
    fn is_block_rotation_10536e80_probes_the_rotation_then_yaw_plus_minus_and_pitch_plus_while_the_script_skips_the_rotation() {
        let ends = RefCell::new(Vec::new());
        let record = |from: Vec3, to: Vec3| -> Option<Hit> {
            ends.borrow_mut().push((from, to));
            None
        };
        let location = Vec3::new(10.0, 20.0, 30.0);
        assert!(!is_block_rotation([100, 200, 0], location, 50.0, 1000, &record));
        let native: Vec<(Vec3, Vec3)> = [[100, 200, 0], [100, 1200, 0], [100, -800, 0], [1100, 200, 0]].into_iter().map(|r| (location, rotation_to_location_with_dist(r, location, 50.0))).collect();
        assert_eq!(*ends.borrow(), native);
        ends.borrow_mut().clear();
        assert!(!script_is_block_rotation([100, 200, 0], location, 50.0, 1000, &record));
        let script: Vec<(Vec3, Vec3)> = [[100, 1200, 0], [100, -800, 0], [1100, 200, 0], [-900, 200, 0]].into_iter().map(|r| (location, location - rotator_forward(r) * 50.0)).collect();
        assert_eq!(*ends.borrow(), script);
        let calls = Cell::new(0);
        let fourth = |_from: Vec3, _to: Vec3| -> Option<Hit> {
            calls.set(calls.get() + 1);
            (calls.get() == 4).then_some(Hit { distance: 1.0, terrain: false })
        };
        assert!(is_block_rotation([0, 0, 0], location, 50.0, 10, &fourth));
        assert_eq!(calls.get(), 4);
        calls.set(0);
        let first = |_from: Vec3, _to: Vec3| -> Option<Hit> {
            calls.set(calls.get() + 1);
            Some(Hit { distance: 1.0, terrain: false })
        };
        assert!(is_block_rotation([0, 0, 0], location, 50.0, 10, &first));
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn volume_camera_10537300_without_a_block_puts_the_eye_at_the_whole_view_distance() {
        let mut o = facing_x(TARGET, volume_settings(1000));
        let (eye, dist) = o.volume_camera(&|_: Vec3, _: Vec3| -> Option<Hit> { None });
        assert_eq!((eye, dist), (Vec3::new(770.0, 0.0, 0.0), 230.0));
        let behind = OrbitCamera::behind(TARGET, 0, volume_settings(1000));
        assert_eq!(behind.distance, 230.0);
        assert!(((behind.volume.old_eye - TARGET).length() - 230.0).abs() < 1e-2);
        assert_eq!(behind.volume.old_rotation, [-2700, 0, 0]);
    }

    #[test]
    fn volume_camera_with_the_old_eye_farther_than_the_hit_keeps_the_rotation_and_pulls_to_the_projection() {
        let mut o = facing_x(TARGET, volume_settings(1000));
        o.volume.old_eye = Vec3::ZERO;
        let (eye, dist) = o.volume_camera(&along_x(10.0, 100.0));
        assert_eq!((eye, dist), (Vec3::new(900.0, 0.0, 0.0), 100.0));
        assert_eq!((o.zoom, o.ue_yaw(), o.ue_pitch()), (ENGINE_DEFAULT_ZOOM, 0, 0));
    }

    #[test]
    fn volume_camera_bisects_toward_the_old_rotation_and_shrinks_the_radius_by_the_turn() {
        let mut o = facing_x(TARGET, volume_settings(1000));
        o.volume.old_eye = Vec3::new(770.0, 30.0, 0.0);
        o.volume.old_rotation = [0, 16384, 0];
        o.volume.radius = 20000;
        let probe = o;
        let (eye, dist) = o.volume_camera(&along_x(10.0, 100.0));
        let half = std::f32::consts::FRAC_1_SQRT_2;
        let expected = TARGET - Vec3::new(200.0 * half, 200.0 * half, 0.0);
        assert_eq!(dist, 200.0);
        assert!((eye - expected).length() < 1e-3, "{eye:?}");
        assert_eq!((o.ue_yaw(), o.ue_pitch(), o.zoom, o.volume.radius), (8192, 0, -50.0, 11808));
        let mut o = probe;
        o.collide(along_x(10.0, 100.0));
        assert_eq!(o.distance, 200.0);
        assert_eq!(o.volume.old_rotation, [0, 8192, 0]);
        assert!((o.volume.old_eye - expected).length() < 1e-3);
        assert!((o.camera().eye - expected).length() < 1e-2);
        let mut o = probe;
        o.volume.radius = 5000;
        o.volume_camera(&along_x(10.0, 100.0));
        assert_eq!(o.volume.radius, ENGINE_VOLUME_RADIUS_FLOOR);
    }

    #[test]
    fn volume_camera_blocked_twice_falls_back_to_the_old_rotation() {
        let mut o = facing_x(TARGET, volume_settings(1000));
        o.volume.old_eye = Vec3::new(770.0, 30.0, 0.0);
        o.volume.old_rotation = [0, 1024, 0];
        o.volume.radius = 10;
        let (eye, dist) = o.volume_camera(&along_x(10.0, 100.0));
        assert_eq!(dist, 200.0);
        assert!((eye - rotation_to_location_with_dist([0, 1024, 0], TARGET, 200.0)).length() < 1e-3);
        assert_eq!((o.ue_yaw(), o.zoom, o.volume.radius), (1024, -50.0, 10));
    }

    #[test]
    fn calc_volume_zooming_distance_eases_toward_desired_and_undoes_the_step_when_blocked() {
        let fresh = || {
            let mut o = facing_x(Vec3::ZERO, volume_settings(1000));
            o.volume.old_rotation = [0, 0, 0];
            o.desired_zoom = 100.0;
            o
        };
        let mut o = fresh();
        o.volume_zooming(0.1, &along_x(3.0, 150.0));
        assert_eq!((o.zoom, o.desired_zoom, o.volume.radius), (-100.0, 100.0, 1000));
        let mut o = fresh();
        o.volume_zooming(0.1, &|_: Vec3, _: Vec3| -> Option<Hit> { Some(Hit { distance: 1.0, terrain: false }) });
        assert_eq!((o.zoom, o.volume.radius), (ENGINE_DEFAULT_ZOOM, 0));
        let mut o = fresh();
        o.volume_zooming(0.1, &|_: Vec3, _: Vec3| -> Option<Hit> { None });
        assert_eq!((o.zoom, o.volume.radius), (80.0, 1000));
        o.desired_zoom = o.zoom;
        o.volume_zooming(0.1, &|_: Vec3, _: Vec3| -> Option<Hit> { Some(Hit { distance: 1.0, terrain: false }) });
        assert_eq!(o.zoom, 80.0);
    }

    #[test]
    fn player_tick_schedules_the_volume_zoom_only_outside_the_default_move_and_collide_runs_it() {
        let mut o = facing_x(Vec3::ZERO, volume_settings(1000));
        o.desired_zoom = 100.0;
        o.player_tick(0.1);
        assert_eq!(o.volume.zoom_tick, Some(0.1));
        o.collide(|_, _| None);
        assert_eq!(o.volume.zoom_tick, None);
        assert_eq!((o.zoom, o.desired_zoom, o.distance), (80.0, 100.0, 330.0));
        o.default_move.moving = true;
        o.player_tick(0.1);
        assert_eq!(o.volume.zoom_tick, None);
        let mut plain = OrbitCamera::behind(Vec3::ZERO, 0, CameraSettings::default());
        plain.player_tick(0.1);
        assert_eq!(plain.volume.zoom_tick, None);
    }

    #[test]
    fn player_calc_view_copies_the_desired_zoom_only_without_the_volume_camera() {
        let mut o = OrbitCamera::behind(Vec3::ZERO, 0, CameraSettings::default());
        o.zoom = 300.0;
        o.collide(|_, _| None);
        assert_eq!((o.zoom, o.distance), (ENGINE_DEFAULT_ZOOM, 200.0));
        let mut v = OrbitCamera::behind(Vec3::ZERO, 0, volume_settings(1000));
        v.zoom = 300.0;
        v.collide(|_, _| None);
        assert_eq!((v.zoom, v.desired_zoom, v.distance), (300.0, ENGINE_DEFAULT_ZOOM, 550.0));
    }

    #[test]
    fn zoom_hold_1057ab4c_moves_both_zooms_by_speed_times_delta_within_minus_200_and_1000() {
        let mut o = OrbitCamera::behind(Vec3::ZERO, 0, CameraSettings::from_ini(&killer_ui::ini::parse_ini(include_str!("../../../crates/killer-ui/tests/fixtures/user.ini"))));
        assert!(o.exec_input("ZoomInHold Speed=1000.0", InputAction::Hold, 0.1));
        assert_eq!((o.zoom, o.desired_zoom), (-120.0, -120.0));
        assert!(o.exec_input(" zoominhold speed=1000", InputAction::Hold, 1.0));
        assert_eq!((o.zoom, o.desired_zoom, o.lift), (-200.0, -200.0, 0.0));
        assert!(!o.exec_input("ZoomInHold Speed=1000.0", InputAction::Press, 1.0));
        assert!(!o.exec_input("ZoomInHold", InputAction::Release, 1.0));
        assert!(o.exec_input("ZoomOutHold", InputAction::Hold, 0.5));
        assert_eq!(o.zoom, -199.5);
        assert!(o.exec_input("ZoomOutHold Speed=5000", InputAction::Hold, 1.0));
        assert_eq!((o.zoom, o.desired_zoom), (ENGINE_HOLD_MAX_ZOOM, ENGINE_HOLD_MAX_ZOOM));
        assert!(!o.exec_input("ZoomInHolding", InputAction::Hold, 1.0));
        assert!(!o.exec_input("Arrow On", InputAction::Hold, 1.0));
        o.collide(|_, _| None);
        assert_eq!(o.distance, 1220.0);
        o.default_move.disable_zoom = true;
        assert!(o.exec_input("ZoomInHold Speed=10", InputAction::Hold, 1.0));
        assert_eq!(o.zoom, ENGINE_HOLD_MAX_ZOOM);
        o.default_move.disable_zoom = false;
        o.special.special_move = true;
        assert!(o.exec_input("ZoomInHold Speed=10", InputAction::Hold, 1.0));
        assert_eq!(o.zoom, ENGINE_HOLD_MAX_ZOOM);
        o.special.special_move = false;
        assert!(o.exec_input("ZoomInHold Speed=10", InputAction::Hold, 1.0));
        assert_eq!(o.zoom, 990.0);
    }

    #[test]
    fn the_wheel_moves_the_desired_zoom_with_the_current_one() {
        let mut o = OrbitCamera::behind(Vec3::ZERO, 0, CameraSettings { wheel_up: 20.0, wheel_down: -20.0, ..CameraSettings::default() });
        o.wheel(1.0, None);
        assert_eq!((o.zoom, o.desired_zoom), (0.0, 0.0));
        for _ in 0..20 {
            o.wheel(-1.0, None);
        }
        assert_eq!((o.zoom, o.desired_zoom), (-200.0, -200.0));
    }
}
