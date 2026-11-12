#[path = "siege_layout_generated.rs"]
mod layout;

use std::collections::BTreeMap;

use crate::button::TextButton;
use crate::color::Argb;
use crate::combo::ComboBox;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::tab_button::TabButton;
use crate::window::Rect;

pub const SIZE: (i32, i32) = layout::SIZE;
pub const PAGE: Rect = layout::RECT_70C;
pub const LIST: Rect = Rect::new(7, 18, 242, 200);
pub const COMBO: Rect = Rect::new(80, 60, 126, 15);
pub const TABS: Rect = layout::RECT_708;
pub const TAB_ID: u32 = 0xf22036;
pub const DATE_ID: u32 = 0xa0f311;
pub const SCROLL_UP_ID: u32 = 0xf0f01406;
pub const SCROLL_DOWN_ID: u32 = 0xf0f01407;
pub const BUTTON_IDS: [u32; 4] = [0xf023a0, 0xf023a1, 0xf023a2, 0xf023a3];
pub const BUTTON_RECTS: [Rect; 4] =
    [Rect::new(158, 224, 76, 23), Rect::new(158, 224, 76, 23), Rect::new(80, 224, 76, 23), Rect::new(158, 224, 76, 23)];
pub const SYSSTRINGS: [u32; 24] = [
    567, 568, 571, 572, 573, 574, 575, 576, 577, 578, 579, 580, 582, 583, 584, 585, 586, 587, 588, 589, 590, 591, 595,
    581,
];
pub const TITLE_SYSSTRING: u32 = 593;
pub const SCROLL_UP: Rect = Rect::new(227, 19, 15, 15);
pub const SCROLL_DOWN: Rect = Rect::new(227, 185, 15, 15);
pub const SCROLL_TRACK: Rect = Rect::new(227, 34, 15, 151);

pub fn default_position(viewport_height: i32) -> (i32, i32) {
    (0, crate::window_restore::x87_to_int(f64::from(viewport_height as f32) * 0.5 - 252.0))
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, y) = default_position(viewport.1);
    crate::initial_position::InitialPosition {
        enabled: true,
        horizontal_mode: 1,
        vertical_mode: 5,
        x: viewport.0.wrapping_sub(x),
        y: (viewport.1 / 2).wrapping_sub(y),
    }
}

pub fn compare_c_text(left: &str, right: &str) -> std::cmp::Ordering {
    fn units(text: &str) -> impl Iterator<Item = u16> + '_ {
        text.encode_utf16().take_while(|unit| *unit != 0).map(
            |unit| {
                if (65..=90).contains(&unit) { unit + 32 } else { unit }
            },
        )
    }
    units(left).cmp(units(right))
}

pub fn parse_number_with(text: &str, is_space: impl Fn(u16) -> bool) -> i32 {
    let mut units = text.encode_utf16().peekable();
    while units.peek().is_some_and(|unit| *unit != 0 && is_space(*unit)) {
        units.next();
    }
    let negative = units.peek() == Some(&45);
    if matches!(units.peek(), Some(43 | 45)) {
        units.next();
    }
    let mut value = 0i32;
    for unit in units {
        let Some(digit) = [
            0x30u16, 0x660, 0x6f0, 0x966, 0x9e6, 0xa66, 0xae6, 0xb66, 0xc66, 0xce6, 0xd66, 0xe50, 0xed0, 0xf20, 0x1040,
            0x17e0, 0x1810, 0xff10,
        ]
        .into_iter()
        .find_map(|start| unit.checked_sub(start).filter(|digit| *digit < 10)) else {
            break;
        };
        value = value.wrapping_mul(10).wrapping_add(i32::from(digit));
    }
    if negative { value.wrapping_neg() } else { value }
}

pub fn parse_c_number(text: &str) -> i32 {
    parse_number_with(text, |unit| matches!(unit, 9..=13 | 32 | 160))
}

pub const CAPACITY: usize = 256;
pub const VISIBLE_ROWS: usize = 12;
pub const BACK: &str = "L2UI_ch3.SiegeWnd.Siege_Back1";
pub const TAB_BACK: &str = "L2UI_CH3.SiegeWnd.Siege_Back2";
pub const DATE_BACK: &str = "L2UI_ch3.SiegeWnd.siege_back31";
pub const PART_BACK: &str = "L2UI_ch3.SiegeWnd.siege_back32";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum SiegeTab {
    #[default]
    Date,
    Attackers,
    Defenders,
}

impl SiegeTab {
    pub fn from_index(index: i32) -> Option<Self> {
        match index {
            0 => Some(Self::Date),
            1 => Some(Self::Attackers),
            2 => Some(Self::Defenders),
            _ => None,
        }
    }

