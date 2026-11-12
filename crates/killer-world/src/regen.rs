const REGEN_TICK: f64 = 0.666;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct RegenFill {
    pub interval: f64,
    pub remaining: f64,
    pub amount: f64,
    pub rate: f64,
    pub stamp: f64,
}

impl RegenFill {
    pub fn start(count: i32, interval: i32, rate: f64, now: f64) -> Self {
        let interval = f64::from(interval);
        let remaining = f64::from(count) - REGEN_TICK * (interval / 2.0).trunc();
        Self { interval, remaining, amount: remaining / (interval * REGEN_TICK) * rate, rate, stamp: now }
    }

    pub fn decay(&mut self, now: f64) {
        if self.remaining <= 0.0 || self.amount < 0.0 {
            return;
        }
        let dt = now - self.stamp;
        self.remaining -= dt;
        self.amount -= self.rate / self.interval * dt * (1.0 / REGEN_TICK);
        if self.remaining <= 0.0 || self.amount < 0.0 {
            self.remaining = 0.0;
            self.amount = 0.0;
        }
        self.stamp = now;
    }

    pub fn clear(&mut self) {
        *self = Self { stamp: self.stamp, ..Self::default() };
    }

    pub fn fill_width(&self, width: i32, max: i32) -> i32 {
        if self.amount == 0.0 || max == 0 { 0 } else { (f64::from(width) * self.amount / f64::from(max)) as i32 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn regen_status_1010d490_previews_the_hp_left_to_regenerate_and_decays_on_hp_changes() {
        let mut fill = RegenFill::start(10, 3, 30.0, 100.0);
        assert!((fill.remaining - (10.0 - 0.666)).abs() < 1e-12);
        assert!((fill.amount - 30.0 * fill.remaining / (0.666 * 3.0)).abs() < 1e-9);
        assert_eq!(fill.amount, fill.remaining / (3.0 * 0.666) * 30.0);
        assert_eq!(1.0 / REGEN_TICK, 1.5015015015015014);
        let before = fill.amount;
        fill.decay(101.0);
        assert!((fill.remaining - (9.334 - 1.0)).abs() < 1e-9);
        assert!((before - fill.amount - 10.0 / 0.666).abs() < 1e-9);
        assert_eq!(fill.fill_width(150, 1000), (150.0 * fill.amount / 1000.0) as i32);
        fill.decay(200.0);
        assert_eq!((fill.remaining, fill.amount), (0.0, 0.0));
        assert_eq!(fill.fill_width(150, 1000), 0);
    }
}
