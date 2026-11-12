use std::f32::consts::FRAC_PI_2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fade {
    pub color1: [u8; 4],
    pub color2: [u8; 4],
    pub period: f32,
    pub phase: f32,
    pub sinusoidal: bool,
}

impl Fade {
    #[must_use]
    pub fn at(&self, time: f32) -> [f32; 4] {
        let x = (time + self.phase) / self.period.max(f32::EPSILON);
        let f = if self.sinusoidal {
            ((x * FRAC_PI_2).cos() + 1.0) * 0.5
        } else {
            let whole = x.floor();
            let frac = x - whole;
            if (whole as i64) & 1 == 1 { frac } else { 1.0 - frac }
        };
        let mut color = [0.0; 4];
        for (channel, (one, two)) in color.iter_mut().zip(self.color1.into_iter().zip(self.color2)) {
            *channel = (f32::from(one) * f + f32::from(two) * (1.0 - f)) / 255.0;
            *channel = channel.clamp(0.0, 1.0);
        }
        color
    }

    #[must_use]
    pub fn bytes(&self, time: f32) -> [u8; 4] {
        self.at(time).map(|channel| (channel * 255.0).round() as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RED: [u8; 4] = [255, 0, 0, 255];
    const BLUE: [u8; 4] = [0, 0, 255, 51];

    fn close(a: [f32; 4], b: [f32; 4]) -> bool {
        a.iter().zip(b).all(|(a, b)| (a - b).abs() < 1e-4)
    }

    #[test]
    fn a_linear_fade_is_a_triangle_wave_from_color1() {
        let fade = Fade { color1: RED, color2: BLUE, period: 1.0, phase: 0.0, sinusoidal: false };
        assert!(close(fade.at(0.0), [1.0, 0.0, 0.0, 1.0]));
        assert!(close(fade.at(1.0), [0.0, 0.0, 1.0, 0.2]));
        assert!(close(fade.at(2.0), [1.0, 0.0, 0.0, 1.0]));
        assert!(close(fade.at(0.5), [0.5, 0.0, 0.5, 0.6]));
        assert!(close(fade.at(1.5), [0.5, 0.0, 0.5, 0.6]));
    }

    #[test]
    fn a_sinusoidal_fade_reaches_color2_after_two_periods() {
        let fade = Fade { color1: RED, color2: BLUE, period: 0.5, phase: 0.0, sinusoidal: true };
        assert!(close(fade.at(0.0), [1.0, 0.0, 0.0, 1.0]));
        assert!(close(fade.at(1.0), [0.0, 0.0, 1.0, 0.2]));
        assert!(close(fade.at(2.0), [1.0, 0.0, 0.0, 1.0]));
        assert!(close(fade.at(0.5), [0.5, 0.0, 0.5, 0.6]));
    }

    #[test]
    fn the_phase_shifts_time_and_bytes_are_clamped() {
        let fade = Fade { color1: RED, color2: BLUE, period: 1.0, phase: 1.0, sinusoidal: false };
        assert!(close(fade.at(0.0), [0.0, 0.0, 1.0, 0.2]));
        assert!(close(fade.at(-1.0), [1.0, 0.0, 0.0, 1.0]));
        assert_eq!(fade.bytes(0.0), BLUE);
        assert_eq!(fade.bytes(0.5), [128, 0, 128, 153]);
    }
}
