use std::time::{Duration, Instant};
use winit::dpi::PhysicalPosition;
use winit::event::{Touch, TouchPhase};

const ZONE_WIDTH: f64 = 0.38;
const ZONE_TOP: f64 = 0.2;
const RADIUS_POINTS: f64 = 56.0;
const SLOP_POINTS: f64 = 10.0;
const DEAD_ZONE: f64 = 0.18;
const HOLD: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Default)]
enum State {
    #[default]
    Idle,
    Pending { touch: Touch, since: Instant },
    Active { id: u64, origin: PhysicalPosition<f64>, current: PhysicalPosition<f64> },
    Passed { id: u64 },
}

#[derive(Debug, Default)]
pub struct Joystick {
    state: State,
    scale: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Knob {
    pub origin: PhysicalPosition<f64>,
    pub current: PhysicalPosition<f64>,
    pub radius: f64,
}

fn distance(a: PhysicalPosition<f64>, b: PhysicalPosition<f64>) -> f64 {
    (a.x - b.x).hypot(a.y - b.y)
}

impl Joystick {
    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    pub fn radius(&self) -> f64 {
        RADIUS_POINTS * self.scale.max(1.0)
    }

    fn slop(&self) -> f64 {
        SLOP_POINTS * self.scale.max(1.0)
    }

    fn in_zone(location: PhysicalPosition<f64>, screen: (f64, f64)) -> bool {
        location.x < screen.0 * ZONE_WIDTH && location.y > screen.1 * ZONE_TOP
    }

    pub fn handle(&mut self, touch: Touch, screen: (f64, f64), scale: f64) -> Vec<Touch> {
        self.scale = scale;
        match (&self.state, touch.phase) {
            (State::Idle, TouchPhase::Started) if Self::in_zone(touch.location, screen) => {
                self.state = State::Pending { touch, since: Instant::now() };
                Vec::new()
            }
            (State::Pending { touch: first, .. }, _) if first.id == touch.id => self.pending(touch),
            (State::Active { id, .. }, _) if *id == touch.id => {
                self.active(touch);
                Vec::new()
            }
            (State::Passed { id }, phase) if *id == touch.id => {
                if matches!(phase, TouchPhase::Ended | TouchPhase::Cancelled) {
                    self.state = State::Idle;
                }
                vec![touch]
            }
            _ => vec![touch],
        }
    }

    fn pending(&mut self, touch: Touch) -> Vec<Touch> {
        let State::Pending { touch: first, since } = std::mem::take(&mut self.state) else { return vec![touch] };
        match touch.phase {
            TouchPhase::Moved if distance(first.location, touch.location) > self.slop() => {
                self.state = State::Active { id: touch.id, origin: first.location, current: touch.location };
                Vec::new()
            }
            TouchPhase::Moved => {
                self.state = State::Pending { touch: first, since };
                Vec::new()
            }
            TouchPhase::Ended => vec![first, touch],
            TouchPhase::Cancelled => Vec::new(),
            TouchPhase::Started => vec![touch],
        }
    }

    fn active(&mut self, touch: Touch) {
        let State::Active { id, origin, .. } = self.state else { return };
        match touch.phase {
            TouchPhase::Ended | TouchPhase::Cancelled => self.state = State::Idle,
            _ => self.state = State::Active { id, origin, current: touch.location },
        }
    }

    pub fn tick(&mut self, now: Instant) -> Vec<Touch> {
        let State::Pending { touch, since } = &self.state else { return Vec::new() };
        if now.duration_since(*since) < HOLD {
            return Vec::new();
        }
        let touch = *touch;
        self.state = State::Passed { id: touch.id };
        vec![touch]
    }

    pub fn reset(&mut self) {
        self.state = State::Idle;
    }

    pub fn knob(&self) -> Option<Knob> {
        let State::Active { origin, current, .. } = self.state else { return None };
        let radius = self.radius();
        let length = distance(origin, current);
        let reach = if length > radius { radius / length } else { 1.0 };
        let clamped = PhysicalPosition::new(origin.x + (current.x - origin.x) * reach, origin.y + (current.y - origin.y) * reach);
        Some(Knob { origin, current: clamped, radius })
    }

