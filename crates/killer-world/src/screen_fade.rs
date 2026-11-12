const MINIMUM_FRACTION: f32 = f32::from_bits(0x38d1b717);
const BLINK: i32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FadeDone {
    pub id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScreenFade {
    duration: f32,
    elapsed: f32,
    fading_out: bool,
    fading_in: bool,
    color: [u8; 4],
    id: i32,
    hold: f32,
    in_duration: f32,
    flash_scale: f32,
    flash_fog: [f32; 3],
}

impl Default for ScreenFade {
    fn default() -> Self {
        Self { duration: 0.0, elapsed: 0.0, fading_out: false, fading_in: false, color: [0; 4], id: 0, hold: 0.0, in_duration: 0.0, flash_scale: 1.0, flash_fog: [0.0; 3] }
    }
}

fn clamp(value: f32) -> f32 {
    value.clamp(MINIMUM_FRACTION, 1.0)
}

impl ScreenFade {
    pub fn flash(&self) -> (f32, [f32; 3]) {
        (self.flash_scale, self.flash_fog)
    }

    pub fn active(&self) -> bool {
        self.fading_out || self.fading_in
    }

    fn idle(&self) -> bool {
        !self.fading_out && !self.fading_in
    }

    pub fn blink(&mut self, duration: f32, color: [u8; 4], in_duration: f32, hold: f32) {
        if self.idle() {
            self.duration = duration;
            self.in_duration = in_duration;
            self.elapsed = 0.0;
            self.color = color;
            self.fading_out = true;
            self.hold = hold;
            self.id = BLINK;
        }
    }

    pub fn fade_out(&mut self, duration: f32, color: [u8; 4], id: i32) {
        if self.idle() {
            self.duration = duration;
            self.elapsed = 0.0;
            self.color = color;
            self.fading_out = true;
            self.id = id;
        }
    }

    pub fn fade_in(&mut self, duration: f32, id: i32) {
        if self.idle() {
            self.duration = duration;
            self.elapsed = 0.0;
            self.fading_in = true;
            self.id = id;
        }
    }

    fn fog(&self) -> [f32; 3] {
        let scale = f32::from_bits(0x3b808081);
        [self.color[2], self.color[1], self.color[0]].map(|byte| (f64::from(byte) * f64::from(scale)) as f32)
    }

    pub fn update(&mut self, dt: f32) -> Option<FadeDone> {
        if self.idle() {
            return None;
        }
        let milliseconds = (f64::from(dt) * 1000.0) as f32;
        self.elapsed = (f64::from(milliseconds) + f64::from(self.elapsed)) as f32;
        let mut fraction = clamp(self.elapsed / self.duration);
        let mut done = None;
        if self.fading_out {
            self.flash_scale = 1.0 - fraction;
            self.flash_fog = self.fog();
            if fraction >= 1.0 {
                let id = self.id;
                if id == BLINK {
                    self.hold = (f64::from(self.hold) - f64::from(milliseconds)) as f32;
                    if self.hold >= 0.0 {
                        return None;
                    }
                    self.elapsed = -self.hold;
                    self.duration = self.in_duration;
                    fraction = clamp(self.elapsed / self.duration);
                    self.fading_in = true;
                }
                self.fading_out = false;
                done = Some(FadeDone { id });
                self.id = 0;
            }
        }
        if self.fading_in {
            self.flash_fog = self.fog();
            self.flash_scale = fraction;
            if fraction >= 1.0 {
                self.fading_in = false;
                self.id = 0;
            }
        }
        done
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(fade: &mut ScreenFade, seconds: f32, step: f32) -> Vec<(f32, [f32; 3])> {
        let mut samples = Vec::new();
        let mut elapsed = 0.0;
        while elapsed < seconds {
            fade.update(step);
            samples.push(fade.flash());
            elapsed += step;
        }
        samples
    }

    #[test]
    fn fade_update_1055c5e0_fades_out_with_the_scale_going_to_zero_and_the_fog_set_to_the_color() {
        let mut fade = ScreenFade::default();
        fade.fade_out(1000.0, [10, 20, 30, 255], 7);
        fade.update(0.5);
        let (scale, fog) = fade.flash();
        assert!((scale - 0.5).abs() < 1e-6);
        assert!((fog[0] - 30.0 / 255.0).abs() < 1e-6 && (fog[2] - 10.0 / 255.0).abs() < 1e-6);
        assert_eq!(fade.update(0.6), Some(FadeDone { id: 7 }));
        assert_eq!(fade.flash().0, 0.0);
        assert!(!fade.active());
    }

    #[test]
    fn fade_update_1055c5e0_blink_holds_the_black_and_then_fades_back_in() {
        let mut fade = ScreenFade::default();
        fade.blink(1000.0, [255; 4], 500.0, 300.0);
        for _ in 0..9 {
            assert!(fade.update(0.1).is_none());
        }
        assert!(fade.update(0.1).is_none());
        assert_eq!(fade.flash().0, 0.0);
        assert!(fade.update(0.1).is_none());
        assert!(fade.update(0.1).is_none());
        assert_eq!(fade.flash().0, 0.0);
        assert_eq!(fade.update(0.1), Some(FadeDone { id: 2 }));
        assert!(fade.active());
        assert!((fade.flash().0 - 0.2).abs() < 1e-5, "{}", fade.flash().0);
        let samples = run(&mut fade, 0.6, 0.1);
        assert!(samples.windows(2).all(|pair| pair[1].0 >= pair[0].0));
        assert_eq!(fade.flash().0, 1.0);
        assert!(!fade.active());
    }

    #[test]
    fn screen_fade_calls_are_ignored_while_a_fade_runs() {
        let mut fade = ScreenFade::default();
        fade.fade_out(1000.0, [1, 2, 3, 4], 1);
        fade.blink(10.0, [9; 4], 10.0, 10.0);
        fade.fade_in(10.0, 3);
        fade.update(0.5);
        assert!((fade.flash().0 - 0.5).abs() < 1e-6);
    }

    #[test]
    fn the_fade_fraction_never_reaches_zero() {
        let mut fade = ScreenFade::default();
        fade.fade_in(1000.0, 0);
        fade.update(0.0);
        assert_eq!(fade.flash().0, MINIMUM_FRACTION);
    }
}
