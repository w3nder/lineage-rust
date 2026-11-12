use crate::button::TextButton;
use crate::color::{Argb, TEXT};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::html::{self, Line};
use crate::npc_viewer::paint_html_lines;
use crate::window::Rect;

pub const CONTAINER_SIZE: (i32, i32) = (316, 542);
pub const SELECT_SIZE: (i32, i32) = (316, 330);
pub const INFO_SIZE: (i32, i32) = (316, 205);
pub const INFO_TOP: i32 = 337;
pub const SELECT_BACK: &str = "L2UI_CH3.Serverselectwnd.Serverselect_back";
pub const HELP_RECT: Rect = Rect::new(7, INFO_TOP + 7, 302, 191);
pub const HELP_WHEEL_LINES: i32 = 3;
pub const HELP_FILE: &str = "..\\L2text\\server_help.htm";
pub const INFO_BACK: &str = "L2UI_CH3.Serverselectwnd.Serverinfo_back";
pub const LIST_RECT: Rect = Rect::new(8, 10, 300, 274);
pub const HEADER_HEIGHT: i32 = 19;
pub const ROW_HEIGHT: i32 = 17;
pub const STRIPE: &str = "L2UI_CH3.Etc.textbackline";
pub const SELECT: &str = "L2UI_CH3.ListCtrl.TextSelect";
pub const SELECT_END: &str = "L2UI_CH3.ListCtrl.TextSelect2";
pub const CELL_TEXT: (i32, i32) = (10, 3);
pub const COLUMN_WIDTHS: [i32; 5] = [32, 111, 59, 49, 49];
pub const QUALITY_RECT: Rect = Rect::new(41, 297, 76, 23);
pub const OK_RECT: Rect = Rect::new(121, 297, 76, 23);
pub const CANCEL_RECT: Rect = Rect::new(201, 297, 76, 23);
pub const BUTTON_NORMAL: &str = "L2UI_CH3.Button.Btn1_normal";
pub const BUTTON_DOWN: &str = "L2UI_CH3.Button.Btn1_normalOn";
pub const TAB_DOWN: [&str; 3] = ["L2UI_CH3.ListCtrl.tab1_down", "L2UI_CH3.ListCtrl.tab2_down", "L2UI_CH3.ListCtrl.tab3_down"];
pub const NUMERIC_COLUMN: usize = 3;
pub const TAB_NORMAL: [&str; 3] = ["L2UI_CH3.ListCtrl.tab1", "L2UI_CH3.ListCtrl.tab2", "L2UI_CH3.ListCtrl.tab3"];
pub const SANDGLASS: &str = "symbol.sandglass";
pub const SANDGLASS_SIZE: (i32, i32) = (10, 12);
pub const LOGIN_LOCK_MS: u32 = 3000;
pub const NO_PING: i32 = 9999;
pub const FULL_PLAYERS: i32 = 3700;
pub const RED: Argb = Argb(0xffff0000);
pub const FULL_COLOR: Argb = Argb(0xffff8080);
pub const NORMAL_COLOR: Argb = Argb(0xffbfae64);
pub const GOOD_COLOR: Argb = Argb(0xff64a3d9);
pub const SYSSTRINGS: [u32; 16] = [416, 451, 452, 601, 602, 603, 140, 141, 456, 457, 458, 466, 459, 460, 461, 462];
pub const TEEN_SYSSTRING: u32 = 539;
pub const SERVER_TEST_FLAG: u32 = 4;
pub const SERVER_GM_FLAG: u32 = 8;
pub const SERVER_HOURGLASS_FLAG: u32 = 2;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ServerLabels {
    pub header: [String; 5],
    pub quality: String,
    pub ok: String,
    pub cancel: String,
    pub down: String,
    pub normal: String,
    pub full: String,
    pub good: String,
    pub pvp_on: String,
    pub pvp_off: String,
    pub over_18: String,
    pub over_15: String,
    pub teen: String,
    pub title: String,
}

