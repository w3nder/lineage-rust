use glam::Vec3;
use l2_net::game::SpecialCamera;

const UNITS_PER_TURN: f32 = 65536.0;
const ORBIT_MARGIN: f32 = 220.0;
const NEAR_SPEED_DISTANCE: f32 = 40.0;
const NEAR_SPEED: f32 = 100.0;
const DISTANCE_RATE: f32 = 2.5;
const ANGLE_STEP_RANGE: i32 = 2000;
const ANGLE_STEP_RATE: f32 = 5000.0;
const ANGLE_EASE_RATE: f32 = 2.5;
const INSTANT_DISTANCE_SPEED: f32 = 1_000_000.0;
const INSTANT_ANGLE_SPEED: i32 = 0x320000;
const MILLISECOND: f32 = 0.001;

pub fn wrap16(mut value: i32) -> i32 {
    while value > 32768 {
        value -= 65536;
    }
    while value < -32767 {
        value += 65536;
    }
    value
}

fn units(degrees: i32) -> i32 {
    degrees.wrapping_shl(16) / 360
}

fn angle(units: i32) -> f32 {
    units as f32 * std::f32::consts::TAU / UNITS_PER_TURN
}

pub fn rotator_vector(pitch: i32, yaw: i32) -> Vec3 {
    let (sp, cp) = angle(pitch).sin_cos();
    let (sy, cy) = angle(yaw).sin_cos();
    Vec3::new(cp * cy, cp * sy, sp)
}

pub fn vector_rotator(v: Vec3) -> (i32, i32) {
    let v = v.as_dvec3();
    let units = |radians: f64| (radians * 10430.219195527361) as i32;
    (units(v.z.atan2((v.x * v.x + v.y * v.y).sqrt())), units(v.y.atan2(v.x)))
}

fn ease_step(delta_time: f32, diff: i32) -> i32 {
    let step = if (-ANGLE_STEP_RANGE..0).contains(&diff) {
        delta_time * -ANGLE_STEP_RATE
    } else if diff > 0 && diff <= ANGLE_STEP_RANGE {
        delta_time * ANGLE_STEP_RATE
    } else {
        diff as f32 * delta_time * ANGLE_EASE_RATE
    } as i32;
    if diff < 0 { step.max(diff) } else { step.min(diff) }
}

fn speed_step(delta_time: f32, speed: i32, diff: i32) -> i32 {
    let step = (speed as f32 * delta_time) as i32;
    if diff < 0 { (-step).max(diff) } else { step.min(diff) }
}

