use crate::state::RadarState;
use std::collections::{BTreeMap, BTreeSet};

pub const INPUT_SECTION: &str = "Engine.Input";
pub const LINE_MAX: i32 = 0x200;
pub const ON_RELEASE: &str = "OnRelease";

pub const KEY_NAMES: &[(u32, &str)] = &[
    (0x08, "Backspace"),
    (0x09, "Tab"),
    (0x0d, "Enter"),
    (0x10, "Shift"),
    (0x11, "Ctrl"),
    (0x12, "Alt"),
    (0x1b, "Escape"),
    (0x20, "Space"),
    (0x21, "PageUp"),
    (0x22, "PageDown"),
    (0x23, "End"),
    (0x24, "Home"),
    (0x30, "0"),
    (0x31, "1"),
    (0x32, "2"),
    (0x33, "3"),
    (0x34, "4"),
    (0x35, "5"),
    (0x36, "6"),
    (0x37, "7"),
    (0x38, "8"),
    (0x39, "9"),
    (0x41, "A"),
    (0x42, "B"),
    (0x43, "C"),
    (0x44, "D"),
    (0x45, "E"),
    (0x46, "F"),
    (0x47, "G"),
    (0x48, "H"),
    (0x49, "I"),
    (0x4a, "J"),
    (0x4b, "K"),
    (0x4c, "L"),
    (0x4d, "M"),
    (0x4e, "N"),
    (0x4f, "O"),
    (0x50, "P"),
    (0x51, "Q"),
    (0x52, "R"),
    (0x53, "S"),
    (0x54, "T"),
    (0x55, "U"),
    (0x56, "V"),
    (0x57, "W"),
    (0x58, "X"),
    (0x59, "Y"),
    (0x5a, "Z"),
    (0x70, "F1"),
    (0x71, "F2"),
    (0x72, "F3"),
    (0x73, "F4"),
    (0x74, "F5"),
    (0x75, "F6"),
    (0x76, "F7"),
    (0x77, "F8"),
    (0x78, "F9"),
    (0x79, "F10"),
    (0x7a, "F11"),
    (0x7b, "F12"),
    (0xbb, "Equals"),
    (0xbd, "Minus"),
    (0xc0, "Tilde"),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputAction {
    Press,
    Hold,
    Release,
}

pub fn key_name(vk: u32) -> Option<&'static str> {
    KEY_NAMES.iter().find(|(code, _)| *code == vk).map(|(_, name)| *name)
}

fn is_alnum(c: char) -> bool {
    c.is_ascii_alphanumeric()
}

pub fn parse_line(stream: &mut &str) -> Option<String> {
    let mut line = String::new();
    let mut got = false;
    let mut quoted = false;
    let mut ignore = false;
    let mut budget = LINE_MAX;
    let mut rest = *stream;
    while let Some(c) = rest.chars().next() {
        if c == '\n' || c == '\r' {
            break;
        }
        budget -= 1;
        if budget <= 0 {
            break;
        }
        let after = &rest[c.len_utf8()..];
        if !quoted {
            if c == '/' && after.starts_with('/') {
                ignore = true;
            }
            if c == '|' {
                break;
            }
        }
        quoted ^= c == '"';
        got = true;
        if !ignore {
            line.push(c);
        }
        rest = after;
    }
    rest = rest.trim_start_matches(['\n', '\r', '|']);
    *stream = rest;
    (!rest.is_empty() || got).then_some(line)
}

pub fn parse_command(stream: &mut &str, word: &str) -> bool {
    let original = *stream;
    let text = original.trim_start_matches([' ', '\t']);
    *stream = text;
    let Some(head) = text.get(..word.len()) else { return false };
    if !head.eq_ignore_ascii_case(word) {
        return false;
    }
    let after = &text[word.len()..];
    if after.chars().next().is_some_and(is_alnum) {
        return false;
    }
    *stream = after.trim_start_matches([' ', '\t']);
    true
}

pub fn find(text: &str, key: &str) -> Option<usize> {
    let mut previous_alnum = false;
    for (index, c) in text.char_indices() {
        if !previous_alnum && text.get(index..index + key.len()).is_some_and(|head| head.eq_ignore_ascii_case(key)) {
            return Some(index);
        }
        let upper = c.to_ascii_uppercase();
        previous_alnum = upper.is_ascii_uppercase() || upper.is_ascii_digit();
    }
    None
}

pub fn atof(text: &str) -> f64 {
    let text = text.trim_start_matches([' ', '\t', '\n', '\x0b', '\x0c', '\r']);
    let bytes = text.as_bytes();
    let mut end = 0;
    if matches!(bytes.first(), Some(b'+' | b'-')) {
        end = 1;
    }
    let digits_start = end;
    while bytes.get(end).is_some_and(u8::is_ascii_digit) {
        end += 1;
    }
    let mut digits = end - digits_start;
    if bytes.get(end) == Some(&b'.') {
        end += 1;
        let fraction = end;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        digits += end - fraction;
    }
    if digits == 0 {
        return 0.0;
    }
    if matches!(bytes.get(end), Some(b'e' | b'E')) {
        let mut exponent = end + 1;
        if matches!(bytes.get(exponent), Some(b'+' | b'-')) {
            exponent += 1;
        }
        let exponent_digits = exponent;
        while bytes.get(exponent).is_some_and(u8::is_ascii_digit) {
            exponent += 1;
        }
        if exponent > exponent_digits {
            end = exponent;
        }
    }
    let number = &text[..end];
    let number = number.strip_suffix('.').unwrap_or(number);
    number.parse().unwrap_or(0.0)
}

