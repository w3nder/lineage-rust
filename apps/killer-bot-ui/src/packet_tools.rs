use killer_bot_core::packets::{Logged, PacketLog, parse_hex};
use l2_net::game::fields::{decode, field_count};
use l2_net::game::packet_table::{CLIENT, SERVER};
use serde::Serialize;
use std::time::Duration;

pub const LOG_FILE: &str = "pacotes.log";
pub const FILTER_FILE: &str = "filtros-de-pacotes.txt";

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Filters {
    pub server: Vec<u16>,
    pub client: Vec<u16>,
    pub whitelist: bool,
}

pub fn time_text(at: Duration) -> String {
    let total = at.as_millis();
    format!("{:02}:{:02}:{:02}.{:03}", total / 3_600_000 % 24, total / 60_000 % 60, total / 1000 % 60, total % 1000)
}

pub fn parse_id(text: &str) -> Option<u16> {
    let clean = text.trim().trim_start_matches("0x").trim_start_matches("0X");
    if clean.is_empty() {
        return None;
    }
    let hex = text.trim().starts_with("0x") || text.trim().starts_with("0X") || clean.chars().any(|c| c.is_ascii_hexdigit() && !c.is_ascii_digit());
    if hex { u16::from_str_radix(clean, 16).ok() } else { clean.parse().ok().or_else(|| u16::from_str_radix(clean, 16).ok()) }
}

pub fn packet_id(entry: &Logged) -> u16 {
    if entry.body.first() == Some(&0xfe) && entry.body.len() >= 3 {
        return u16::from_le_bytes([entry.body[1], entry.body[2]]);
    }
    entry.opcode()
}

pub fn passes(entry: &Logged, filters: &Filters, filter_on: bool, hide_server: bool, hide_client: bool) -> bool {
    if (entry.outgoing && hide_client) || (!entry.outgoing && hide_server) {
        return false;
    }
    if !filter_on {
        return true;
    }
    let list = if entry.outgoing { &filters.client } else { &filters.server };
    let listed = list.contains(&packet_id(entry));
    if filters.whitelist { listed } else { !listed }
}

pub fn name_of(server: bool, id: u16) -> String {
    let table = if server { SERVER } else { CLIENT };
    table.iter().find(|(op, _, _)| *op == id).map_or_else(|| "?".to_owned(), |(_, name, _)| (*name).to_owned())
}

pub fn matches(entry: &Logged, query: &str, client: bool, by_name: bool) -> bool {
    if entry.outgoing != client {
        return false;
    }
    let query = query.trim();
    if query.is_empty() {
        return false;
    }
    if by_name {
        return entry.name().to_lowercase().contains(&query.to_lowercase());
    }
    parse_id(query).is_some_and(|id| packet_id(entry) == id)
}

pub fn search(entries: &[&Logged], from: Option<usize>, forward: bool, query: &str, client: bool, by_name: bool) -> Option<usize> {
    let indices: Vec<usize> = match (from, forward) {
        (Some(at), true) => (at + 1..entries.len()).collect(),
        (None, true) => (0..entries.len()).collect(),
        (Some(at), false) => (0..at.min(entries.len())).rev().collect(),
        (None, false) => (0..entries.len()).rev().collect(),
    };
    indices.into_iter().find(|index| matches(entries[*index], query, client, by_name))
}

pub fn parse_with(format: &str, entry: &Logged) -> Result<String, String> {
    let clean = format.trim();
    if clean.is_empty() || !clean.chars().all(|c| "cbhdfQSs*;0123456789".contains(c)) {
        return Err("formato inválido (use c h d f Q S)".to_owned());
    }
    let (fields, used) = decode(clean, &entry.body);
    let expected = field_count(clean);
    if fields.len() < expected {
        return Err(format!("o pacote acabou no campo {} de {expected}", fields.len() + 1));
    }
    let mut text = fields.iter().map(|field| format!("{:04X}  {}  {}", field.offset, field.kind, field.text)).collect::<Vec<_>>().join("\r\n");
    if used < entry.body.len() {
        text.push_str(&format!("\r\n(+{} bytes sem formato)", entry.body.len() - used));
    }
    Ok(text)
}