fn approach_angle(current: i32, target: i32, step: impl Fn(i32) -> i32) -> i32 {
    let diff = wrap16(target - current);
    if diff == 0 { current } else { wrap16(current + step(diff)) }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Request {
    pub dist: f32,
    pub yaw: i32,
    pub pitch: i32,
    pub time: f32,
    pub duration: f32,
    pub turn: i32,
    pub rise: i32,
    pub wide: bool,
    pub flag: bool,
}

impl Request {
    pub fn from_packet(p: &SpecialCamera) -> Self {
        Self {
            dist: p.distance as f32,
            yaw: units(-p.yaw),
            pitch: units(-p.pitch),
            time: p.time as f32 * MILLISECOND,
            duration: p.duration as f32 * MILLISECOND,
            turn: units(p.turn),
            rise: units(p.rise),
            wide: p.widescreen > 0,
            flag: p.unknown > 0,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rest {
    pub yaw: i32,
    pub pitch: i32,
    pub dist: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SpecialCameraState {
    pub special_move: bool,
    pub moving_to_special: bool,
    pub view_target: Option<i32>,
    pub wide: bool,
    pub yaw: i32,
    pub pitch: i32,
    pub dist: f32,
    pub dist_speed: f32,
    pub yaw_speed: i32,
    pub pitch_speed: i32,
    pub duration: f32,
    pub cur_yaw: i32,
    pub cur_pitch: i32,
    pub cur_dist: f32,
    pub rel_yaw: i32,
    pub rel_pitch: i32,
    pub turn_speed: i32,
    pub rise_speed: i32,
    pub turn: i32,
    pub rise: i32,
    pub location: Vec3,
    pub rotation: [i32; 3],
}

impl SpecialCameraState {
    pub fn active(&self) -> bool {
        self.special_move || self.moving_to_special
    }

    pub fn eye(&self) -> Vec3 {
        self.location
    }

    pub fn look_at(&self) -> Vec3 {
        self.location + rotator_vector(self.rotation[0], self.rotation[1])
    }

    pub fn request(&mut self, target: i32, req: &Request) {
        let seconds = req.time;
        let (dist_speed, angle_speed) = if seconds == 0.0 {
            (INSTANT_DISTANCE_SPEED, None)
        } else {
            (0.0, Some(seconds))
        };
        let speed = |diff: i32| match angle_speed {
            Some(time) => (wrap16(diff).abs() as f32 / time) as i32,
            None => INSTANT_ANGLE_SPEED,
        };
        if self.special_move && self.view_target == Some(target) {
            self.dist_speed = match angle_speed {
                Some(time) => (self.cur_dist - req.dist).abs() / time,
                None => dist_speed,
            };
            self.yaw_speed = speed(req.yaw - self.cur_yaw);
            self.pitch_speed = speed(req.pitch - self.cur_pitch);
            self.turn_speed = speed(req.turn - self.turn);
            self.rise_speed = speed(req.rise - self.rise);
            self.moving_to_special = false;
        } else {
            self.turn = 0;
            self.rise = 0;
            self.cur_dist = req.dist;
            self.cur_pitch = req.pitch;
            self.cur_yaw = req.yaw;
            self.turn_speed = speed(req.turn);
            self.rise_speed = speed(req.rise);
        }
        self.special_move = true;
        if self.view_target != Some(target) {
            self.view_target = Some(target);
            self.moving_to_special = true;
        }
        self.wide = req.wide;
        self.dist = req.dist;
        self.duration = req.duration;
        self.pitch = req.pitch;
        self.yaw = req.yaw;
        self.rel_yaw = req.turn;
        self.rel_pitch = req.rise;
    }

    pub fn normal_camera(&mut self, rest: Rest) -> bool {
        if !self.special_move {
            return false;
        }
        self.special_move = false;
        self.view_target = None;
        self.moving_to_special = true;
        self.yaw = rest.yaw;
        self.pitch = rest.pitch;
        self.dist = rest.dist;
        self.rel_pitch = 0;
        self.rel_yaw = 0;
        self.wide = false;
        true
    }

    pub fn tick(&mut self, delta_time: f32, target: Option<Vec3>, rest: Rest) -> bool {
        let Some(target) = target else { return false };
        self.rotation[1] = wrap16(self.rotation[1] - self.turn);
        self.rotation[0] = wrap16(self.rotation[0] - self.rise);
        if self.moving_to_special {
            if self.approach_target(delta_time, target) {
                return false;
            }
        } else if self.special_move {
            self.follow_target(delta_time, target);
        }
        self.rotation[1] = wrap16(self.rotation[1] + self.turn);
        self.rotation[0] = wrap16(self.rotation[0] + self.rise);
        if self.special_move {
            self.duration -= delta_time;
            if self.duration <= 0.0 {
                self.normal_camera(rest);
                self.duration = 0.0;
                return true;
            }
        }
        false
    }

    fn approach_target(&mut self, delta_time: f32, target: Vec3) -> bool {
        let desired = target - rotator_vector(self.pitch, self.yaw) * (self.dist + ORBIT_MARGIN);
        let remaining = (desired - self.location).length();
        if remaining > 0.0 {
            let step = if remaining > NEAR_SPEED_DISTANCE { delta_time * remaining * DISTANCE_RATE } else { delta_time * NEAR_SPEED };
            self.location = if remaining <= step { desired } else { self.location + (desired - self.location) / remaining * step };
        }
        let (look_pitch, look_yaw) = vector_rotator(target - self.location);
        let yaw = approach_angle(self.rotation[1], look_yaw, |diff| ease_step(delta_time, diff));
        let pitch = approach_angle(self.rotation[0], look_pitch, |diff| ease_step(delta_time, diff));
        let reached = yaw == self.rotation[1] && pitch == self.rotation[0];
        self.rotation[1] = yaw;
        self.rotation[0] = pitch;
        self.turn = approach_angle(self.turn, self.rel_yaw, |diff| ease_step(delta_time, diff));
        self.rise = approach_angle(self.rise, self.rel_pitch, |diff| ease_step(delta_time, diff));
        if remaining == 0.0 && reached {
            self.moving_to_special = false;
            self.rotation[1] = wrap16(self.rotation[1] + self.turn);
            self.rotation[0] = wrap16(self.rotation[0] + self.rise);
            return true;
        }
        false
    }

    fn follow_target(&mut self, delta_time: f32, target: Vec3) {
        let diff = self.dist - self.cur_dist;
        let step = self.dist_speed * delta_time;
        self.cur_dist = if diff.abs() <= step { self.dist } else { self.cur_dist + step.copysign(diff) };
        let (yaw_speed, pitch_speed) = (self.yaw_speed, self.pitch_speed);
        self.cur_yaw = approach_angle(self.cur_yaw, self.yaw, |diff| speed_step(delta_time, yaw_speed, diff));
        self.cur_pitch = approach_angle(self.cur_pitch, self.pitch, |diff| speed_step(delta_time, pitch_speed, diff));
        self.location = target - rotator_vector(self.cur_pitch, self.cur_yaw) * (self.cur_dist + ORBIT_MARGIN);
        let (look_pitch, look_yaw) = vector_rotator(target - self.location);
        self.rotation = [look_pitch, look_yaw, 0];
        let (turn_speed, rise_speed) = (self.turn_speed, self.rise_speed);
        self.turn = approach_angle(self.turn, self.rel_yaw, |diff| speed_step(delta_time, turn_speed, diff));
        self.rise = approach_angle(self.rise, self.rel_pitch, |diff| speed_step(delta_time, rise_speed, diff));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet() -> SpecialCamera {
        SpecialCamera { target_id: 7, distance: 300, yaw: 90, pitch: 10, time: 2000, duration: 5000, turn: 45, rise: -90, widescreen: 1, unknown: 0 }
    }

    #[test]
    fn handler_conversions_follow_the_engine() {
        let r = Request::from_packet(&packet());
        assert_eq!(r.yaw, -16384);
        assert_eq!(r.pitch, -1820);
        assert_eq!(r.turn, 8192);
        assert_eq!(r.rise, -16384);
        assert_eq!(r.time, 2.0);
        assert_eq!(r.duration, 5.0);
        assert!(r.wide && !r.flag);
        assert_eq!(r.dist, 300.0);
    }

    #[test]
    fn new_request_sets_current_values_and_speeds() {
        let mut s = SpecialCameraState::default();
        s.request(7, &Request::from_packet(&packet()));
        assert!(s.special_move && s.moving_to_special && s.active());
        assert_eq!((s.cur_yaw, s.cur_pitch, s.cur_dist), (-16384, -1820, 300.0));
        assert_eq!((s.turn_speed, s.rise_speed), (4096, 8192));
        assert_eq!(s.duration, 5.0);
        assert!(s.wide);
    }

    #[test]
    fn zero_time_uses_the_instant_speeds() {
        let mut s = SpecialCameraState::default();
        let mut p = packet();
        p.time = 0;
        s.request(7, &Request::from_packet(&p));
        assert_eq!((s.turn_speed, s.rise_speed), (0x320000, 0x320000));
        s.moving_to_special = false;
        s.request(7, &Request::from_packet(&p));
        assert_eq!((s.yaw_speed, s.pitch_speed), (0x320000, 0x320000));
        assert_eq!(s.dist_speed, 1_000_000.0);
    }

    #[test]
    fn same_target_request_blends_from_the_current_values() {
        let mut s = SpecialCameraState::default();
        s.request(7, &Request::from_packet(&packet()));
        s.moving_to_special = false;
        s.cur_dist = 100.0;
        let mut p = packet();
        p.distance = 400;
        p.yaw = 0;
        p.pitch = 0;
        s.request(7, &Request::from_packet(&p));
        assert!(!s.moving_to_special);
        assert_eq!(s.dist_speed, 150.0);
        assert_eq!(s.yaw_speed, 8192);
        assert_eq!(s.pitch_speed, 910);
    }

    #[test]
    fn ease_step_matches_the_engine_branches() {
        assert_eq!(ease_step(0.01, 10), 10);
        assert_eq!(ease_step(0.01, 5000), 125);
        assert_eq!(ease_step(0.1, 1000), 500);
        assert_eq!(ease_step(0.1, -1000), -500);
        assert_eq!(ease_step(0.0, 100), 0);
        assert_eq!(ease_step(1.0, -3), -3);
    }

    #[test]
    fn camera_flies_to_the_target_then_orbits_it_until_the_duration_ends() {
        let mut s = SpecialCameraState { location: Vec3::new(1000.0, 0.0, 0.0), ..Default::default() };
        s.request(7, &Request::from_packet(&packet()));
        let target = Vec3::new(0.0, 0.0, 100.0);
        let rest = Rest::default();
        for _ in 0..600 {
            s.tick(0.016, Some(target), rest);
            if !s.moving_to_special {
                break;
            }
        }
        assert!(!s.moving_to_special);
        let expected = target - rotator_vector(s.pitch, s.yaw) * (s.dist + ORBIT_MARGIN);
        assert!((s.location - expected).length() < 1e-3);
        let mut finished = false;
        for _ in 0..700 {
            finished = s.tick(0.016, Some(target), rest);
            if finished {
                break;
            }
        }
        assert!(finished);
        assert!(!s.special_move);
        assert_eq!(s.duration, 0.0);
    }

    #[test]
    fn normal_camera_is_idempotent_and_returns_to_the_rest_view() {
        let mut s = SpecialCameraState::default();
        assert!(!s.normal_camera(Rest::default()));
        s.request(7, &Request::from_packet(&packet()));
        let rest = Rest { yaw: 100, pitch: -200, dist: 350.0 };
        assert!(s.normal_camera(rest));
        assert_eq!((s.yaw, s.pitch, s.dist, s.rel_yaw, s.rel_pitch), (100, -200, 350.0, 0, 0));
        assert!(s.moving_to_special && !s.special_move && !s.wide);
        assert!(!s.normal_camera(rest));
    }
}
