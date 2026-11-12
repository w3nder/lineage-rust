use std::fmt::Write;
use std::time::{Duration, Instant};

pub const REPORT_FRAMES: u32 = 60;

pub struct FrameProfile {
    frames: u32,
    stages: Vec<(&'static str, Duration)>,
    frame_started: Instant,
    last: Instant,
    current: Vec<(&'static str, Duration)>,
    worst: Vec<(&'static str, Duration)>,
}

impl FrameProfile {
    pub fn new(now: Instant) -> Self {
        Self { frames: 0, stages: Vec::new(), frame_started: now, last: now, current: Vec::new(), worst: Vec::new() }
    }

    pub fn begin(&mut self, now: Instant) {
        self.frame_started = now;
        self.last = now;
        self.current.clear();
    }

    pub fn lap(&mut self, stage: &'static str, now: Instant) {
        let elapsed = now.saturating_duration_since(self.last);
        self.last = now;
        match self.stages.iter_mut().find(|(name, _)| *name == stage) {
            Some((_, total)) => *total += elapsed,
            None => self.stages.push((stage, elapsed)),
        }
        match self.current.iter_mut().find(|(name, _)| *name == stage) {
            Some((_, total)) => *total += elapsed,
            None => self.current.push((stage, elapsed)),
        }
    }

    pub fn end(&mut self, now: Instant) -> Option<String> {
        self.lap("resto", now);
        let current: Duration = self.current.iter().map(|(_, d)| *d).sum();
        let worst: Duration = self.worst.iter().map(|(_, d)| *d).sum();
        if current > worst {
            self.worst.clone_from(&self.current);
        }
        self.frames += 1;
        if self.frames < REPORT_FRAMES {
            return None;
        }
        let total: Duration = self.stages.iter().map(|(_, d)| *d).sum();
        let frames = f64::from(self.frames);
        let mut report = format!("{:.1} ms/frame:", total.as_secs_f64() * 1000.0 / frames);
        for (name, duration) in &self.stages {
            let _ = write!(report, " {name} {:.2}", duration.as_secs_f64() * 1000.0 / frames);
        }
        if let Some((name, slow)) = self.worst.iter().max_by_key(|(_, d)| *d) {
            let _ = write!(report, " | pior quadro {:.1} ms (etapa {name} {:.1})", self.worst.iter().map(|(_, d)| d.as_secs_f64()).sum::<f64>() * 1000.0, slow.as_secs_f64() * 1000.0);
        }
        self.frames = 0;
        self.stages.clear();
        self.worst.clear();
        Some(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn averages_each_stage_over_the_report_window() {
        let start = Instant::now();
        let mut profile = FrameProfile::new(start);
        let mut now = start;
        let mut report = None;
        for _ in 0..REPORT_FRAMES {
            profile.begin(now);
            now += Duration::from_millis(2);
            profile.lap("tick", now);
            now += Duration::from_millis(3);
            profile.lap("render", now);
            now += Duration::from_millis(1);
            report = profile.end(now);
        }
        let report = report.unwrap();
        assert_eq!(report, "6.0 ms/frame: tick 2.00 render 3.00 resto 1.00 | pior quadro 6.0 ms (etapa render 3.0)");
        assert!(profile.end(now).is_none());
    }
}
