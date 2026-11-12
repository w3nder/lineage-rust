use winit::dpi::PhysicalPosition;
use winit::event::{Touch, TouchPhase};

const MARGIN_POINTS: f64 = 8.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    Attack,
    Target,
    Pickup,
}

impl Button {
    pub const ALL: [Button; 3] = [Button::Attack, Button::Target, Button::Pickup];

    pub fn command(self) -> &'static str {
        match self {
            Button::Attack => "/attack",
            Button::Target => "/targetnext",
            Button::Pickup => "/pickup",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Button::Attack => "ATACAR",
            Button::Target => "ALVO",
            Button::Pickup => "PEGAR",
        }
    }

    fn placement(self) -> (f64, f64, f64) {
        match self {
            Button::Attack => (120.0, 110.0, 44.0),
            Button::Target => (215.0, 60.0, 29.0),
            Button::Pickup => (120.0, 205.0, 26.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Circle {
    pub center: PhysicalPosition<f64>,
    pub radius: f64,
}

pub fn circle(button: Button, screen: (f64, f64), scale: f64) -> Circle {
    let unit = scale.max(1.0);
    let (from_right, from_bottom, radius) = button.placement();
    Circle { center: PhysicalPosition::new(screen.0 - from_right * unit, screen.1 - from_bottom * unit), radius: radius * unit }
}

fn inside(circle: Circle, point: PhysicalPosition<f64>, margin: f64) -> bool {
    (point.x - circle.center.x).hypot(point.y - circle.center.y) <= circle.radius + margin
}

#[derive(Debug, Default)]
pub struct TouchHud {
    held: Option<(u64, Button)>,
}

impl TouchHud {
    pub fn pressed(&self) -> Option<Button> {
        self.held.map(|(_, button)| button)
    }

    pub fn reset(&mut self) {
        self.held = None;
    }

    pub fn handle(&mut self, touch: Touch, screen: (f64, f64), scale: f64) -> (bool, Option<Button>) {
        let margin = MARGIN_POINTS * scale.max(1.0);
        match (self.held, touch.phase) {
            (None, TouchPhase::Started) => {
                let hit = Button::ALL.into_iter().find(|&button| inside(circle(button, screen, scale), touch.location, margin));
                if let Some(button) = hit {
                    self.held = Some((touch.id, button));
                }
                (hit.is_some(), None)
            }
            (Some((id, button)), TouchPhase::Ended) if id == touch.id => {
                self.held = None;
                (true, inside(circle(button, screen, scale), touch.location, margin).then_some(button))
            }
            (Some((id, _)), TouchPhase::Cancelled) if id == touch.id => {
                self.held = None;
                (true, None)
            }
            (Some((id, _)), _) if id == touch.id => (true, None),
            _ => (false, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use winit::event::DeviceId;

    const SCREEN: (f64, f64) = (2556.0, 1179.0);

    fn touch(id: u64, phase: TouchPhase, x: f64, y: f64) -> Touch {
        Touch { device_id: DeviceId::dummy(), phase, location: PhysicalPosition::new(x, y), force: None, id }
    }

    fn at(button: Button) -> PhysicalPosition<f64> {
        circle(button, SCREEN, 3.0).center
    }

    #[test]
    fn buttons_fit_the_screen_and_do_not_overlap() {
        for a in Button::ALL {
            let c = circle(a, SCREEN, 3.0);
            assert!(c.center.x - c.radius > 0.0 && c.center.x + c.radius < SCREEN.0);
            assert!(c.center.y - c.radius > 0.0 && c.center.y + c.radius < SCREEN.1);
            for b in Button::ALL.into_iter().filter(|b| *b != a) {
                let d = circle(b, SCREEN, 3.0);
                assert!((c.center.x - d.center.x).hypot(c.center.y - d.center.y) > c.radius + d.radius);
            }
        }
    }

    #[test]
    fn releasing_on_the_button_fires_it_and_nothing_goes_to_the_world() {
        let mut hud = TouchHud::default();
        let p = at(Button::Attack);
        assert_eq!(hud.handle(touch(1, TouchPhase::Started, p.x, p.y), SCREEN, 3.0), (true, None));
        assert_eq!(hud.pressed(), Some(Button::Attack));
        assert_eq!(hud.handle(touch(1, TouchPhase::Ended, p.x, p.y), SCREEN, 3.0), (true, Some(Button::Attack)));
        assert_eq!(hud.pressed(), None);
    }

    #[test]
    fn sliding_off_the_button_cancels_it() {
        let mut hud = TouchHud::default();
        let p = at(Button::Target);
        hud.handle(touch(1, TouchPhase::Started, p.x, p.y), SCREEN, 3.0);
        assert_eq!(hud.handle(touch(1, TouchPhase::Ended, 100.0, 100.0), SCREEN, 3.0), (true, None));
    }

    #[test]
    fn touches_elsewhere_are_not_consumed() {
        let mut hud = TouchHud::default();
        assert_eq!(hud.handle(touch(1, TouchPhase::Started, 300.0, 300.0), SCREEN, 3.0), (false, None));
        let p = at(Button::Pickup);
        hud.handle(touch(2, TouchPhase::Started, p.x, p.y), SCREEN, 3.0);
        assert_eq!(hud.handle(touch(3, TouchPhase::Started, 300.0, 300.0), SCREEN, 3.0), (false, None));
    }
}
