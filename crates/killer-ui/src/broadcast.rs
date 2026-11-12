use crate::color::{Argb, TEXT};
use crate::detail_status::class_name_sysstring;
use crate::draw::DrawList;
use crate::true_font::LineFont;
use crate::window::Rect;

pub const TEXTURE_PREFIX: &str = "L2UI_CH3.BroadcastObs.";
pub const DEBUFF_FRAME: &str = "L2UI_CH3.PlayerStatusWnd.debuff";
pub const TEAMS: usize = 2;
pub const USERS: usize = 9;
pub const RED: Argb = Argb(0xFFFF3737);
pub const WHITE: Argb = TEXT;
pub const YELLOW: Argb = Argb(0xFFFFD23C);
pub const BLACK: Argb = Argb(0xFF000000);

pub const TOP_HEIGHT: i32 = 113;
pub const TOP_MARGIN: i32 = 60;
pub const TOP_STRIP_HEIGHT: i32 = 128;
pub const TOP_CENTER_SPAN: i32 = 420;
pub const SCORE_SIZE: i32 = 128;
pub const SCORE_Y: i32 = 28;
pub const SCORE_FIRST_DX: i32 = -105;
pub const SCORE_SECOND_DX: i32 = 5;
pub const COLON_DX: i32 = -51;
pub const COLON_Y: i32 = 18;
pub const TOP_NAME_Y: i32 = 40;
pub const TOP_SECOND_NAME_DX: i32 = 520;
pub const TOP_NAME_WRAP_EXTRA: i32 = 0x82;
pub const TOP_SLIDE_STEP: i32 = 3;
pub const TOP_HOLD_MS: i32 = 5000;

pub const PLAYER_SIZE: (i32, i32) = (520, 224);
pub const PLAYER_Y: i32 = 0x62;
pub const PLAYER_STRIP: (i32, i32) = (32, 256);
pub const PLAYER_LEVEL_POS: (i32, i32) = (20, 10);
pub const PLAYER_NAME_POS: (i32, i32) = (20, 40);
pub const PLAYER_BAR_X: i32 = 10;
pub const PLAYER_BAR_WIDTH: i32 = 500;
pub const PLAYER_BAR_HEIGHT: i32 = 12;
pub const PLAYER_BAR_YS: [i32; 3] = [98, 112, 126];

pub const BUFF_SIZE: (i32, i32) = (400, 80);
pub const BUFF_CELL: i32 = 40;
pub const BUFF_COLUMNS: usize = 10;
pub const BUFF_MAX: usize = 30;
pub const BUFF_FRAME_UV: i32 = 26;
pub const BUFF_OFFSET: i32 = 10;

pub const PARTY_LIST_SIZE: (i32, i32) = (280, 630);
pub const PARTY_LIST_Y: i32 = 0x154;
pub const PARTY_ROW: i32 = 70;
pub const PARTY_STRIP: (i32, i32) = (32, 128);
pub const PARTY_ICON_SIZE: i32 = 64;
pub const PARTY_ICON_POS: [(i32, i32); 2] = [(7, 9), (231, 9)];
pub const PARTY_TEXT_X: [i32; 2] = [56, 5];
pub const PARTY_TEXT_Y: i32 = 5;
pub const PARTY_BAR_WIDTH: i32 = 220;
pub const PARTY_TEXT_MARGIN: i32 = 61;
pub const PARTY_ALPHA_UNSELECTED: f32 = 128.0 / 255.0;

pub const MSG_Y: i32 = 971;
pub const MSG_HEIGHT: i32 = 236;
pub const MSG_RING: usize = 6;
pub const MSG_LINE: i32 = 30;
pub const MSG_TTL: i32 = 10;
pub const MSG_X: i32 = 20;

pub const GM_MSG_SIZE: (i32, i32) = (720, 88);
pub const GM_MSG_STRIP_HEIGHT: i32 = 128;
pub const GM_MSG_TEXT_Y: i32 = 15;
pub const GM_MSG_HOLD_MS: i32 = 5000;

pub const SPECIAL_SIZE: (i32, i32) = (720, 320);
pub const SPECIAL_STRIP_HEIGHT: i32 = 512;
pub const SPECIAL_Y: i32 = 32;
pub const SPECIAL_START_HOLD_MS: i32 = 1000;
pub const SPECIAL_HOLD_MS: i32 = 5000;

pub fn texture(name: &str) -> String {
    format!("{TEXTURE_PREFIX}{name}")
}

fn indexed(name: &str, number: usize) -> String {
    format!("{TEXTURE_PREFIX}{name}{number}")
}

pub fn top_rect(parent_width: i32) -> Rect {
    Rect::new(TOP_MARGIN, 0, parent_width - 2 * TOP_MARGIN, TOP_HEIGHT)
}

pub fn player_rect(team: usize, parent_width: i32) -> Rect {
    let x = if team == 0 { 0 } else { parent_width - PLAYER_SIZE.0 };
    Rect::new(x, PLAYER_Y, PLAYER_SIZE.0, PLAYER_SIZE.1)
}

pub fn party_list_rect(team: usize, parent_width: i32, visible_rows: i32) -> Rect {
    let x = if team == 0 { 0 } else { parent_width - PARTY_LIST_SIZE.0 };
    Rect::new(x, PARTY_LIST_Y, PARTY_LIST_SIZE.0, PARTY_ROW * visible_rows)
}

pub fn msg_rect(parent_width: i32) -> Rect {
    Rect::new(0, MSG_Y, parent_width, MSG_HEIGHT)
}

pub fn gm_msg_rect(viewport: (i32, i32), state: i32) -> Rect {
    let x = (viewport.0 as f32 * 0.5 - 360.0) as i32;
    let y = if state == 1 { 834 } else { (viewport.1 as f32 * 0.5 + 100.0) as i32 };
    Rect::new(x, y, GM_MSG_SIZE.0, GM_MSG_SIZE.1)
}

