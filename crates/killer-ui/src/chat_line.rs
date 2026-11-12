use crate::chat::{CHAT_TELL, CHAT_PREFIXES, TAB_CHANNELS};

pub const COMMAND_PREFIXES: [char; 2] = ['/', '\u{ff0f}'];
pub const TOKEN_SEPARATORS: [char; 4] = [' ', ',', '\t', '\n'];
pub const FULL_WIDTH_QUOTE: char = '\u{ff02}';
pub const TELL_NAME_MAX: usize = 24;
pub const TELL_NAMES_MAX: usize = 64;
pub const FLOOD_MESSAGE: u32 = 0x3c6;
pub const FLOOD_LIMITED_MESSAGES: [u32; 2] = [964, 1078];
pub const FLOOD_RELEASED_MESSAGE: u32 = 965;

const KIND_NORMAL: i32 = 0;
const KIND_SHOUT: i32 = 1;
const KIND_PARTY: i32 = 3;
const KIND_PLEDGE: i32 = 4;
const KIND_MARKET: i32 = 8;
const KIND_ALLY: i32 = 9;
const KIND_HERO: i32 = 17;
const KIND_PETITION: i32 = 6;
const KIND_PARTY_ROOM: i32 = 7;

pub const TAB_KINDS: [i32; 5] = TAB_CHANNELS;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Limits {
    pub limit: i32,
    pub threshold: i32,
    pub decay: i32,
    pub per_char: i32,
    pub per_line: i32,
    pub per_repeat: i32,
    pub factor: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatLimiter {
    points: i32,
    pub limited: bool,
    items: Vec<(String, i32)>,
    limit: i32,
    threshold: i32,
    decay: i32,
    per_char: i32,
    per_line: i32,
    per_repeat: i32,
    factor_bits: u32,
}

impl Default for ChatLimiter {
    fn default() -> Self {
        ChatLimiter {
            points: 0,
            limited: false,
            items: Vec::new(),
            limit: 20000,
            threshold: 10000,
            decay: 200,
            per_char: 60,
            per_line: 100,
            per_repeat: 2000,
            factor_bits: 0x3DCCCCCD,
        }
    }
}

impl ChatLimiter {
    pub fn clear(&mut self) {
        self.points = 0;
        self.limited = false;
        self.items.clear();
    }

    pub fn set_limits(&mut self, limits: Limits) {
        self.clear();
        self.limit = limits.limit;
        self.threshold = limits.threshold;
        self.decay = limits.decay;
        self.per_char = limits.per_char;
        self.per_line = limits.per_line;
        self.per_repeat = limits.per_repeat;
        self.factor_bits = limits.factor.to_bits();
    }

    pub fn tick(&mut self) -> bool {
        self.points = (self.points - self.decay).max(0);
        let mut under = self.points <= self.threshold;
        for item in &mut self.items {
            item.1 = (item.1 - self.decay).max(0);
            if self.threshold < item.1 {
                under = false;
            }
        }
        if self.limited && under {
            self.limited = false;
            return true;
        }
        false
    }

    pub fn add_line(&mut self, length: i32, text: &str) -> Vec<u32> {
        let mut messages = Vec::new();
        let old = self.points;
        let step = f64::from(self.per_char * length + self.per_line) + f64::from(old) * f64::from(f32::from_bits(self.factor_bits));
        self.points = old.wrapping_add(step as i64 as i32);
        if !self.limited && self.points > self.limit {
            self.limited = true;
            messages.extend(FLOOD_LIMITED_MESSAGES);
        }
        if let Some(item) = self.items.iter_mut().find(|item| item.0 == text) {
            let bonus = f64::from(item.1) * f64::from(f32::from_bits(self.factor_bits)) + f64::from(self.per_repeat);
            item.1 = item.1.wrapping_add(bonus as i64 as i32);
            if !self.limited && item.1 > self.limit {
                self.limited = true;
                messages.extend(FLOOD_LIMITED_MESSAGES);
            }
            return messages;
        }
        self.items.push((text.to_string(), self.per_repeat));
        messages
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TellNames {
    pub names: Vec<String>,
    pub index: i32,
    pub browsing: bool,
    pub last: String,
}

impl TellNames {
    pub fn clear(&mut self) {
        self.names.clear();
        self.browsing = false;
    }

    pub fn set_last(&mut self, name: &str) {
        if !name.is_empty() && name.encode_utf16().count() < TELL_NAME_MAX {
            self.last = format!("{name} ");
        }
    }

    pub fn add(&mut self, name: &str) {
        if name.is_empty() || name.encode_utf16().count() >= TELL_NAME_MAX || name.starts_with('-') {
            return;
        }
        if let Some(found) = self.names.iter().position(|known| known.to_lowercase() == name.to_lowercase()) {
            let known = self.names.remove(found);
            self.names.push(known);
        } else {
            if self.names.len() > TELL_NAMES_MAX - 1 {
                self.names.remove(0);
            }
            self.names.push(name.to_string());
        }
        self.index = self.names.len() as i32 - 1;
    }

    pub fn begin(&mut self, name: Option<&str>) -> Option<String> {
        let text = match name {
            Some(name) => Some(format!("{name} ")),
            None => self.names.last().map(|name| format!("{name} ")),
        };
        self.browsing = true;
        text
    }

    pub fn browse(&mut self, up: bool) -> Option<String> {
        if !self.browsing {
            return None;
        }
        let last = self.names.len() as i32 - 1;
        let tell = prefix_text(CHAT_TELL);
        if up {
            self.index = (self.index - 1).max(0);
            if self.names.is_empty() {
                return None;
            }
        } else {
            if self.index == last {
                self.index += 1;
                return Some(tell);
            }
            if self.index > last {
                return None;
            }
            self.index = (self.index + 1).min(last);
            if self.names.is_empty() {
                return None;
            }
        }
        let name = self.names.get(usize::try_from(self.index).ok()?)?;
        Some(format!("{tell}{name} "))
    }
}

pub fn prefix_text(kind: i32) -> String {
    CHAT_PREFIXES.iter().find(|(_, k)| *k == kind).map_or_else(String::new, |(c, _)| c.to_string())
}

fn prefix_kind(c: char) -> Option<i32> {
    CHAT_PREFIXES.iter().find(|(p, _)| *p == c).map(|(_, kind)| *kind)
}

fn is_prefix(c: char, kind: i32) -> bool {
    prefix_kind(c) == Some(kind)
}

pub fn is_tab_prefix(c: char) -> bool {
    [KIND_PARTY, KIND_PLEDGE, KIND_MARKET, KIND_ALLY].iter().any(|kind| is_prefix(c, *kind))
}

pub fn tab_prefix_kind(tab: usize) -> i32 {
    TAB_KINDS[tab.min(TAB_KINDS.len() - 1)]
}

pub fn tokens(line: &str) -> Vec<&str> {
    line.split(TOKEN_SEPARATORS).filter(|token| !token.is_empty()).collect()
}

pub fn expand_line(line: &str, expand: &dyn Fn(&str) -> Option<String>) -> String {
    let parts = tokens(line);
    let Some(first) = parts.first() else { return line.to_string() };
    let mut found = [false; 2];
    let mut words: Vec<String> = parts.iter().map(|part| (*part).to_string()).collect();
    let replace = |index: usize, words: &mut Vec<String>| -> bool {
        match words.get(index).and_then(|word| expand(word)) {
            Some(name) => {
                words[index] = name;
                true
            }
            None => false,
        }
    };
    found[0] = replace(0, &mut words);
    let mut out = if first.starts_with('$') {
        if found[0] { format!("${}", words[0]) } else { format!("$ {}", words[0]) }
    } else {
        words[0].clone()
    };
    let mut unchanged = true;
    if words.len() > 1 {
        found[1] = replace(1, &mut words);
    }
    if found[0] || found[1] {
        unchanged = false;
    }
    let mut index = 0usize;
    let mut next = 1usize;
    while next < words.len() {
        let slot = index % 2;
        if found[slot] {
            out.push_str(&words[next]);
        } else {
            out.push(' ');
            out.push_str(&words[next]);
        }
        next += 1;
        found[slot] = next < words.len() && replace(next, &mut words);
        index += 1;
        if found[0] || found[1] {
            unchanged = false;
        }
    }
    if unchanged { line.to_string() } else { out }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineEffect {
    Chat { kind: i32, text: String, target: String },
    Command(String),
    System(u32),
    Text(String),
    StoreSearch(String),
    Builder(String),
    Console(String),
}

pub const CONSOLE_TEXT_COLOR: u32 = 0xFFDCDCDC;
pub const SETCL_USAGE: &str = "[howto] ///setcl start end server sec char enter senten weight";
pub const YO_ECHO: &str = "search private store!!!!!!!!!!";
pub const BUILDER_STAT_ONLY_LEVEL: i32 = 9;

pub struct LineContext {
    pub tab: usize,
    pub old_chat: bool,
    pub builder_level: i32,
}

pub fn is_command(line: &str) -> bool {
    line.chars().next().is_some_and(|c| COMMAND_PREFIXES.contains(&c))
}

fn slashes(line: &str) -> usize {
    line.chars().take_while(|c| COMMAND_PREFIXES.contains(c)).count()
}

fn exec_line(text: &str, builder_level: i32, limiter: &mut ChatLimiter, effects: &mut Vec<LineEffect>) {
    let body: String = text.chars().skip(3).collect();
    let lower = body.to_lowercase();
    if lower.starts_with("yo") {
        let word = body.split(TOKEN_SEPARATORS).filter(|word| !word.is_empty()).nth(1).unwrap_or_default();
        effects.push(LineEffect::StoreSearch(word.to_string()));
        effects.push(LineEffect::Text(YO_ECHO.to_string()));
    } else if lower.starts_with("setcl") {
        let words: Vec<&str> = body.split(TOKEN_SEPARATORS).filter(|word| !word.is_empty()).skip(1).collect();
        let number = |index: usize| words.get(index).map(|word| crate::html::atoi_text(word));
        let required = [number(0), number(1), number(2), number(3), number(4)];
        let mut all = required.iter().all(Option::is_some);
        let enter = number(5).or_else(|| {
            all = false;
            number(0)
        });
        let senten = number(6).or_else(|| {
            all = false;
            number(0)
        });
        match words.get(7).filter(|_| all) {
            Some(weight) => limiter.set_limits(Limits {
                limit: required[0].unwrap_or(0),
                threshold: required[1].unwrap_or(0),
                decay: required[3].unwrap_or(0),
                per_char: required[4].unwrap_or(0),
                per_line: enter.unwrap_or(0),
                per_repeat: senten.unwrap_or(0),
                factor: weight.parse::<f32>().unwrap_or(0.0),
            }),
            None => effects.push(LineEffect::Text(SETCL_USAGE.to_string())),
        }
    }
    let after: String = text.chars().skip(4).collect();
    if builder_level != BUILDER_STAT_ONLY_LEVEL || after.eq_ignore_ascii_case("stat fps") {
        effects.push(LineEffect::Console(body));
    }
}

fn send(limiter: &mut ChatLimiter, effects: &mut Vec<LineEffect>, kind: i32, text: &str, target: &str, counted: bool) {
    if counted {
        let length = text.encode_utf16().count() as i32;
        let messages = limiter.add_line(length, text);
        effects.extend(messages.into_iter().map(LineEffect::System));
        if limiter.limited {
            effects.push(LineEffect::System(FLOOD_MESSAGE));
            return;
        }
    }
    effects.push(LineEffect::Chat { kind, text: text.to_string(), target: target.to_string() });
}

pub fn process_line(
    line: &str,
    context: &LineContext,
    limiter: &mut ChatLimiter,
    tells: &mut TellNames,
    expand: &dyn Fn(&str) -> Option<String>,
) -> Vec<LineEffect> {
    let mut effects = Vec::new();
    if is_command(line) {
        match slashes(line) {
            1 => effects.push(LineEffect::Command(line.to_string())),
            2 => {
                if context.builder_level != 0 {
                    effects.push(LineEffect::Builder(line.chars().skip(2).collect()));
                }
            }
            _ => {
                if context.builder_level != 0 {
                    exec_line(line, context.builder_level, limiter, &mut effects);
                }
            }
        }
        return effects;
    }
    let chat = expand_line(line, expand);
    let chars: Vec<char> = chat.chars().collect();
    let Some(&first) = chars.first() else { return effects };
    let mut kind = if context.old_chat { KIND_NORMAL } else { tab_prefix_kind(context.tab) };
    if is_prefix(first, KIND_SHOUT) {
        kind = KIND_SHOUT;
    } else if is_prefix(first, KIND_PLEDGE) {
        kind = KIND_PLEDGE;
    } else if is_prefix(first, KIND_PARTY) {
        kind = KIND_PARTY;
    } else if first == FULL_WIDTH_QUOTE || is_prefix(first, CHAT_TELL) {
        kind = CHAT_TELL;
    } else if is_prefix(first, KIND_PETITION) {
        kind = KIND_PETITION;
    } else if is_prefix(first, KIND_PARTY_ROOM) {
        kind = KIND_PARTY_ROOM;
    } else if is_prefix(first, KIND_MARKET) {
        kind = KIND_MARKET;
    } else if is_prefix(first, KIND_ALLY) {
        kind = KIND_ALLY;
    } else if is_prefix(first, KIND_HERO) {
        kind = KIND_HERO;
    }
    if !context.old_chat {
        if is_prefix(first, KIND_NORMAL) {
            kind = KIND_NORMAL;
        }
    } else if kind == KIND_NORMAL {
        send(limiter, &mut effects, KIND_NORMAL, &chat, "", true);
        return effects;
    }
    if kind == CHAT_TELL {
        let mut name_len = 0usize;
        if chars.get(1) != Some(&' ') {
            let mut cursor = 1usize;
            loop {
                if chars.get(cursor) == Some(&'\u{3000}') || chars.len().saturating_sub(1) <= name_len {
                    break;
                }
                cursor += 1;
                name_len += 1;
                if chars.get(cursor) == Some(&' ') {
                    break;
                }
            }
        }
        if name_len == 0 {
            send(limiter, &mut effects, CHAT_TELL, &chat, "", true);
            return effects;
        }
        let name: String = chars[1..1 + name_len].iter().collect();
        if chars.len() <= name_len + 2 {
            return effects;
        }
        let text: String = chars[name_len + 2..].iter().collect();
        tells.set_last(&name);
        tells.add(&name);
        send(limiter, &mut effects, CHAT_TELL, &text, &name, true);
        return effects;
    }
    let message_start = if context.old_chat || prefix_kind(first).is_some() || first == FULL_WIDTH_QUOTE { 1 } else { 0 };
    let message: String = chars.iter().skip(message_start).collect();
    if message.is_empty() {
        return effects;
    }
    let counted = matches!(kind, KIND_SHOUT | KIND_MARKET | KIND_NORMAL);
    send(limiter, &mut effects, kind, &message, "", counted);
    effects
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(line: &str, tab: usize, old_chat: bool) -> Vec<LineEffect> {
        let mut limiter = ChatLimiter::default();
        let mut tells = TellNames::default();
        process_line(line, &LineContext { tab, old_chat, builder_level: 0 }, &mut limiter, &mut tells, &|_| None)
    }

    fn chat(kind: i32, text: &str, target: &str) -> LineEffect {
        LineEffect::Chat { kind, text: text.to_string(), target: target.to_string() }
    }

    #[test]
    fn the_flood_control_blocks_at_twenty_thousand_points_and_releases_under_ten_thousand() {
        let mut limiter = ChatLimiter::default();
        let mut blocked = None;
        for round in 0..200 {
            let messages = limiter.add_line(200, &"x".repeat(200 + round % 3));
            if !messages.is_empty() {
                blocked = Some((round, messages));
                break;
            }
        }
        let (_, messages) = blocked.expect("limite alcançado");
        assert_eq!(messages, vec![964, 1078]);
        assert!(limiter.limited);
        let mut seconds = 0;
        while !limiter.tick() {
            seconds += 1;
            assert!(seconds < 10_000);
        }
        assert!(!limiter.limited);
        assert!(seconds > 10);
    }

    #[test]
    fn a_repeated_text_scores_two_thousand_and_grows_by_a_tenth_of_its_points() {
        let mut limiter = ChatLimiter::default();
        limiter.add_line(5, "hello");
        assert_eq!(limiter.items[0].1, 2000);
        limiter.add_line(5, "hello");
        assert_eq!(limiter.items[0].1, 2000 + 200 + 2000);
        assert_eq!(limiter.points, 840);
    }

    #[test]
    fn the_tick_fades_points_by_two_hundred_and_never_below_zero() {
        let mut limiter = ChatLimiter::default();
        limiter.add_line(1, "a");
        limiter.tick();
        assert_eq!(limiter.points, 0);
        assert_eq!(limiter.items[0].1, 1800);
    }

    #[test]
    fn the_old_chat_sends_the_whole_line_as_normal_without_a_prefix_and_strips_one_prefix_char_otherwise() {
        assert_eq!(run("hello", 2, true), vec![chat(0, "hello", "")]);
        assert_eq!(run("#hello", 0, true), vec![chat(3, "hello", "")]);
        assert_eq!(run("hello", 2, false), vec![chat(3, "hello", "")]);
        assert_eq!(run("~hello", 2, false), vec![chat(0, "hello", "")]);
        assert_eq!(run("!hi", 0, false), vec![chat(1, "hi", "")]);
        assert_eq!(run("%hi", 0, false), vec![chat(17, "hi", "")]);
        assert_eq!(run("#", 0, true), Vec::<LineEffect>::new());
    }

    #[test]
    fn a_whisper_takes_its_name_up_to_the_first_space_and_nothing_goes_without_text() {
        assert_eq!(run("\"bob hi there", 0, false), vec![chat(2, "hi there", "bob")]);
        assert_eq!(run("\"bob\u{3000}hi", 0, false), vec![chat(2, "hi", "bob")]);
        assert_eq!(run("\"bob", 0, false), Vec::<LineEffect>::new());
        assert_eq!(run("\"bob ", 0, false), Vec::<LineEffect>::new());
        assert_eq!(run("\" hi", 0, false), vec![chat(2, "\" hi", "")]);
        assert_eq!(run("\u{ff02}bob hi", 0, false), vec![chat(2, "hi", "bob")]);
    }

    #[test]
    fn the_party_pledge_and_ally_lines_are_not_counted_by_the_flood_control() {
        let mut limiter = ChatLimiter::default();
        let mut tells = TellNames::default();
        let context = LineContext { tab: 0, old_chat: false, builder_level: 0 };
        for _ in 0..500 {
            let effects = process_line("#aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", &context, &mut limiter, &mut tells, &|_| None);
            assert_eq!(effects.len(), 1);
        }
        assert!(!limiter.limited);
        let mut counted = Vec::new();
        for _ in 0..40 {
            counted.extend(process_line("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", &context, &mut limiter, &mut tells, &|_| None));
        }
        assert!(counted.contains(&LineEffect::System(FLOOD_MESSAGE)));
        assert!(counted.contains(&LineEffect::System(964)));
    }

    #[test]
    fn slash_lines_are_commands_and_double_slash_lines_are_builder_commands_a_player_cannot_run() {
        assert_eq!(run("/sit", 0, true), vec![LineEffect::Command("/sit".to_string())]);
        assert_eq!(run("\u{ff0f}sit", 0, true), vec![LineEffect::Command("\u{ff0f}sit".to_string())]);
        assert_eq!(run("//gmspeed 3", 0, true), Vec::<LineEffect>::new());
        assert_eq!(run("///yo name", 0, true), Vec::<LineEffect>::new());
        let gm = |line: &str, level: i32| {
            let mut limiter = ChatLimiter::default();
            let mut tells = TellNames::default();
            let effects = process_line(line, &LineContext { tab: 0, old_chat: true, builder_level: level }, &mut limiter, &mut tells, &|_| None);
            (effects, limiter)
        };
        assert_eq!(gm("//admin_help x", 1).0, vec![LineEffect::Builder("admin_help x".to_string())]);
        assert_eq!(gm("///yo bob", 1).0, vec![LineEffect::StoreSearch("bob".to_string()), LineEffect::Text(YO_ECHO.to_string()), LineEffect::Console("yo bob".to_string())]);
        assert_eq!(gm("///show all", 9).0, Vec::<LineEffect>::new());
        assert_eq!(gm("///show all", 3).0, vec![LineEffect::Console("show all".to_string())]);
        assert_eq!(gm("///setcl 1 2 3", 1).0, vec![LineEffect::Text(SETCL_USAGE.to_string()), LineEffect::Console("setcl 1 2 3".to_string())]);
        let (effects, limiter) = gm("///setcl 5000 100 7 50 100 20 30 0.5", 1);
        assert_eq!(effects, vec![LineEffect::Console("setcl 5000 100 7 50 100 20 30 0.5".to_string())]);
        let mut changed = limiter;
        assert_eq!(changed.add_line(10, "x"), Vec::<u32>::new());
        assert_eq!(changed.add_line(100, "y"), vec![964, 1078]);
    }

    #[test]
    fn target_words_in_a_chat_line_are_replaced_and_the_joins_follow_the_original_slots() {
        let expand = |word: &str| (word.eq_ignore_ascii_case("%target")).then(|| "Bob".to_string());
        assert_eq!(expand_line("hello", &expand), "hello");
        assert_eq!(expand_line("go %target now", &expand), "go Bobnow");
        assert_eq!(expand_line("%target hi", &expand), "Bobhi");
        assert_eq!(expand_line("a  b", &expand), "a  b");
        assert_eq!(expand_line("hi %target", &expand), "hi Bob");
        assert_eq!(expand_line("$hi %target", &expand), "$ $hi Bob");
    }

    #[test]
    fn the_tell_names_move_a_known_name_last_keep_sixty_four_and_walk_with_up_and_down() {
        let mut tells = TellNames::default();
        for index in 0..70 {
            tells.add(&format!("n{index}"));
        }
        assert_eq!(tells.names.len(), 64);
        assert_eq!(tells.names[0], "n6");
        tells.add("N10");
        assert_eq!(tells.names.last().map(String::as_str), Some("n10"));
        assert_eq!(tells.names.len(), 64);
        tells.add("-hidden");
        tells.add(&"x".repeat(24));
        assert_eq!(tells.names.len(), 64);
        assert_eq!(tells.browse(true), None);
        assert_eq!(tells.begin(None), Some("n10 ".to_string()));
        assert_eq!(tells.browse(true), Some("\"n69 ".to_string()));
        assert_eq!(tells.browse(true), Some("\"n68 ".to_string()));
        assert_eq!(tells.browse(false), Some("\"n69 ".to_string()));
        assert_eq!(tells.browse(false), Some("\"n10 ".to_string()));
        assert_eq!(tells.browse(false), Some("\"".to_string()));
        assert_eq!(tells.browse(false), None);
    }
}
