use glam::Vec3;
use killer_assets::engine_math::rotator_forward;

pub const VIEWPORT_INDEX: i32 = 0;
pub const VIEWPORT_SIZE: (i32, i32) = (256, 256);
pub const TRACE_HALF_HEIGHT: f32 = 50.0;
pub const CAMERA_DISTANCE: f32 = 200.0;
pub const CAMERA_YAW_TURN: i32 = 0x8000;
pub const FALLBACK_PITCH: i32 = -5400;
pub const FALLBACK_YAW_MODULO: i32 = 0xffff;
pub const FALLBACK_LIFT: f32 = 10.0;
pub const FALLBACK_DISTANCE: f32 = 100.0;
pub const FOV_BASE_DEGREES: f64 = 60.0;
pub const FOV_HALF_WIDTH: f64 = 126.0;
pub const FLOAT_MESH: &str = "LineageDecos.float_m00";
pub const FLOAT_CLASS: &str = "AL2Float";
pub const LINE_CLASS: &str = "ANLine";
pub const LINE_END_PARAMETER: i32 = 30;
pub const ROD_WEAPON_SLOT: i32 = 8;
pub const ROD_ANIMATION: &str = "Fishing_wait";
pub const FLOAT_IDLE_ANIMATION_RATE: f32 = 1.0;
pub const FLOAT_ANIMATION_BLEND: f32 = 0.1;
pub const CALC_STEP_CAST: i32 = 1;
pub const CALC_STEP_COMBAT: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ViewportParam {
    pub index: i32,
    pub size: (i32, i32),
    pub location: Vec3,
    pub rotation: [i32; 3],
    pub fov: f32,
}

pub fn fov_degrees(screen_x: i32) -> f32 {
    let half = f64::from(screen_x / 2);
    let tangent = (FOV_BASE_DEGREES.to_radians() / 2.0).tan();
    (2.0 * (tangent * FOV_HALF_WIDTH / half).atan() * 180.0 / std::f64::consts::PI) as f32
}

pub fn camera_with_controller(pawn_location: Vec3, pawn_rotation: [i32; 3]) -> (Vec3, [i32; 3]) {
    let location = pawn_location + rotator_forward(pawn_rotation) * CAMERA_DISTANCE;
    let rotation = [pawn_rotation[0], pawn_rotation[1].wrapping_add(CAMERA_YAW_TURN), pawn_rotation[2]];
    (location, rotation)
}

pub fn camera_without_controller(pawn_location: Vec3, pawn_rotation: [i32; 3]) -> (Vec3, [i32; 3]) {
    let rotation = [FALLBACK_PITCH, (pawn_rotation[1].wrapping_add(CAMERA_YAW_TURN)) % FALLBACK_YAW_MODULO, pawn_rotation[2]];
    let origin = Vec3::new(pawn_location.x, pawn_location.y, pawn_location.z - FALLBACK_LIFT);
    (origin - rotator_forward(rotation) * FALLBACK_DISTANCE, rotation)
}

pub fn viewport_param(pawn_location: Vec3, pawn_rotation: [i32; 3], screen_x: i32, controller: bool) -> ViewportParam {
    let (location, rotation) = if controller { camera_with_controller(pawn_location, pawn_rotation) } else { camera_without_controller(pawn_location, pawn_rotation) };
    ViewportParam { index: VIEWPORT_INDEX, size: VIEWPORT_SIZE, location, rotation, fov: fov_degrees(screen_x) }
}

pub fn trace_ends(packet: Vec3) -> (Vec3, Vec3) {
    (Vec3::new(packet.x, packet.y, packet.z + TRACE_HALF_HEIGHT), Vec3::new(packet.x, packet.y, packet.z - TRACE_HALF_HEIGHT))
}

pub fn float_location(packet: Vec3, hit: Option<Vec3>) -> Vec3 {
    match hit {
        Some(point) if point.z != 0.0 => point,
        _ => packet,
    }
}