    pub fn vector(&self) -> Option<(f32, f32)> {
        let knob = self.knob()?;
        let dx = knob.current.x - knob.origin.x;
        let dy = knob.current.y - knob.origin.y;
        let length = dx.hypot(dy);
        if length / knob.radius < DEAD_ZONE {
            return None;
        }
        Some(((dx / length) as f32, (-dy / length) as f32))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::event::DeviceId;

    const SCREEN: (f64, f64) = (2400.0, 1080.0);

    fn touch(id: u64, phase: TouchPhase, x: f64, y: f64) -> Touch {
        Touch { device_id: DeviceId::dummy(), phase, location: PhysicalPosition::new(x, y), force: None, id }
    }

    #[test]
    fn a_touch_outside_the_zone_goes_through() {
        let mut joystick = Joystick::default();
        let out = joystick.handle(touch(1, TouchPhase::Started, 1800.0, 700.0), SCREEN, 3.0);
        assert_eq!(out.len(), 1);
        assert!(joystick.knob().is_none());
    }

    #[test]
    fn a_tap_in_the_zone_is_replayed_when_the_finger_lifts() {
        let mut joystick = Joystick::default();
        assert!(joystick.handle(touch(1, TouchPhase::Started, 300.0, 700.0), SCREEN, 3.0).is_empty());
        let out = joystick.handle(touch(1, TouchPhase::Ended, 300.0, 700.0), SCREEN, 3.0);
        let phases: Vec<_> = out.iter().map(|t| t.phase).collect();
        assert_eq!(phases, [TouchPhase::Started, TouchPhase::Ended]);
        assert!(joystick.knob().is_none());
    }

    #[test]
    fn dragging_in_the_zone_becomes_a_stick_and_nothing_goes_through() {
        let mut joystick = Joystick::default();
        joystick.handle(touch(1, TouchPhase::Started, 300.0, 700.0), SCREEN, 3.0);
        assert!(joystick.handle(touch(1, TouchPhase::Moved, 300.0, 600.0), SCREEN, 3.0).is_empty());
        let knob = joystick.knob().unwrap();
        assert_eq!(knob.origin, PhysicalPosition::new(300.0, 700.0));
        let (right, forward) = joystick.vector().unwrap();
        assert!(right.abs() < 1e-6 && (forward - 1.0).abs() < 1e-6);
        assert!(joystick.handle(touch(1, TouchPhase::Ended, 300.0, 600.0), SCREEN, 3.0).is_empty());
        assert!(joystick.vector().is_none());
    }

    #[test]
    fn the_knob_stays_inside_the_radius_and_the_dead_zone_gives_no_direction() {
        let mut joystick = Joystick::default();
        joystick.handle(touch(1, TouchPhase::Started, 300.0, 700.0), SCREEN, 3.0);
        joystick.handle(touch(1, TouchPhase::Moved, 340.0, 700.0), SCREEN, 3.0);
        assert!(joystick.vector().is_some());
        joystick.handle(touch(1, TouchPhase::Moved, 900.0, 700.0), SCREEN, 3.0);
        let knob = joystick.knob().unwrap();
        assert!((distance(knob.origin, knob.current) - knob.radius).abs() < 1e-6);
        joystick.handle(touch(1, TouchPhase::Moved, 305.0, 700.0), SCREEN, 3.0);
        assert!(joystick.vector().is_none());
    }

    #[test]
    fn a_second_finger_elsewhere_does_not_disturb_the_stick() {
        let mut joystick = Joystick::default();
        joystick.handle(touch(1, TouchPhase::Started, 300.0, 700.0), SCREEN, 3.0);
        joystick.handle(touch(1, TouchPhase::Moved, 300.0, 600.0), SCREEN, 3.0);
        let out = joystick.handle(touch(2, TouchPhase::Started, 1800.0, 700.0), SCREEN, 3.0);
        assert_eq!(out.len(), 1);
        assert!(joystick.vector().is_some());
    }

    #[test]
    fn holding_still_hands_the_touch_over_for_the_long_press() {
        let mut joystick = Joystick::default();
        joystick.handle(touch(1, TouchPhase::Started, 300.0, 700.0), SCREEN, 3.0);
        assert!(joystick.tick(Instant::now()).is_empty());
        let out = joystick.tick(Instant::now() + Duration::from_millis(600));
        assert_eq!(out.len(), 1);
        let rest = joystick.handle(touch(1, TouchPhase::Ended, 300.0, 700.0), SCREEN, 3.0);
        assert_eq!(rest.len(), 1);
        assert!(joystick.handle(touch(2, TouchPhase::Started, 300.0, 700.0), SCREEN, 3.0).is_empty());
    }
}