pub fn parse_float(text: &str, key: &str) -> Option<f32> {
    let at = find(text, key)?;
    Some(atof(&text[at + key.len()..]) as f32)
}

pub fn exec_input_commands(bind: &str, action: InputAction) -> Vec<String> {
    let mut commands = Vec::new();
    let mut stream = bind;
    while let Some(line) = parse_line(&mut stream) {
        let mut command = line.as_str();
        if action == InputAction::Release {
            parse_command(&mut command, ON_RELEASE);
        }
        commands.push(command.to_string());
    }
    commands
}

pub fn exec_radar(radar: &mut RadarState, command: &str, has_pawn: bool) -> bool {
    let mut cmd = command;
    if parse_command(&mut cmd, "Arrow") && has_pawn {
        if parse_command(&mut cmd, "On") {
            radar.enabled = true;
        } else if parse_command(&mut cmd, "Off") {
            radar.enabled = false;
        }
        let mut target = radar.target.unwrap_or_default();
        let mut written = false;
        for (axis, key) in ["X=", "Y=", "Z="].into_iter().enumerate() {
            if let Some(value) = parse_float(cmd, key) {
                target[axis] = value;
                written = true;
            }
        }
        if written || radar.target.is_some() {
            radar.target = Some(target);
        }
        return true;
    }
    if parse_command(&mut cmd, "RadarTime") {
        radar.show_time = !radar.show_time;
        return true;
    }
    false
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct KeyBinds {
    binds: BTreeMap<u32, String>,
    held: BTreeSet<u32>,
}

impl KeyBinds {
    pub fn from_ini<'a>(get: impl Fn(&str) -> Option<&'a str>) -> Self {
        let binds = KEY_NAMES
            .iter()
            .filter_map(|(vk, name)| get(name).filter(|bind| !bind.is_empty()).map(|bind| (*vk, bind.to_string())))
            .collect();
        Self { binds, held: BTreeSet::new() }
    }

    fn process(&self, vk: u32, action: InputAction) -> Vec<String> {
        self.binds.get(&vk).map(|bind| exec_input_commands(bind, action)).unwrap_or_default()
    }

    pub fn press(&mut self, vk: u32) -> Vec<String> {
        if !self.held.insert(vk) {
            return Vec::new();
        }
        self.process(vk, InputAction::Press)
    }

    pub fn release(&mut self, vk: u32) -> Vec<String> {
        if !self.held.remove(&vk) {
            return Vec::new();
        }
        self.process(vk, InputAction::Release)
    }

    pub fn hold(&self, delta_seconds: f32) -> Vec<String> {
        if delta_seconds == 0.0 {
            return Vec::new();
        }
        self.held.iter().flat_map(|vk| self.process(*vk, InputAction::Hold)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_line_10155950_splits_on_bars_skips_comments_and_keeps_quoted_bars() {
        let mut stream = "Arrow On | RadarTime|| Zoom \"a|b\" | x // y | z\nrest";
        assert_eq!(parse_line(&mut stream).as_deref(), Some("Arrow On "));
        assert_eq!(parse_line(&mut stream).as_deref(), Some(" RadarTime"));
        assert_eq!(parse_line(&mut stream).as_deref(), Some(" Zoom \"a|b\" "));
        assert_eq!(parse_line(&mut stream).as_deref(), Some(" x "));
        assert_eq!(parse_line(&mut stream).as_deref(), Some(" z"));
        assert_eq!(parse_line(&mut stream).as_deref(), Some("rest"));
        assert_eq!(parse_line(&mut stream), None);
        let mut empty = "";
        assert_eq!(parse_line(&mut empty), None);
    }

    #[test]
    fn parse_line_stops_after_511_characters() {
        let long = "a".repeat(600);
        let mut stream = long.as_str();
        assert_eq!(parse_line(&mut stream).map(|line| line.len()), Some(511));
        assert_eq!(stream.len(), 89);
    }

    #[test]
    fn parse_command_101555a0_needs_a_whole_word_and_eats_the_spaces() {
        let mut cmd = "  arrow  on x=1";
        assert!(parse_command(&mut cmd, "Arrow"));
        assert_eq!(cmd, "on x=1");
        assert!(parse_command(&mut cmd, "On"));
        assert_eq!(cmd, "x=1");
        let mut word = " Arrows";
        assert!(!parse_command(&mut word, "Arrow"));
        assert_eq!(word, "Arrows");
        let mut short = "Arr";
        assert!(!parse_command(&mut short, "Arrow"));
        let mut tail = "RadarTime-1";
        assert!(parse_command(&mut tail, "RadarTime"));
        assert_eq!(tail, "-1");
    }

    #[test]
    fn parse_10155100_finds_the_key_at_a_word_start_and_reads_atof() {
        assert_eq!(parse_float("Arrow X=100.5 Y=-20", "X="), Some(100.5));
        assert_eq!(parse_float("Arrow X=100.5 Y=-20", "y="), Some(-20.0));
        assert_eq!(parse_float("Arrow MaxX=5 X=7", "X="), Some(7.0));
        assert_eq!(parse_float("Arrow X=abc", "X="), Some(0.0));
        assert_eq!(parse_float("Arrow X= 3e2z", "X="), Some(300.0));
        assert_eq!(parse_float("Arrow", "X="), None);
        assert_eq!(parse_float("_X=4", "X="), Some(4.0));
    }

    #[test]
    fn atof_takes_the_longest_number_prefix() {
        assert_eq!(atof("12.5abc"), 12.5);
        assert_eq!(atof("  -7."), -7.0);
        assert_eq!(atof(".5"), 0.5);
        assert_eq!(atof("1e"), 1.0);
        assert_eq!(atof("-"), 0.0);
        assert_eq!(atof("2E-1"), 0.2);
    }

    #[test]
    fn exec_input_commands_10572280_routes_press_hold_and_on_release() {
        let bind = "Arrow On | OnRelease RadarTime| RadarTime";
        assert_eq!(exec_input_commands(bind, InputAction::Press), ["Arrow On ", " OnRelease RadarTime", " RadarTime"]);
        assert_eq!(exec_input_commands(bind, InputAction::Hold), ["Arrow On ", " OnRelease RadarTime", " RadarTime"]);
        assert_eq!(exec_input_commands(bind, InputAction::Release), ["Arrow On ", "RadarTime", "RadarTime"]);
        let mut radar = RadarState::default();
        for command in exec_input_commands(bind, InputAction::Hold) {
            exec_radar(&mut radar, &command, true);
        }
        assert!(radar.enabled && radar.show_time);
    }

    #[test]
    fn arrow_10581188_sets_the_flag_and_edits_the_target_axis_by_axis() {
        let mut radar = RadarState::default();
        assert!(exec_radar(&mut radar, "Arrow On", true));
        assert!(radar.enabled);
        assert_eq!(radar.target, None);
        assert!(exec_radar(&mut radar, "arrow x=100 Z=-5", true));
        assert_eq!(radar.target, Some([100.0, 0.0, -5.0]));
        assert!(exec_radar(&mut radar, "Arrow Off Y=7.5", true));
        assert!(!radar.enabled);
        assert_eq!(radar.target, Some([100.0, 7.5, -5.0]));
        radar.apply_control(0, 0, [1, 2, 3]);
        assert!(exec_radar(&mut radar, "Arrow", true));
        assert_eq!((radar.enabled, radar.target), (true, Some([1.0, 2.0, 3.0])));
        assert!(!exec_radar(&mut radar, "Arrows On", true));
        assert!(radar.enabled);
    }

    #[test]
    fn arrow_without_a_pawn_falls_into_the_radar_time_test() {
        let mut radar = RadarState::default();
        assert!(!exec_radar(&mut radar, "Arrow On", false));
        assert!(!radar.enabled);
        assert!(exec_radar(&mut radar, "Arrow RadarTime", false));
        assert!(radar.show_time);
    }

    #[test]
    fn radar_time_10581360_toggles_the_console_flag() {
        let mut radar = RadarState::default();
        assert!(exec_radar(&mut radar, " RadarTime", true));
        assert!(radar.show_time);
        assert!(exec_radar(&mut radar, "radartime", true));
        assert!(!radar.show_time);
    }

    #[test]
    fn key_binds_follow_preprocess_and_hold_every_frame_in_key_order() {
        let ini = [("F12", "RadarTime"), ("A", "OnRelease Arrow On"), ("B", "")];
        let mut binds = KeyBinds::from_ini(|name| ini.iter().find(|(key, _)| *key == name).map(|(_, bind)| *bind));
        assert_eq!(binds.press(0x7b), ["RadarTime"]);
        assert!(binds.press(0x7b).is_empty());
        assert_eq!(binds.press(0x41), ["OnRelease Arrow On"]);
        assert!(binds.press(0x42).is_empty());
        assert_eq!(binds.hold(0.016), ["OnRelease Arrow On", "RadarTime"]);
        assert!(binds.hold(0.0).is_empty());
        assert_eq!(binds.release(0x41), ["Arrow On"]);
        assert!(binds.release(0x41).is_empty());
        assert_eq!(binds.release(0x7b), ["RadarTime"]);
        assert!(binds.hold(0.016).is_empty());
    }

    #[test]
    fn key_names_are_the_einputkey_names_without_ik() {
        assert_eq!(key_name(0x7b), Some("F12"));
        assert_eq!(key_name(0xc0), Some("Tilde"));
        assert_eq!(key_name(0x12), Some("Alt"));
        assert_eq!(key_name(0x5b), None);
    }
}