    pub fn index(self) -> usize {
        match self {
            Self::Date => 0,
            Self::Attackers => 1,
            Self::Defenders => 2,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SiegeInfo {
    pub castle_id: i32,
    pub owner_flags: i32,
    pub owner_id: i32,
    pub leader_name: String,
    pub pledge_name: String,
    pub ally_id: i32,
    pub ally_name: String,
    pub now: i32,
    pub siege_time: i32,
    pub choices: Vec<i32>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SiegeClan {
    pub pledge_id: i32,
    pub name: String,
    pub leader_name: String,
    pub ally_name: String,
    pub ally_leader_name: String,
    pub kind: i32,
    pub crest_texture: Option<String>,
    pub ally_crest_texture: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SiegeRequest {
    Attackers { castle_id: i32 },
    Defenders { castle_id: i32 },
    Join { castle_id: i32, attacker: bool, join: bool },
    Approve { castle_id: i32, pledge_id: i32, approved: bool },
    SetTime { castle_id: i32, time: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiegeConfirmation {
    pub system_message: u32,
    pub text_parameter: String,
    pub request: SiegeRequest,
}

impl SiegeConfirmation {
    pub fn dialog_type(&self) -> i32 {
        match self.request {
            SiegeRequest::SetTime { .. } => 61,
            SiegeRequest::Join { attacker: true, join: true, .. } => 62,
            SiegeRequest::Join { attacker: true, join: false, .. } => 63,
            SiegeRequest::Join { attacker: false, join: true, .. } => 64,
            SiegeRequest::Join { attacker: false, join: false, .. } => 65,
            SiegeRequest::Approve { approved: true, .. } => 66,
            SiegeRequest::Approve { approved: false, .. } => 67,
            SiegeRequest::Attackers { .. } | SiegeRequest::Defenders { .. } => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SiegeAction {
    Request(SiegeRequest),
    Confirm(SiegeConfirmation),
    Close,
}

#[derive(Debug, Default, Clone)]
pub struct SiegePartInfo {
    pub rows: Vec<SiegeClan>,
    pub received_count: usize,
    pub confirmed: usize,
    pub pending: usize,
    pub registered: bool,
    pub requested: bool,
    pub selected: Option<usize>,
    pub first: usize,
    pub scroll_grab: Option<i32>,
    pub descending_next: [bool; 2],
    pub hovered: Option<usize>,
}

impl SiegePartInfo {
    pub fn sort_by(&mut self, column: usize, compare: impl Fn(&SiegeClan, &SiegeClan) -> std::cmp::Ordering) {
        let Some(descending) = self.descending_next.get_mut(column) else { return };
        for end in (1..self.rows.len()).rev() {
            for index in 0..end {
                let order = compare(&self.rows[index], &self.rows[index + 1]);
                if (!*descending && order.is_gt()) || (*descending && order.is_lt()) {
                    self.rows.swap(index, index + 1);
                    self.selected = match self.selected {
                        Some(selected) if selected == index => Some(index + 1),
                        Some(selected) if selected == index + 1 => Some(index),
                        other => other,
                    };
                }
            }
        }
        *descending = !*descending;
    }

    pub fn receive(&mut self, rows: &[SiegeClan], own_pledge: i32) {
        let requested = self.requested;
        let descending_next = self.descending_next;
        *self = Self { requested, descending_next, ..Self::default() };
        self.received_count = rows.len();
        self.confirmed = rows.iter().filter(|row| row.kind == 3).count();
        self.pending = rows.iter().filter(|row| matches!(row.kind, 1 | 2)).count();
        self.registered = rows.iter().any(|row| row.pledge_id == own_pledge);
        self.rows.extend(rows.iter().take(CAPACITY).cloned());
    }

    pub fn buttons(&self, defenders: bool, owner_flags: i32) -> [bool; 4] {
        if owner_flags != 0 {
            [false, false, defenders, defenders]
        } else {
            [!self.registered, self.registered, false, false]
        }
    }

    pub fn tooltip_row(&self, resources_loaded: bool) -> Option<&SiegeClan> {
        let index = self.hovered?;
        if resources_loaded && index < CAPACITY && self.selected == Some(index) { self.rows.get(index) } else { None }
    }

    pub fn tooltip_anchor(&self, resources_loaded: bool) -> Option<(i32, i32)> {
        self.tooltip_row(resources_loaded)?;
        let index = self.hovered? as i32;
        Some((0, (index.wrapping_sub(self.first as i32) % VISIBLE_ROWS as i32) * 17 + 22))
    }

    pub fn scroll(&mut self, notches: i32) {
        self.first = (self.first as i64 - i64::from(notches))
            .clamp(0, self.rows.len().saturating_sub(VISIBLE_ROWS) as i64) as usize;
    }

    pub fn slider(&self) -> crate::scroll::SliderBar {
        let mut slider = crate::scroll::SliderBar::new(SCROLL_TRACK.h);
        slider.set_range(self.rows.len() as i32, VISIBLE_ROWS as i32);
        slider.set_pos(self.first as i32);
        slider
    }

    pub fn scroll_down(&mut self, x: i32, y: i32) -> bool {
        if SCROLL_UP.contains(x, y) {
            self.scroll(1);
            return true;
        }
        if SCROLL_DOWN.contains(x, y) {
            self.scroll(-1);
            return true;
        }
        if SCROLL_TRACK.contains(x, y) {
            let mut slider = self.slider();
            slider.mouse_down(y - SCROLL_TRACK.y);
            self.scroll_grab = slider.dragging.then_some(slider.grab);
            return true;
        }
        false
    }

    pub fn scroll_drag(&mut self, y: i32) -> bool {
        let Some(grab) = self.scroll_grab else {
            return false;
        };
        let mut slider = self.slider();
        slider.dragging = true;
        slider.grab = grab;
        if let Some(first) = slider.mouse_move(y - SCROLL_TRACK.y) {
            self.first = first.max(0) as usize;
        }
        true
    }

    pub fn row_at(&self, x: i32, y: i32) -> Option<usize> {
        if !(0..LIST.w).contains(&x) || !(22..=LIST.h).contains(&y) {
            return None;
        }
        let row = ((y - 22) / 14) as usize;
        let index = self.first + row;
        (row < VISIBLE_ROWS && index < self.rows.len()).then_some(index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SiegeHit {
    Close,
    Tab(i32),
    DateBody,
    DateArrow,
    Button(u32),
    ScrollUp,
    ScrollDown,
    ScrollTrack,
    Row(usize),
    Header(usize),
}

#[derive(Debug, Clone)]
pub struct SiegeView {
    pub info: SiegeInfo,
    pub tab: SiegeTab,
    pub attackers: SiegePartInfo,
    pub defenders: SiegePartInfo,
    pub date: ComboBox,
    pub frame: FrameCtrl,
    pub labels: BTreeMap<u32, String>,
    pub castle_name: String,
    pub current_time_text: String,
    pub siege_time_text: String,
    pub owner_crest: Option<String>,
    pub owner_ally_crest: Option<String>,
    pub pressed: Option<u32>,
    pub captured: Option<SiegeHit>,
    pub header_hover: Option<usize>,
}

impl SiegeView {
    pub fn new(title: &str, mut sysstring: impl FnMut(u32) -> String, text_height: i32) -> Self {
        let labels: BTreeMap<_, _> = SYSSTRINGS.into_iter().map(|id| (id, sysstring(id))).collect();
        let mut date = ComboBox::new(COMBO, vec![labels.get(&585).cloned().unwrap_or_default()], text_height);
        date.selected = 0;
        Self {
            info: SiegeInfo::default(),
            tab: SiegeTab::Date,
            attackers: SiegePartInfo::default(),
            defenders: SiegePartInfo::default(),
            date,
            frame: FrameCtrl::titled(title, true),
            labels,
            castle_name: String::new(),
            current_time_text: String::new(),
            siege_time_text: String::new(),
            owner_crest: None,
            owner_ally_crest: None,
            pressed: None,
            captured: None,
            header_hover: None,
        }
    }

    pub fn label(&self, id: u32) -> &str {
        self.labels.get(&id).map(String::as_str).unwrap_or_default()
    }

    pub fn reset(&mut self) {
        self.info = SiegeInfo::default();
        self.attackers = SiegePartInfo { descending_next: self.attackers.descending_next, ..Default::default() };
        self.defenders = SiegePartInfo { descending_next: self.defenders.descending_next, ..Default::default() };
        self.date.clear_entries();
        self.date.entries.push(self.label(585).to_owned());
        self.date.selected = 0;
        self.castle_name.clear();
        self.current_time_text.clear();
        self.siege_time_text.clear();
        self.owner_crest = None;
        self.owner_ally_crest = None;
        self.pressed = None;
        self.captured = None;
        self.header_hover = None;
    }

    pub fn on_show(&mut self) -> u32 {
        self.reset();
        self.tab = SiegeTab::Date;
        5
    }

    pub fn on_hide(&mut self) {
        self.reset();
    }

    pub fn receive_info(&mut self, info: SiegeInfo, castle_name: String, mut format_time: impl FnMut(i32) -> String) {
        self.reset();
        self.castle_name = castle_name;
        self.current_time_text = format_time(info.now);
        self.siege_time_text = if info.siege_time > 0 { format_time(info.siege_time) } else { String::new() };
        self.date.entries.extend(info.choices.iter().copied().map(format_time));
        self.info = info;
    }

    pub fn receive_list(&mut self, tab: SiegeTab, castle_id: i32, rows: &[SiegeClan], own_pledge: i32) -> bool {
        if castle_id != self.info.castle_id {
            return false;
        }
        match tab {
            SiegeTab::Attackers => self.attackers.receive(rows, own_pledge),
            SiegeTab::Defenders => self.defenders.receive(rows, own_pledge),
            SiegeTab::Date => return false,
        }
        true
    }

    pub fn select_tab(&mut self, index: i32) -> Option<SiegeRequest> {
        let tab = SiegeTab::from_index(index)?;
        self.tab = tab;
        let castle_id = self.info.castle_id;
        let (part, request) = match tab {
            SiegeTab::Date => return None,
            SiegeTab::Attackers => (&mut self.attackers, SiegeRequest::Attackers { castle_id }),
            SiegeTab::Defenders => (&mut self.defenders, SiegeRequest::Defenders { castle_id }),
        };
        let send = !part.requested;
        part.requested = true;
        send.then_some(request)
    }

    pub fn active_part(&self) -> Option<&SiegePartInfo> {
        match self.tab {
            SiegeTab::Date => None,
            SiegeTab::Attackers => Some(&self.attackers),
            SiegeTab::Defenders => Some(&self.defenders),
        }
    }

    pub fn active_part_mut(&mut self) -> Option<&mut SiegePartInfo> {
        match self.tab {
            SiegeTab::Date => None,
            SiegeTab::Attackers => Some(&mut self.attackers),
            SiegeTab::Defenders => Some(&mut self.defenders),
        }
    }

    pub fn date_visible(&self) -> bool {
        self.info.siege_time == 0 && self.info.owner_flags != 0
    }

    pub fn date_confirmation(&self, index: i32) -> Option<SiegeConfirmation> {
        let choice = usize::try_from(index.checked_sub(1)?).ok()?;
        let time = *self.info.choices.get(choice)?;
        let text_parameter = self.date.entries.get(choice + 1)?.clone();
        Some(SiegeConfirmation {
            system_message: 663,
            text_parameter,
            request: SiegeRequest::SetTime { castle_id: self.info.castle_id, time },
        })
    }

    pub fn button_confirmation(&self, id: u32) -> Option<SiegeConfirmation> {
        let part = self.active_part()?;
        let button = BUTTON_IDS.iter().position(|candidate| *candidate == id)?;
        if !part.buttons(self.tab == SiegeTab::Defenders, self.info.owner_flags)[button] {
            return None;
        }
        let castle_id = self.info.castle_id;
        if button < 2 {
            return Some(SiegeConfirmation {
                system_message: if button == 1 {
                    669
                } else if self.tab == SiegeTab::Defenders {
                    668
                } else {
                    667
                },
                text_parameter: self.castle_name.clone(),
                request: SiegeRequest::Join { castle_id, attacker: self.tab == SiegeTab::Attackers, join: button == 0 },
            });
        }
        let row = part.rows.get(part.selected?)?;
        if row.pledge_id <= 0 || (button == 2 && row.kind != 2) || (button == 3 && !matches!(row.kind, 2 | 3)) {
            return None;
        }
        Some(SiegeConfirmation {
            system_message: if button == 2 { 671 } else { 670 },
            text_parameter: row.name.clone(),
            request: SiegeRequest::Approve { castle_id, pledge_id: row.pledge_id, approved: button == 2 },
        })
    }

    pub fn sort_column(&mut self, column: usize) {
        self.sort_column_with(column, compare_c_text, parse_c_number);
    }

    pub fn sort_column_with(
        &mut self,
        column: usize,
        compare_text: impl Fn(&str, &str) -> std::cmp::Ordering,
        parse_number: impl Fn(&str) -> i32,
    ) {
        let defenders = self.tab == SiegeTab::Defenders;
        let labels = [
            self.label(588).to_owned(),
            self.label(568).to_owned(),
            self.label(567).to_owned(),
            self.label(579).to_owned(),
        ];
        if let Some(part) = self.active_part_mut() {
            part.sort_by(column, |left, right| {
                if column == 0 {
                    compare_text(&left.name, &right.name)
                } else {
                    let text = |row: &SiegeClan| {
                        if defenders {
                            row.kind
                                .checked_sub(1)
                                .and_then(|index| labels.get(index as usize))
                                .map(|label| parse_number(label))
                                .unwrap_or(0)
                        } else {
                            parse_number(&row.ally_name)
                        }
                    };
                    text(left).cmp(&text(right))
                }
            });
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<SiegeHit> {
        let (px, py) = (x - PAGE.x, y - PAGE.y);
        if self.tab == SiegeTab::Date && self.date_visible() && self.date.expanded_rect().contains(px, py) {
            return Some(if self.date.arrow_rect().contains(px, py) {
                SiegeHit::DateArrow
            } else {
                SiegeHit::DateBody
            });
        }
        if Rect::new(
            SIZE.0 - crate::frame::CLOSE_INSET,
            crate::frame::BUTTON_Y,
            crate::frame::BUTTON_SIZE,
            crate::frame::BUTTON_SIZE,
        )
        .contains(x, y)
        {
            return Some(SiegeHit::Close);
        }
        for i in 0..3 {
            if Rect::new(TABS.x + i * 75, TABS.y, 74, 23).contains(x, y) {
                return Some(SiegeHit::Tab(i));
            }
        }
        let part = self.active_part()?;
        let buttons = part.buttons(self.tab == SiegeTab::Defenders, self.info.owner_flags);
        for (index, rect) in BUTTON_RECTS.into_iter().enumerate() {
            if buttons[index] && rect.contains(px, py) {
                return Some(SiegeHit::Button(BUTTON_IDS[index]));
            }
        }
        let (lx, ly) = (px - LIST.x, py - LIST.y);
        if (0..19).contains(&ly) && (0..LIST.w).contains(&lx) {
            return Some(SiegeHit::Header(usize::from(lx >= 130)));
        }
        for (rect, hit) in [
            (SCROLL_UP, SiegeHit::ScrollUp),
            (SCROLL_DOWN, SiegeHit::ScrollDown),
            (SCROLL_TRACK, SiegeHit::ScrollTrack),
        ] {
            if rect.contains(lx, ly) {
                return Some(hit);
            }
        }
        part.row_at(lx, ly).map(SiegeHit::Row)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<SiegeAction> {
        let hit = self.hit(x, y);
        self.captured = hit;
        self.header_hover = match hit {
            Some(SiegeHit::Header(column)) => Some(column),
            _ => None,
        };
        self.pressed = match hit {
            Some(SiegeHit::Button(id)) => Some(id),
            Some(SiegeHit::DateArrow) => Some(DATE_ID),
            Some(SiegeHit::ScrollUp) => Some(SCROLL_UP_ID),
            Some(SiegeHit::ScrollDown) => Some(SCROLL_DOWN_ID),
            _ => None,
        };
        if !matches!(hit, Some(SiegeHit::DateBody | SiegeHit::DateArrow)) {
            self.date.lose_focus();
        }
        match hit {
            Some(SiegeHit::Tab(index)) => self.select_tab(index).map(SiegeAction::Request),
            Some(SiegeHit::DateBody) => {
                self.date.mouse_down().and_then(|index| self.date_confirmation(index)).map(SiegeAction::Confirm)
            }
            Some(SiegeHit::Row(index)) => {
                if let Some(part) = self.active_part_mut() {
                    part.selected = Some(index);
                    part.hovered = Some(index);
                }
                None
            }
            Some(SiegeHit::ScrollTrack) => {
                if let Some(part) = self.active_part_mut() {
                    part.scroll_down(x - PAGE.x - LIST.x, y - PAGE.y - LIST.y);
                }
                None
            }
            None if LIST.contains(x - PAGE.x, y - PAGE.y) && y >= PAGE.y + LIST.y + 19 => {
                if let Some(part) = self.active_part_mut() {
                    part.selected = None;
                }
                None
            }
            _ => None,
        }
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        self.header_hover = match self.hit(x, y) {
            Some(SiegeHit::Header(column)) => Some(column),
            _ => None,
        };
        if self.tab == SiegeTab::Date && self.date_visible() {
            self.date.mouse_move(x - PAGE.x, y - PAGE.y);
        }
        if let Some(part) = self.active_part_mut() {
            part.scroll_drag(y - PAGE.y - LIST.y);
            part.hovered = part.row_at(x - PAGE.x - LIST.x, y - PAGE.y - LIST.y);
        }
        self.pressed = if self.hit(x, y) == self.captured {
            match self.captured {
                Some(SiegeHit::Button(id)) => Some(id),
                Some(SiegeHit::DateArrow) => Some(DATE_ID),
                Some(SiegeHit::ScrollUp) => Some(SCROLL_UP_ID),
                Some(SiegeHit::ScrollDown) => Some(SCROLL_DOWN_ID),
                _ => None,
            }
        } else {
            None
        };
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<SiegeAction> {
        let captured = self.captured.take();
        self.pressed = None;
        if let Some(part) = self.active_part_mut() {
            part.scroll_grab = None;
        }
        if captured != self.hit(x, y) {
            return None;
        }
        match captured {
            Some(SiegeHit::Header(column)) => {
                self.sort_column(column);
                None
            }
            Some(SiegeHit::Close) => Some(SiegeAction::Close),
            Some(SiegeHit::Button(id)) => self.button_confirmation(id).map(SiegeAction::Confirm),
            Some(SiegeHit::DateArrow) => {
                self.date.open_list();
                None
            }
            Some(SiegeHit::ScrollUp) => {
                if let Some(part) = self.active_part_mut() {
                    part.scroll(1);
                }
                None
            }
            Some(SiegeHit::ScrollDown) => {
                if let Some(part) = self.active_part_mut() {
                    part.scroll(-1);
                }
                None
            }
            _ => None,
        }
    }

    pub fn click(&mut self, x: i32, y: i32) -> Option<SiegeAction> {
        self.mouse_move(x, y);
        let down = self.mouse_down(x, y);
        let up = self.mouse_up(x, y);
        down.or(up)
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        list.draw_texture(x, y + 20, SIZE.0, 105, 0, 0, SIZE.0, 105, BACK, alpha);
        list.draw_texture(x, y + 125, SIZE.0, 23, 0, 0, SIZE.0, 23, TAB_BACK, alpha);
        self.frame.paint(list, x, y, SIZE.0, SIZE.1, font, alpha);
        list.draw_text(
            x + 120 - font.measure(&self.castle_name) as i32 / 2,
            y + 41,
            Argb(0xffb09b79),
            &self.castle_name,
            font,
        );
        for (id, dy) in [(588, 70), (589, 86), (590, 102)] {
            list.draw_text(x + 20, y + dy, Argb(0xffa3a3a3), self.label(id), font);
        }
        if self.info.leader_name.is_empty() {
            list.draw_text(x + 80, y + 70, Argb(0xffdcdcdc), self.label(595), font);
        } else {
            list.draw_text(x + 80, y + 70, Argb(0xffdcdcdc), &self.info.leader_name, font);
            let pledge_x = if let Some(texture) = &self.owner_crest {
                list.draw_texture(x + 80, y + 86, 16, 12, 0, 4, 16, 12, texture, alpha);
                100
            } else {
                80
            };
            list.draw_text(x + pledge_x, y + 86, Argb(0xffdcdcdc), &self.info.pledge_name, font);
            let ally_x = if !self.info.ally_name.is_empty()
                && let Some(texture) = &self.owner_ally_crest
            {
                list.draw_texture(x + 88, y + 102, 8, 12, 0, 4, 8, 12, texture, alpha);
                100
            } else {
                80
            };
            list.draw_text(
                x + ally_x,
                y + 102,
                Argb(0xffdcdcdc),
                if self.info.ally_name.is_empty() { self.label(591) } else { &self.info.ally_name },
                font,
            );
        }
        for (i, id) in [583, 586, 587].into_iter().enumerate() {
            TabButton {
                rect: Rect::new(TABS.x + i as i32 * 75, TABS.y, 74, 23),
                textures: [
                    Some("L2UI_ch3.SiegeWnd.siege_Tab3".into()),
                    Some(if i == 0 { "L2UI_ch3.SiegeWnd.siege_Tab1" } else { "L2UI_ch3.SiegeWnd.siege_Tab2" }.into()),
                    None,
                ],
                source: (0, 0),
                state: i32::from(self.tab.index() == i),
                enabled: true,
                label: self.label(id).into(),
                text_offset: (2, 1),
            }
            .paint(list, origin, font);
        }
        let page_origin = (x + PAGE.x, y + PAGE.y);
        list.draw_texture(
            page_origin.0,
            page_origin.1,
            PAGE.w,
            PAGE.h,
            0,
            0,
            PAGE.w,
            PAGE.h,
            if self.tab == SiegeTab::Date { DATE_BACK } else { PART_BACK },
            alpha,
        );
        if self.tab == SiegeTab::Date {
            self.paint_date(list, page_origin, font);
        } else {
            self.paint_part(list, page_origin, font);
        }
    }

    fn paint_date(&self, list: &mut DrawList, (x, y): (i32, i32), font: &BitmapFont) {
        for (id, dy) in [(582, 30), (583, 50)] {
            list.draw_text(x + 10, y + dy, Argb(0xffdcdcdc), self.label(id), font);
        }
        list.draw_text(x + 80, y + 30, Argb(0xffdcdcdc), &self.current_time_text, font);
        if self.info.siege_time > 0 {
            list.draw_text(x + 80, y + 50, Argb(0xffdcdcdc), &self.siege_time_text, font);
        } else if self.info.siege_time == 0 && self.info.owner_flags == 0 {
            list.draw_text(x + 80, y + 50, Argb(0xffdcdcdc), self.label(584), font);
        }
        if self.date_visible() {
            self.date.paint(list, (x, y), font, true, self.pressed == Some(DATE_ID));
        }
    }

    fn paint_part(&self, list: &mut DrawList, (x, y): (i32, i32), font: &BitmapFont) {
        let Some(part) = self.active_part() else {
            return;
        };
        let defenders = self.tab == SiegeTab::Defenders;
        let count = if defenders {
            format!("{}/{} : {}/{}", self.label(577), self.label(578), part.confirmed, part.pending)
        } else {
            format!("{} : {}", self.label(576), part.received_count)
        };
        list.draw_text(x + 14, y + 2, Argb(0xffdcdcdc), &count, font);
        let lx = x + LIST.x;
        let ly = y + LIST.y;
        for (column, (dx, width, id)) in
            [(0, 130, 580), (130, 112, if defenders { 581 } else { 571 })].into_iter().enumerate()
        {
            let down = self.captured == Some(SiegeHit::Header(column)) && self.header_hover == Some(column);
            let textures = if down {
                ["L2UI_CH3.ListCtrl.tab1_down", "L2UI_CH3.ListCtrl.tab2_down", "L2UI_CH3.ListCtrl.tab3_down"]
            } else if self.header_hover == Some(column) {
                ["L2UI_CH3.ListCtrl.tab1_over", "L2UI_CH3.ListCtrl.tab2_over", "L2UI_CH3.ListCtrl.tab3_over"]
            } else {
                ["L2UI_CH3.ListCtrl.tab1", "L2UI_CH3.ListCtrl.tab2", "L2UI_CH3.ListCtrl.tab3"]
            };
            list.slices3_h(lx + dx, ly, width, 19, (textures[0], 8), textures[1], (textures[2], 8), 19, 1.0);
            let text = self.label(id);
            let tx = width / 2 - font.measure(text) as i32 / 2;
            let ty = (9.5 - f64::from(font.line_height as i32 / 2)).trunc() as i32 + 1;
            let mut header = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
            header.draw_text(lx + dx + tx, ly + ty, Argb(0xffe6dcbe), text, font);
            list.append_clipped(header, Rect::new(lx + dx, ly, width, 19));
        }
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        for (line, (index, row)) in part.rows.iter().enumerate().skip(part.first).take(VISIBLE_ROWS).enumerate() {
            let ry = ly + 22 + line as i32 * 14;
            if index % 2 == 1 {
                body.draw_texture(lx, ry - 1, LIST.w, 15, 0, 0, 8, 15, "L2UI_CH3.Etc.textbackline", 1.0);
            }
            if part.selected == Some(index) {
                body.draw_texture(lx, ry, LIST.w - 49, 13, 0, 0, 16, 13, "L2UI_CH3.ListCtrl.TextSelect", 1.0);
                body.draw_texture(lx + LIST.w - 49, ry, 32, 13, 0, 0, 32, 13, "L2UI_CH3.ListCtrl.TextSelect2", 1.0);
            }
            if let Some(texture) = &row.crest_texture {
                body.draw_texture(lx, ry + 1, 16, 12, 0, 4, 16, 12, texture, 1.0);
            }
            body.draw_text_clipped(lx + 17, ry + 1, 113, Argb(0xffdcdcdc), &row.name, font);
            let second = if defenders {
                self.label(match row.kind {
                    1 => 588,
                    2 => 568,
                    3 => 567,
                    4 => 579,
                    _ => 0,
                })
            } else {
                &row.ally_name
            };
            let offset = if defenders { 0 } else { 9 };
            if !defenders && let Some(texture) = &row.ally_crest_texture {
                body.draw_texture(lx + 130, ry + 1, 8, 12, 0, 4, 8, 12, texture, 1.0);
            }
            body.draw_text_clipped(lx + 130 + offset, ry + 1, 112 - offset, Argb(0xffdcdcdc), second, font);
        }
        list.append_clipped(body, Rect::new(lx, ly + 19, LIST.w, LIST.h - 19));
        for (rect, texture) in [
            (
                SCROLL_UP,
                if self.pressed == Some(SCROLL_UP_ID) { crate::scroll::UP_ON_BTN } else { crate::scroll::UP_BTN },
            ),
            (
                SCROLL_DOWN,
                if self.pressed == Some(SCROLL_DOWN_ID) { crate::scroll::DOWN_ON_BTN } else { crate::scroll::DOWN_BTN },
            ),
        ] {
            list.draw_texture(lx + rect.x, ly + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, texture, 1.0);
        }
        part.slider().paint(list, lx + SCROLL_TRACK.x, ly + SCROLL_TRACK.y);
        for (index, visible) in part.buttons(defenders, self.info.owner_flags).into_iter().enumerate() {
            if visible {
                TextButton::new(
                    BUTTON_RECTS[index],
                    "L2UI_ch3.button.btn1_normal",
                    "L2UI_ch3.button.btn1_normalon",
                    self.label(572 + index as u32),
                )
                .paint(list, x, y, self.pressed == Some(BUTTON_IDS[index]), font);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> BitmapFont {
        use crate::font::{Glyphs, gly};
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("font".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn view() -> SiegeView {
        SiegeView::new("Siege", |id| format!("s{id}"), 13)
    }

    fn clan(id: i32, kind: i32) -> SiegeClan {
        SiegeClan { pledge_id: id, kind, name: format!("Clan{id}"), ..Default::default() }
    }

    #[test]
    fn headers_capture_release_sort_and_paint_native_text_and_pressed_slices() {
        let mut v = view();
        v.tab = SiegeTab::Attackers;
        v.attackers.rows = vec![
            SiegeClan { name: "b".into(), ..Default::default() },
            SiegeClan { name: "a".into(), ..Default::default() },
        ];
        let (x, y) = (PAGE.x + LIST.x + 8, PAGE.y + LIST.y + 8);
        v.mouse_down(x, y);
        assert_eq!(v.attackers.rows[0].name, "b");
        let mut list = DrawList::default();
        v.paint(&mut list, (0, 0), &font(), 1.0);
        assert!(list.quads.iter().any(|q| q.texture == "L2UI_CH3.ListCtrl.tab1_down"));
        v.mouse_up(x, y);
        assert_eq!(v.attackers.rows[0].name, "a");
        v.mouse_down(x, y);
        v.mouse_move(-1, -1);
        v.mouse_up(-1, -1);
        assert_eq!(v.attackers.rows[0].name, "a");
        v.click(x, y);
        assert_eq!(v.attackers.rows[0].name, "b");
        assert_eq!(v.hit(PAGE.x + LIST.x + 130, PAGE.y + LIST.y), Some(SiegeHit::Header(1)));
        assert_eq!(v.hit(PAGE.x + LIST.x + 242, PAGE.y + LIST.y), None);
        v.sort_column(usize::MAX);
    }

    #[test]
    fn row_clear_and_window_reset_preserve_header_sort_direction() {
        let mut v = view();
        v.tab = SiegeTab::Attackers;
        v.sort_column(0);
        v.attackers.receive(&[clan(1, 0)], 1);
        assert_eq!(v.attackers.descending_next, [true, false]);
        v.on_hide();
        assert_eq!(v.attackers.descending_next, [true, false]);
        v.on_show();
        assert_eq!(v.attackers.descending_next, [true, false]);
    }

    #[test]
    fn tooltip_gate_requires_hovered_selection_and_loaded_native_resources() {
        let mut p = SiegePartInfo { rows: vec![clan(1, 1), clan(2, 2)], ..Default::default() };
        for selected in [None, Some(0), Some(1), Some(2), Some(CAPACITY)] {
            for hovered in [None, Some(0), Some(1), Some(2), Some(CAPACITY)] {
                for loaded in [false, true] {
                    p.selected = selected;
                    p.hovered = hovered;
                    assert_eq!(
                        p.tooltip_row(loaded).is_some(),
                        loaded && selected == hovered && hovered.is_some_and(|row| row < 2)
                    );
                }
            }
        }
        let mut v = view();
        v.tab = SiegeTab::Attackers;
        v.attackers.rows = vec![clan(1, 1)];
        v.click(PAGE.x + LIST.x, PAGE.y + LIST.y + 22);
        assert!(v.attackers.tooltip_row(true).is_some());
        v.mouse_move(-1, -1);
        assert!(v.attackers.tooltip_row(true).is_none());
    }

    #[test]
    fn numeric_cells_follow_native_sign_wrapping_digits_and_locale_boundary() {
        for (text, expected) in [
            ("", 0),
            ("\t\r -42end", -42),
            ("+2147483648", i32::MIN),
            ("4294967297", 1),
            ("-2147483649", i32::MAX),
            ("12\0more", 12),
            ("+１２", 12),
            ("-١٢", -12),
            ("+", 0),
            ("name", 0),
        ] {
            assert_eq!(parse_c_number(text), expected, "{text}");
        }
        assert_eq!(parse_number_with("\u{3000}１２", |unit| unit == 0x3000), 12);
        assert_eq!(compare_c_text("Ab\0zzz", "aB"), std::cmp::Ordering::Equal);
        assert_ne!(compare_c_text("Ä", "ä"), std::cmp::Ordering::Equal);
        let mut v = view();
        v.tab = SiegeTab::Defenders;
        v.defenders.rows = (1..=4).map(|kind| clan(kind, kind)).collect();
        v.sort_column(1);
        assert_eq!(v.defenders.rows.iter().map(|row| row.kind).collect::<Vec<_>>(), [1, 2, 3, 4]);
        v.sort_column(1);
        assert_eq!(v.defenders.rows.iter().map(|row| row.kind).collect::<Vec<_>>(), [1, 2, 3, 4]);
    }

    #[test]
    fn select_tab_requests_once_until_full_reset_and_ignores_unknown_tabs() {
        let mut v = view();
        v.info.castle_id = 3;
        for index in [-1, 3, i32::MAX] {
            assert_eq!(v.select_tab(index), None);
            assert_eq!(v.tab, SiegeTab::Date);
        }
        assert_eq!(v.select_tab(1), Some(SiegeRequest::Attackers { castle_id: 3 }));
        assert_eq!(v.select_tab(1), None);
        assert_eq!(v.select_tab(2), Some(SiegeRequest::Defenders { castle_id: 3 }));
        v.select_tab(0);
        assert_eq!(v.select_tab(1), None);
        assert_eq!(v.on_show(), 5);
        assert_eq!(v.tab, SiegeTab::Date);
        assert_eq!(v.select_tab(1), Some(SiegeRequest::Attackers { castle_id: 0 }));
    }

    #[test]
    fn list_keeps_full_counts_beyond_native_display_capacity() {
        let mut p = SiegePartInfo::default();
        let rows: Vec<_> = (0..300).map(|i| clan(i + 1, i % 5)).collect();
        p.receive(&rows, 300);
        assert_eq!((p.rows.len(), p.received_count, p.confirmed, p.pending), (256, 300, 60, 120));
        assert!(p.registered);
        p.selected = Some(2);
        p.first = 3;
        p.receive(&[], 300);
        assert_eq!((p.selected, p.first, p.registered), (None, 0, false));
    }

    #[test]
    fn all_confirmation_branches_preserve_c4_message_ids_and_requests() {
        let mut v = view();
        v.info.castle_id = 5;
        v.castle_name = "Castle".into();
        for (tab, message) in [(SiegeTab::Attackers, 667), (SiegeTab::Defenders, 668)] {
            v.tab = tab;
            let apply = v.button_confirmation(BUTTON_IDS[0]).unwrap();
            assert_eq!(apply.system_message, message);
            assert_eq!(apply.dialog_type(), if tab == SiegeTab::Attackers { 62 } else { 64 });
            assert_eq!(
                apply.request,
                SiegeRequest::Join { castle_id: 5, attacker: tab == SiegeTab::Attackers, join: true }
            );
            v.active_part_mut().unwrap().registered = true;
            assert!(v.button_confirmation(BUTTON_IDS[0]).is_none());
            assert_eq!(v.button_confirmation(BUTTON_IDS[1]).unwrap().system_message, 669);
        }
        v.info.owner_flags = 1;
        v.tab = SiegeTab::Defenders;
        for id in [-1, 0, 1] {
            for kind in -1..6 {
                v.defenders.receive(&[clan(id, kind)], 99);
                assert!(v.button_confirmation(BUTTON_IDS[2]).is_none());
                v.defenders.selected = Some(0);
                for (button, valid, message) in
                    [(2, id > 0 && kind == 2, 671), (3, id > 0 && matches!(kind, 2 | 3), 670)]
                {
                    let c = v.button_confirmation(BUTTON_IDS[button]);
                    assert_eq!(c.is_some(), valid, "id={id} kind={kind} button={button}");
                    if let Some(c) = c {
                        assert_eq!(c.system_message, message);
                        assert_eq!(c.dialog_type(), if button == 2 { 66 } else { 67 });
                        assert_eq!(
                            c.request,
                            SiegeRequest::Approve { castle_id: 5, pledge_id: id, approved: button == 2 }
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn date_choices_preserve_placeholder_indices_and_reset_on_hide() {
        let mut v = view();
        v.receive_info(
            SiegeInfo { castle_id: 9, owner_flags: 1, choices: vec![123, 456], ..Default::default() },
            "Castle".into(),
            |n| format!("time{n}"),
        );
        assert_eq!(v.date.entries, ["s585", "time123", "time456"]);
        for index in [-1, 0, 3, i32::MAX] {
            assert!(v.date_confirmation(index).is_none());
        }
        for (index, time) in [(1, 123), (2, 456)] {
            let c = v.date_confirmation(index).unwrap();
            assert_eq!(c.system_message, 663);
            assert_eq!(c.dialog_type(), 61);
            assert_eq!(c.request, SiegeRequest::SetTime { castle_id: 9, time });
        }
        v.on_hide();
        assert_eq!(v.info, SiegeInfo::default());
        assert_eq!(v.date.entries, ["s585"]);
    }

    #[test]
    fn mouse_buttons_release_inside_and_preserve_capture_outside() {
        let mut v = view();
        v.select_tab(1);
        let (x, y) = (160, PAGE.y + 226);
        assert_eq!(v.mouse_down(x, y), None);
        assert_eq!(v.pressed, Some(BUTTON_IDS[0]));
        v.mouse_move(-10, -10);
        assert_eq!(v.pressed, None);
        assert_eq!(v.captured, Some(SiegeHit::Button(BUTTON_IDS[0])));
        v.mouse_move(x, y);
        assert_eq!(v.pressed, Some(BUTTON_IDS[0]));
        assert!(matches!(v.mouse_up(x, y), Some(SiegeAction::Confirm(_))));
        assert_eq!(v.mouse_up(x, y), None);
        v.mouse_down(x, y);
        assert_eq!(v.mouse_up(-10, -10), None);
        assert_eq!(v.captured, None);
    }

    #[test]
    fn mouse_tabs_notify_on_down_but_combo_arrow_opens_on_up() {
        let mut v = view();
        assert_eq!(
            v.mouse_down(TABS.x + 76, TABS.y + 1),
            Some(SiegeAction::Request(SiegeRequest::Attackers { castle_id: 0 }))
        );
        assert_eq!(v.mouse_up(TABS.x + 76, TABS.y + 1), None);
        v.receive_info(
            SiegeInfo { owner_flags: 1, choices: vec![100, 200], ..Default::default() },
            String::new(),
            |n| n.to_string(),
        );
        v.select_tab(0);
        let arrow = v.date.arrow_rect();
        let (x, y) = (PAGE.x + arrow.x + 1, PAGE.y + arrow.y + 1);
        v.mouse_down(x, y);
        assert!(!v.date.open);
        v.mouse_up(x, y);
        assert!(v.date.open);
        let body = (PAGE.x + COMBO.x + 2, PAGE.y + COMBO.y + COMBO.h + 18);
        v.mouse_move(body.0, body.1);
        assert_eq!(v.date.hovered, 1);
        assert!(matches!(
            v.mouse_down(body.0, body.1),
            Some(SiegeAction::Confirm(SiegeConfirmation { system_message: 663, .. }))
        ));
        assert_eq!(v.mouse_up(body.0, body.1), None);
        assert!(!v.date.open);
    }

    #[test]
    fn mouse_scroll_arrows_release_and_thumb_drag_never_confirm() {
        let mut v = view();
        v.select_tab(1);
        v.attackers.receive(&(0..100).map(|id| clan(id, 0)).collect::<Vec<_>>(), -1);
        let (x, y) = (PAGE.x + LIST.x + SCROLL_DOWN.x + 1, PAGE.y + LIST.y + SCROLL_DOWN.y + 1);
        assert_eq!(v.mouse_down(x, y), None);
        assert_eq!(v.attackers.first, 0);
        assert_eq!(v.mouse_up(x, y), None);
        assert_eq!(v.attackers.first, 1);
        let (tx, ty) =
            (PAGE.x + LIST.x + SCROLL_TRACK.x + 1, PAGE.y + LIST.y + SCROLL_TRACK.y + v.attackers.slider().thumb_y + 2);
        assert_eq!(v.mouse_down(tx, ty), None);
        assert!(v.attackers.scroll_grab.is_some());
        v.mouse_move(tx + 100, ty + 70);
        assert!(v.attackers.first > 1);
        assert_eq!(v.mouse_up(tx + 100, ty + 70), None);
        assert_eq!(v.attackers.scroll_grab, None);
    }

    #[test]
    fn painter_uses_all_three_pages_and_native_background_rectangles() {
        use crate::font::{Glyphs, gly};
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let font = BitmapFont::new("font".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap());
        let mut v = view();
        for (tab, texture) in
            [(SiegeTab::Date, DATE_BACK), (SiegeTab::Attackers, PART_BACK), (SiegeTab::Defenders, PART_BACK)]
        {
            v.tab = tab;
            let mut list = DrawList::default();
            v.paint(&mut list, (10, 20), &font, 1.0);
            for (tex, rect) in [
                (BACK, (10.0, 40.0, 256.0, 105.0)),
                (TAB_BACK, (10.0, 145.0, 256.0, 23.0)),
                (texture, (10.0, 168.0, 256.0, 253.0)),
            ] {
                let quad = list.quads.iter().find(|quad| quad.texture == tex).unwrap();
                assert_eq!((quad.x, quad.y, quad.w, quad.h), rect);
            }
        }
    }
}
