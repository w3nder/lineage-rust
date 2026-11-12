use killer_world::PacketRecord;
use l2_net::game::fields::{Field, client_kind, decode, hex_dump, server_kind};
use std::collections::VecDeque;
use std::time::Duration;

pub const CAPACITY: usize = 20_000;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Logged {
    pub number: u64,
    pub outgoing: bool,
    pub at: Duration,
    pub body: Vec<u8>,
}

impl Logged {
    pub fn opcode(&self) -> u16 {
        self.body.first().copied().map_or(0, u16::from)
    }

    pub fn name(&self) -> String {
        let kind = if self.outgoing { client_kind(&self.body) } else { server_kind(&self.body) };
        kind.map_or_else(|| format!("Unknown {:#04x}", self.opcode()), |kind| kind.name)
    }

    pub fn format(&self) -> String {
        let kind = if self.outgoing { client_kind(&self.body) } else { server_kind(&self.body) };
        kind.map(|kind| if kind.format.starts_with("ch") || kind.format.starts_with('c') { kind.format } else { format!("c{}", kind.format) }).unwrap_or_default()
    }

    pub fn fields(&self) -> Vec<Field> {
        decode(&self.format(), &self.body).0
    }

    pub fn detail(&self) -> String {
        let mut text = format!("{} ({})\r\n", self.name(), if self.outgoing { "client -> server" } else { "server -> client" });
        for field in self.fields() {
            text.push_str(&format!("{:04X}  {}  {}\r\n", field.offset, field.kind, field.text));
        }
        text.push_str("\r\n");
        text.push_str(&hex_dump(&self.body).replace('\n', "\r\n"));
        text
    }
}

#[derive(Debug, Default)]
pub struct PacketLog {
    entries: VecDeque<Logged>,
    next: u64,
    pub recording: bool,
}

impl PacketLog {
    pub fn add(&mut self, record: PacketRecord, at: Duration) {
        if !self.recording {
            return;
        }
        self.next += 1;
        if self.entries.len() == CAPACITY {
            self.entries.pop_front();
        }
        self.entries.push_back(Logged { number: self.next, outgoing: record.outgoing, at, body: record.body });
    }

    pub fn entries(&self) -> impl Iterator<Item = &Logged> {
        self.entries.iter()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn save(&self) -> String {
        self.entries
            .iter()
            .map(|entry| format!("{}\t{}\t{}\t{}", entry.number, if entry.outgoing { "C" } else { "S" }, entry.at.as_millis(), entry.body.iter().map(|byte| format!("{byte:02x}")).collect::<String>()))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn load(&mut self, text: &str) -> usize {
        let mut loaded = 0;
        for line in text.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            let [number, side, at, hex] = parts[..] else { continue };
            let (Ok(number), Ok(at)) = (number.parse::<u64>(), at.parse::<u64>()) else { continue };
            let Some(body) = parse_hex(hex) else { continue };
            if self.entries.len() == CAPACITY {
                self.entries.pop_front();
            }
            self.entries.push_back(Logged { number, outgoing: side == "C", at: Duration::from_millis(at), body });
            self.next = self.next.max(number);
            loaded += 1;
        }
        loaded
    }
}

pub fn parse_hex(text: &str) -> Option<Vec<u8>> {
    let clean: String = text.chars().filter(|c| !c.is_whitespace() && *c != ',').collect::<String>().replace("0x", "");
    if clean.is_empty() || !clean.len().is_multiple_of(2) {
        return None;
    }
    (0..clean.len()).step_by(2).map(|index| u8::from_str_radix(&clean[index..index + 2], 16).ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(outgoing: bool, body: &[u8]) -> PacketRecord {
        PacketRecord { outgoing, body: body.to_vec() }
    }

    #[test]
    fn nothing_is_kept_until_recording_starts_and_numbers_follow_arrival() {
        let mut log = PacketLog::default();
        log.add(record(false, &[0x05]), Duration::ZERO);
        assert!(log.is_empty());
        log.recording = true;
        log.add(record(false, &[0x07, 1, 0, 0, 0]), Duration::from_millis(10));
        log.add(record(true, &[0x09]), Duration::from_millis(20));
        let numbers: Vec<_> = log.entries().map(|entry| (entry.number, entry.outgoing)).collect();
        assert_eq!(numbers, vec![(1, false), (2, true)]);
        assert_eq!(log.entries().next().unwrap().name(), "RevivePacket");
        log.clear();
        assert!(log.is_empty());
    }

    #[test]
    fn the_log_is_a_ring_of_the_last_twenty_thousand_packets() {
        let mut log = PacketLog { recording: true, ..Default::default() };
        for index in 0..(CAPACITY + 5) {
            log.add(record(false, &[0x07, (index % 200) as u8]), Duration::ZERO);
        }
        assert_eq!(log.len(), CAPACITY);
        assert_eq!(log.entries().next().unwrap().number, 6);
    }

    #[test]
    fn detail_shows_name_direction_fields_and_hex() {
        let mut log = PacketLog { recording: true, ..Default::default() };
        let mut body = vec![0x07u8];
        body.extend_from_slice(&42i32.to_le_bytes());
        log.add(record(false, &body), Duration::ZERO);
        let detail = log.entries().next().unwrap().detail();
        assert!(detail.contains("RevivePacket") && detail.contains("server -> client"));
        assert!(detail.contains("42"));
        assert!(detail.contains("07 2A 00 00 00"));
    }

    #[test]
    fn saved_logs_load_back_and_bad_lines_are_skipped() {
        let mut log = PacketLog { recording: true, ..Default::default() };
        log.add(record(false, &[0x07, 1, 0, 0, 0]), Duration::from_millis(5));
        log.add(record(true, &[0x09]), Duration::from_millis(9));
        let text = log.save();
        let mut other = PacketLog::default();
        assert_eq!(other.load(&format!("{text}\nlixo\n3\tS\tx\t00")), 2);
        assert_eq!(other.entries().map(|entry| entry.body.clone()).collect::<Vec<_>>(), vec![vec![7, 1, 0, 0, 0], vec![9]]);
    }

    #[test]
    fn hex_text_is_parsed_with_spaces_commas_and_prefixes_and_rejects_garbage() {
        assert_eq!(parse_hex("07 2A 00"), Some(vec![7, 0x2a, 0]));
        assert_eq!(parse_hex("0x07,0x2a"), Some(vec![7, 0x2a]));
        assert_eq!(parse_hex("7"), None);
        assert_eq!(parse_hex("zz"), None);
        assert_eq!(parse_hex(""), None);
    }
}
