use killer_platform::time::Instant;

pub const SUN_RISE_MINUTES: i32 = 0x1a4;
pub const SUN_SET_MINUTES: i32 = 0x3c;

#[derive(Debug, Clone, Copy)]
pub struct GameClock {
    pub minutes_at_sync: i32,
    pub ratio: i32,
    pub synced_at: Instant,
}

impl GameClock {
    pub fn new(minutes_at_sync: i32, ratio: i32, synced_at: Instant) -> GameClock {
        GameClock { minutes_at_sync, ratio, synced_at }
    }

    pub fn hour(&self, now: Instant) -> f32 {
        let elapsed_real_seconds = now.saturating_duration_since(self.synced_at).as_secs_f32();
        let minutes = self.minutes_at_sync as f32 + elapsed_real_seconds * self.ratio as f32 / 60.0;
        minutes.rem_euclid(1440.0) / 60.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use killer_platform::time::Duration;

    #[test]
    fn ten_real_minutes_at_ratio_six_from_six_is_seven() {
        let sync = Instant::now();
        let clock = GameClock::new(6 * 60, 6, sync);
        let now = sync + Duration::from_secs(10 * 60);
        assert!((clock.hour(now) - 7.0).abs() < 1e-3, "{}", clock.hour(now));
    }

    #[test]
    fn hour_wraps_past_midnight() {
        let sync = Instant::now();
        let clock = GameClock::new(23 * 60, 6, sync);
        let now = sync + Duration::from_secs(15 * 60);
        assert!((clock.hour(now) - 0.5).abs() < 1e-3, "{}", clock.hour(now));
    }

    #[test]
    fn no_elapsed_time_keeps_the_synced_hour() {
        let sync = Instant::now();
        let clock = GameClock::new(6 * 60, 6, sync);
        assert!((clock.hour(sync) - 6.0).abs() < 1e-3);
    }
}