impl ServerLabels {
    pub fn from_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> ServerLabels {
        let text = |id: u32| lookup(id).unwrap_or_default();
        ServerLabels {
            header: [text(SYSSTRINGS[0]), text(SYSSTRINGS[1]), text(SYSSTRINGS[2]), text(SYSSTRINGS[3]), text(SYSSTRINGS[4])],
            quality: text(SYSSTRINGS[5]),
            ok: text(SYSSTRINGS[6]),
            cancel: text(SYSSTRINGS[7]),
            down: text(SYSSTRINGS[8]),
            normal: text(SYSSTRINGS[9]),
            full: text(SYSSTRINGS[10]),
            good: text(SYSSTRINGS[11]),
            pvp_on: text(SYSSTRINGS[12]),
            pvp_off: text(SYSSTRINGS[13]),
            over_18: text(SYSSTRINGS[14]),
            over_15: text(SYSSTRINGS[15]),
            teen: text(TEEN_SYSSTRING),
            title: text(449),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerEntry {
    pub id: u8,
    pub ip: u32,
    pub port: u32,
    pub age: u8,
    pub pvp: u8,
    pub current: i32,
    pub max: i32,
    pub status: u8,
    pub flags: u32,
    pub brackets: u8,
    pub name: String,
    pub forbidden: bool,
    pub ping: i32,
}

pub fn server_number(id: u8, flags: u32) -> String {
    if flags & SERVER_GM_FLAG != 0 {
        format!("{:03}", 127)
    } else if flags & SERVER_TEST_FLAG != 0 {
        "1".to_string()
    } else if id <= 9 {
        format!("0{id}")
    } else {
        id.to_string()
    }
}

pub fn server_name(id: u8, flags: u32, record: impl Fn(u32) -> Option<String>) -> String {
    let index = if flags & SERVER_GM_FLAG != 0 {
        126
    } else if flags & SERVER_TEST_FLAG != 0 {
        125
    } else {
        u32::from(id).saturating_sub(1)
    };
    record(index).unwrap_or_default()
}

pub fn bracketed_name(brackets: u8, idc: impl Fn(u32) -> Option<String>, name: &str) -> String {
    if brackets == 0 {
        return name.to_string();
    }
    format!("[{}]{name}", idc(u32::from(brackets)).unwrap_or_default())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Down,
    Normal,
    Full,
    Good,
    Unknown,
}

impl ServerEntry {
    pub fn load_status(&self) -> Status {
        match self.status {
            0 => Status::Down,
            1 if self.max > 0 => {
                let percent = f64::from(self.current) / f64::from(self.max) * 100.0;
                if percent >= 80.0 || self.current >= FULL_PLAYERS {
                    Status::Full
                } else if percent > 50.0 {
                    Status::Normal
                } else {
                    Status::Good
                }
            }
            1 => Status::Normal,
            _ => Status::Unknown,
        }
    }
}

pub fn quality_order(entries: &mut [ServerEntry], preferred: Option<usize>) -> Option<usize> {
    if entries.is_empty() {
        return None;
    }
    let preferred = preferred.filter(|index| *index < entries.len()).unwrap_or(0);
    entries.swap(0, preferred);
    if entries.len() > 1 {
        entries[1..].sort_by_key(|entry| entry.ping);
        let group = |ping: i32| match ping {
            ..=9 => 0,
            10..=50 => 1,
            51..=9998 => 2,
            _ => 3,
        };
        let mut start = 1;
        while start < entries.len() {
            let kind = group(entries[start].ping);
            let end = (start..entries.len()).find(|&i| group(entries[i].ping) != kind).unwrap_or(entries.len());
            if kind != 3 {
                entries[start..end].sort_by_key(|entry| entry.current);
            }
            start = end;
        }
    }
    if entries.len() > 1
        && let Some(first) = entries.iter().skip(1).position(|entry| entry.flags & SERVER_GM_FLAG != 0)
    {
        let last = entries.len() - 1;
        entries.swap(first + 1, last);
    }
    Some(0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    Header(usize),
    Row(usize),
    Quality,
    Ok,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerAction {
    Choose { id: u8, ip: u32, port: u32 },
    Cancel,
    Help(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerSelect {
    pub screen: (i32, i32),
    pub labels: ServerLabels,
    pub entries: Vec<ServerEntry>,
    pub selected: Option<usize>,
    pub last_server: Option<u8>,
    pub locked: bool,
    pub timer_ms: u32,
    pub first_row: usize,
    pub pressed: Option<Hit>,
    pub help_lines: Vec<Line>,
    pub help_height: i32,
    pub help_step: i32,
    pub help_scroll: i32,
    pub help_pressed: Option<String>,
    pub help_bar: crate::scroll::ScrollWnd,
    pub help_bar_pressed: Option<crate::scroll::ScrollHit>,
    pub pending_pings: usize,
    pub ascending: [bool; 5],
}

impl ServerSelect {
    pub fn new(screen: (i32, i32), labels: ServerLabels) -> ServerSelect {
        ServerSelect { screen, labels, entries: Vec::new(), selected: None, last_server: None, locked: false, timer_ms: 0, first_row: 0, pressed: None, help_lines: Vec::new(), help_height: 0, help_scroll: 0, help_step: 0, help_pressed: None, help_bar: crate::scroll::ScrollWnd::new(HELP_RECT.w, HELP_RECT.h), help_bar_pressed: None, pending_pings: 0, ascending: [true; 5] }
    }

    pub fn origin(&self) -> (i32, i32) {
        (self.screen.0 / 2 - CONTAINER_SIZE.0 / 2, self.screen.1 / 2 - CONTAINER_SIZE.1 / 2)
    }

    pub fn visible_rows() -> usize {
        ((LIST_RECT.h - HEADER_HEIGHT) / ROW_HEIGHT) as usize
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.selected = None;
        self.first_row = 0;
        self.locked = false;
    }

    pub fn start_pings(&mut self) {
        self.pending_pings = self.entries.len();
    }

    pub fn set_ping(&mut self, id: u8, ms: i32) {
        let Some(entry) = self.entries.iter_mut().find(|entry| entry.id == id) else { return };
        entry.ping = ms;
        self.pending_pings = self.pending_pings.saturating_sub(1);
        if self.pending_pings == 0 {
            self.quality();
        }
    }

    pub fn finish(&mut self) {
        self.quality();
    }

    pub fn tick(&mut self, delta_ms: u32) {
        self.timer_ms = self.timer_ms.saturating_add(delta_ms);
        if self.timer_ms >= LOGIN_LOCK_MS {
            self.timer_ms %= LOGIN_LOCK_MS;
            self.locked = false;
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<Hit> {
        let origin = self.origin();
        let (x, y) = (x - origin.0, y - origin.1);
        for (rect, hit) in [(QUALITY_RECT, Hit::Quality), (OK_RECT, Hit::Ok), (CANCEL_RECT, Hit::Cancel)] {
            if rect.contains(x, y) {
                return Some(hit);
            }
        }
        if Rect::new(LIST_RECT.x, LIST_RECT.y, COLUMN_WIDTHS.iter().sum(), HEADER_HEIGHT).contains(x, y) {
            let mut left = LIST_RECT.x;
            for (index, width) in COLUMN_WIDTHS.iter().enumerate() {
                if x < left + width {
                    return Some(Hit::Header(index));
                }
                left += width;
            }
        }
        let rows = Rect::new(LIST_RECT.x, LIST_RECT.y + HEADER_HEIGHT, LIST_RECT.w, LIST_RECT.h - HEADER_HEIGHT);
        if rows.contains(x, y) {
            let index = self.first_row + ((y - rows.y) / ROW_HEIGHT) as usize;
            if index < self.entries.len() {
                return Some(Hit::Row(index));
            }
        }
        None
    }

    pub fn set_help(&mut self, text: &str, font: &BitmapFont) {
        self.set_help_with(text, font, &|_, _| None);
    }

    pub fn set_help_with(&mut self, text: &str, font: &BitmapFont, resolve: &dyn Fn(html::Marker, i32) -> Option<String>) {
        self.help_lines = html::layout(&html::parse_with(text, resolve), HELP_RECT.w, font);
        self.help_height = html::height(&self.help_lines, font);
        self.help_step = font.line_height as i32;
        self.help_scroll = 0;
        self.sync_help_bar();
    }

    fn sync_help_bar(&mut self) {
        self.help_bar.slider.set_range(self.help_height, HELP_RECT.h);
        self.help_bar.slider.set_pos(self.help_scroll);
    }

    fn set_help_scroll(&mut self, scroll: i32) {
        self.help_scroll = scroll.clamp(0, (self.help_height - HELP_RECT.h).max(0));
        self.sync_help_bar();
    }

    pub fn help_bar_down(&mut self, x: i32, y: i32) -> bool {
        let origin = self.origin();
        let (lx, ly) = (x - origin.0 - HELP_RECT.x, y - origin.1 - HELP_RECT.y);
        if self.help_height <= HELP_RECT.h {
            return false;
        }
        let Some(hit) = self.help_bar.hit(HELP_RECT.w, HELP_RECT.h, lx, ly) else { return false };
        self.help_bar_pressed = Some(hit);
        self.help_bar.up_pressed = hit == crate::scroll::ScrollHit::Up;
        self.help_bar.down_pressed = hit == crate::scroll::ScrollHit::Down;
        if let crate::scroll::ScrollHit::Slider(slider_y) = hit {
            self.help_bar.slider.mouse_down(slider_y);
        }
        true
    }

    pub fn help_bar_move(&mut self, y: i32) -> bool {
        if !self.help_bar.slider.dragging {
            return false;
        }
        let origin = self.origin();
        if let Some(position) = self.help_bar.slider.mouse_move(y - origin.1 - HELP_RECT.y - crate::scroll::BUTTON_SIZE) {
            self.help_scroll = position.clamp(0, (self.help_height - HELP_RECT.h).max(0));
        }
        true
    }

    pub fn help_bar_up(&mut self, x: i32, y: i32) -> bool {
        let Some(pressed) = self.help_bar_pressed.take() else { return false };
        let origin = self.origin();
        let hit = self.help_bar.hit(HELP_RECT.w, HELP_RECT.h, x - origin.0 - HELP_RECT.x, y - origin.1 - HELP_RECT.y);
        self.help_bar.up_pressed = false;
        self.help_bar.down_pressed = false;
        self.help_bar.slider.mouse_up();
        if hit == Some(pressed) {
            match pressed {
                crate::scroll::ScrollHit::Up => self.set_help_scroll(self.help_scroll - self.help_step),
                crate::scroll::ScrollHit::Down => self.set_help_scroll(self.help_scroll + self.help_step),
                crate::scroll::ScrollHit::Slider(_) => {}
            }
        }
        true
    }

    fn help_point(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        let origin = self.origin();
        let (x, y) = (x - origin.0, y - origin.1);
        HELP_RECT.contains(x, y).then_some((x - HELP_RECT.x, y - HELP_RECT.y))
    }

    fn help_link(&self, x: i32, y: i32, font: &BitmapFont) -> Option<String> {
        let (lx, ly) = self.help_point(x, y)?;
        html::link_at(&self.help_lines, lx, ly + self.help_scroll, font).and_then(html::local_link_target).map(str::to_string)
    }

    pub fn wheel_at(&mut self, x: i32, y: i32, notches: i32) {
        if self.help_point(x, y).is_some() {
            self.set_help_scroll(self.help_scroll - notches * HELP_WHEEL_LINES * self.help_step);
        } else {
            self.wheel(notches);
        }
    }

    pub fn mouse_down_help(&mut self, x: i32, y: i32, font: &BitmapFont) {
        self.help_pressed = self.help_link(x, y, font);
    }

    pub fn mouse_up_help(&mut self, x: i32, y: i32, font: &BitmapFont) -> Option<ServerAction> {
        let pressed = self.help_pressed.take()?;
        (self.help_link(x, y, font).as_deref() == Some(pressed.as_str())).then_some(ServerAction::Help(pressed))
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        self.pressed = self.hit(x, y);
        if let Some(Hit::Row(index)) = self.pressed {
            self.selected = Some(index);
        }
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<ServerAction> {
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            Hit::Row(_) => None,
            Hit::Header(column) => {
                self.sort_by_column(column);
                None
            }
            Hit::Quality => {
                self.quality();
                None
            }
            Hit::Ok => self.confirm(),
            Hit::Cancel => Some(ServerAction::Cancel),
        }
    }

    pub fn sort_by_column(&mut self, column: usize) {
        let selected = self.selected.and_then(|index| self.entries.get(index)).map(|entry| entry.id);
        let ascending = self.ascending[column];
        let texts: Vec<String> = self.entries.iter().map(|entry| self.cells(entry)[column].0.clone()).collect();
        let mut order: Vec<usize> = (0..self.entries.len()).collect();
        let count = order.len();
        for pass in 0..count {
            for at in 0..count.saturating_sub(pass).min(count.saturating_sub(1)) {
                let (a, b) = (&texts[order[at]], &texts[order[at + 1]]);
                if a.is_empty() || b.is_empty() {
                    continue;
                }
                let ordering = if column == NUMERIC_COLUMN { crate::list_ctrl::wtoi(a).cmp(&crate::list_ctrl::wtoi(b)) } else { crate::list_ctrl::wcsicmp(a, b) };
                let swap = if ascending { ordering == std::cmp::Ordering::Greater } else { ordering == std::cmp::Ordering::Less };
                if swap {
                    order.swap(at, at + 1);
                }
            }
        }
        if let Some(special) = order.iter().position(|&index| self.entries[index].flags & SERVER_GM_FLAG != 0) {
            let last = order.len() - 1;
            order.swap(special, last);
        }
        let mut sorted: Vec<ServerEntry> = order.iter().map(|&index| self.entries[index].clone()).collect();
        std::mem::swap(&mut self.entries, &mut sorted);
        self.ascending[column] = !ascending;
        if let Some(id) = selected {
            self.selected = self.entries.iter().position(|entry| entry.id == id);
        }
    }

    pub fn double_click(&mut self, x: i32, y: i32) -> Option<ServerAction> {
        match self.hit(x, y) {
            Some(Hit::Row(index)) => {
                self.selected = Some(index);
                self.confirm()
            }
            _ => None,
        }
    }

    pub fn enter(&mut self) -> Option<ServerAction> {
        self.confirm()
    }

    pub fn wheel(&mut self, delta: i32) {
        let max = self.entries.len().saturating_sub(Self::visible_rows());
        let next = self.first_row as i32 - delta.signum() * 3;
        self.first_row = next.clamp(0, max as i32) as usize;
    }

    pub fn quality(&mut self) {
        let preferred = self.last_server.and_then(|id| self.entries.iter().position(|entry| entry.id == id));
        self.selected = quality_order(&mut self.entries, preferred);
        self.first_row = 0;
    }

    fn confirm(&mut self) -> Option<ServerAction> {
        let entry = self.entries.get(self.selected?)?;
        if self.locked {
            return None;
        }
        self.locked = true;
        Some(ServerAction::Choose { id: entry.id, ip: entry.ip, port: entry.port })
    }

    pub fn paint(&self, list: &mut DrawList, font: &BitmapFont) {
        let (ox, oy) = self.origin();
        list.draw_texture(ox, oy, SELECT_SIZE.0, SELECT_SIZE.1, 0, 0, SELECT_SIZE.0, SELECT_SIZE.1, SELECT_BACK, 1.0);
        self.paint_header(list, font, ox, oy);
        self.paint_rows(list, font, ox, oy);
        let pressed = |hit: Hit| self.pressed == Some(hit);
        TextButton::new(QUALITY_RECT, BUTTON_NORMAL, BUTTON_DOWN, &self.labels.quality).paint(list, ox, oy, pressed(Hit::Quality), font);
        TextButton::new(OK_RECT, BUTTON_NORMAL, BUTTON_DOWN, &self.labels.ok).paint(list, ox, oy, pressed(Hit::Ok), font);
        TextButton::new(CANCEL_RECT, BUTTON_NORMAL, BUTTON_DOWN, &self.labels.cancel).paint(list, ox, oy, pressed(Hit::Cancel), font);
        list.draw_texture(ox, oy + INFO_TOP, INFO_SIZE.0, INFO_SIZE.1, 0, 0, INFO_SIZE.0, INFO_SIZE.1, INFO_BACK, 1.0);
        paint_html_lines(list, ox + HELP_RECT.x, oy + HELP_RECT.y, (HELP_RECT.w, HELP_RECT.h), &self.help_lines, self.help_scroll, font, 1.0);
        if self.help_height > HELP_RECT.h {
            self.help_bar.paint(list, ox + HELP_RECT.x, oy + HELP_RECT.y, HELP_RECT.w, HELP_RECT.h);
        }
    }

    fn paint_header(&self, list: &mut DrawList, font: &BitmapFont, ox: i32, oy: i32) {
        let mut x = ox + LIST_RECT.x;
        for (index, width) in COLUMN_WIDTHS.iter().enumerate() {
            let y = oy + LIST_RECT.y;
            let tabs = if self.pressed == Some(Hit::Header(index)) { &TAB_DOWN } else { &TAB_NORMAL };
            list.draw_texture(x, y, 8, HEADER_HEIGHT, 0, 0, 8, HEADER_HEIGHT, tabs[0], 1.0);
            list.draw_texture(x + 8, y, width - 16, HEADER_HEIGHT, 0, 0, 8, HEADER_HEIGHT, tabs[1], 1.0);
            list.draw_texture(x + width - 8, y, 8, HEADER_HEIGHT, 0, 0, 8, HEADER_HEIGHT, tabs[2], 1.0);
            let label = &self.labels.header[index];
            let tx = x + width / 2 - font.measure(label) as i32 / 2;
            let ty = y + HEADER_HEIGHT / 2 - font.line_height as i32 / 2;
            list.draw_text(tx, ty, TEXT, label, font);
            x += width;
        }
    }

    fn cells(&self, entry: &ServerEntry) -> [(String, Argb); 5] {
        let status = match entry.load_status() {
            Status::Down => (self.labels.down.clone(), RED),
            Status::Full => (self.labels.full.clone(), FULL_COLOR),
            Status::Normal => (self.labels.normal.clone(), NORMAL_COLOR),
            Status::Good => (self.labels.good.clone(), GOOD_COLOR),
            Status::Unknown => (String::new(), TEXT),
        };
        let number = if entry.flags & SERVER_GM_FLAG != 0 { String::new() } else { server_number(entry.id, entry.flags) };
        [
            (number, TEXT),
            (entry.name.clone(), if entry.forbidden { RED } else { TEXT }),
            status,
            (entry.ping.to_string(), TEXT),
            (String::new(), TEXT),
        ]
    }

    fn paint_rows(&self, list: &mut DrawList, font: &BitmapFont, ox: i32, oy: i32) {
        let top = oy + LIST_RECT.y + HEADER_HEIGHT;
        for (offset, entry) in self.entries.iter().enumerate().skip(self.first_row).take(Self::visible_rows()) {
            let y = top + (offset - self.first_row) as i32 * ROW_HEIGHT;
            if offset % 2 == 1 {
                list.draw_texture(ox + LIST_RECT.x, y - 1, LIST_RECT.w, 16, 0, 0, 8, 16, STRIPE, 1.0);
            }
            if self.selected == Some(offset) {
                let x = ox + LIST_RECT.x;
                list.draw_texture(x, y, LIST_RECT.w - 32, 16, 0, 0, 16, 13, SELECT, 1.0);
                list.draw_texture(x + LIST_RECT.w - 32, y, 32, 16, 0, 0, 32, 13, SELECT_END, 1.0);
            }
            let mut x = ox + LIST_RECT.x;
            for (index, ((text, color), width)) in self.cells(entry).into_iter().zip(COLUMN_WIDTHS).enumerate() {
                if !text.is_empty() {
                    list.draw_text_clipped(x + CELL_TEXT.0, y + CELL_TEXT.1, width - CELL_TEXT.0, color, &text, font);
                }
                if index == 4 && entry.flags & SERVER_HOURGLASS_FLAG != 0 {
                    list.draw_texture(x + CELL_TEXT.0, y + 2, SANDGLASS_SIZE.0, SANDGLASS_SIZE.1, 0, 0, SANDGLASS_SIZE.0, SANDGLASS_SIZE.1, SANDGLASS, 1.0);
                }
                x += width;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn entry(id: u8, current: i32, max: i32, status: u8, flags: u32, ping: i32) -> ServerEntry {
        ServerEntry { id, ip: 0x7f000001, port: 7777, age: 15, pvp: 0, current, max, status, flags, brackets: 0, name: format!("S{id}"), forbidden: false, ping }
    }

    fn select() -> ServerSelect {
        ServerSelect::new((1024, 768), ServerLabels::from_sysstrings(|id| Some(format!("s{id}"))))
    }

    #[test]
    fn number_and_name_follow_the_flags_of_servername_0x100fffc0() {
        assert_eq!(server_number(3, 0), "03");
        assert_eq!(server_number(12, 0), "12");
        assert_eq!(server_number(3, SERVER_TEST_FLAG), "1");
        assert_eq!(server_number(3, SERVER_GM_FLAG), "127");
        let record = |index: u32| Some(format!("r{index}"));
        assert_eq!(server_name(3, 0, record), "r2");
        assert_eq!(server_name(3, SERVER_TEST_FLAG, record), "r125");
        assert_eq!(server_name(3, SERVER_GM_FLAG, record), "r126");
    }

    #[test]
    fn a_server_type_puts_its_idc_name_in_brackets_before_the_name_like_formatserverlabel() {
        let idc = |index: u32| (index == 2).then(|| "IDC".to_string());
        assert_eq!(bracketed_name(0, idc, "Killer"), "Killer");
        assert_eq!(bracketed_name(2, idc, "Killer"), "[IDC]Killer");
        assert_eq!(bracketed_name(1, idc, "Killer"), "[]Killer");
    }

    #[test]
    fn load_status_uses_the_percentage_and_the_3700_players_cap() {
        assert_eq!(entry(1, 0, 1000, 0, 0, 1).load_status(), Status::Down);
        assert_eq!(entry(1, 500, 1000, 1, 0, 1).load_status(), Status::Good);
        assert_eq!(entry(1, 501, 1000, 1, 0, 1).load_status(), Status::Normal);
        assert_eq!(entry(1, 800, 1000, 1, 0, 1).load_status(), Status::Full);
        assert_eq!(entry(1, 3700, 9000, 1, 0, 1).load_status(), Status::Full);
        assert_eq!(entry(1, 5, 0, 1, 0, 1).load_status(), Status::Normal);
        assert_eq!(entry(1, 5, 10, 7, 0, 1).load_status(), Status::Unknown);
    }

    #[test]
    fn quality_puts_the_preferred_first_then_orders_by_ping_group_and_players() {
        let mut entries = vec![
            entry(1, 100, 1000, 1, 0, 9999),
            entry(2, 900, 1000, 1, 0, 30),
            entry(3, 200, 1000, 1, 0, 30),
            entry(4, 50, 1000, 1, 0, 5),
            entry(5, 10, 1000, 1, 0, 120),
        ];
        let moved = quality_order(&mut entries, Some(0));
        assert_eq!(moved, Some(0));
        assert_eq!(entries.iter().map(|e| e.id).collect::<Vec<_>>(), [1, 4, 3, 2, 5]);
    }

    #[test]
    fn quality_swaps_the_first_gm_server_with_the_last_row() {
        let mut entries = vec![entry(1, 1, 10, 1, 0, 9999), entry(2, 1, 10, 1, SERVER_GM_FLAG, 9999), entry(3, 1, 10, 1, 0, 9999)];
        quality_order(&mut entries, None);
        assert_eq!(entries.iter().map(|e| e.id).collect::<Vec<_>>(), [1, 3, 2]);
    }

    #[test]
    fn quality_swaps_the_last_server_with_the_first_row_and_keeps_a_special_first_row_in_place() {
        let mut entries: Vec<_> = (1..=4).map(|id| entry(id, 1, 10, 1, 0, 9999)).collect();
        assert_eq!(quality_order(&mut entries, Some(2)), Some(0));
        assert_eq!(entries.iter().map(|e| e.id).collect::<Vec<_>>(), [3, 2, 1, 4]);
        let mut entries = vec![entry(1, 1, 10, 1, SERVER_GM_FLAG, 9999), entry(2, 1, 10, 1, 0, 9999), entry(3, 1, 10, 1, 0, 9999)];
        assert_eq!(quality_order(&mut entries, None), Some(0));
        assert_eq!(entries.iter().map(|e| e.id).collect::<Vec<_>>(), [1, 2, 3]);
        assert_eq!(quality_order(&mut Vec::new(), None), None);
    }

    #[test]
    fn ok_and_double_click_choose_once_until_the_timer_releases_the_lock() {
        let mut select = select();
        select.entries = vec![entry(1, 1, 10, 1, 0, 9999), entry(2, 1, 10, 1, 0, 9999)];
        select.finish();
        assert_eq!(select.selected, Some(0));
        let (ox, oy) = select.origin();
        let row1 = (ox + LIST_RECT.x + 3, oy + LIST_RECT.y + HEADER_HEIGHT + ROW_HEIGHT + 2);
        select.mouse_down(row1.0, row1.1);
        assert_eq!(select.selected, Some(1));
        assert_eq!(select.double_click(row1.0, row1.1), Some(ServerAction::Choose { id: 2, ip: 0x7f000001, port: 7777 }));
        assert_eq!(select.enter(), None);
        select.tick(LOGIN_LOCK_MS);
        assert!(matches!(select.enter(), Some(ServerAction::Choose { id: 2, .. })));
    }

    #[test]
    fn cancel_button_and_the_last_server_selection_on_finish() {
        let mut select = select();
        select.last_server = Some(3);
        select.entries = vec![entry(1, 1, 10, 1, 0, 9999), entry(3, 1, 10, 1, 0, 9999)];
        select.finish();
        assert_eq!(select.entries[0].id, 3);
        assert_eq!(select.selected, Some(0));
        let (ox, oy) = select.origin();
        let (x, y) = (ox + CANCEL_RECT.x + 1, oy + CANCEL_RECT.y + 1);
        select.mouse_down(x, y);
        assert_eq!(select.mouse_up(x, y), Some(ServerAction::Cancel));
    }

    #[test]
    fn rows_paint_the_status_text_in_its_color_and_the_forbidden_name_in_red() {
        let mut select = select();
        let mut forbidden = entry(2, 900, 1000, 1, 0, 9999);
        forbidden.forbidden = true;
        select.entries = vec![entry(1, 0, 1000, 0, 0, 9999), forbidden];
        let mut list = DrawList { capture_text: true, ..Default::default() };
        select.paint(&mut list, &font());
        let color = |text: &str| list.text_runs.iter().find(|run| run.text == text).map(|run| run.color);
        assert_eq!(color("s456"), Some(RED.rgba()));
        assert_eq!(color("s458"), Some(FULL_COLOR.rgba()));
        assert_eq!(color("S2"), Some(RED.rgba()));
    }

    #[test]
    fn the_help_viewer_lays_out_html_scrolls_inside_its_rect_and_reports_clicked_links() {
        let mut screen = ServerSelect::new((1024, 768), ServerLabels::default());
        let font = font();
        let rows: String = (0..40).map(|i| format!("linha {i}<br>")).collect();
        screen.set_help(&format!("<html><body><a link=\"..\\L2text\\server_help_5.htm\">Relax</a><br>{rows}</body></html>"), &font);
        assert!(screen.help_height > HELP_RECT.h);
        let (ox, oy) = screen.origin();
        let (px, py) = (ox + HELP_RECT.x + 3, oy + HELP_RECT.y + 3);
        screen.mouse_down_help(px, py, &font);
        assert_eq!(screen.mouse_up_help(px, py, &font), Some(ServerAction::Help("..\\L2text\\server_help_5.htm".into())));
        screen.mouse_down_help(px, py, &font);
        assert_eq!(screen.mouse_up_help(px + 200, py + 100, &font), None);
        let bar_x = ox + HELP_RECT.x + HELP_RECT.w - 5;
        let bar_down = oy + HELP_RECT.y + HELP_RECT.h - 5;
        assert!(screen.help_bar_down(bar_x, bar_down));
        assert!(screen.help_bar_up(bar_x, bar_down));
        assert_eq!(screen.help_scroll, screen.help_step);
        let thumb_y = oy + HELP_RECT.y + crate::scroll::BUTTON_SIZE + screen.help_bar.slider.thumb_y + 2;
        assert!(screen.help_bar_down(bar_x, thumb_y));
        assert!(screen.help_bar_move(thumb_y + 30));
        assert!(screen.help_scroll > screen.help_step);
        assert!(screen.help_bar_up(bar_x, thumb_y + 30));
        assert!(!screen.help_bar_move(0));
        screen.set_help_scroll(0);
        screen.wheel_at(px, py, -1);
        assert_eq!(screen.help_scroll, HELP_WHEEL_LINES * screen.help_step);
        screen.wheel_at(px, py, 1);
        assert_eq!(screen.help_scroll, 0);
        screen.wheel_at(px, py, 1);
        assert_eq!(screen.help_scroll, 0);
        screen.entries = (1..=20).map(|id| entry(id, 1, 100, 1, 0, 9999)).collect();
        screen.wheel_at(ox + LIST_RECT.x + 5, oy + LIST_RECT.y + 30, -1);
        assert_eq!(screen.first_row, 3);
        assert_eq!(screen.help_scroll, 0);
    }

    #[test]
    fn pings_fill_the_column_and_the_last_reply_sorts_by_quality_like_the_console_handler() {
        let mut screen = ServerSelect::new((1024, 768), ServerLabels::default());
        screen.entries = vec![entry(1, 10, 100, 1, 0, NO_PING), entry(2, 10, 100, 1, 0, NO_PING), entry(3, 10, 100, 1, 0, NO_PING)];
        screen.start_pings();
        screen.set_ping(3, 120);
        screen.set_ping(9, 5);
        assert_eq!(screen.pending_pings, 2);
        assert_eq!(screen.entries.iter().map(|entry| entry.id).collect::<Vec<_>>(), [1, 2, 3]);
        screen.set_ping(1, NO_PING);
        screen.set_ping(2, 30);
        assert_eq!(screen.pending_pings, 0);
        let ids: Vec<u8> = screen.entries.iter().map(|entry| entry.id).collect();
        assert_eq!(ids[0], 1);
        assert_eq!(&ids[1..], [2, 3]);
        assert_eq!(screen.entries.iter().find(|entry| entry.id == 3).map(|entry| entry.ping), Some(120));
    }

    #[test]
    fn a_header_click_sorts_with_the_ncserverselectlistctrl_rules_0x101013a0() {
        let mut select = select();
        select.entries = vec![entry(3, 10, 3000, 1, 0, 80), entry(1, 20, 3000, 1, 8, 30), entry(2, 5, 3000, 1, 0, 50), entry(4, 5, 3000, 1, 0, 10)];
        select.selected = Some(2);
        select.sort_by_column(NUMERIC_COLUMN);
        assert_eq!(select.entries.iter().map(|e| e.id).collect::<Vec<_>>(), vec![4, 3, 2, 1]);
        assert_eq!(select.selected, Some(2));
        select.sort_by_column(NUMERIC_COLUMN);
        assert_eq!(select.entries.iter().map(|e| e.id).collect::<Vec<_>>(), vec![3, 2, 4, 1]);
        assert_eq!(select.selected, Some(1));
        assert!(select.ascending[NUMERIC_COLUMN]);
    }

    #[test]
    fn the_header_cells_are_hit_by_column_and_press_swaps_the_tab_textures() {
        let mut select = select();
        let (ox, oy) = select.origin();
        let (x, y) = (ox + LIST_RECT.x + COLUMN_WIDTHS[0] + 5, oy + LIST_RECT.y + 3);
        assert_eq!(select.hit(x, y), Some(Hit::Header(1)));
        select.mouse_down(x, y);
        let mut list = DrawList::default();
        select.paint(&mut list, &font());
        assert!(list.quads.iter().any(|q| &*q.texture == TAB_DOWN[0]));
        assert_eq!(select.mouse_up(x, y), None);
        let mut list = DrawList::default();
        select.paint(&mut list, &font());
        assert!(list.quads.iter().all(|q| &*q.texture != TAB_DOWN[0]));
    }
}