pub fn line_middle(float: Vec3, pawn: Vec3) -> Vec3 {
    float + (pawn - float) * 0.5
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CastState {
    Waiting,
    FloatSpawned,
    Done,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FishStartAction {
    pub location: Vec3,
    pub kind: i32,
    pub state: CastState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CastStep {
    Continue,
    SpawnFloat { location: Vec3, kind: i32, base: Vec3 },
    RodAnimation,
    SpawnLine { middle: Vec3, parameter: i32 },
    Leave,
}

impl FishStartAction {
    pub fn new(location: Vec3, kind: i32) -> FishStartAction {
        FishStartAction { location, kind, state: CastState::Waiting }
    }

    pub fn execute(&mut self, fishing: bool, throwing: bool, has_float: bool, has_rod: bool, float: Vec3, pawn: Vec3) -> Vec<CastStep> {
        if !fishing {
            self.state = CastState::Done;
            return vec![CastStep::Leave];
        }
        match self.state {
            CastState::Waiting => {
                if throwing {
                    return vec![CastStep::Continue];
                }
                self.state = CastState::FloatSpawned;
                vec![CastStep::SpawnFloat { location: self.location, kind: self.kind, base: self.location }, CastStep::RodAnimation, CastStep::Continue]
            }
            CastState::FloatSpawned => {
                self.state = CastState::Done;
                if !has_float || !has_rod {
                    return vec![CastStep::Leave];
                }
                vec![CastStep::SpawnLine { middle: line_middle(float, pawn), parameter: LINE_END_PARAMETER }, CastStep::Leave]
            }
            CastState::Done => vec![CastStep::Leave],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FishingView {
    pub active: bool,
    pub pawn_location: Vec3,
    pub pawn_rotation: [i32; 3],
    pub param: Option<ViewportParam>,
    pub float: Option<Vec3>,
    pub line_middle: Option<Vec3>,
    pub calc_step: i32,
    pub kind: i32,
    pub gut: bool,
    pub pull: Option<u8>,
}

impl FishingView {
    pub fn cast(&mut self, pawn_location: Vec3, pawn_rotation: [i32; 3], float: Vec3, kind: i32) {
        self.active = true;
        self.pawn_location = pawn_location;
        self.pawn_rotation = pawn_rotation;
        self.param = None;
        self.float = Some(float);
        self.line_middle = Some(line_middle(float, pawn_location));
        self.calc_step = CALC_STEP_CAST;
        self.kind = kind;
        self.gut = false;
        self.pull = None;
    }

    pub fn refresh(&mut self, screen_x: i32) {
        if self.active && screen_x > 0 {
            self.param = Some(viewport_param(self.pawn_location, self.pawn_rotation, screen_x, true));
        }
    }

    pub fn combat(&mut self, mode: u8) {
        if self.active {
            self.calc_step = CALC_STEP_COMBAT;
            self.gut = mode & 1 != 0;
        }
    }

    pub fn regen(&mut self, mode: u8, action: u8) {
        if self.active {
            self.gut = mode & 1 != 0;
            if action != 0 {
                self.pull = Some(action);
            }
        }
    }

    pub fn end(&mut self) {
        *self = FishingView::default();
    }

    pub fn update_pawn(&mut self, pawn_location: Vec3) {
        self.pawn_location = pawn_location;
        if let Some(float) = self.float {
            self.line_middle = Some(line_middle(float, pawn_location));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fov_matches_the_engine_formula_for_common_widths() {
        for (width, expected) in [(1024, 16.173179835071874f64), (800, 20.614923320558443), (1280, 12.969481404880465)] {
            let value = f64::from(fov_degrees(width));
            assert!((value - expected).abs() < 1.0e-4, "{width}: {value}");
        }
        let exact = 2.0 * ((std::f64::consts::PI / 6.0).tan() * 126.0 / 512.0).atan() * 180.0 / std::f64::consts::PI;
        assert_eq!(fov_degrees(1024), exact as f32);
        assert_eq!(fov_degrees(1025), fov_degrees(1024));
    }

    #[test]
    fn camera_stands_in_front_of_the_pawn_looking_back() {
        let (location, rotation) = camera_with_controller(Vec3::new(100.0, 200.0, 30.0), [0, 0, 0]);
        assert!((location - Vec3::new(300.0, 200.0, 30.0)).length() < 1.0e-3);
        assert_eq!(rotation, [0, 0x8000, 0]);
        let (location, rotation) = camera_with_controller(Vec3::ZERO, [0, 0x4000, 0]);
        assert!((location - Vec3::new(0.0, 200.0, 0.0)).length() < 1.0e-3);
        assert_eq!(rotation[1], 0xc000);
    }

    #[test]
    fn fallback_camera_looks_down_with_the_pitch_5400() {
        let (location, rotation) = camera_without_controller(Vec3::new(0.0, 0.0, 100.0), [0, 0, 7]);
        assert_eq!(rotation, [-5400, 0x8000, 7]);
        assert!(location.z > 90.0);
    }

    #[test]
    fn viewport_param_is_256_square_at_index_zero() {
        let param = viewport_param(Vec3::new(1.0, 2.0, 3.0), [0, 0x2000, 0], 1024, true);
        assert_eq!((param.index, param.size), (0, (256, 256)));
        assert_eq!(param.rotation[1], 0xa000);
    }

    #[test]
    fn the_trace_and_float_location_follow_the_water_hit() {
        let packet = Vec3::new(10.0, 20.0, 30.0);
        let (top, bottom) = trace_ends(packet);
        assert_eq!((top.z, bottom.z), (80.0, -20.0));
        assert_eq!(float_location(packet, Some(Vec3::new(10.0, 20.0, 25.0))), Vec3::new(10.0, 20.0, 25.0));
        assert_eq!(float_location(packet, Some(Vec3::new(1.0, 2.0, 0.0))), packet);
        assert_eq!(float_location(packet, None), packet);
    }

    #[test]
    fn fish_start_action_waits_for_the_cast_animation_then_spawns_float_and_line() {
        let mut action = FishStartAction::new(Vec3::new(1.0, 2.0, 3.0), 5);
        assert_eq!(action.execute(false, false, false, false, Vec3::ZERO, Vec3::ZERO), vec![CastStep::Leave]);
        let mut action = FishStartAction::new(Vec3::new(1.0, 2.0, 3.0), 5);
        assert_eq!(action.execute(true, true, false, false, Vec3::ZERO, Vec3::ZERO), vec![CastStep::Continue]);
        let steps = action.execute(true, false, false, false, Vec3::ZERO, Vec3::ZERO);
        assert_eq!(steps, vec![CastStep::SpawnFloat { location: Vec3::new(1.0, 2.0, 3.0), kind: 5, base: Vec3::new(1.0, 2.0, 3.0) }, CastStep::RodAnimation, CastStep::Continue]);
        assert_eq!(action.execute(true, false, false, true, Vec3::ZERO, Vec3::ZERO), vec![CastStep::Leave]);
        let mut action = FishStartAction::new(Vec3::ZERO, 0);
        action.execute(true, false, false, false, Vec3::ZERO, Vec3::ZERO);
        let steps = action.execute(true, false, true, true, Vec3::new(0.0, 0.0, 0.0), Vec3::new(10.0, 20.0, 30.0));
        assert_eq!(steps, vec![CastStep::SpawnLine { middle: Vec3::new(5.0, 10.0, 15.0), parameter: 30 }, CastStep::Leave]);
    }

    #[test]
    fn view_tracks_cast_combat_and_end() {
        let mut view = FishingView::default();
        view.cast(Vec3::new(0.0, 0.0, 0.0), [0, 0, 0], Vec3::new(100.0, 0.0, 0.0), 2);
        assert_eq!(view.calc_step, CALC_STEP_CAST);
        assert_eq!(view.param, None);
        view.refresh(0);
        assert_eq!(view.param, None);
        view.refresh(1024);
        assert_eq!(view.param.map(|param| param.size), Some((256, 256)));
        assert_eq!(view.line_middle, Some(Vec3::new(50.0, 0.0, 0.0)));
        view.update_pawn(Vec3::new(0.0, 100.0, 0.0));
        assert_eq!(view.line_middle, Some(Vec3::new(50.0, 50.0, 0.0)));
        assert_eq!(view.kind, 2);
        view.combat(1);
        assert_eq!(view.calc_step, CALC_STEP_COMBAT);
        assert!(view.gut);
        view.regen(0, 2);
        assert!(!view.gut);
        assert_eq!(view.pull, Some(2));
        view.regen(1, 0);
        assert_eq!(view.pull, Some(2));
        view.end();
        assert_eq!(view, FishingView::default());
        view.combat(1);
        assert_eq!(view.calc_step, 0);
    }
}
