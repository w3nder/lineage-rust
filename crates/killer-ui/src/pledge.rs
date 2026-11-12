use crate::button::TextButton;
use crate::color::{Argb, GOLD, TEXT};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::header_ctrl::HeaderCtrl;
use crate::list_ctrl::{ColumnContract, TAB_TEXTURES};
use crate::window::Rect;

pub const PAGE_SIZE: (i32, i32) = (256, 335);
pub const PLEDGE_BACK: &str = "L2UI_CH3.BloodHoodWnd.BloodHood_Back";
pub const LOGON: &str = "L2UI_CH3.BloodHoodWnd.BloodHood_Logon";
pub const LOGOFF: &str = "L2UI_CH3.BloodHoodWnd.BloodHood_Logoff";
pub const HEADER_TEXTURE: &str = "L2UI_CH3.ListCtrl.tab1";
pub const HEADER_DOWN_TEXTURE: &str = "L2UI_CH3.ListCtrl.tab1_down";
pub const ROW_BACK: &str = "L2UI_ch3.etc.textbackline";
pub const SELECT_LEFT: &str = "L2UI_CH3.ListCtrl.TextSelect";
pub const SELECT_RIGHT: &str = "L2UI_CH3.ListCtrl.TextSelect2";
pub const BTN_NORMAL: &str = "L2UI_ch3.button.btn1_normal";
pub const BTN_DOWN: &str = "L2UI_ch3.button.btn1_normalon";
pub const LABEL_COLOR: Argb = Argb(0xFFA3A3A3);
pub const LIST_RECT: Rect = Rect::new(8, 62, 240, 137);
pub const HEADER_HEIGHT: i32 = 19;
pub const SCROLL_RECT: Rect = Rect::new(LIST_RECT.x, LIST_RECT.y + HEADER_HEIGHT, LIST_RECT.w, LIST_RECT.h - HEADER_HEIGHT);
pub const ROW_HEIGHT: i32 = 17;
pub const VISIBLE_ROWS: usize = 7;
pub const COLUMN_WIDTHS: [i32; 4] = [124, 32, 32, 52];
pub const COLUMN_SYSSTRINGS: [u32; 4] = [50, 88, 391, 346];
pub const COLUMN_CONTRACTS: [ColumnContract; 4] = [
    ColumnContract { width: 124, height: 19, sysstring: 50, flag_c: 1, flag_10: 0, textures: TAB_TEXTURES },
    ColumnContract { width: 32, height: 19, sysstring: 88, flag_c: 1, flag_10: 0, textures: TAB_TEXTURES },
    ColumnContract { width: 32, height: 19, sysstring: 391, flag_c: 1, flag_10: 0, textures: TAB_TEXTURES },
    ColumnContract { width: 52, height: 19, sysstring: 346, flag_c: 1, flag_10: 0, textures: TAB_TEXTURES },
];
pub const CELL_TEXT_X: i32 = 10;
pub const CELL_TEXT_Y: i32 = 3;
pub const STATUS_ICON_SIZE: (i32, i32) = (32, 11);
pub const CLASS_ICON_SIZE: i32 = 11;
pub const BUTTON_SIZE: (i32, i32) = (76, 23);
pub const BUTTON_XS: [i32; 3] = [11, 91, 171];
pub const BUTTON_YS: [i32; 5] = [202, 227, 252, 281, 306];
pub const BUTTON_SYSSTRINGS: [u32; 15] =
    [330, 331, 842, 338, 666, 667, 1205, 1206, 337, 332, 843, 1207, 1208, 1209, 387];
pub const NO_CLAN_SYSSTRING: u32 = 431;
pub const CASTLE_EXISTS_SYSSTRING: u32 = 345;
pub const HIDEOUT_EXISTS_SYSSTRING: u32 = 344;
pub const VIEW_RIGHTS_SYSSTRING: u32 = 883;
pub const GM_HIDEOUT_GAP: i32 = 63;
pub const GM_ENABLED_BUTTONS: [PledgeButton; 5] =
    [PledgeButton::Authorize, PledgeButton::SetInsignia, PledgeButton::DeleteInsignia, PledgeButton::Penalty, PledgeButton::Community];
pub const LEADER_SYSSTRING: u32 = 342;
pub const DISPERSING_SYSSTRING: u32 = 341;
pub const AT_PEACE_SYSSTRING: u32 = 894;
pub const IN_WAR_SYSSTRING: u32 = 340;
pub const LV_SYSSTRING: u32 = 88;
pub const BASE_SYSSTRING: u32 = 343;
pub const NONE_SYSSTRING: u32 = 27;
pub const DISSOLVING_STATE: i32 = 3;
pub const NAME_POS: (i32, i32) = (19, 10);
pub const LEADER_LABEL_POS: (i32, i32) = (19, 26);
pub const LEADER_POS: (i32, i32) = (61, 26);
pub const STATE_POS: (i32, i32) = (192, 10);
pub const LV_LABEL_POS: (i32, i32) = (192, 26);
pub const LV_POS: (i32, i32) = (208, 26);
pub const BASE_LABEL_POS: (i32, i32) = (19, 42);
pub const BASE_POS: (i32, i32) = (61, 42);
pub const BASE_SEPARATOR: &str = ",";
pub const BASE_SEPARATOR_ADVANCE: i32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PledgeButton {
    Invite,
    Title,
    DeleteTitle,
    Dismiss,
    Privileges,
    Authorize,
    DeclareWar,
    EndWar,
    Leave,
    SetCrest,
    DeleteCrest,
    Penalty,
    SetInsignia,
    DeleteInsignia,
    Community,
}

