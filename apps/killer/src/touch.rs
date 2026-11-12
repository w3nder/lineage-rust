use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use winit::dpi::PhysicalPosition;
use winit::event::{DeviceId, ElementState, MouseButton, MouseScrollDelta, Touch, TouchPhase, WindowEvent};

const PINCH_PIXELS_PER_TICK: f64 = 48.0;
const SLOP_POINTS: f64 = 10.0;
const LONG_PRESS: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum Single {
    #[default]
    Idle,
    Pending,
    Dragging,
    Spent,
}

#[derive(Debug, Default)]
pub struct Touches {
    fingers: BTreeMap<u64, PhysicalPosition<f64>>,
    primary: Option<u64>,
    single: Single,
    origin: PhysicalPosition<f64>,
    down_at: Option<Instant>,
    device: Option<DeviceId>,
    camera: bool,
    pinch: Option<f64>,
    scale: f64,
    direct: bool,
    direct_finger: Option<u64>,
}

impl Touches {
    pub fn set_direct(&mut self, direct: bool) {
        self.direct = direct;
    }

    fn handle_direct(&mut self, touch: Touch) -> Vec<WindowEvent> {
        let device_id = touch.device_id;
        self.device = Some(device_id);
        let mut out = Vec::new();
        match touch.phase {
            TouchPhase::Started if self.direct_finger.is_none() => {
                self.direct_finger = Some(touch.id);
                out.push(moved(device_id, touch.location));
                out.push(button(device_id, ElementState::Pressed, MouseButton::Left));
            }
            TouchPhase::Moved if self.direct_finger == Some(touch.id) => out.push(moved(device_id, touch.location)),
            TouchPhase::Ended | TouchPhase::Cancelled if self.direct_finger == Some(touch.id) => {
                self.direct_finger = None;
                out.push(moved(device_id, touch.location));
                out.push(button(device_id, ElementState::Released, MouseButton::Left));
            }
            _ => {}
        }
        out
    }

    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    fn slop(&self) -> f64 {
        SLOP_POINTS * self.scale.max(1.0)
    }

    pub fn handle(&mut self, touch: Touch, now: Instant) -> Vec<WindowEvent> {
        if self.direct || self.direct_finger.is_some() {
            return self.handle_direct(touch);
        }
        let device_id = touch.device_id;
        self.device = Some(device_id);
        let mut out = Vec::new();
        match touch.phase {
            TouchPhase::Started => {
                self.fingers.insert(touch.id, touch.location);
                match self.fingers.len() {
                    1 => {
                        self.primary = Some(touch.id);
                        self.single = Single::Pending;
                        self.origin = touch.location;
                        self.down_at = Some(now);
                        out.push(moved(device_id, touch.location));
                    }
                    2 => {
                        if self.single == Single::Dragging {
                            out.push(button(device_id, ElementState::Released, MouseButton::Left));
                        }
                        self.single = Single::Idle;
                        self.primary = None;
                        self.down_at = None;
                        out.push(moved(device_id, self.center()));
                        out.push(button(device_id, ElementState::Pressed, MouseButton::Right));
                        self.camera = true;
                        self.pinch = self.spread();
                    }
                    _ => {}
                }
            }
            TouchPhase::Moved => {
                if !self.fingers.contains_key(&touch.id) {
                    return out;
                }
                self.fingers.insert(touch.id, touch.location);
                if self.camera {
                    out.push(moved(device_id, self.center()));
                    if let (Some(before), Some(now)) = (self.pinch, self.spread()) {
                        let ticks = ((now - before) / PINCH_PIXELS_PER_TICK) as f32;
                        if ticks != 0.0 {
                            out.push(WindowEvent::MouseWheel { device_id, delta: MouseScrollDelta::LineDelta(0.0, ticks), phase: TouchPhase::Moved });
                            self.pinch = Some(now);
                        }
                    }
                } else if self.primary == Some(touch.id) {
                    if self.single == Single::Pending && distance(self.origin, touch.location) > self.slop() {
                        out.push(moved(device_id, self.origin));
                        out.push(button(device_id, ElementState::Pressed, MouseButton::Left));
                        self.single = Single::Dragging;
                        self.down_at = None;
                    }
                    if self.single == Single::Dragging {
                        out.push(moved(device_id, touch.location));
                    }
                }
            }
            TouchPhase::Ended | TouchPhase::Cancelled => {
                if self.fingers.remove(&touch.id).is_none() {
                    return out;
                }
                if self.camera {
                    out.push(moved(device_id, touch.location));
                    out.push(button(device_id, ElementState::Released, MouseButton::Right));
                    self.camera = false;
                    self.pinch = None;
                    self.primary = None;
                } else if self.primary == Some(touch.id) {
                    let cancelled = touch.phase == TouchPhase::Cancelled;
                    match self.single {
                        Single::Pending if !cancelled => {
                            out.push(moved(device_id, self.origin));
                            out.push(button(device_id, ElementState::Pressed, MouseButton::Left));
                            out.push(button(device_id, ElementState::Released, MouseButton::Left));
                        }
                        Single::Dragging => {
                            out.push(moved(device_id, touch.location));
                            out.push(button(device_id, ElementState::Released, MouseButton::Left));
                        }
                        _ => {}
                    }
                    self.single = Single::Idle;
                    self.primary = None;
                    self.down_at = None;
                }
            }
        }
        out
    }

