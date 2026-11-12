use l2_net::game::describe_server_opcode;
use std::collections::HashMap;
use std::fmt::Write;
use std::time::{Duration, Instant};

pub const REPORT_INTERVAL: Duration = Duration::from_secs(60);
pub const HUD_TOP: usize = 5;

#[derive(Debug, Clone)]
pub struct IgnoredOpcodes {
    counts: HashMap<u8, u32>,
    next_report: Instant,
}

impl IgnoredOpcodes {
    pub fn new(now: Instant) -> Self {
        Self { counts: HashMap::new(), next_report: now + REPORT_INTERVAL }
    }

    pub fn note(&mut self, opcode: u8) {
        *self.counts.entry(opcode).or_insert(0) += 1;
    }

    pub fn total(&self) -> u32 {
        self.counts.values().sum()
    }

    pub fn distinct(&self) -> usize {
        self.counts.len()
    }

    pub fn ranked(&self) -> Vec<(u8, u32)> {
        let mut ranked: Vec<(u8, u32)> = self.counts.iter().map(|(&opcode, &count)| (opcode, count)).collect();
        ranked.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
        ranked
    }

    pub fn top(&self, count: usize) -> Vec<(u8, u32)> {
        let mut ranked = self.ranked();
        ranked.truncate(count);
        ranked
    }

    pub fn report(&self) -> Option<String> {
        if self.counts.is_empty() {
            return None;
        }
        let mut line = format!("pacotes ignorados: {} em {} opcodes:", self.total(), self.distinct());
        for (opcode, count) in self.ranked() {
            let _ = write!(line, " {}×{count}", describe_server_opcode(opcode));
        }
        Some(line)
    }

    pub fn due(&mut self, now: Instant) -> Option<String> {
        if now < self.next_report {
            return None;
        }
        self.next_report = now + REPORT_INTERVAL;
        self.report()
    }

    pub fn take_report(&mut self) -> Option<String> {
        let report = self.report();
        self.counts.clear();
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counted(start: Instant) -> IgnoredOpcodes {
        let mut ignored = IgnoredOpcodes::new(start);
        for opcode in [0x4a, 0x4a, 0x0e, 0x4a, 0xfe, 0x0e, 0x01] {
            ignored.note(opcode);
        }
        ignored
    }

    #[test]
    fn opcodes_are_ranked_by_count_then_by_opcode() {
        let ignored = counted(Instant::now());
        assert_eq!(ignored.ranked(), [(0x4a, 3), (0x0e, 2), (0x01, 1), (0xfe, 1)]);
        assert_eq!(ignored.top(2), [(0x4a, 3), (0x0e, 2)]);
        assert_eq!((ignored.total(), ignored.distinct()), (7, 4));
        assert_eq!(ignored.report().as_deref(), Some("pacotes ignorados: 7 em 4 opcodes: Say2 (0x4a)×3 StatusUpdate (0x0e)×2 MoveToLocation (0x01)×1 Extended (0xfe)×1"));
        assert_eq!(IgnoredOpcodes::new(Instant::now()).report(), None);
    }

    #[test]
    fn the_report_is_due_every_minute_and_taking_it_clears_the_counts() {
        let start = Instant::now();
        let mut ignored = counted(start);
        assert_eq!(ignored.due(start), None);
        assert!(ignored.due(start + REPORT_INTERVAL).is_some());
        assert_eq!(ignored.due(start + REPORT_INTERVAL + Duration::from_secs(1)), None);
        assert!(ignored.due(start + 2 * REPORT_INTERVAL).is_some());
        assert!(ignored.take_report().is_some());
        assert_eq!(ignored.take_report(), None);
        assert_eq!(ignored.total(), 0);
    }
}