pub const BUTTONS: [PledgeButton; 15] = [
    PledgeButton::Invite,
    PledgeButton::Title,
    PledgeButton::DeleteTitle,
    PledgeButton::Dismiss,
    PledgeButton::Privileges,
    PledgeButton::Authorize,
    PledgeButton::DeclareWar,
    PledgeButton::EndWar,
    PledgeButton::Leave,
    PledgeButton::SetCrest,
    PledgeButton::DeleteCrest,
    PledgeButton::Penalty,
    PledgeButton::SetInsignia,
    PledgeButton::DeleteInsignia,
    PledgeButton::Community,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PledgeRights {
    pub leader: bool,
    pub invite: bool,
    pub title: bool,
    pub crest: bool,
    pub war: bool,
    pub in_clan: bool,
    pub noble: bool,
}

pub fn button_rect(index: usize) -> Rect {
    Rect::new(BUTTON_XS[index % 3], BUTTON_YS[index / 3], BUTTON_SIZE.0, BUTTON_SIZE.1)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PledgeHit {
    Button(PledgeButton),
    Row(usize),
    Header(usize),
    Page,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MemberRow {
    pub name: String,
    pub level: i32,
    pub class_id: i32,
    pub online: bool,
    pub object_id: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PledgeView {
    pub name: Option<String>,
    pub leader: String,
    pub level: i32,
    pub dissolving: bool,
    pub at_war: bool,
    pub castle: Option<String>,
    pub hideout: Option<String>,
    pub members: Vec<MemberRow>,
    pub selected: Option<usize>,
    pub scroll_row: usize,
    pub scroll: crate::scroll::ScrollWnd,
    pub scroll_pressed: Option<crate::scroll::ScrollHit>,
    pub pressed: Option<PledgeButton>,
    pub texts: Vec<(u32, String)>,
    pub gm: bool,
    pub gm_castle: bool,
    pub gm_hideout: bool,
    pub enabled: [bool; 15],
    pub headers: Vec<HeaderCtrl>,
    pub sort_forward: [bool; 4],
    pub header_pressed: Option<usize>,
    pub header_left_held: bool,
    pub hover: Option<usize>,
    pub class_names: Vec<String>,
}

fn column_headers() -> Vec<HeaderCtrl> {
    let mut x = 0;
    COLUMN_CONTRACTS
        .iter()
        .enumerate()
        .map(|(index, column)| {
            let mut header = HeaderCtrl::new(Rect::new(x, 0, column.width, column.height));
            header.set_info(&column.header_info());
            header.notify_id = index as i32;
            x += column.width;
            header
        })
        .collect()
}

impl Default for PledgeView {
    fn default() -> PledgeView {
        PledgeView {
            name: None,
            leader: String::new(),
            level: 0,
            dissolving: false,
            at_war: false,
            castle: None,
            hideout: None,
            members: Vec::new(),
            selected: None,
            scroll_row: 0,
            scroll: crate::scroll::ScrollWnd::default(),
            scroll_pressed: None,
            pressed: None,
            texts: Vec::new(),
            gm: false,
            gm_castle: false,
            gm_hideout: false,
            enabled: [true; 15],
            headers: column_headers(),
            sort_forward: [true; 4],
            header_pressed: None,
            header_left_held: false,
            hover: None,
            class_names: Vec::new(),
        }
    }
}

impl PledgeView {
    pub fn button_enabled(&self, index: usize) -> bool {
        if self.gm { GM_ENABLED_BUTTONS.contains(&BUTTONS[index]) } else { self.enabled[index] }
    }

    pub fn show(&mut self, rights: &PledgeRights) {
        let set = |enabled: &mut [bool; 15], list: &[PledgeButton], value: bool| {
            for button in list {
                if let Some(index) = BUTTONS.iter().position(|candidate| candidate == button) {
                    enabled[index] = value;
                }
            }
        };
        use PledgeButton::*;
        let e = &mut self.enabled;
        if rights.leader {
            set(e, &[Invite, Title, SetCrest, DeclareWar, EndWar, Dismiss, SetInsignia, DeleteInsignia, DeleteTitle, DeleteCrest, Privileges, Authorize, Community], true);
            set(e, &[Leave], false);
            set(e, &[Penalty], true);
        } else {
            set(e, &[Invite, Title, SetCrest, DeclareWar, EndWar, SetInsignia, DeleteInsignia, Dismiss, Authorize], false);
            set(e, &[Leave, Community, Penalty, Privileges], true);
        }
        if rights.invite {
            set(e, &[Invite], true);
        }
        if rights.title {
            set(e, &[Title], true);
        }
        if rights.crest {
            set(e, &[SetCrest], true);
        }
        if rights.war {
            set(e, &[DeclareWar, EndWar], true);
        }
        if !rights.in_clan {
            set(e, &BUTTONS, false);
        }
        if rights.noble {
            set(e, &[Title, DeleteTitle], true);
        }
    }

    pub fn with_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> PledgeView {
        let ids = [
            NO_CLAN_SYSSTRING,
            LEADER_SYSSTRING,
            DISPERSING_SYSSTRING,
            AT_PEACE_SYSSTRING,
            IN_WAR_SYSSTRING,
            LV_SYSSTRING,
            BASE_SYSSTRING,
            NONE_SYSSTRING,
            CASTLE_EXISTS_SYSSTRING,
            HIDEOUT_EXISTS_SYSSTRING,
            VIEW_RIGHTS_SYSSTRING,
        ];
        let texts = ids
            .into_iter()
            .chain(COLUMN_SYSSTRINGS)
            .chain(BUTTON_SYSSTRINGS)
            .map(|id| (id, lookup(id).unwrap_or_else(|| format!("sysstring {id}"))))
            .collect();
        let class_names = (0..crate::party_room::CLASS_NAME_COUNT).map(|class| lookup(crate::detail_status::class_name_sysstring(class)).unwrap_or_default()).collect();
        PledgeView { texts, class_names, ..Default::default() }
    }

    pub fn text(&self, id: u32) -> &str {
        self.texts.iter().find(|(k, _)| *k == id).map_or("", |(_, t)| t.as_str())
    }

    pub fn buttons(&self) -> Vec<TextButton> {
        BUTTON_SYSSTRINGS
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let id = if self.gm && BUTTONS[i] == PledgeButton::Authorize { VIEW_RIGHTS_SYSSTRING } else { *id };
                let mut button = TextButton::new(button_rect(i), BTN_NORMAL, BTN_DOWN, self.text(id));
                button.enabled = self.button_enabled(i);
                button
            })
            .collect()
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<PledgeHit> {
        if !Rect::new(0, 0, PAGE_SIZE.0, PAGE_SIZE.1).contains(x, y) {
            return None;
        }
        for (i, button) in BUTTONS.iter().enumerate() {
            if button_rect(i).contains(x, y) && self.button_enabled(i) {
                return Some(PledgeHit::Button(*button));
            }
        }
        if LIST_RECT.contains(x, y) {
            if self.scrollbar_shown() && x >= SCROLL_RECT.x + SCROLL_RECT.w - crate::scroll::SCROLL_WIDTH && y >= SCROLL_RECT.y {
                return Some(PledgeHit::Page);
            }
            let ly = y - LIST_RECT.y;
            if ly < HEADER_HEIGHT {
                return Some(match self.header_at(x - LIST_RECT.x, ly) {
                    Some(column) => PledgeHit::Header(column),
                    None => PledgeHit::Page,
                });
            }
            let row = self.scroll_row + ((ly - HEADER_HEIGHT) / ROW_HEIGHT) as usize;
            if row < self.members.len() && row < self.scroll_row + VISIBLE_ROWS {
                return Some(PledgeHit::Row(row));
            }
        }
        Some(PledgeHit::Page)
    }

    fn header_at(&self, x: i32, y: i32) -> Option<usize> {
        self.headers.iter().position(|header| header.contains(x, y))
    }

    pub fn header_down(&mut self, x: i32, y: i32) -> Option<usize> {
        let (px, py) = (x - LIST_RECT.x, y - LIST_RECT.y);
        let column = self.header_at(px, py)?;
        self.header_left_held = true;
        self.header_pressed = Some(column);
        self.headers[column].on_lbutton_down(px, py).sound
    }

    pub fn row_tooltip(&self) -> Option<crate::tooltip::Tooltip> {
        let index = self.hover?;
        if self.selected != Some(index) {
            return None;
        }
        let member = self.members.get(index)?;
        let class = usize::try_from(member.class_id).ok().and_then(|class| self.class_names.get(class)).cloned().unwrap_or_default();
        Some(crate::tooltip::Tooltip {
            row: true,
            row_style: 0x8000,
            lines: vec![crate::tooltip::TooltipLine { label: self.text(COLUMN_SYSSTRINGS[2]).to_string(), value: class, color: crate::tooltip::VALUE_COLOR }],
            ..Default::default()
        })
    }

    pub fn header_move(&mut self, x: i32, y: i32) {
        self.hover = match self.hit(x, y) {
            Some(PledgeHit::Row(row)) => Some(row),
            _ => None,
        };
        let (px, py) = (x - LIST_RECT.x, y - LIST_RECT.y);
        let held = self.header_left_held;
        for header in &mut self.headers {
            header.on_mouse_move(px, py, held);
        }
    }

    pub fn header_up(&mut self, x: i32, y: i32) {
        let (px, py) = (x - LIST_RECT.x, y - LIST_RECT.y);
        self.header_left_held = false;
        if let Some(column) = self.header_pressed.take()
            && let Some(notify) = self.headers[column].on_lbutton_up(px, py).notify
        {
            self.sort(notify as usize);
        }
    }

    fn sort_key(row: &MemberRow, column: usize) -> String {
        match column {
            0 => row.name.clone(),
            1 => row.level.to_string(),
            2 => row.class_id.to_string(),
            _ => if row.online { "1" } else { "2" }.to_string(),
        }
    }

    pub fn sort(&mut self, column: usize) {
        if column >= 4 {
            return;
        }
        let forward = self.sort_forward[column];
        let selected = self.selected.and_then(|index| self.members.get(index)).cloned();
        let count = self.members.len();
        for pass in 0..count {
            let mut at = 0;
            while at + pass < count && at + 1 < count {
                let ordering = crate::list_ctrl::wcsicmp(&Self::sort_key(&self.members[at], column), &Self::sort_key(&self.members[at + 1], column));
                let swap = if forward { ordering == std::cmp::Ordering::Greater } else { ordering == std::cmp::Ordering::Less };
                if swap {
                    self.members.swap(at, at + 1);
                }
                at += 1;
            }
        }
        self.sort_forward[column] = !forward;
        if let Some(row) = selected {
            self.selected = self.members.iter().position(|member| *member == row);
        }
    }

    pub fn apply_members(&mut self, incoming: Vec<MemberRow>, reset: bool) {
        let same = |a: &str, b: &str| crate::list_ctrl::wcsicmp(a, b) == std::cmp::Ordering::Equal;
        if reset {
            self.members = incoming;
            self.selected = None;
            self.scroll_row = 0;
        } else {
            let selected = self.selected.and_then(|index| self.members.get(index)).map(|member| member.name.clone());
            let mut ordered: Vec<MemberRow> = self
                .members
                .iter()
                .filter_map(|old| incoming.iter().find(|new| same(&new.name, &old.name)).cloned())
                .collect();
            for new in incoming {
                if !ordered.iter().any(|known| same(&known.name, &new.name)) {
                    ordered.push(new);
                }
            }
            self.members = ordered;
            self.selected = selected.and_then(|name| self.members.iter().position(|member| same(&member.name, &name)));
        }
        self.clamp_scroll();
    }

    pub fn wheel(&mut self, delta: i32) {
        self.scroll_row = crate::inventory::item_wnd_wheel(self.scroll_row as i32, VISIBLE_ROWS as i32, self.members.len() as i32, delta).max(0) as usize;
        self.sync_scroll();
    }

    pub fn clamp_scroll(&mut self) {
        self.scroll_row = self.scroll_row.min(self.members.len().saturating_sub(VISIBLE_ROWS));
        self.sync_scroll();
    }

    fn sync_scroll(&mut self) {
        if self.scroll.slider.height == 0 {
            self.scroll = crate::scroll::ScrollWnd::new(SCROLL_RECT.w, SCROLL_RECT.h);
        }
        self.scroll.slider.set_range(self.members.len() as i32, VISIBLE_ROWS as i32);
        self.scroll.slider.set_pos(self.scroll_row as i32);
    }

    fn scrollbar_shown(&self) -> bool {
        self.members.len() > VISIBLE_ROWS
    }

    pub fn scroll_down(&mut self, x: i32, y: i32) -> bool {
        if !self.scrollbar_shown() {
            return false;
        }
        self.sync_scroll();
        let r = SCROLL_RECT;
        let Some(hit) = self.scroll.hit(r.w, r.h, x - r.x, y - r.y) else { return false };
        self.scroll_pressed = Some(hit);
        self.scroll.up_pressed = hit == crate::scroll::ScrollHit::Up;
        self.scroll.down_pressed = hit == crate::scroll::ScrollHit::Down;
        if let crate::scroll::ScrollHit::Slider(slider_y) = hit {
            self.scroll.slider.mouse_down(slider_y);
        }
        true
    }

    pub fn scroll_move(&mut self, y: i32) -> bool {
        if !self.scroll.slider.dragging {
            return false;
        }
        if let Some(first) = self.scroll.slider.mouse_move(y - SCROLL_RECT.y - crate::scroll::BUTTON_SIZE) {
            self.scroll_row = (first.max(0) as usize).min(self.members.len().saturating_sub(VISIBLE_ROWS));
        }
        true
    }

    pub fn scroll_up(&mut self, over: Option<(i32, i32)>) {
        let pressed = self.scroll_pressed.take();
        let r = SCROLL_RECT;
        let hit = over.and_then(|(x, y)| self.scroll.hit(r.w, r.h, x - r.x, y - r.y));
        self.scroll.up_pressed = false;
        self.scroll.down_pressed = false;
        self.scroll.slider.mouse_up();
        if pressed == hit {
            let first = self.scroll_row as i32;
            let (visible, total) = (VISIBLE_ROWS as i32, self.members.len() as i32);
            self.scroll_row = match pressed {
                Some(crate::scroll::ScrollHit::Up) => crate::scroll::line_up(first, visible, total),
                Some(crate::scroll::ScrollHit::Down) => crate::scroll::line_down(first, visible, total),
                _ => first,
            }
            .max(0) as usize;
            self.sync_scroll();
        }
    }

    pub fn state_text(&self) -> &str {
        if self.dissolving {
            self.text(DISPERSING_SYSSTRING)
        } else if self.at_war {
            self.text(IN_WAR_SYSSTRING)
        } else {
            self.text(AT_PEACE_SYSSTRING)
        }
    }

    fn paint_gm_header(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        if let Some(name) = self.name.as_ref().filter(|name| !name.is_empty()) {
            list.draw_text(x + NAME_POS.0, y + NAME_POS.1, GOLD, name, font);
        }
        list.draw_text(x + LEADER_LABEL_POS.0, y + LEADER_LABEL_POS.1, LABEL_COLOR, self.text(LEADER_SYSSTRING), font);
        if !self.leader.is_empty() {
            list.draw_text(x + LEADER_POS.0, y + LEADER_POS.1, GOLD, &self.leader, font);
        }
        list.draw_text(x + LV_LABEL_POS.0, y + LV_LABEL_POS.1, LABEL_COLOR, self.text(LV_SYSSTRING), font);
        list.draw_text(x + LV_POS.0, y + LV_POS.1, GOLD, &self.level.to_string(), font);
        list.draw_text(x + BASE_LABEL_POS.0, y + BASE_LABEL_POS.1, LABEL_COLOR, self.text(BASE_SYSSTRING), font);
        match (self.gm_castle, self.gm_hideout) {
            (true, hideout) => {
                list.draw_text(x + BASE_POS.0, y + BASE_POS.1, GOLD, self.text(CASTLE_EXISTS_SYSSTRING), font);
                if hideout {
                    let width = font.measure(self.text(CASTLE_EXISTS_SYSSTRING)) as i32;
                    list.draw_text(x + width + GM_HIDEOUT_GAP, y + BASE_POS.1, GOLD, self.text(HIDEOUT_EXISTS_SYSSTRING), font);
                }
            }
            (false, true) => {
                list.draw_text(x + BASE_POS.0, y + BASE_POS.1, GOLD, self.text(HIDEOUT_EXISTS_SYSSTRING), font);
            }
            (false, false) => {}
        }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let (w, h) = PAGE_SIZE;
        list.draw_texture(x, y, w, h, 0, 0, w, h, PLEDGE_BACK, 1.0);
        if self.gm {
            self.paint_gm_header(list, x, y, font);
            self.paint_list(list, x + LIST_RECT.x, y + LIST_RECT.y, font);
            if self.scrollbar_shown() {
                self.scroll.paint(list, x + SCROLL_RECT.x, y + SCROLL_RECT.y, SCROLL_RECT.w, SCROLL_RECT.h);
            }
            for (button, text_button) in BUTTONS.iter().zip(self.buttons()) {
                text_button.paint(list, x, y, self.pressed == Some(*button), font);
            }
            return;
        }
        let has_name = self.name.as_ref().is_some_and(|name| !name.is_empty());
        let has_leader = !self.leader.is_empty();
        let name = match self.name.as_ref().filter(|name| !name.is_empty()) {
            Some(name) => name.clone(),
            None => self.text(NO_CLAN_SYSSTRING).to_string(),
        };
        list.draw_text(x + NAME_POS.0, y + NAME_POS.1, GOLD, &name, font);
        list.draw_text(x + LEADER_LABEL_POS.0, y + LEADER_LABEL_POS.1, LABEL_COLOR, self.text(LEADER_SYSSTRING), font);
        if has_leader {
            list.draw_text(x + LEADER_POS.0, y + LEADER_POS.1, GOLD, &self.leader, font);
            list.draw_text(x + STATE_POS.0, y + STATE_POS.1, GOLD, self.state_text(), font);
        }
        list.draw_text(x + LV_LABEL_POS.0, y + LV_LABEL_POS.1, LABEL_COLOR, self.text(LV_SYSSTRING), font);
        if has_name {
            list.draw_text(x + LV_POS.0, y + LV_POS.1, GOLD, &self.level.to_string(), font);
        }
        list.draw_text(x + BASE_LABEL_POS.0, y + BASE_LABEL_POS.1, LABEL_COLOR, self.text(BASE_SYSSTRING), font);
        if has_leader {
            match (&self.castle, &self.hideout) {
                (None, None) => {
                    list.draw_text(x + BASE_POS.0, y + BASE_POS.1, GOLD, self.text(NONE_SYSSTRING), font);
                }
                (Some(castle), None) => {
                    list.draw_text(x + BASE_POS.0, y + BASE_POS.1, GOLD, castle, font);
                }
                (castle, Some(hideout)) => {
                    list.draw_text(x + BASE_POS.0, y + BASE_POS.1, GOLD, hideout, font);
                    if let Some(castle) = castle {
                        let hideout_width = font.measure(hideout) as i32;
                        list.draw_text(x + BASE_POS.0 + hideout_width, y + BASE_POS.1, GOLD, BASE_SEPARATOR, font);
                        list.draw_text(x + BASE_POS.0 + hideout_width + BASE_SEPARATOR_ADVANCE, y + BASE_POS.1, GOLD, castle, font);
                    }
                }
            }
        }
        self.paint_list(list, x + LIST_RECT.x, y + LIST_RECT.y, font);
        if self.scrollbar_shown() {
            self.scroll.paint(list, x + SCROLL_RECT.x, y + SCROLL_RECT.y, SCROLL_RECT.w, SCROLL_RECT.h);
        }
        for (button, text_button) in BUTTONS.iter().zip(self.buttons()) {
            text_button.paint(list, x, y, self.pressed == Some(*button), font);
        }
    }

    fn paint_list(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        for header in &self.headers {
            let label = usize::try_from(header.label).ok().map(|id| self.text(id as u32).to_string());
            header.paint(list, (x, y), label.as_deref(), font);
        }
        for (slot, row) in self.members.iter().enumerate().skip(self.scroll_row).take(VISIBLE_ROWS) {
            let visual = slot - self.scroll_row;
            let ry = y + HEADER_HEIGHT + visual as i32 * ROW_HEIGHT;
            if visual % 2 == 1 {
                list.draw_texture(x, ry - 1, LIST_RECT.w, ROW_HEIGHT, 0, 0, 8, 15, ROW_BACK, 1.0);
            }
            if self.selected == Some(slot) {
                list.draw_texture(x + 1, ry - 1, 16, ROW_HEIGHT, 0, 0, 16, 13, SELECT_LEFT, 1.0);
                list.draw_texture(x + 17, ry - 1, LIST_RECT.w - 17, ROW_HEIGHT, 0, 0, 32, 13, SELECT_RIGHT, 1.0);
            }
            let mut cx = x;
            list.draw_text(cx + CELL_TEXT_X, ry + CELL_TEXT_Y, TEXT, &row.name, font);
            cx += COLUMN_WIDTHS[0];
            list.draw_text(cx + CELL_TEXT_X, ry + CELL_TEXT_Y, TEXT, &row.level.to_string(), font);
            cx += COLUMN_WIDTHS[1];
            if let Some(texture) = crate::class_icon::texture(row.class_id) {
                let size = CLASS_ICON_SIZE;
                list.draw_texture(cx + COLUMN_WIDTHS[2] / 2 - size / 2, ry + ROW_HEIGHT / 2 - size / 2, size, size, 0, 0, size, size, texture, 1.0);
            }
            cx += COLUMN_WIDTHS[2];
            let (iw, ih) = STATUS_ICON_SIZE;
            let icon = if row.online { LOGON } else { LOGOFF };
            list.draw_texture(cx + COLUMN_WIDTHS[3] / 2 - iw / 2, ry + ROW_HEIGHT / 2 - ih / 2, iw, ih, 0, 0, iw, ih, icon, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let bytes = gly(&[(0x20, &glyphs)], (1024, 128));
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&bytes).unwrap())
    }

    fn view() -> PledgeView {
        let mut view = PledgeView::with_sysstrings(|id| Some(format!("s{id}")));
        view.name = Some("Canastra".into());
        view.leader = "Killer".into();
        view.level = 5;
        view.members = (0..9)
            .map(|i| MemberRow { name: format!("M{i}"), level: 40 + i, class_id: i, online: i % 2 == 0, object_id: 0 })
            .collect();
        view
    }

    #[test]
    fn buttons_are_76_by_23_in_three_columns_and_five_rows_with_their_sysstrings() {
        assert_eq!(button_rect(0), Rect::new(11, 202, 76, 23));
        assert_eq!(button_rect(5), Rect::new(171, 227, 76, 23));
        assert_eq!(button_rect(12), Rect::new(11, 306, 76, 23));
        assert_eq!(button_rect(14), Rect::new(171, 306, 76, 23));
        let view = view();
        assert_eq!(view.hit(20, 210), Some(PledgeHit::Button(PledgeButton::Invite)));
        assert_eq!(view.hit(180, 260), Some(PledgeHit::Button(PledgeButton::Leave)));
        assert_eq!(view.hit(100, 310), Some(PledgeHit::Button(PledgeButton::DeleteInsignia)));
        assert_eq!(view.hit(10, 70), Some(PledgeHit::Header(0)));
        assert_eq!(view.hit(8 + 124 + 32 + 5, 70), Some(PledgeHit::Header(2)));
        assert_eq!(view.hit(8 + 240 + 2, 70), Some(PledgeHit::Page));
        assert_eq!(view.hit(10, 62 + 19 + 1), Some(PledgeHit::Row(0)));
        assert_eq!(view.hit(10, 62 + 19 + 17 * 6 + 1), Some(PledgeHit::Row(6)));
        assert_eq!(view.hit(10, 62 + 19 + 17 * 7 + 1), Some(PledgeHit::Page));
        let scrolled = PledgeView { scroll_row: 2, ..view };
        assert_eq!(scrolled.hit(10, 62 + 19 + 1), Some(PledgeHit::Row(2)));
        assert_eq!(scrolled.hit(10, 62 + 19 + 17 * 6 + 1), Some(PledgeHit::Row(8)));
        assert_eq!(view_no_clan().hit(300, 10), None);
    }

    fn view_no_clan() -> PledgeView {
        PledgeView::with_sysstrings(|id| Some(format!("s{id}")))
    }

    #[test]
    fn on_paint_1004d4f0_moves_all_residence_branches_with_the_window() {
        for castle in [None, Some("Giran".to_owned())] {
            for hideout in [None, Some("Hall".to_owned())] {
                let mut view = view();
                view.castle = castle.clone();
                view.hideout = hideout;
                let mut origin = DrawList::default();
                let mut moved = DrawList::default();
                view.paint(&mut origin, 0, 0, &font());
                view.paint(&mut moved, 391, 127, &font());
                assert!(!origin.quads.is_empty());
                assert_eq!(origin.quads.len(), moved.quads.len());
                for (mut expected, actual) in origin.quads.into_iter().zip(moved.quads) {
                    expected.x += 391.0;
                    expected.y += 127.0;
                    assert_eq!(expected, actual);
                }
            }
        }
    }

    #[test]
    fn paint_puts_the_texts_of_on_paint_and_seven_rows_with_logon_icons() {
        let font = font();
        let view = view();
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 66, &font);
        assert_eq!((list.quads[0].texture.as_str(), list.quads[0].y, list.quads[0].h), (PLEDGE_BACK, 66.0, 335.0));
        let text: Vec<_> = list.quads.iter().filter(|q| q.texture == font.atlas).collect();
        assert_eq!((text[0].x, text[0].y, text[0].color), (19.0, 76.0, GOLD.rgba()));
        assert!(text.iter().any(|q| q.x == 192.0 && q.y == 76.0));
        assert!(text.iter().any(|q| q.x == 208.0 && q.y == 92.0));
        assert!(text.iter().any(|q| q.x == 61.0 && q.y == 108.0));
        let caps: Vec<_> = list.quads.iter().filter(|q| q.texture == HEADER_TEXTURE).collect();
        assert_eq!(caps.len(), 4);
        assert_eq!((caps[0].x, caps[0].y, caps[0].w, caps[0].h), (8.0, 128.0, 8.0, 19.0));
        assert_eq!(caps[3].x, 8.0 + 124.0 + 32.0 + 32.0);
        let middles: Vec<_> = list.quads.iter().filter(|q| q.texture == "L2UI_CH3.ListCtrl.tab2").collect();
        assert_eq!((middles[0].x, middles[0].w), (16.0, 108.0));
        let rights: Vec<_> = list.quads.iter().filter(|q| q.texture == "L2UI_CH3.ListCtrl.tab3").collect();
        assert_eq!((rights[0].x, rights[0].w), (8.0 + 124.0 - 8.0, 8.0));
        let logon = list.quads.iter().filter(|q| q.texture == LOGON).count();
        let logoff = list.quads.iter().filter(|q| q.texture == LOGOFF).count();
        let icons = list.quads.iter().filter(|q| q.texture.starts_with("L2UI_CH3.PartyWnd.party_styleicon")).count();
        assert_eq!(icons, 7);
        assert_eq!((logon, logoff), (4, 3));
        assert_eq!(list.quads.iter().filter(|q| q.texture == ROW_BACK).count(), 3);
        let names: Vec<_> = text.iter().filter(|q| q.x == 18.0).collect();
        assert_eq!(names[0].y, (66 + 62 + 19 + 3) as f32);
        let buttons = list.quads.iter().filter(|q| q.texture == BTN_NORMAL).count();
        assert_eq!(buttons, 15);
        let empty = view_no_clan();
        let mut list = DrawList::default();
        empty.paint(&mut list, 0, 0, &font);
        assert!(list.quads.iter().all(|q| q.texture != LOGON));
    }

    fn enabled_names(view: &PledgeView) -> Vec<PledgeButton> {
        BUTTONS.iter().enumerate().filter(|(i, _)| view.button_enabled(*i)).map(|(_, b)| *b).collect()
    }

    #[test]
    fn show_window_1004c680_enables_the_buttons_by_leader_flag_rights_clan_and_noble() {
        use PledgeButton::*;
        let mut view = view();
        let member = PledgeRights { in_clan: true, ..Default::default() };
        view.show(&member);
        assert_eq!(enabled_names(&view), vec![DeleteTitle, Privileges, Leave, DeleteCrest, Penalty, Community]);
        view.show(&PledgeRights { invite: true, war: true, ..member });
        assert_eq!(enabled_names(&view), vec![Invite, DeleteTitle, Privileges, DeclareWar, EndWar, Leave, DeleteCrest, Penalty, Community]);
        view.show(&PledgeRights { title: true, crest: true, ..member });
        assert_eq!(enabled_names(&view), vec![Title, DeleteTitle, Privileges, Leave, SetCrest, DeleteCrest, Penalty, Community]);
        view.show(&PledgeRights { leader: true, in_clan: true, ..Default::default() });
        assert_eq!(enabled_names(&view).len(), 14);
        assert!(!enabled_names(&view).contains(&Leave));
        view.show(&PledgeRights { in_clan: false, ..member });
        assert!(enabled_names(&view).is_empty());
        view.show(&PledgeRights { noble: true, ..Default::default() });
        assert_eq!(enabled_names(&view), vec![Title, DeleteTitle]);
        view.show(&PledgeRights { leader: true, noble: true, ..Default::default() });
        assert_eq!(enabled_names(&view), vec![Title, DeleteTitle]);
    }

    #[test]
    fn row_tip_8000_shows_the_class_name_only_when_the_hovered_row_is_the_selected_one() {
        let mut view = PledgeView::with_sysstrings(|id| Some(format!("s{id}")));
        view.members = vec![MemberRow { name: "A".into(), level: 1, class_id: 2, online: true, object_id: 0 }, MemberRow { name: "B".into(), level: 1, class_id: 3, online: true, object_id: 0 }];
        let row_y = |row: i32| LIST_RECT.y + HEADER_HEIGHT + row * ROW_HEIGHT + 3;
        view.header_move(20, row_y(0));
        assert!(view.row_tooltip().is_none());
        view.selected = Some(1);
        assert!(view.row_tooltip().is_none());
        view.header_move(20, row_y(1));
        let tip = view.row_tooltip().unwrap();
        assert!(tip.row);
        assert_eq!((tip.lines[0].label.as_str(), tip.lines[0].value.as_str()), ("s391", view.class_names[3].as_str()));
        assert_eq!(view.class_names[3], format!("s{}", crate::detail_status::class_name_sysstring(3)));
        view.header_move(20, 5);
        assert!(view.row_tooltip().is_none());
    }

    #[test]
    fn a_non_leader_keeps_the_delete_title_and_delete_crest_buttons_it_had_before() {
        use PledgeButton::*;
        let mut view = view();
        view.show(&PledgeRights { in_clan: false, ..Default::default() });
        view.show(&PledgeRights { in_clan: true, ..Default::default() });
        assert_eq!(enabled_names(&view), vec![Privileges, Leave, Penalty, Community]);
        view.show(&PledgeRights { leader: true, in_clan: true, ..Default::default() });
        view.show(&PledgeRights { in_clan: true, ..Default::default() });
        assert_eq!(enabled_names(&view), vec![DeleteTitle, Privileges, Leave, DeleteCrest, Penalty, Community]);
    }

    #[test]
    fn disabled_buttons_are_painted_disabled_and_do_not_hit() {
        let mut view = view();
        view.show(&PledgeRights { in_clan: true, ..Default::default() });
        assert_eq!(view.hit(20, 210), Some(PledgeHit::Page));
        assert_eq!(view.hit(180, 260), Some(PledgeHit::Button(PledgeButton::Leave)));
        let buttons = view.buttons();
        assert!(!buttons[0].enabled);
        assert!(buttons[8].enabled);
    }

    #[test]
    fn on_paint_1004d4f0_draws_the_hideout_then_a_comma_then_the_castle_and_needs_a_leader_for_the_base() {
        let font = font();
        let mut both = view();
        both.castle = Some("Giran".into());
        both.hideout = Some("Hall".into());
        let mut list = DrawList::default();
        both.paint(&mut list, 0, 0, &font);
        let text: Vec<_> = list.quads.iter().filter(|q| q.texture == font.atlas && q.y == 42.0 && q.x >= 61.0).collect();
        let hall = "Hall".chars().count();
        assert!(text.iter().take(hall).all(|q| q.x >= 61.0));
        let width = font.measure("Hall") as f32;
        assert_eq!(text[hall].x, 61.0 + width);
        assert_eq!(text[hall + 1].x, 61.0 + width + BASE_SEPARATOR_ADVANCE as f32);
        assert_eq!(BASE_SEPARATOR, ",");
        let mut no_leader = view();
        no_leader.leader.clear();
        no_leader.castle = Some("Giran".into());
        let mut list = DrawList::default();
        no_leader.paint(&mut list, 0, 0, &font);
        assert!(list.quads.iter().filter(|q| q.texture == font.atlas).all(|q| !(q.y == 42.0 && q.x >= 61.0)));
        assert!(list.quads.iter().filter(|q| q.texture == font.atlas).all(|q| !(q.x == 192.0 && q.y == 10.0)));
        let mut empty_name = view();
        empty_name.name = Some(String::new());
        let mut list = DrawList::default();
        empty_name.paint(&mut list, 0, 0, &font);
        assert!(list.quads.iter().filter(|q| q.texture == font.atlas).all(|q| !(q.x == 208.0 && q.y == 26.0)));
    }

    #[test]
    fn nc_list_ctrl_10023000_sorts_the_member_rows_by_the_released_header_and_flips_the_direction() {
        let mut view = PledgeView::with_sysstrings(|id| Some(format!("s{id}")));
        let member = |name: &str, level, class_id, online| MemberRow { name: name.into(), level, class_id, online, object_id: 0 };
        view.members = vec![member("beta", 9, 12, false), member("Alfa", 40, 3, true), member("delta", 100, 20, true), member("Charlie", 2, 7, false)];
        view.selected = Some(2);
        let names = |view: &PledgeView| view.members.iter().map(|m| m.name.clone()).collect::<Vec<_>>();
        let (x, y) = (LIST_RECT.x + 10, LIST_RECT.y + 5);
        assert!(view.header_down(x, y).is_some());
        assert_eq!(view.headers[0].state, 1);
        view.header_up(x, y);
        assert_eq!(names(&view), ["Alfa", "beta", "Charlie", "delta"]);
        assert_eq!(view.selected, Some(3));
        assert_eq!(view.headers[0].state, 2);
        view.header_down(x, y);
        view.header_up(x, y);
        assert_eq!(names(&view), ["delta", "Charlie", "beta", "Alfa"]);
        assert_eq!(view.selected, Some(0));
        let level_x = LIST_RECT.x + COLUMN_WIDTHS[0] + 4;
        view.header_down(level_x, y);
        view.header_up(level_x, y);
        assert_eq!(view.members.iter().map(|m| m.level).collect::<Vec<_>>(), [100, 2, 40, 9]);
        let status_x = LIST_RECT.x + COLUMN_WIDTHS[0] + COLUMN_WIDTHS[1] + COLUMN_WIDTHS[2] + 4;
        view.header_down(status_x, y);
        view.header_up(status_x, y);
        assert_eq!(view.members.iter().map(|m| m.online).collect::<Vec<_>>(), [true, true, false, false]);
        let class_x = LIST_RECT.x + COLUMN_WIDTHS[0] + COLUMN_WIDTHS[1] + 4;
        view.header_down(class_x, y);
        view.header_up(class_x, y);
        assert_eq!(view.members.iter().map(|m| m.class_id).collect::<Vec<_>>(), [12, 20, 3, 7]);
        view.header_down(x, y);
        view.header_up(LIST_RECT.x + 500, y);
        assert_eq!(view.members.iter().map(|m| m.class_id).collect::<Vec<_>>(), [12, 20, 3, 7]);
        assert_eq!(view.sort_forward, [true, false, false, false]);
    }

    #[test]
    fn the_member_list_scrolls_by_wheel_buttons_and_slider_and_stays_inside_the_rows() {
        let members = (0..10).map(|i| MemberRow { name: format!("m{i}"), level: 1, class_id: 0, online: false, object_id: 0 }).collect();
        let mut view = PledgeView { members, ..Default::default() };
        view.clamp_scroll();
        view.wheel(-120);
        assert_eq!(view.scroll_row, 1);
        view.wheel(-120 * 20);
        assert_eq!(view.scroll_row, 3);
        view.wheel(120);
        assert_eq!(view.scroll_row, 2);
        let (px, py) = (SCROLL_RECT.x + SCROLL_RECT.w - 5, SCROLL_RECT.y + SCROLL_RECT.h - 5);
        assert!(view.scroll_down(px, py));
        view.scroll_up(Some((px, py)));
        assert_eq!(view.scroll_row, 3);
        let (ux, uy) = (SCROLL_RECT.x + SCROLL_RECT.w - 5, SCROLL_RECT.y + 5);
        assert!(view.scroll_down(ux, uy));
        view.scroll_up(Some((ux, uy)));
        assert_eq!(view.scroll_row, 2);
        assert_eq!(view.hit(ux, SCROLL_RECT.y + 40), Some(PledgeHit::Page));
        assert!(!view.scroll_down(LIST_RECT.x + 5, uy));
        view.members.truncate(4);
        view.clamp_scroll();
        assert_eq!(view.scroll_row, 0);
        assert!(!view.scroll_down(px, py));
        let mut list = DrawList::default();
        let font = BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &(0..95).map(|i| [i * 7, 6, 0, 13]).collect::<Vec<_>>())], (1024, 128))).unwrap());
        view.paint(&mut list, 0, 0, &font);
        assert!(!list.quads.iter().any(|quad| quad.texture == crate::scroll::UP_BTN));
        view.members = (0..10).map(|i| MemberRow { name: format!("m{i}"), level: 1, class_id: 0, online: false, object_id: 0 }).collect();
        view.clamp_scroll();
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font);
        assert!(list.quads.iter().any(|quad| quad.texture == crate::scroll::UP_BTN));
    }
}