pub const PARSER_HELP: &str = "c = byte\r\nh = short (2 bytes)\r\nd = int (4 bytes)\r\nf = double (8 bytes)\r\nQ = int64 (8 bytes)\r\nS = texto unicode terminado em zero\r\nd*3 = repete 3 vezes\r\nExemplo: cdhS";

pub fn convert(text: &str) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    let trimmed = text.trim();
    if let Some(bytes) = parse_hex(trimmed) {
        if bytes.len() >= 4 {
            rows.push(("int32".to_owned(), i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]).to_string()));
        }
        if bytes.len() >= 2 {
            rows.push(("int16".to_owned(), i16::from_le_bytes([bytes[0], bytes[1]]).to_string()));
        }
        if bytes.len() >= 8 {
            rows.push(("double".to_owned(), f64::from_le_bytes(bytes[..8].try_into().unwrap_or([0; 8])).to_string()));
        }
        let units: Vec<u16> = bytes.chunks_exact(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]])).take_while(|unit| *unit != 0).collect();
        rows.push(("texto".to_owned(), String::from_utf16_lossy(&units)));
    }
    if let Ok(number) = trimmed.parse::<i32>() {
        rows.push(("hex (le)".to_owned(), number.to_le_bytes().iter().map(|byte| format!("{byte:02X}")).collect::<Vec<_>>().join(" ")));
    }
    if !trimmed.is_empty() {
        rows.push(("texto em hex (unicode)".to_owned(), trimmed.encode_utf16().flat_map(u16::to_le_bytes).map(|byte| format!("{byte:02X}")).collect::<Vec<_>>().join(" ")));
    }
    rows
}

pub fn save_filters(filters: &Filters) -> String {
    let mut text = format!("whitelist\t{}\n", u8::from(filters.whitelist));
    for id in &filters.server {
        text.push_str(&format!("S\t{id:#06x}\n"));
    }
    for id in &filters.client {
        text.push_str(&format!("C\t{id:#06x}\n"));
    }
    text
}

pub fn load_filters(text: &str) -> Filters {
    let mut filters = Filters::default();
    for line in text.lines() {
        let Some((kind, value)) = line.split_once('\t') else { continue };
        match kind {
            "whitelist" => filters.whitelist = value.trim() == "1",
            "S" => filters.server.extend(parse_id(value)),
            "C" => filters.client.extend(parse_id(value)),
            _ => {}
        }
    }
    filters
}

