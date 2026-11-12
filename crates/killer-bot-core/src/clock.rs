use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub trait Clock: Send + Sync {
    fn now(&self) -> Duration;
}

#[derive(Debug, Clone)]
pub struct SystemClock {
    start: Instant,
}

impl Default for SystemClock {
    fn default() -> Self {
        Self { start: Instant::now() }
    }
}

impl Clock for SystemClock {
    fn now(&self) -> Duration {
        self.start.elapsed()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ManualClock {
    elapsed: Arc<Mutex<Duration>>,
}

impl ManualClock {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn advance(&self, by: Duration) {
        if let Ok(mut elapsed) = self.elapsed.lock() {
            *elapsed += by;
        }
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Duration {
        self.elapsed.lock().map_or(Duration::ZERO, |elapsed| *elapsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_clock_only_moves_when_advanced() {
        let clock = ManualClock::new();
        assert_eq!(clock.now(), Duration::ZERO);
        let twin = clock.clone();
        clock.advance(Duration::from_millis(250));
        assert_eq!(clock.now(), Duration::from_millis(250));
        assert_eq!(twin.now(), Duration::from_millis(250));
    }

    #[test]
    fn system_clock_never_goes_back() {
        let clock = SystemClock::default();
        let first = clock.now();
        assert!(clock.now() >= first);
    }
}