    pub fn tick(&mut self, now: Instant) -> Vec<WindowEvent> {
        let (Some(device_id), Some(down_at)) = (self.device, self.down_at) else { return Vec::new(); };
        if self.single != Single::Pending || now.duration_since(down_at) < LONG_PRESS {
            return Vec::new();
        }
        self.single = Single::Spent;
        self.down_at = None;
        vec![
            moved(device_id, self.origin),
            button(device_id, ElementState::Pressed, MouseButton::Right),
            button(device_id, ElementState::Released, MouseButton::Right),
        ]
    }

    fn center(&self) -> PhysicalPosition<f64> {
        let count = self.fingers.len().max(1) as f64;
        let (x, y) = self.fingers.values().fold((0.0, 0.0), |(x, y), p| (x + p.x, y + p.y));
        PhysicalPosition::new(x / count, y / count)
    }

    fn spread(&self) -> Option<f64> {
        let mut points = self.fingers.values();
        let (a, b) = (points.next()?, points.next()?);
        Some(distance(*a, *b))
    }
}

pub fn nearest(x: i32, y: i32, radius: i32, hit: impl Fn(i32, i32) -> bool) -> (i32, i32) {
    if hit(x, y) {
        return (x, y);
    }
    let mut best: Option<(i32, (i32, i32))> = None;
    for dy in (-radius..=radius).step_by(2) {
        for dx in (-radius..=radius).step_by(2) {
            let squared = dx * dx + dy * dy;
            if squared <= radius * radius && best.is_none_or(|(closest, _)| squared < closest) && hit(x + dx, y + dy) {
                best = Some((squared, (x + dx, y + dy)));
            }
        }
    }
    best.map_or((x, y), |(_, point)| point)
}

fn distance(a: PhysicalPosition<f64>, b: PhysicalPosition<f64>) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

fn moved(device_id: DeviceId, position: PhysicalPosition<f64>) -> WindowEvent {
    WindowEvent::CursorMoved { device_id, position }
}