pub const ROW_LIMIT: usize = 400;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RowDto {
    pub number: u64,
    pub client: bool,
    pub time: String,
    pub size: usize,
    pub name: String,
    pub opcode: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FilterEntry {
    pub id: u16,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FiltersDto {
    pub server: Vec<FilterEntry>,
    pub client: Vec<FilterEntry>,
    pub whitelist: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PacketsDto {
    pub recording: bool,
    pub total: usize,
    pub shown: usize,
    pub rows: Vec<RowDto>,
    pub selected: Option<u64>,
    pub detail: String,
    pub status: String,
    pub filters: FiltersDto,
    pub filter_on: bool,
    pub hide_server: bool,
    pub hide_client: bool,
    pub converter: Vec<(String, String)>,
    pub help: &'static str,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PacketUi {
    pub filters: Filters,
    pub filter_on: bool,
    pub hide_server: bool,
    pub hide_client: bool,
    pub selected: Option<u64>,
    pub detail: Option<String>,
    pub status: String,
    pub converter: Vec<(String, String)>,
}

impl PacketUi {
    pub fn visible<'a>(&self, log: &'a PacketLog) -> Vec<&'a Logged> {
        log.entries().filter(|entry| passes(entry, &self.filters, self.filter_on, self.hide_server, self.hide_client)).collect()
    }

    pub fn dto(&self, log: &PacketLog) -> PacketsDto {
        let visible = self.visible(log);
        let shown = visible.len();
        let selected = self.selected.and_then(|number| visible.iter().find(|entry| entry.number == number));
        let detail = self.detail.clone().or_else(|| selected.map(|entry| entry.detail())).unwrap_or_default();
        let rows = visible
            .iter()
            .skip(shown.saturating_sub(ROW_LIMIT))
            .map(|entry| RowDto { number: entry.number, client: entry.outgoing, time: time_text(entry.at), size: entry.body.len(), name: entry.name(), opcode: format!("0x{:02X}", entry.opcode()) })
            .collect();
        let entries = |list: &[u16], server: bool| list.iter().map(|id| FilterEntry { id: *id, name: name_of(server, *id) }).collect();
        PacketsDto {
            recording: log.recording,
            total: log.len(),
            shown,
            rows,
            selected: self.selected,
            detail,
            status: self.status.clone(),
            filters: FiltersDto { server: entries(&self.filters.server, true), client: entries(&self.filters.client, false), whitelist: self.filters.whitelist },
            filter_on: self.filter_on,
            hide_server: self.hide_server,
            hide_client: self.hide_client,
            converter: self.converter.clone(),
            help: PARSER_HELP,
        }
    }

    pub fn select(&mut self, number: u64) {
        self.selected = Some(number);
        self.detail = None;
        self.status = "...".to_owned();
    }

    pub fn find(&mut self, log: &PacketLog, query: &str, client: bool, by_name: bool, direction: &str) {
        let visible = self.visible(log);
        let current = self.selected.and_then(|number| visible.iter().position(|entry| entry.number == number));
        let (from, forward) = match direction {
            "first" => (None, true),
            "prev" => (current, false),
            _ => (current, true),
        };
        match search(&visible, from, forward, query, client, by_name) {
            Some(index) => {
                self.selected = Some(visible[index].number);
                self.detail = None;
                self.status = format!("achado no pacote {}", visible[index].number);
            }
            None => self.status = "nada encontrado".to_owned(),
        }
    }

    pub fn parse(&mut self, log: &PacketLog, format: &str) {
        let visible = self.visible(log);
        let Some(entry) = self.selected.and_then(|number| visible.iter().find(|entry| entry.number == number)) else {
            self.status = "escolha um pacote".to_owned();
            return;
        };
        match parse_with(format, entry) {
            Ok(text) => {
                self.detail = Some(text);
                self.status = "ok".to_owned();
            }
            Err(error) => self.status = error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(number: u64, outgoing: bool, body: &[u8]) -> Logged {
        Logged { number, outgoing, at: Duration::from_millis(3_723_456), body: body.to_vec() }
    }

    #[test]
    fn times_ids_and_names_are_shown_like_the_original_columns() {
        assert_eq!(time_text(Duration::from_millis(3_723_456)), "01:02:03.456");
        assert_eq!(parse_id("0x1F"), Some(31));
        assert_eq!(parse_id("1f"), Some(31));
        assert_eq!(parse_id("31"), Some(31));
        assert_eq!(parse_id(""), None);
        assert_eq!(parse_id("zz"), None);
        assert_eq!(name_of(true, 0x05), "AttackPacket");
        assert_eq!(name_of(true, 0xfff), "?");
    }

    #[test]
    fn filters_hide_by_side_and_work_as_black_or_white_list() {
        let server = entry(1, false, &[0x05, 0, 0, 0, 0]);
        let client = entry(2, true, &[0x09]);
        let mut filters = Filters { server: vec![0x05], client: vec![], whitelist: false };
        assert!(passes(&server, &filters, false, false, false));
        assert!(!passes(&server, &filters, true, false, false));
        assert!(passes(&client, &filters, true, false, false));
        filters.whitelist = true;
        assert!(passes(&server, &filters, true, false, false));
        assert!(!passes(&client, &filters, true, false, false));
        assert!(!passes(&server, &Filters::default(), false, true, false));
        assert!(!passes(&client, &Filters::default(), false, false, true));
    }

    #[test]
    fn extended_server_packets_are_filtered_by_their_sub_opcode() {
        let ex = entry(1, false, &[0xfe, 0x01, 0x00, 0, 0, 0, 0]);
        assert_eq!(packet_id(&ex), 1);
        assert_eq!(packet_id(&entry(2, false, &[0x05])), 5);
    }

    #[test]
    fn search_walks_up_and_down_by_id_or_name_on_the_chosen_side() {
        let list = [entry(1, false, &[0x05]), entry(2, true, &[0x09]), entry(3, false, &[0x07]), entry(4, false, &[0x05])];
        let refs: Vec<&Logged> = list.iter().collect();
        assert_eq!(search(&refs, None, true, "0x05", false, false), Some(0));
        assert_eq!(search(&refs, Some(0), true, "5", false, false), Some(3));
        assert_eq!(search(&refs, Some(3), false, "5", false, false), Some(0));
        assert_eq!(search(&refs, None, true, "revive", false, true), Some(2));
        assert_eq!(search(&refs, None, true, "9", true, false), Some(1));
        assert_eq!(search(&refs, None, true, "9", false, false), None);
        assert_eq!(search(&refs, None, true, "", false, false), None);
    }

    #[test]
    fn the_parser_decodes_a_custom_format_and_explains_errors() {
        let mut body = vec![0x07u8];
        body.extend_from_slice(&9i32.to_le_bytes());
        let packet = entry(1, false, &body);
        let text = parse_with("cd", &packet).unwrap();
        assert!(text.contains("0001  d  9"));
        assert!(parse_with("c", &packet).unwrap().contains("+4 bytes"));
        assert!(parse_with("cddd", &packet).unwrap_err().contains("acabou"));
        assert!(parse_with("xyz", &packet).unwrap_err().contains("inválido"));
        assert!(parse_with("", &packet).is_err());
    }

    #[test]
    fn the_converter_reads_hex_numbers_and_text() {
        let from_hex = convert("2A 00 00 00 41 00");
        assert!(from_hex.iter().any(|(label, value)| label == "int32" && value == "42"));
        assert!(from_hex.iter().any(|(label, value)| label == "texto" && value.starts_with('*')));
        let from_number = convert("42");
        assert!(from_number.iter().any(|(label, value)| label == "hex (le)" && value == "2A 00 00 00"));
        assert!(convert("").is_empty());
    }

    #[test]
    fn the_window_state_lists_the_visible_packets_and_follows_the_search() {
        let mut log = PacketLog::default();
        log.recording = true;
        for opcode in [0x05u8, 0x09, 0x07, 0x05] {
            log.add(killer_world::PacketRecord { outgoing: opcode == 0x09, body: vec![opcode, 0, 0, 0, 0] }, Duration::from_millis(5));
        }
        let mut ui = PacketUi::default();
        assert_eq!(ui.dto(&log).rows.len(), 4);
        ui.find(&log, "5", false, false, "first");
        assert_eq!(ui.selected, Some(1));
        ui.find(&log, "5", false, false, "next");
        assert_eq!(ui.selected, Some(4));
        assert!(ui.dto(&log).detail.contains("server -> client"));
        ui.parse(&log, "cd");
        assert!(ui.dto(&log).detail.contains("0001  d  0"));
        ui.hide_server = true;
        assert_eq!(ui.dto(&log).shown, 1);
    }

    #[test]
    fn filter_files_round_trip() {
        let filters = Filters { server: vec![5, 0x0e], client: vec![0x09], whitelist: true };
        assert_eq!(load_filters(&save_filters(&filters)), filters);
        assert_eq!(load_filters("lixo\nS\tzz\n"), Filters::default());
    }
}