pub fn special_rect(viewport: (i32, i32)) -> Rect {
    Rect::new((viewport.0 as f32 * 0.5 - 360.0) as i32, 390, SPECIAL_SIZE.0, SPECIAL_SIZE.1)
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BroadcastUser {
    pub id: i32,
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub cp: i32,
    pub max_cp: i32,
    pub level: i32,
    pub class_id: i32,
    pub skills: Vec<i32>,
    pub times: Vec<i32>,
}

impl BroadcastUser {
    pub fn dead(&self) -> bool {
        self.hp <= 0
    }

    fn set_stats(&mut self, values: &[i32]) {
        self.hp = values[0];
        self.max_hp = values[1];
        self.mp = values[2];
        self.max_mp = values[3];
        self.cp = values[4];
        self.max_cp = values[5];
        self.level = values[6];
        self.class_id = values[7];
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BroadcastTeam {
    pub score: i32,
    pub name: String,
    pub users: [Option<BroadcastUser>; USERS],
    pub retained: [(Vec<i32>, Vec<i32>); USERS],
}

impl BroadcastTeam {
    pub fn free_slot(&mut self, index: usize) -> Option<i32> {
        let user = self.users.get_mut(index)?.take()?;
        self.retained[index] = (user.skills, user.times);
        Some(user.id)
    }

    pub fn free_all(&mut self) -> Vec<i32> {
        (0..USERS).filter_map(|index| self.free_slot(index)).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgEntry {
    pub caster: String,
    pub target: String,
    pub skill: String,
    pub caster_team: i32,
    pub target_team: i32,
    pub caster_index: i32,
    pub target_index: i32,
}

impl MsgEntry {
    pub fn empty() -> Self {
        Self {
            caster: String::new(),
            target: String::new(),
            skill: String::new(),
            caster_team: -1,
            target_team: -1,
            caster_index: -1,
            target_index: -1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MsgRing {
    pub entries: [MsgEntry; MSG_RING],
    pub index: usize,
}

impl Default for MsgRing {
    fn default() -> Self {
        Self { entries: std::array::from_fn(|_| MsgEntry::empty()), index: 0 }
    }
}

impl MsgRing {
    fn push(&mut self, entry: MsgEntry) {
        self.entries[self.index] = entry;
        self.index = (self.index + 1) % MSG_RING;
    }

    pub fn ordered(&self) -> impl Iterator<Item = &MsgEntry> {
        (0..MSG_RING).map(move |k| &self.entries[(self.index + k) % MSG_RING])
    }

    fn oldest(&self, team: i32) -> Option<usize> {
        (0..MSG_RING).map(|k| (self.index + k) % MSG_RING).find(|&slot| {
            let entry = &self.entries[slot];
            if team == -1 { entry.caster_team != -1 } else { entry.caster_team == team }
        })
    }

    fn clear_oldest(&mut self, team: i32) -> bool {
        match self.oldest(team) {
            Some(slot) => {
                self.entries[slot] = MsgEntry::empty();
                true
            }
            None => false,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TopPhase {
    Idle,
    Down,
    Hold(i32),
    Up,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TopSlide {
    pub phase: TopPhase,
    pub y: i32,
}

impl TopSlide {
    pub fn show(&mut self) {
        self.phase = TopPhase::Down;
    }

    pub fn tick(&mut self, elapsed_ms: i32, height: i32) {
        let mut remaining = elapsed_ms;
        while remaining > 0 {
            match self.phase {
                TopPhase::Idle => return,
                TopPhase::Down => {
                    if self.y < 0 {
                        self.y += TOP_SLIDE_STEP;
                        remaining -= 1;
                    }
                    if self.y >= 0 {
                        self.phase = TopPhase::Hold(TOP_HOLD_MS);
                    }
                }
                TopPhase::Hold(left) => {
                    if remaining >= left {
                        remaining -= left;
                        self.phase = TopPhase::Up;
                    } else {
                        self.phase = TopPhase::Hold(left - remaining);
                        return;
                    }
                }
                TopPhase::Up => {
                    if self.y + height > 0 {
                        self.y -= TOP_SLIDE_STEP;
                        remaining -= 1;
                    } else {
                        self.phase = TopPhase::Idle;
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MsgMode {
    All,
    Teams,
    Off,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BroadcastObserver {
    pub game_id: i32,
    pub teams: [BroadcastTeam; TEAMS],
    pub filling: Option<usize>,
    pub spell: Option<(usize, usize)>,
    pub show_class_or_name: bool,
    pub selected: [Option<usize>; TEAMS],
    pub ring_all: MsgRing,
    pub ring_teams: [MsgRing; TEAMS],
    pub msg_mode: MsgMode,
    pub msg_ttl: i32,
    pub locked: bool,
    pub top: TopSlide,
    second_ms: i32,
    msg_ms: i32,
}

impl Default for BroadcastObserver {
    fn default() -> Self {
        Self {
            game_id: -1,
            teams: Default::default(),
            filling: None,
            spell: None,
            show_class_or_name: true,
            selected: [None; TEAMS],
            ring_all: MsgRing::default(),
            ring_teams: Default::default(),
            msg_mode: MsgMode::All,
            msg_ttl: -1,
            locked: false,
            top: TopSlide { phase: TopPhase::Idle, y: 0 },
            second_ms: 0,
            msg_ms: 0,
        }
    }
}

impl BroadcastObserver {
    pub fn user(&self, team: usize, index: usize) -> Option<&BroadcastUser> {
        self.teams.get(team)?.users.get(index)?.as_ref()
    }

    pub fn find_user(&self, id: i32) -> Option<(usize, usize)> {
        for (team, data) in self.teams.iter().enumerate() {
            for (index, user) in data.users.iter().enumerate() {
                if user.as_ref().is_some_and(|user| user.id == id) {
                    return Some((team, index));
                }
            }
        }
        None
    }

    pub fn selected_user(&self, team: usize) -> Option<&BroadcastUser> {
        self.user(team, self.selected.get(team).copied().flatten()?)
    }

    pub fn visible_rows(&self, team: usize) -> i32 {
        self.teams[team].users.iter().filter(|user| user.is_some()).count() as i32
    }

    pub fn start_observer(&mut self, game_id: i32, first: &str, second: &str) {
        self.game_id = game_id;
        self.teams[0].name = first.to_string();
        self.teams[1].name = second.to_string();
        for team in &mut self.teams {
            team.score = 0;
            team.free_all();
        }
    }

    pub fn score(&mut self, game_id: i32, first: i32, second: i32) -> bool {
        if self.game_id != game_id {
            return false;
        }
        self.teams[0].score = first;
        self.teams[1].score = second;
        self.top.show();
        true
    }

    pub fn team_info_start(&mut self, game_id: i32, flag: i32) {
        if self.game_id != game_id {
            return;
        }
        match flag {
            1 => self.filling = Some(0),
            2 => self.filling = Some(1),
            _ => {}
        }
    }

    pub fn team_info(&mut self, index: i32, user_id: i32, name: &str, values: &[i32]) {
        let Some(team) = self.filling else { return };
        if !(0..USERS as i32).contains(&index) || values.len() < 8 {
            return;
        }
        let slot = index as usize;
        let (skills, times) = match self.teams[team].users[slot].take() {
            Some(previous) => (previous.skills, previous.times),
            None => std::mem::take(&mut self.teams[team].retained[slot]),
        };
        let mut user = BroadcastUser { id: user_id, name: name.to_string(), skills, times, ..Default::default() };
        user.set_stats(values);
        self.teams[team].users[slot] = Some(user);
    }

    pub fn user_info(&mut self, user_id: i32, name: &str, values: &[i32]) {
        let Some((team, index)) = self.find_user(user_id) else { return };
        if values.len() < 8 {
            return;
        }
        if let Some(user) = self.teams[team].users[index].as_mut() {
            user.name = name.to_string();
            user.set_stats(values);
        }
    }

    pub fn spell_info_start(&mut self, game_id: i32, user_id: i32) {
        self.spell = if self.game_id == game_id { self.find_user(user_id) } else { None };
        if let Some((team, index)) = self.spell
            && let Some(user) = self.teams[team].users[index].as_mut()
        {
            user.skills.clear();
            user.times.clear();
        }
    }

    pub fn spell_info(&mut self, skill_id: i32, amount: i32) {
        let Some((team, index)) = self.spell else { return };
        if let Some(user) = self.teams[team].users[index].as_mut() {
            user.skills.push(skill_id);
            user.times.push(amount);
        }
    }

    pub fn spell_info_end(&mut self) -> Option<(usize, usize)> {
        self.spell.take()
    }

    pub fn select_user(&mut self, team: usize, index: usize) {
        if team < TEAMS && index < USERS {
            self.selected[team] = Some(index);
        }
    }

    pub fn add_skill_msg(&mut self, caster: (i32, usize, usize), target: (i32, usize, usize), skill: &str, names: (&str, &str)) {
        let _ = caster.0;
        let entry = MsgEntry {
            caster: names.0.to_string(),
            target: names.1.to_string(),
            skill: skill.to_string(),
            caster_team: caster.1 as i32,
            target_team: target.1 as i32,
            caster_index: caster.2 as i32,
            target_index: target.2 as i32,
        };
        self.ring_all.push(entry.clone());
        match caster.1 {
            0 | 1 => self.ring_teams[caster.1].push(entry),
            _ => {}
        }
        self.msg_ttl = MSG_TTL;
    }

    pub fn switch_msg_mode(&mut self) {
        match self.msg_mode {
            MsgMode::All => {
                self.msg_mode = MsgMode::Teams;
                self.msg_ttl = MSG_TTL;
            }
            MsgMode::Teams => self.msg_mode = MsgMode::Off,
            MsgMode::Off => {
                self.msg_mode = MsgMode::All;
                self.msg_ttl = MSG_TTL;
            }
        }
    }

    pub fn toggle_class_or_name(&mut self) {
        self.show_class_or_name = !self.show_class_or_name;
    }

    pub fn toggle_lock(&mut self) {
        self.locked = !self.locked;
    }

    pub fn tick(&mut self, elapsed_ms: i32) {
        self.top.tick(elapsed_ms, TOP_HEIGHT);
        self.second_ms += elapsed_ms;
        while self.second_ms >= 1000 {
            self.second_ms -= 1000;
            for team in &mut self.teams {
                for user in team.users.iter_mut().flatten() {
                    for time in &mut user.times {
                        *time -= 1;
                    }
                }
                for (_, times) in &mut team.retained {
                    for time in times {
                        *time -= 1;
                    }
                }
            }
        }
        self.msg_ms += elapsed_ms;
        while self.msg_ms >= 1000 {
            self.msg_ms -= 1000;
            self.msg_second();
        }
    }

    fn msg_second(&mut self) {
        if self.msg_ttl > 0 {
            self.msg_ttl -= 1;
            return;
        }
        if self.msg_ttl != 0 {
            return;
        }
        match self.msg_mode {
            MsgMode::All => {
                if self.ring_all.clear_oldest(-1) {
                    self.msg_ttl = MSG_TTL;
                }
            }
            MsgMode::Teams => {
                let first = self.ring_teams[0].clear_oldest(0);
                let second = self.ring_teams[1].clear_oldest(1);
                if first || second {
                    self.msg_ttl = MSG_TTL;
                }
            }
            MsgMode::Off => {}
        }
    }

    pub fn team_unlocked(&mut self, game_id: i32, code: i32) -> Option<(usize, Vec<i32>)> {
        if self.game_id != game_id {
            return None;
        }
        let team = match code {
            1 => 0,
            2 => 1,
            _ => return None,
        };
        let ids = self.teams[team].free_all();
        Some((team, ids))
    }

    pub fn team_info_end(&mut self) -> Option<usize> {
        self.filling.take()
    }
}

pub fn fill_width(value: i32, maximum: i32, full: i32) -> i32 {
    if maximum == 0 { 0 } else { (i64::from(value) * i64::from(full) / i64::from(maximum)) as i32 }
}

fn strip_at(list: &mut DrawList, origin: (i32, i32), names: [String; 3], width: i32, geometry: (i32, i32), alpha: f32) {
    let (side, height) = geometry;
    let (x, y) = origin;
    list.draw_texture(x, y, side, height, 0, 0, side, height, &names[0], alpha);
    list.draw_texture(x + side, y, width - 2 * side, height, 0, 0, side, height, &names[1], alpha);
    list.draw_texture(x + width - side, y, side, height, 0, 0, side, height, &names[2], alpha);
}

fn triple(prefix: &str) -> [String; 3] {
    [indexed(prefix, 1), indexed(prefix, 2), indexed(prefix, 3)]
}

pub const TOP_FONT_TYPE: i32 = 4;
pub const PLAYER_LEVEL_FONT_TYPE: i32 = 7;
pub const PLAYER_NAME_FONT_TYPE: i32 = 4;
pub const PARTY_FONT_TYPE: i32 = 7;
pub const GM_MSG_FONT_TYPE: i32 = 7;
pub const MSG_FONT_TYPE: i32 = 6;
pub const MSG_BACK_SOURCE: (i32, i32) = (0x20, 0x100);

pub fn paint_top(observer: &BroadcastObserver, list: &mut DrawList, origin: (i32, i32), width: i32, font: &dyn LineFont) {
    let (x, y) = origin;
    let a = (width as f32 * 0.5 - TOP_CENTER_SPAN as f32) as i32;
    let h = TOP_STRIP_HEIGHT;
    let pieces = [
        (1, 0, 256, 256),
        (2, 256, a, 256),
        (3, a + 256, 64, 64),
        (4, a + 320, 200, 32),
        (5, a + 520, 64, 64),
        (2, a + 584, a, 256),
        (6, a + 584 + a, 256, 256),
    ];
    for (number, dx, w, uw) in pieces {
        list.draw_texture(x + dx, y, w, h, 0, 0, uw, h, &indexed("br_top_back", number), 1.0);
    }
    let digit = |team: usize| {
        let score = observer.teams[team].score;
        indexed("br_score", if (0..=9).contains(&score) { score as usize } else { 0 })
    };
    let first_x = (width as f32 * 0.5 + SCORE_FIRST_DX as f32) as i32;
    let second_x = (width as f32 * 0.5 + SCORE_SECOND_DX as f32) as i32;
    let colon_x = (width as f32 * 0.5 + COLON_DX as f32) as i32;
    let s = SCORE_SIZE;
    list.draw_texture(x + first_x, y + SCORE_Y, s, s, 0, 0, s, s, &digit(0), 1.0);
    list.draw_texture(x + colon_x, y + COLON_Y, s, s, 0, 0, s, s, &texture("br_scorecolon"), 1.0);
    list.draw_texture(x + second_x, y + SCORE_Y, s, s, 0, 0, s, s, &digit(1), 1.0);
    let name = &observer.teams[0].name;
    let name_width = font.measure_text(name);
    let wrap = a + TOP_NAME_WRAP_EXTRA;
    crate::text_align::draw_true_font_wrapped(list, x + first_x - name_width, y + TOP_NAME_Y, RED, name, wrap, font);
    crate::text_align::draw_true_font_wrapped(list, x + a + TOP_SECOND_NAME_DX, y + TOP_NAME_Y, WHITE, &observer.teams[1].name, wrap, font);
}

pub fn paint_player(
    observer: &BroadcastObserver,
    team: usize,
    list: &mut DrawList,
    origin: (i32, i32),
    width: i32,
    fonts: (&dyn LineFont, &dyn LineFont),
    lookup: &dyn Fn(u32) -> Option<String>,
) {
    let (x, y) = origin;
    let (level_font, name_font) = fonts;
    let user = observer.selected_user(team);
    let back = if user.is_some_and(BroadcastUser::dead) { "br_dead1_back" } else { "br_party1_back" };
    strip_at(list, (x, y), triple(back), width, PLAYER_STRIP, 1.0);
    if let Some(user) = user {
        let class = lookup(class_name_sysstring(user.class_id.max(0) as u32)).unwrap_or_default();
        level_font.draw(list, x + PLAYER_LEVEL_POS.0, y + PLAYER_LEVEL_POS.1, WHITE, &format!("Lv{} {}", user.level, class));
        crate::text_align::draw_true_font_wrapped(list, x + PLAYER_NAME_POS.0, y + PLAYER_NAME_POS.1, YELLOW, &user.name, width, name_font);
    }
    let bars = [("cp", user.map(|u| (u.cp, u.max_cp))), ("hp", user.map(|u| (u.hp, u.max_hp))), ("mp", user.map(|u| (u.mp, u.max_mp)))];
    for ((kind, value), by) in bars.into_iter().zip(PLAYER_BAR_YS) {
        list.draw_texture(
            x + PLAYER_BAR_X,
            y + by,
            PLAYER_BAR_WIDTH,
            PLAYER_BAR_HEIGHT,
            0,
            0,
            8,
            16,
            &texture(&format!("br_bar1back_{kind}")),
            1.0,
        );
        if let Some((current, maximum)) = value {
            let width = fill_width(current, maximum, PLAYER_BAR_WIDTH);
            list.draw_texture(x + PLAYER_BAR_X, y + by, width, PLAYER_BAR_HEIGHT, 0, 0, 8, 16, &texture(&format!("br_bar1_{kind}")), 1.0);
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BuffCell {
    pub skill: i32,
    pub icon: String,
    pub debuff: bool,
    pub alpha: u8,
    pub blink_down: bool,
    pub time: i32,
}

pub const BUFF_ANCHOR_BOTTOM_LEFT: u32 = 7;
pub const BUFF_ANCHOR_BOTTOM_RIGHT: u32 = 9;

pub fn anchor_origin(mode: u32, offset: (i32, i32), parent: (i32, i32), child: (i32, i32)) -> Option<(i32, i32)> {
    let half = |parent: i32, child: i32| ((parent as f32 * 0.5) - (child as f32 * 0.5)) as i32;
    let (x, y) = match mode {
        1 => (0, 0),
        2 => (half(parent.0, child.0), 0),
        3 => (parent.0 - child.0, 0),
        4 => (0, half(parent.1, child.1)),
        5 => (half(parent.0, child.0), half(parent.1, child.1)),
        6 => (parent.0 - child.0, half(parent.1, child.1)),
        7 => (0, parent.1 - child.1),
        8 => (half(parent.0, child.0), parent.1 - child.1),
        9 => (parent.0 - child.0, parent.1 - child.1),
        _ => return None,
    };
    Some((x + offset.0, y + offset.1))
}

pub fn buff_origin(team: usize, player_size: (i32, i32)) -> (i32, i32) {
    let (mode, dx) = if team == 0 { (BUFF_ANCHOR_BOTTOM_LEFT, BUFF_OFFSET) } else { (BUFF_ANCHOR_BOTTOM_RIGHT, -BUFF_OFFSET) };
    anchor_origin(mode, (dx, 0), player_size, BUFF_SIZE).unwrap_or_default()
}

pub const BUFF_INITIAL_ICON_SIZE: i32 = 24;
pub const BUFF_TOOLTIP_ICON_SIZE: i32 = 26;

pub fn buff_hit(count: usize, team: usize, player_size: (i32, i32), icon_size: i32, x: i32, y: i32) -> Option<usize> {
    let (bx, by) = buff_origin(team, player_size);
    let (lx, ly) = (x - bx, y - by);
    if !(0..BUFF_SIZE.0).contains(&lx) || !(0..BUFF_SIZE.1).contains(&ly) {
        return None;
    }
    let index = crate::abnormal::grid_index_sized(lx, ly, icon_size).filter(|index| (0..=crate::abnormal::HIT_MAX_INDEX).contains(index))? as usize;
    let count = count.min(BUFF_MAX);
    let rows = match count {
        0 => 0,
        1..=10 => 10,
        11..=20 => 20,
        _ => 30,
    };
    (index < rows && index < count).then_some(index)
}

pub fn blink_step(cell: &mut BuffCell) {
    if !(0..=30).contains(&cell.time) {
        cell.alpha = 255;
        return;
    }
    let delta: i32 = if cell.blink_down { -25 } else { 25 };
    let next = i32::from(cell.alpha) + delta;
    if next > 255 {
        cell.alpha = 255;
        cell.blink_down = true;
    } else if next < 0 {
        cell.alpha = 0;
        cell.blink_down = false;
    } else {
        cell.alpha = next as u8;
    }
}

pub fn paint_buffs(list: &mut DrawList, origin: (i32, i32), cells: &[BuffCell]) {
    let (x, y) = origin;
    for (index, cell) in cells.iter().take(BUFF_MAX).enumerate() {
        let (cx, cy) = (x + (index % BUFF_COLUMNS) as i32 * BUFF_CELL, y + (index / BUFF_COLUMNS) as i32 * BUFF_CELL);
        let alpha = f32::from(cell.alpha) / 255.0;
        list.draw_texture(cx, cy, BUFF_CELL, BUFF_CELL, 0, 0, BUFF_CELL, BUFF_CELL, &cell.icon, alpha);
        if cell.debuff {
            list.draw_texture(cx, cy, BUFF_CELL, BUFF_CELL, 0, 0, BUFF_FRAME_UV, BUFF_FRAME_UV, DEBUFF_FRAME, alpha);
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PartyHit {
    Row(usize),
}

pub fn party_hit(observer: &BroadcastObserver, team: usize, x: i32, y: i32) -> Option<PartyHit> {
    if !(0..PARTY_LIST_SIZE.0).contains(&x) || y < 0 {
        return None;
    }
    let row = (y / PARTY_ROW) as usize;
    observer.user(team, row).map(|_| PartyHit::Row(row))
}

pub fn paint_party(
    observer: &BroadcastObserver,
    team: usize,
    list: &mut DrawList,
    origin: (i32, i32),
    font: &dyn LineFont,
    lookup: &dyn Fn(u32) -> Option<String>,
) {
    let width = PARTY_LIST_SIZE.0;
    for index in 0..USERS {
        let Some(user) = observer.user(team, index) else { continue };
        let top = origin.1 + index as i32 * PARTY_ROW;
        let start = list.quads.len();
        let selected = observer.selected[team] == Some(index);
        let alpha = if selected { 1.0 } else { PARTY_ALPHA_UNSELECTED };
        let back = if user.dead() { "br_dead2_back" } else { "br_party2_back" };
        strip_at(list, (origin.0, top), triple(back), width, PARTY_STRIP, alpha);
        let icon = if user.dead() { texture("br_party_X") } else { indexed(&format!("br_party{}_", team + 1), index + 1) };
        let (ix, iy) = PARTY_ICON_POS[team];
        let size = PARTY_ICON_SIZE;
        list.draw_texture(origin.0 + ix, top + iy, size, size, 0, 0, size, size, &icon, alpha);
        let bx = PARTY_TEXT_X[team];
        let color = if user.hp > 0 { WHITE } else { BLACK };
        let mut text = if observer.show_class_or_name {
            lookup(class_name_sysstring(user.class_id.max(0) as u32)).unwrap_or_default()
        } else {
            user.name.clone()
        };
        while font.measure_text(&text) > width - PARTY_TEXT_MARGIN && !text.is_empty() {
            text.pop();
        }
        let text_width = font.measure_text(&text);
        let tx = if team == 0 { bx } else { bx - text_width + PARTY_BAR_WIDTH };
        font.draw(list, origin.0 + tx, top + PARTY_TEXT_Y, color, &text);
        let bars = [
            ("cp", user.cp, user.max_cp, 38, 6),
            ("hp", user.hp, user.max_hp, 46, 8),
            ("mp", user.mp, user.max_mp, 56, 8),
        ];
        for (kind, current, maximum, by, bh) in bars {
            list.draw_texture(
                origin.0 + bx,
                top + by,
                PARTY_BAR_WIDTH,
                bh,
                0,
                0,
                8,
                8,
                &texture(&format!("br_bar2back_{kind}")),
                alpha,
            );
            let filled = fill_width(current, maximum, PARTY_BAR_WIDTH);
            let fx = if team == 0 { bx } else { bx - filled + PARTY_BAR_WIDTH };
            list.draw_texture(origin.0 + fx, top + by, filled, bh, 0, 0, 8, 8, &texture(&format!("br_bar2_{kind}")), alpha);
        }
        if selected {
            strip_at(list, (origin.0, top), triple("br_partyon_back"), width, PARTY_STRIP, 128.0 / 255.0);
        }
        list.clip_from(start, Rect::new(origin.0, top, width, PARTY_ROW));
    }
}

fn skill_text(list: &mut DrawList, entry: &MsgEntry, x: i32, y: i32, with_names: bool, font: &dyn LineFont) {
    let team_color = |team: i32| match team {
        0 => Some(RED),
        1 => Some(WHITE),
        _ => None,
    };
    let mut cursor = x;
    let caster = if with_names { format!("{}{}>", entry.caster_index + 1, entry.caster) } else { format!("{}>", entry.caster_index + 1) };
    if let Some(color) = team_color(entry.caster_team) {
        font.draw(list, cursor, y, color, &caster);
    }
    cursor += font.measure_text(&caster);
    let target = if with_names { format!("{}{}", entry.target_index + 1, entry.target) } else { format!("{}", entry.target_index + 1) };
    if let Some(color) = team_color(entry.target_team) {
        font.draw(list, cursor, y, color, &target);
    }
    cursor += font.measure_text(&target);
    font.draw(list, cursor, y, RED, ": ");
    cursor += font.measure_text(": ");
    font.draw(list, cursor, y, YELLOW, &entry.skill);
}

pub fn paint_msg(observer: &BroadcastObserver, list: &mut DrawList, origin: (i32, i32), width: i32, font: &dyn LineFont) {
    let (x, y) = origin;
    list.draw_texture(x, y, width, MSG_HEIGHT, 0, 0, MSG_BACK_SOURCE.0, MSG_BACK_SOURCE.1, &texture("br_msg3_back"), 1.0);
    match observer.msg_mode {
        MsgMode::All => {
            for (k, entry) in observer.ring_all.ordered().enumerate() {
                if !entry.caster.is_empty() {
                    skill_text(list, entry, x + MSG_X, y + k as i32 * MSG_LINE, true, font);
                }
            }
        }
        MsgMode::Teams => {
            let second_x = (width as f32 * 0.5 + x as f32 + 20.0) as i32;
            for (team, base) in [(0, x + MSG_X), (1, second_x)] {
                for (k, entry) in observer.ring_teams[team].ordered().enumerate() {
                    if !entry.caster.is_empty() {
                        skill_text(list, entry, base, y + k as i32 * MSG_LINE, false, font);
                    }
                }
            }
        }
        MsgMode::Off => {}
    }
}

pub fn paint_gm_msg(list: &mut DrawList, origin: (i32, i32), width: i32, text: &str, font: &dyn LineFont) {
    let (x, y) = origin;
    strip_at(list, (x, y), triple("br_msg2_back"), width, (32, GM_MSG_STRIP_HEIGHT), 1.0);
    let text_width = crate::text_align::true_font_width(text, width, font);
    let tx = (width as f32 * 0.5 - ((text_width - (text_width >> 31)) >> 1) as f32) as i32;
    crate::text_align::draw_true_font_wrapped(list, x + tx, y + GM_MSG_TEXT_Y, WHITE, text, width, font);
}

pub fn special_picture(code: i32) -> Option<(String, i32, i32)> {
    match code {
        1 => Some((texture("br_msg1_finish"), 512, 256)),
        2 => Some((texture("br_msg1_start"), 512, 512)),
        3 => Some((texture("br_msg1_gameover"), 512, 256)),
        4..=8 => Some((indexed("br_msg1_count", (code - 3) as usize), 256, 256)),
        _ => None,
    }
}

pub fn paint_special(list: &mut DrawList, origin: (i32, i32), width: i32, code: i32) {
    let (x, y) = origin;
    strip_at(list, (x, y), triple("br_msg1_back"), width, (32, SPECIAL_STRIP_HEIGHT), 1.0);
    if let Some((name, w, h)) = special_picture(code) {
        let px = (width as f32 * 0.5 - if w == 512 { 256.0 } else { 128.0 }) as i32;
        list.draw_texture(x + px, y + SPECIAL_Y, w, h, 0, 0, w, h, &name, 1.0);
    }
}

pub fn special_hold_ms(code: i32) -> i32 {
    if code == 2 { SPECIAL_START_HOLD_MS } else { SPECIAL_HOLD_MS }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{BitmapFont, Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn lookup(id: u32) -> Option<String> {
        Some(format!("c{id}"))
    }

    fn values(hp: i32) -> Vec<i32> {
        vec![hp, 100, 40, 50, 10, 20, 61, 2, 0]
    }

    fn filled() -> BroadcastObserver {
        let mut observer = BroadcastObserver::default();
        observer.start_observer(7, "Red", "Blue");
        observer.team_info_start(7, 1);
        observer.team_info(0, 501, "Ana", &values(50));
        observer.team_info(1, 502, "Bo", &values(0));
        observer.team_info_start(7, 2);
        observer.team_info(0, 601, "Cy", &values(100));
        observer
    }

    #[test]
    fn notifications_only_apply_to_the_current_game_and_a_filling_team() {
        let mut observer = BroadcastObserver::default();
        observer.start_observer(7, "Red", "Blue");
        observer.team_info(0, 1, "x", &values(1));
        assert!(observer.user(0, 0).is_none());
        observer.team_info_start(9, 1);
        assert_eq!(observer.filling, None);
        assert!(!observer.score(9, 1, 2));
        assert!(observer.score(7, 3, 4));
        assert_eq!((observer.teams[0].score, observer.teams[1].score), (3, 4));
        let observer = filled();
        assert_eq!(observer.find_user(502), Some((0, 1)));
        assert_eq!(observer.find_user(601), Some((1, 0)));
        assert_eq!(observer.visible_rows(0), 2);
        assert_eq!(observer.user(0, 0).unwrap().class_id, 2);
    }

    #[test]
    fn spell_info_collects_skills_and_times_that_fall_one_per_second() {
        let mut observer = filled();
        observer.spell_info_start(9, 501);
        assert_eq!(observer.spell_info_end(), None);
        observer.spell_info_start(7, 501);
        observer.spell_info(1204, 20);
        observer.spell_info(1040, 3);
        assert_eq!(observer.spell_info_end(), Some((0, 0)));
        observer.spell_info(1, 1);
        assert_eq!(observer.user(0, 0).unwrap().skills, vec![1204, 1040]);
        observer.tick(2500);
        assert_eq!(observer.user(0, 0).unwrap().times, vec![18, 1]);
    }

    #[test]
    fn a_freed_slot_keeps_its_effect_lists_until_the_effects_of_a_new_player_replace_them() {
        let mut observer = filled();
        observer.spell_info_start(7, 501);
        observer.spell_info(1204, 20);
        observer.spell_info_end();
        observer.team_info_start(7, 1);
        observer.team_info(0, 777, "Dee", &values(10));
        assert_eq!(observer.user(0, 0).unwrap().skills, vec![1204]);
        observer.team_unlocked(7, 1);
        assert!(observer.user(0, 0).is_none());
        observer.tick(3000);
        observer.team_info_start(7, 1);
        observer.team_info(0, 888, "Eve", &values(10));
        let user = observer.user(0, 0).unwrap();
        assert_eq!((user.skills.clone(), user.times.clone()), (vec![1204], vec![17]));
        observer.start_observer(7, "a", "b");
        observer.team_info_start(7, 1);
        observer.team_info(0, 999, "Fay", &values(10));
        assert_eq!(observer.user(0, 0).unwrap().skills, vec![1204]);
    }

    #[test]
    fn nobody_is_followed_until_a_slot_is_selected() {
        let mut observer = filled();
        assert!(observer.selected_user(0).is_none());
        observer.select_user(0, 9);
        assert!(observer.selected_user(0).is_none());
        observer.select_user(0, 1);
        assert_eq!(observer.selected_user(0).map(|user| user.id), Some(502));
    }

    #[test]
    fn the_buff_hit_uses_the_base_class_grid_twelve_pixels_left_of_the_icons() {
        let (bx, by) = buff_origin(0, PLAYER_SIZE);
        let size = BUFF_TOOLTIP_ICON_SIZE;
        assert_eq!(buff_hit(3, 0, PLAYER_SIZE, size, bx + 12, by), Some(0));
        assert_eq!(buff_hit(3, 0, PLAYER_SIZE, size, bx + 12 + 27 * 2, by + 5), Some(2));
        assert_eq!(buff_hit(3, 0, PLAYER_SIZE, size, bx + 12 + 27 * 3, by), None);
        assert_eq!(buff_hit(12, 0, PLAYER_SIZE, size, bx + 12 + 27, by + 27), Some(11));
        assert_eq!(buff_hit(12, 0, PLAYER_SIZE, size, bx + 12 + 27 * 2, by + 27), None);
        assert_eq!(buff_hit(3, 0, PLAYER_SIZE, size, bx - 1, by), None);
        assert_eq!(buff_hit(3, 0, PLAYER_SIZE, BUFF_INITIAL_ICON_SIZE, bx + 12 + 25 * 2, by), Some(2));
        assert_eq!(buff_hit(3, 0, PLAYER_SIZE, size, bx + 12 + 25 * 2, by), Some(1));
    }

    #[test]
    fn skill_messages_go_to_all_and_to_the_caster_team_ring_and_expire_one_by_one() {
        let mut observer = filled();
        observer.add_skill_msg((501, 0, 0), (601, 1, 0), "Heal", ("Ana", "Cy"));
        observer.add_skill_msg((601, 1, 0), (501, 0, 0), "Bolt", ("Cy", "Ana"));
        assert_eq!(observer.ring_all.ordered().filter(|e| !e.caster.is_empty()).count(), 2);
        assert_eq!(observer.ring_teams[0].ordered().filter(|e| !e.caster.is_empty()).count(), 1);
        assert_eq!(observer.ring_teams[1].ordered().filter(|e| !e.caster.is_empty()).count(), 1);
        assert_eq!(observer.msg_ttl, MSG_TTL);
        observer.tick(9000);
        assert_eq!(observer.msg_ttl, 1);
        observer.tick(1000);
        assert_eq!(observer.msg_ttl, 0);
        assert_eq!(observer.ring_all.ordered().filter(|e| !e.caster.is_empty()).count(), 2);
        observer.tick(1000);
        assert_eq!(observer.ring_all.ordered().filter(|e| !e.caster.is_empty()).count(), 1);
        assert_eq!(observer.msg_ttl, MSG_TTL);
    }

    #[test]
    fn the_g_key_cycles_three_modes_and_only_two_of_them_rearm_the_ttl() {
        let mut observer = BroadcastObserver { msg_ttl: 3, ..Default::default() };
        observer.switch_msg_mode();
        assert_eq!((observer.msg_mode, observer.msg_ttl), (MsgMode::Teams, MSG_TTL));
        observer.msg_ttl = 4;
        observer.switch_msg_mode();
        assert_eq!((observer.msg_mode, observer.msg_ttl), (MsgMode::Off, 4));
        observer.switch_msg_mode();
        assert_eq!((observer.msg_mode, observer.msg_ttl), (MsgMode::All, MSG_TTL));
    }

    #[test]
    fn the_top_slides_down_three_pixels_per_millisecond_holds_five_seconds_and_slides_up() {
        let mut slide = TopSlide { phase: TopPhase::Idle, y: -113 };
        slide.show();
        slide.tick(38, TOP_HEIGHT);
        assert_eq!((slide.y, slide.phase), (1, TopPhase::Hold(TOP_HOLD_MS)));
        slide.tick(4999, TOP_HEIGHT);
        assert_eq!(slide.phase, TopPhase::Hold(1));
        slide.tick(1, TOP_HEIGHT);
        assert_eq!(slide.phase, TopPhase::Up);
        slide.tick(100, TOP_HEIGHT);
        assert!(slide.y + TOP_HEIGHT <= 0);
        slide.tick(10, TOP_HEIGHT);
        assert_eq!(slide.phase, TopPhase::Idle);
    }

    #[test]
    fn the_top_paints_seven_strips_that_add_up_to_the_width_and_the_score_digits() {
        let mut observer = filled();
        observer.score(7, 3, 12);
        let mut list = DrawList::default();
        paint_top(&observer, &mut list, (0, 0), 1000, &font());
        let strips: Vec<_> = list.quads.iter().filter(|q| q.texture.contains("br_top_back")).collect();
        assert_eq!(strips.len(), 7);
        let a = 80.0;
        assert_eq!(strips[6].x, 2.0 * a + 584.0);
        assert_eq!(strips[6].x + strips[6].w, 1000.0);
        assert!(list.quads.iter().any(|q| q.texture.ends_with("br_score3") && q.x == 395.0 && q.y == 28.0));
        assert!(list.quads.iter().any(|q| q.texture.ends_with("br_score0") && q.x == 505.0));
        assert!(list.quads.iter().any(|q| q.texture.ends_with("br_scorecolon") && q.x == 449.0 && q.y == 18.0));
    }

    #[test]
    fn the_player_panel_uses_the_dead_strips_and_bar_fills_by_signed_division() {
        let mut observer = filled();
        let mut list = DrawList::default();
        paint_player(&observer, 0, &mut list, (0, 0), 520, (&font(), &font()), &lookup);
        assert!(list.quads.iter().any(|q| q.texture.ends_with("br_party1_back1")));
        assert!(!list.quads.iter().any(|q| q.texture.ends_with("br_bar1_hp") && q.w > 0.0));
        observer.select_user(0, 1);
        let mut list = DrawList::default();
        paint_player(&observer, 0, &mut list, (0, 0), 520, (&font(), &font()), &lookup);
        assert!(list.quads.iter().any(|q| q.texture.ends_with("br_dead1_back1")));
        assert!(list.quads.iter().any(|q| q.texture.ends_with("br_bar1_hp") && q.w == 0.0));
        observer.select_user(0, 0);
        let mut list = DrawList::default();
        paint_player(&observer, 0, &mut list, (0, 0), 520, (&font(), &font()), &lookup);
        let hp = list.quads.iter().find(|q| q.texture.ends_with("br_bar1_hp")).unwrap();
        assert_eq!((hp.x, hp.y, hp.w, hp.h), (10.0, 112.0, 250.0, 12.0));
        assert_eq!(fill_width(5, 0, 500), 0);
    }

    #[test]
    fn the_party_rows_align_the_second_team_to_the_right_and_mark_the_selected_one() {
        let mut observer = filled();
        observer.select_user(1, 0);
        let mut list = DrawList::default();
        paint_party(&observer, 1, &mut list, (0, 0), &font(), &lookup);
        let hp = list.quads.iter().find(|q| q.texture.ends_with("br_bar2_hp")).unwrap();
        assert_eq!((hp.x, hp.w, hp.y), (5.0, 220.0, 46.0));
        assert!(list.quads.iter().any(|q| q.texture.ends_with("br_partyon_back1")));
        let icon = list.quads.iter().find(|q| q.texture.ends_with("br_party2_1")).unwrap();
        assert_eq!((icon.x, icon.y), (231.0, 9.0));
        assert_eq!(party_hit(&observer, 1, 10, 5), Some(PartyHit::Row(0)));
        assert_eq!(party_hit(&observer, 1, 10, 75), None);
    }

    #[test]
    fn the_message_line_prints_numbers_names_colon_and_skill_with_team_colors() {
        let mut observer = filled();
        observer.add_skill_msg((501, 0, 0), (601, 1, 0), "Heal", ("Ana", "Cy"));
        let mut list = DrawList::default();
        paint_msg(&observer, &mut list, (0, 0), 1000, &font());
        let reds = list.quads.iter().filter(|q| q.color == RED.rgba()).count();
        assert!(reds > 0);
        assert!(list.quads.iter().any(|q| q.color == YELLOW.rgba()));
        assert!(list.quads.iter().any(|q| q.color == WHITE.rgba()));
    }

    #[test]
    fn paint_buffs_lays_cells_in_ten_columns_of_forty_pixels_with_the_debuff_frame_and_caps_at_thirty() {
        let cell = |debuff| BuffCell { skill: 1, icon: "i".into(), debuff, alpha: 255, blink_down: true, time: 100 };
        let mut cells: Vec<BuffCell> = (0..35).map(|_| cell(false)).collect();
        cells[11] = cell(true);
        let mut list = DrawList::default();
        paint_buffs(&mut list, (7, 9), &cells);
        let icons: Vec<_> = list.quads.iter().filter(|q| q.texture == "i").collect();
        assert_eq!(icons.len(), BUFF_MAX);
        assert_eq!((icons[11].x, icons[11].y), (47.0, 49.0));
        assert_eq!((icons[29].x, icons[29].y), (367.0, 89.0));
        assert_eq!(list.quads.iter().filter(|q| q.texture == DEBUFF_FRAME).count(), 1);
    }

    #[test]
    fn buff_cells_blink_between_zero_and_full_only_in_the_last_thirty_seconds() {
        let mut cell = BuffCell { skill: 1, icon: "i".into(), debuff: false, alpha: 255, blink_down: true, time: 10 };
        for _ in 0..10 {
            blink_step(&mut cell);
        }
        assert_eq!(cell.alpha, 5);
        blink_step(&mut cell);
        assert_eq!((cell.alpha, cell.blink_down), (0, false));
        let mut steady = BuffCell { time: 99, alpha: 40, ..cell.clone() };
        blink_step(&mut steady);
        assert_eq!(steady.alpha, 255);
        assert_eq!(buff_origin(1, PLAYER_SIZE), (110, 144));
        assert_eq!(buff_origin(0, PLAYER_SIZE), (10, 144));
        assert_eq!(anchor_origin(5, (0, 0), (100, 50), (40, 20)), Some((30, 15)));
        assert_eq!(anchor_origin(0, (0, 0), (100, 50), (40, 20)), None);
    }

    #[test]
    fn the_special_message_picks_its_picture_by_code() {
        assert_eq!(special_picture(2).map(|p| (p.1, p.2)), Some((512, 512)));
        assert_eq!(special_picture(5).map(|p| p.0), Some(texture("br_msg1_count2")));
        assert!(special_picture(9).is_none());
        assert_eq!((special_hold_ms(2), special_hold_ms(3)), (1000, 5000));
    }
}
