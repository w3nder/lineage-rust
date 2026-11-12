#[path = "party_match_controls_generated.rs"]
mod controls;
#[path = "party_match_layout_generated.rs"]
mod layout;

#[path = "party_match_background_generated.rs"]
mod background;
#[path = "party_match_frame_generated.rs"]
mod frame_controls;

use crate::button::TextButton;
use crate::color::{Argb, TEXT};
use crate::combo::ComboBox;
use crate::draw::DrawList;
use crate::font::BitmapFont;

use crate::party_room::RoomData;
use crate::window::{Rect, WindowId};
use std::cmp::Ordering;

pub const DEFAULT_TITLE_SYSTEMMSG: u32 = 1398;
pub const WINDOW_ID: WindowId = WindowId(34);
pub const SIZE: (i32, i32) = layout::SIZE;
pub const INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 0, vertical_mode: 3, x: 0, y: 76 };
pub const BACK: &str = "L2UI_CH3.PartyMatchWnd.PartyMatch1_Back";
pub const ICON: crate::child_contract::ChildContract = controls::CHILD_3C;
pub const LIST_RECT: Rect = layout::RECT_F4;
pub const LOCATION_RECT: Rect = layout::RECT_EC;
pub const LEVEL_RECT: Rect = layout::RECT_F0;
pub const CLOSE_RECT: Rect = Rect::new(577, 3, 15, 15);
pub const MINIMIZE_RECT: Rect = Rect::new(560, 3, 15, 15);
pub const HEADER_HEIGHT: i32 = 19;
pub const ROW_HEIGHT: i32 = 17;
pub const ROW_CAPACITY: usize = 15;
pub const VISIBLE_ROWS: usize = 15;
pub const TIMER_ID: u32 = 0x1f8;
pub const TIMER_MS: u32 = 5000;
pub const COLUMN_WIDTHS: [i32; 6] = [34, 240, 100, 98, 72, 41];
pub const COLUMN_SYSSTRINGS: [u32; 6] = [416, 413, 408, 1029, 1030, 1036];
pub const LOCATION_VALUES: [i32; 17] = [-2, -1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
pub const LOCATION_SYSSTRINGS: [u32; 17] =
    [1047, 1046, 1048, 1049, 1050, 1051, 1052, 1053, 1054, 1055, 1056, 1057, 1058, 1059, 1060, 1247, 1248];
pub const BUTTON_RECTS: [Rect; 5] = [
    layout::RECT_F8,
    layout::RECT_FC,
    layout::RECT_100,
    layout::RECT_104,
    layout::RECT_108,
];
pub const BUTTON_SYSSTRINGS: [u32; 5] = [1037, 1038, 1039, 1040, 1041];
pub const STRIPE: &str = "L2UI_CH3.Etc.textbackline";
pub const TAB: [&str; 3] = ["L2UI_CH3.ListCtrl.tab1", "L2UI_CH3.ListCtrl.tab2", "L2UI_CH3.ListCtrl.tab3"];
pub const TAB_DOWN: [&str; 3] =
    ["L2UI_CH3.ListCtrl.tab1_down", "L2UI_CH3.ListCtrl.tab2_down", "L2UI_CH3.ListCtrl.tab3_down"];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Labels {
    pub title: String,
    pub filters: [String; 2],
    pub locations: [String; 17],
    pub levels: [String; 2],
    pub columns: [String; 6],
    pub buttons: [String; 5],
}

impl Labels {
    pub fn from_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id| lookup(id).unwrap_or_default();
        Self {
            title: text(389),
            filters: [text(1034), text(1035)],
            locations: LOCATION_SYSSTRINGS.map(text),
            levels: [text(1045), text(1046)],
            columns: COLUMN_SYSSTRINGS.map(text),
            buttons: BUTTON_SYSSTRINGS.map(text),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    Close,
    Minimize,
    Location,
    Level,
    Header(usize),
    Row(usize),
    Previous,
    Next,
    Search,
    Create,
    Join,
}

const BUTTON_HITS: [Hit; 5] = [Hit::Previous, Hit::Next, Hit::Search, Hit::Create, Hit::Join];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    RequestList { page: i32, location: i32, mode: i32 },
    Join { room_id: i32, location: i32 },
    CreateRoom,
    Close,
    Minimize,
}

pub fn create_levels(level: Option<i32>) -> (i32, i32) {
    match level {
        None => (1, 78),
        Some(level) if level <= 5 => (1, level.wrapping_add(5)),
        Some(level) if level <= 73 => (level - 5, level + 5),
        Some(level) => (level - 5, 78),
    }
}

pub fn grip() -> Rect {
    Rect::new(0, 0, SIZE.0, 20)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchView {
    pub id: WindowId,
    pub page: i32,
    pub labels: Labels,
    pub rooms: Vec<RoomData>,
    pub selected: Option<usize>,
    pub hovered: Option<usize>,
    pub first_row: usize,
    pub location: ComboBox,
    pub level: ComboBox,
    pub buttons: [TextButton; 5],
    pub pressed: Option<Hit>,
    pub sort_ascending: [bool; 6],
    timer_elapsed_bits: u32,
}

impl MatchView {
    pub fn new(labels: Labels, text_height: i32) -> Self {
        let mut location = ComboBox::from_contract(&controls::CHILD_EC, labels.locations.to_vec(), text_height);
        let mut level = ComboBox::from_contract(&controls::CHILD_F0, labels.levels.to_vec(), text_height);
        location.selected = 1;
        level.selected = 1;
        let buttons = std::array::from_fn(|index| {
            let (normal, down) = if index < 2 {
                ("L2UI_CH3.Button.SmallButton2", "L2UI_CH3.Button.SmallButton2_down")
            } else {
                ("L2UI_CH3.Button.Btn1_normal", "L2UI_CH3.Button.Btn1_normalOn")
            };
            TextButton::new(BUTTON_RECTS[index], normal, down, &labels.buttons[index])
        });
        Self {
            id: WINDOW_ID,
            page: 0,
            labels,
            rooms: Vec::new(),
            selected: None,
            hovered: None,
            first_row: 0,
            location,
            level,
            buttons,
            pressed: None,
            sort_ascending: [true; 6],
            timer_elapsed_bits: 0,
        }
    }

    pub fn reset(&mut self) {
        self.page = 0;
        self.rooms.clear();
        self.first_row = 0;
        self.selected = None;
        self.hovered = None;
    }

    pub fn append(&mut self, page: i32, rooms: &[RoomData]) {
        self.page = page;
        self.rooms.extend(rooms.iter().take(ROW_CAPACITY.saturating_sub(self.rooms.len())).cloned());
    }

    pub fn receive(&mut self, page: i32, rooms: &[RoomData]) {
        self.reset();
        self.append(page, rooms);
    }

    pub fn on_timer(&mut self, id: u32) {
        if id == TIMER_ID {
            for index in [0, 1, 2, 4] {
                self.buttons[index].enabled = true;
            }
        }
    }

    pub fn tick(&mut self, delta_seconds: f32) {
        let elapsed = f64::from(f32::from_bits(self.timer_elapsed_bits)) + f64::from(delta_seconds);
        let interval = f64::from(TIMER_MS) * f64::from(f32::from_bits(0x3a83126f));
        self.timer_elapsed_bits = (elapsed as f32).to_bits();
        if elapsed > interval {
            self.timer_elapsed_bits = ((elapsed - interval) as f32).to_bits();
            self.on_timer(TIMER_ID);
        }
    }

    pub fn selected_location(&self) -> Option<i32> {
        usize::try_from(self.location.selected).ok().and_then(|index| LOCATION_VALUES.get(index)).copied()
    }

    pub fn activate(&mut self, hit: Hit) -> Option<Action> {
        let index = BUTTON_HITS.iter().position(|button| *button == hit);
        if index.is_some_and(|index| !self.buttons[index].enabled) {
            return None;
        }
        let action = match hit {
            Hit::Close => Action::Close,
            Hit::Minimize => Action::Minimize,
            Hit::Create => Action::CreateRoom,
            Hit::Previous | Hit::Next | Hit::Search => Action::RequestList {
                page: match hit {
                    Hit::Previous if self.page > 1 => self.page - 1,
                    Hit::Previous => 1,
                    Hit::Next => self.page.wrapping_add(1),
                    _ => 1,
                },
                location: self.selected_location()?,
                mode: self.level.selected,
            },
            Hit::Join => Action::Join { room_id: 0, location: self.selected_location()? },
            _ => return None,
        };
        if matches!(action, Action::RequestList { .. } | Action::Join { .. }) {
            for index in [0, 1, 2, 4] {
                self.buttons[index].enabled = false;
            }
        }
        Some(action)
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<Hit> {
        for (combo, hit) in [(&self.location, Hit::Location), (&self.level, Hit::Level)] {
            if combo.open && combo.expanded_rect().contains(x, y) {
                return Some(hit);
            }
        }
        if CLOSE_RECT.contains(x, y) {
            return Some(Hit::Close);
        }
        if MINIMIZE_RECT.contains(x, y) {
            return Some(Hit::Minimize);
        }
        if LOCATION_RECT.contains(x, y) {
            return Some(Hit::Location);
        }
        if LEVEL_RECT.contains(x, y) {
            return Some(Hit::Level);
        }
        for (index, button) in self.buttons.iter().enumerate() {
            if button.enabled && button.contains(x, y) {
                return Some(BUTTON_HITS[index]);
            }
        }
        if LIST_RECT.contains(x, y) {
            if y < LIST_RECT.y + HEADER_HEIGHT {
                let mut right = LIST_RECT.x;
                return COLUMN_WIDTHS
                    .iter()
                    .position(|width| {
                        right += width;
                        x < right
                    })
                    .map(Hit::Header);
            }
            let index = self.first_row + ((y - LIST_RECT.y - HEADER_HEIGHT) / ROW_HEIGHT) as usize;
            if index < self.rooms.len() && index < self.first_row + VISIBLE_ROWS {
                return Some(Hit::Row(index));
            }
        }
        None
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> bool {
        let hit = self.hit(x, y);
        self.pressed = hit;
        if hit != Some(Hit::Location) {
            self.location.lose_focus();
        }
        if hit != Some(Hit::Level) {
            self.level.lose_focus();
        }
        match hit {
            Some(Hit::Location) => {
                self.location.mouse_move(x, y);
                self.location.mouse_down();
            }
            Some(Hit::Level) => {
                self.level.mouse_move(x, y);
                self.level.mouse_down();
            }
            Some(Hit::Row(index)) => {
                self.selected = Some(index);
                self.hovered = Some(index);
            }
            _ if LIST_RECT.contains(x, y) && y >= LIST_RECT.y + HEADER_HEIGHT => {
                self.selected = None;
                self.hovered = None;
            }
            _ => {}
        }
        hit.is_some()
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        self.location.mouse_move(x, y);
        self.level.mouse_move(x, y);
        self.hovered = match self.hit(x, y) {
            Some(Hit::Row(index)) => Some(index),
            _ => None,
        };
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<Action> {
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        if let Hit::Header(column) = pressed {
            self.sort(column);
            None
        } else {
            self.activate(pressed)
        }
    }

    pub fn double_click(&mut self, x: i32, y: i32) -> Option<Action> {
        let row = match self.hit(x, y) {
            Some(Hit::Row(index)) => Some(index),
            _ => None,
        };
        self.selected = row;
        self.hovered = row;
        row.and_then(|index| self.rooms.get(index)).map(|room| Action::Join { room_id: room.id, location: 0 })
    }

    pub fn wheel(&mut self, delta: i32) {
        let steps = i32::from(delta as i16) / 120;
        self.first_row =
            (self.first_row as i32 - steps).clamp(0, self.rooms.len().saturating_sub(VISIBLE_ROWS) as i32) as usize;
    }

    pub fn sort(&mut self, column: usize) {
        if column >= COLUMN_WIDTHS.len() {
            return;
        }
        let mut rows: Vec<_> = std::mem::take(&mut self.rooms).into_iter().enumerate().collect();
        let ascending = self.sort_ascending[column];
        rows.sort_by(|(_, left), (_, right)| {
            let order = match column {
                0 => left.id.cmp(&right.id),
                _ => compare_text(&self.cell(left, column), &self.cell(right, column)),
            };
            if ascending { order } else { order.reverse() }
        });
        self.selected = self.selected.and_then(|selected| rows.iter().position(|(index, _)| *index == selected));
        self.rooms = rows.into_iter().map(|(_, room)| room).collect();
        self.sort_ascending[column] = !ascending;
    }

    fn cell(&self, room: &RoomData, column: usize) -> String {
        match column {
            0 => room.id.to_string(),
            1 => room.title.clone(),
            2 => room.owner_name.clone(),
            3 => {
                if (1..=15).contains(&room.location) {
                    self.labels.locations[room.location as usize + 1].clone()
                } else {
                    String::new()
                }
            }
            4 => format!("{}-{}", room.min_level, room.max_level),
            5 => format!("{}/{}", room.members, room.max_members),
            _ => String::new(),
        }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        if background::MATCH_BACK.paint(list, (x, y), alpha).is_err() {
            return;
        }
        let contract = frame_controls::FRAME_114;
        let frame = contract.build(|_| self.labels.title.clone());
        contract.paint(&frame, list, (x, y), font, alpha);
        for (index, right) in [80, 260].into_iter().enumerate() {
            list.draw_text(
                x + right - font.measure(&self.labels.filters[index]) as i32,
                y + 54,
                TEXT,
                &self.labels.filters[index],
                font,
            );
        }
        self.paint_list(list, x, y, font);
        for (index, button) in self.buttons.iter().enumerate() {
            button.paint(list, x, y, self.pressed == Some(BUTTON_HITS[index]), font);
        }
        self.location.paint(list, (x, y), font, self.location.open, self.pressed == Some(Hit::Location));
        self.level.paint(list, (x, y), font, self.level.open, self.pressed == Some(Hit::Level));
    }

    fn paint_list(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        let (x, y) = (x + LIST_RECT.x, y + LIST_RECT.y);
        for row in (1..=VISIBLE_ROWS).step_by(2) {
            body.draw_texture(
                x,
                y + HEADER_HEIGHT + row as i32 * ROW_HEIGHT - 1,
                LIST_RECT.w,
                ROW_HEIGHT,
                0,
                0,
                8,
                15,
                STRIPE,
                1.0,
            );
        }
        for (row, room) in self.rooms.iter().enumerate().skip(self.first_row).take(VISIBLE_ROWS) {
            let row_y = y + HEADER_HEIGHT + (row - self.first_row) as i32 * ROW_HEIGHT;
            if self.selected == Some(row) {
                body.draw_texture(
                    x + 1,
                    row_y - 1,
                    LIST_RECT.w - 32,
                    ROW_HEIGHT,
                    0,
                    0,
                    16,
                    13,
                    crate::combo::SELECT,
                    1.0,
                );
                body.draw_texture(
                    x + LIST_RECT.w - 31,
                    row_y - 1,
                    32,
                    ROW_HEIGHT,
                    0,
                    0,
                    32,
                    13,
                    crate::combo::SELECT_END,
                    1.0,
                );
            }
            let mut column_x = x;
            for (column, width) in COLUMN_WIDTHS.into_iter().enumerate() {
                let mut cell = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
                cell.draw_text(column_x + 10, row_y + 3, TEXT, &self.cell(room, column), font);
                body.append_clipped(cell, Rect::new(column_x, row_y + 2, width, 15));
                column_x += width;
            }
        }
        let mut column_x = x;
        for (column, width) in COLUMN_WIDTHS.into_iter().enumerate() {
            let mut header = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
            let textures = if self.pressed == Some(Hit::Header(column)) { TAB_DOWN } else { TAB };
            header.slices3_h(
                column_x,
                y,
                width,
                HEADER_HEIGHT,
                (textures[0], 8),
                textures[1],
                (textures[2], 8),
                HEADER_HEIGHT,
                1.0,
            );
            header.draw_text(
                column_x + width / 2 - font.measure(&self.labels.columns[column]) as i32 / 2,
                y + HEADER_HEIGHT / 2 - font.line_height as i32 / 2 + 1,
                Argb(0xffe6dcbe),
                &self.labels.columns[column],
                font,
            );
            body.append_clipped(header, Rect::new(column_x, y, width, HEADER_HEIGHT));
            column_x += width;
        }
        list.append_clipped(body, Rect::new(x, y, LIST_RECT.w, LIST_RECT.h));
    }
}

fn compare_text(left: &str, right: &str) -> Ordering {
    let fold = |unit: u16| if (65..=90).contains(&unit) { unit + 32 } else { unit };
    left.encode_utf16().map(fold).cmp(right.encode_utf16().map(fold))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_combos_come_from_the_generated_native_combo_state() {
        let view = view();
        for (combo, contract, rect) in [(&view.location, &controls::CHILD_EC, LOCATION_RECT), (&view.level, &controls::CHILD_F0, LEVEL_RECT)] {
            assert_eq!(contract.kind, crate::child_contract::ChildKind::ComboBox);
            assert_eq!(combo.rect, rect);
            assert_eq!((combo.indent, combo.row_height), (5, 17));
            assert_eq!(combo.hovered, -1);
        }
    }

    fn view() -> MatchView {
        MatchView::new(Labels::from_sysstrings(|id| Some(format!("s{id}"))), 13)
    }

    fn room(id: i32) -> RoomData {
        RoomData {
            id,
            title: format!("title{id}"),
            location: 1,
            min_level: id,
            max_level: 78,
            members: id,
            max_members: 9,
            owner_name: format!("owner{id}"),
        }
    }

    #[test]
    fn ncmatchwnd_oncreate_100b45e0_preserves_every_filter_and_button() {
        let mut v = view();
        assert_eq!((v.page, v.location.selected, v.level.selected), (0, 1, 1));
        assert_eq!(v.labels.locations.len(), 17);
        for (index, reserved) in LOCATION_VALUES.into_iter().enumerate() {
            v.location.selected = index as i32;
            assert_eq!(v.selected_location(), Some(reserved));
            assert_eq!(v.location.entries[index], format!("s{}", LOCATION_SYSSTRINGS[index]));
        }
        for (index, hit) in BUTTON_HITS.into_iter().enumerate() {
            let rect = BUTTON_RECTS[index];
            assert_eq!(v.hit(rect.x + rect.w / 2, rect.y + rect.h / 2), Some(hit));
            assert_eq!(v.buttons[index].label, format!("s{}", BUTTON_SYSSTRINGS[index]));
        }
    }

    #[test]
    fn ncmatchwnd_handlers_wait_for_timer_not_response_and_keep_create_active() {
        for (hit, page) in [(Hit::Previous, 3), (Hit::Next, 5), (Hit::Search, 1)] {
            let mut v = view();
            v.page = 4;
            assert_eq!(v.activate(hit), Some(Action::RequestList { page, location: -1, mode: 1 }));
            assert_eq!(v.page, 4);
            assert_eq!(v.activate(Hit::Join), None);
            assert_eq!(v.activate(Hit::Create), Some(Action::CreateRoom));
            v.receive(page, &[room(7)]);
            assert_eq!(v.activate(Hit::Search), None);
            v.on_timer(TIMER_ID + 1);
            assert_eq!(v.activate(Hit::Search), None);
            v.on_timer(TIMER_ID);
            assert!(v.buttons.iter().all(|button| button.enabled));
            assert_eq!(v.page, page);
        }
    }

    #[test]
    fn ncmatchwnd_join_100b2a30_and_list_doubleclick_100b20e0_are_distinct() {
        let mut v = view();
        v.receive(1, &[room(7)]);
        v.location.selected = 16;
        assert_eq!(v.activate(Hit::Join), Some(Action::Join { room_id: 0, location: 15 }));
        assert_eq!(v.double_click(8, 103), Some(Action::Join { room_id: 7, location: 0 }));
        assert_eq!(v.selected, Some(0));
        assert_eq!(v.double_click(8, 120), None);
        assert_eq!(v.selected, None);
        assert_eq!(v.double_click(8, 84), None);
    }

    #[test]
    fn receivelistpartywating_104002a0_resets_before_appending_and_caps_at_fifteen() {
        let mut v = view();
        let rooms: Vec<_> = (0..40).map(room).collect();
        v.receive(3, &rooms);
        assert_eq!(v.rooms.len(), ROW_CAPACITY);
        assert_eq!(v.rooms.last().unwrap().id, 14);
        v.selected = Some(5);
        for delta in [-32768, -240, -120, -1, 0, 120, 32767] {
            v.wheel(delta);
            assert_eq!(v.first_row, 0);
        }
        v.receive(0, &[]);
        assert!(v.rooms.is_empty());
        assert_eq!((v.page, v.selected, v.hovered), (0, None, None));
        v.append(4, &[room(22)]);
        v.append(5, &[room(33)]);
        assert_eq!(v.rooms.iter().map(|r| r.id).collect::<Vec<_>>(), [22, 33]);
        assert_eq!(v.page, 5);
    }

    #[test]
    fn ncmatchwnd_create_100b2990_covers_all_level_branches() {
        assert_eq!(create_levels(None), (1, 78));
        for (level, expected) in [
            (0, (1, 5)),
            (1, (1, 6)),
            (5, (1, 10)),
            (6, (1, 11)),
            (73, (68, 78)),
            (74, (69, 78)),
            (78, (73, 78)),
            (99, (94, 78)),
        ] {
            assert_eq!(create_levels(Some(level)), expected);
        }
    }

    #[test]
    fn nclistctrl_sort_10022fc0_keeps_item_identity_and_per_column_direction() {
        let mut v = view();
        v.receive(1, &[room(9), room(2), room(2), room(11)]);
        v.selected = Some(2);
        v.sort(0);
        assert_eq!(v.rooms.iter().map(|r| r.id).collect::<Vec<_>>(), [2, 2, 9, 11]);
        assert_eq!(v.selected, Some(1));
        v.sort(0);
        assert_eq!(v.rooms.iter().map(|r| r.id).collect::<Vec<_>>(), [11, 9, 2, 2]);
        assert_eq!(v.selected, Some(3));
        assert!(v.sort_ascending[1]);
        assert_eq!(compare_text("abc", "ABC"), Ordering::Equal);
        assert_eq!(compare_text("x10", "x2"), Ordering::Less);
    }

    #[test]
    fn nclistctrl_sortbycolumn_10023000_orders_the_level_and_member_columns_as_text() {
        let mut v = view();
        v.receive(1, &[room(9), room(10), room(2)]);
        v.sort(4);
        assert_eq!(v.rooms.iter().map(|r| r.id).collect::<Vec<_>>(), [10, 2, 9]);
        v.sort(5);
        assert_eq!(v.rooms.iter().map(|r| r.id).collect::<Vec<_>>(), [10, 2, 9]);
    }

    #[test]
    fn combo_dropdown_has_priority_and_confirms_on_press() {
        let mut v = view();
        v.receive(1, &[room(7)]);
        assert!(v.mouse_down(85, 53));
        assert!(v.location.open);
        assert_eq!(v.hit(85, 104), Some(Hit::Location));
        assert!(v.mouse_down(85, 104));
        assert_eq!(v.location.selected, 2);
        assert!(!v.location.open);
        assert_eq!(v.mouse_up(85, 104), None);
        assert_eq!(v.selected, None);
        v.mouse_down(370, 51);
        assert_eq!(v.mouse_up(370, 51), Some(Action::RequestList { page: 1, location: 1, mode: 1 }));
    }

    #[test]
    fn ncmatchwnd_onpaint_100b23b0_preserves_native_quads_cells_and_origin() {
        use crate::font::{Glyphs, gly};
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let font = BitmapFont::new("font".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap());
        let mut v = view();
        v.receive(2, &[room(7)]);
        v.selected = Some(0);
        let mut list = DrawList { capture_text: true, ..Default::default() };
        v.paint(&mut list, 100, 50, &font, 0.75);
        let back = list.quads.iter().find(|q| q.texture == BACK).unwrap();
        assert_eq!((back.x, back.y, back.w, back.h), (100.0, 70.0, 600.0, 411.0));
        assert!(!list.quads.iter().any(|q| q.texture.contains("ScrollBar")));
        assert_eq!(list.quads.iter().filter(|q| q.texture == TAB[0]).count(), 6);
        for value in ["7", "title7", "owner7", "s1048", "7-78", "7/9"] {
            assert!(list.text_runs.iter().any(|t| t.text == value), "{value}");
        }
        let title = list.text_runs.iter().find(|t| t.text == "title7").unwrap();
        assert_eq!((title.rect[0], title.rect[1]), (151.0, 155.0));
        assert_eq!(title.clip, Some(Rect::new(141, 154, 240, 15)));
        assert_eq!(v.hit(591, 355), None);
        assert_eq!(v.hit(592, 103), None);
    }

    #[test]
    fn nconsole_tick_1007fa90_keeps_period_across_receive_and_window_actions() {
        let mut v = view();
        v.tick(4.0);
        assert_eq!(v.activate(Hit::Search), Some(Action::RequestList { page: 1, location: -1, mode: 1 }));
        v.receive(1, &[room(1)]);
        assert!(!v.buttons[2].enabled);
        assert_eq!(v.activate(Hit::Close), Some(Action::Close));
        assert_eq!(v.activate(Hit::Minimize), Some(Action::Minimize));
        assert_eq!(v.hit(567, 10), Some(Hit::Minimize));
        v.tick(1.0);
        assert!(!v.buttons[2].enabled);
        v.tick(0.000001);
        assert!(v.buttons[2].enabled);
        assert!(f32::from_bits(v.timer_elapsed_bits) < 0.000001);
    }

    #[test]
    #[ignore]
    fn ncmatchwnd_original_handlers_unicorn_all_filter_page_and_level_branches() {
        let python = std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into());
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let output = std::process::Command::new(python)
            .arg(root.join("docs/evidence/party-match-extraction/oracle.py"))
            .output()
            .unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let text = String::from_utf8(output.stdout).unwrap();
        let mut count = 0;
        for line in text.lines() {
            count += 1;
            let cells: Vec<_> = line.split('\t').collect();
            let number = |index: usize| cells[index].parse::<i32>().unwrap();
            match cells[0] {
                "create" => {
                    let level = (cells[1] != "none").then(|| number(1));
                    assert_eq!(create_levels(level), (number(2), number(3)), "{line}");
                    assert_eq!(number(4) as u32, DEFAULT_TITLE_SYSTEMMSG, "{line}");
                }
                "tick" => {
                    let bits = |index: usize| cells[index].parse::<u32>().unwrap();
                    let mut v = view();
                    v.activate(Hit::Join);
                    v.timer_elapsed_bits = bits(1);
                    v.tick(f32::from_bits(bits(2)));
                    assert_eq!(v.timer_elapsed_bits, bits(3), "{line}");
                    assert_eq!(v.buttons[4].enabled, number(4) == 1, "{line}");
                }
                "timer" => {
                    let mut v = view();
                    v.activate(Hit::Join);
                    v.on_timer(number(1) as u32);
                    assert_eq!(
                        v.buttons.iter().enumerate().filter(|(i, b)| *i != 3 && b.enabled).count(),
                        number(2) as usize
                    );
                }
                kind => {
                    let mut v = view();
                    v.page = number(1);
                    v.location.selected = LOCATION_VALUES.iter().position(|value| *value == number(2)).unwrap() as i32;
                    v.level.selected = number(3);
                    let hit = match kind {
                        "previous" => Hit::Previous,
                        "next" => Hit::Next,
                        "search" => Hit::Search,
                        "join" => Hit::Join,
                        _ => panic!("{line}"),
                    };
                    let expected = if kind == "join" {
                        Action::Join { room_id: number(4), location: number(5) }
                    } else {
                        Action::RequestList { page: number(4), location: number(5), mode: number(6) }
                    };
                    assert_eq!(v.activate(hit), Some(expected), "{line}");
                }
            }
        }
        assert_eq!(count, 966);
    }
}
