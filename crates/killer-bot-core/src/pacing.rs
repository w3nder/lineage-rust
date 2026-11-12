use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Pacing {
    gap: Duration,
    last: Option<Duration>,
}

impl Pacing {
    pub fn new(gap: Duration) -> Self {
        Self { gap, last: None }
    }

    pub fn allow(&mut self, now: Duration) -> bool {
        let open = self.last.is_none_or(|last| now.saturating_sub(last) >= self.gap);
        if open {
            self.last = Some(now);
        }
        open
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_commands_inside_the_gap_and_allows_after() {
        let mut pacing = Pacing::new(Duration::from_millis(100));
        assert!(pacing.allow(Duration::ZERO));
        assert!(!pacing.allow(Duration::from_millis(50)));
        assert!(pacing.allow(Duration::from_millis(100)));
    }
}