fn button(device_id: DeviceId, state: ElementState, button: MouseButton) -> WindowEvent {
    WindowEvent::MouseInput { device_id, state, button }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn touch(id: u64, phase: TouchPhase, x: f64, y: f64) -> Touch {
        Touch { device_id: DeviceId::dummy(), phase, location: PhysicalPosition::new(x, y), force: None, id }
    }

    fn kinds(events: &[WindowEvent]) -> Vec<String> {
        events.iter().map(|event| match event {
            WindowEvent::CursorMoved { position, .. } => format!("move {} {}", position.x, position.y),
            WindowEvent::MouseInput { state, button, .. } => format!("{button:?} {state:?}"),
            WindowEvent::MouseWheel { delta: MouseScrollDelta::LineDelta(_, y), .. } => format!("wheel {y}"),
            other => format!("{other:?}"),
        }).collect()
    }

    #[test]
    fn direct_mode_presses_on_touch_down_and_releases_on_lift_like_a_mouse() {
        let mut touches = Touches::default();
        touches.set_direct(true);
        let now = Instant::now();
        assert_eq!(kinds(&touches.handle(touch(1, TouchPhase::Started, 10.0, 20.0), now)), ["move 10 20", "Left Pressed"]);
        assert!(touches.handle(touch(2, TouchPhase::Started, 90.0, 90.0), now).is_empty());
        assert_eq!(kinds(&touches.handle(touch(1, TouchPhase::Moved, 14.0, 22.0), now)), ["move 14 22"]);
        assert_eq!(kinds(&touches.handle(touch(1, TouchPhase::Ended, 14.0, 22.0), now)), ["move 14 22", "Left Released"]);
        assert!(touches.tick(now + Duration::from_secs(2)).is_empty());
        touches.set_direct(false);
        assert_eq!(kinds(&touches.handle(touch(3, TouchPhase::Started, 5.0, 5.0), now)), ["move 5 5"]);
    }

    #[test]
    fn leaving_direct_mode_mid_touch_still_releases_the_button() {
        let mut touches = Touches::default();
        touches.set_direct(true);
        let now = Instant::now();
        touches.handle(touch(1, TouchPhase::Started, 10.0, 20.0), now);
        touches.set_direct(false);
        assert_eq!(kinds(&touches.handle(touch(1, TouchPhase::Ended, 10.0, 20.0), now)), ["move 10 20", "Left Released"]);
    }

    #[test]
    fn a_tap_near_a_target_snaps_to_its_closest_point_and_far_taps_stay_put() {
        let hit = |x: i32, y: i32| (100..=120).contains(&x) && (50..=60).contains(&y);
        assert_eq!(nearest(110, 55, 30, hit), (110, 55));
        let (x, y) = nearest(110, 80, 30, hit);
        assert!(hit(x, y) && y == 60 && (x - 110).abs() <= 1);
        assert_eq!(nearest(110, 200, 30, hit), (110, 200));
        let two = |x: i32, y: i32| (0..=10).contains(&x) && ((0..=10).contains(&y) || (30..=40).contains(&y));
        assert_eq!(nearest(5, 24, 30, two).1, 30);
        assert_eq!(nearest(5, 16, 30, two).1, 10);
    }

    #[test]
    fn a_tap_hovers_then_clicks_on_release() {
        let mut touches = Touches::default();
        let now = Instant::now();
        assert_eq!(kinds(&touches.handle(touch(1, TouchPhase::Started, 10.0, 20.0), now)), ["move 10 20"]);
        assert_eq!(kinds(&touches.handle(touch(1, TouchPhase::Moved, 14.0, 20.0), now)), Vec::<String>::new());
        assert_eq!(kinds(&touches.handle(touch(1, TouchPhase::Ended, 14.0, 20.0), now)), ["move 10 20", "Left Pressed", "Left Released"]);
    }

    #[test]
    fn the_slop_is_measured_in_points_so_a_shaky_tap_on_a_3x_screen_still_clicks() {
        let mut touches = Touches::default();
        touches.set_scale(3.0);
        let now = Instant::now();
        touches.handle(touch(1, TouchPhase::Started, 100.0, 100.0), now);
        assert!(touches.handle(touch(1, TouchPhase::Moved, 124.0, 100.0), now).is_empty());
        assert_eq!(kinds(&touches.handle(touch(1, TouchPhase::Ended, 124.0, 100.0), now)), ["move 100 100", "Left Pressed", "Left Released"]);
        touches.handle(touch(1, TouchPhase::Started, 100.0, 100.0), now);
        assert_eq!(kinds(&touches.handle(touch(1, TouchPhase::Moved, 140.0, 100.0), now)), ["move 100 100", "Left Pressed", "move 140 100"]);
    }

    #[test]
    fn moving_past_the_slop_presses_left_and_drags() {
        let mut touches = Touches::default();
        let now = Instant::now();
        touches.handle(touch(1, TouchPhase::Started, 10.0, 20.0), now);
        assert_eq!(kinds(&touches.handle(touch(1, TouchPhase::Moved, 40.0, 20.0), now)), ["move 10 20", "Left Pressed", "move 40 20"]);
        assert_eq!(kinds(&touches.handle(touch(1, TouchPhase::Moved, 50.0, 20.0), now)), ["move 50 20"]);
        assert_eq!(kinds(&touches.handle(touch(1, TouchPhase::Ended, 50.0, 20.0), now)), ["move 50 20", "Left Released"]);
    }

    #[test]
    fn holding_still_is_a_right_click_and_the_release_adds_nothing() {
        let mut touches = Touches::default();
        let now = Instant::now();
        touches.handle(touch(1, TouchPhase::Started, 10.0, 20.0), now);
        assert!(touches.tick(now + Duration::from_millis(200)).is_empty());
        assert_eq!(kinds(&touches.tick(now + Duration::from_millis(600))), ["move 10 20", "Right Pressed", "Right Released"]);
        assert!(touches.tick(now + Duration::from_millis(900)).is_empty());
        assert!(touches.handle(touch(1, TouchPhase::Ended, 10.0, 20.0), now).is_empty());
    }

    #[test]
    fn a_cancelled_pending_touch_does_not_click() {
        let mut touches = Touches::default();
        let now = Instant::now();
        touches.handle(touch(1, TouchPhase::Started, 10.0, 20.0), now);
        assert!(touches.handle(touch(1, TouchPhase::Cancelled, 10.0, 20.0), now).is_empty());
    }

    #[test]
    fn two_fingers_hold_the_right_button_and_pinch_scrolls() {
        let mut touches = Touches::default();
        let now = Instant::now();
        touches.handle(touch(1, TouchPhase::Started, 100.0, 100.0), now);
        assert_eq!(kinds(&touches.handle(touch(2, TouchPhase::Started, 200.0, 100.0), now)), ["move 150 100", "Right Pressed"]);
        assert_eq!(kinds(&touches.handle(touch(2, TouchPhase::Moved, 248.0, 100.0), now)), ["move 174 100", "wheel 1"]);
        assert_eq!(kinds(&touches.handle(touch(2, TouchPhase::Ended, 248.0, 100.0), now)), ["move 248 100", "Right Released"]);
        assert!(touches.handle(touch(1, TouchPhase::Moved, 120.0, 100.0), now).is_empty());
        assert!(touches.handle(touch(1, TouchPhase::Ended, 120.0, 100.0), now).is_empty());
        assert!(touches.tick(now + Duration::from_secs(2)).is_empty());
    }

    #[test]
    fn a_dragging_finger_releases_left_when_the_second_lands() {
        let mut touches = Touches::default();
        let now = Instant::now();
        touches.handle(touch(1, TouchPhase::Started, 0.0, 0.0), now);
        touches.handle(touch(1, TouchPhase::Moved, 40.0, 0.0), now);
        assert_eq!(kinds(&touches.handle(touch(2, TouchPhase::Started, 80.0, 0.0), now)), ["Left Released", "move 60 0", "Right Pressed"]);
    }
}
