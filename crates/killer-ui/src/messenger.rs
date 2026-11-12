use crate::child_contract::{ChildContract, ChildKind};
use crate::color::Argb;
use crate::draw::DrawList;
use crate::edit_box::{EditBox, EditInput, FontMetrics, Metrics, NOTICE_ENTER};
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::frame_contract::{FrameContract, FrameHit, FrameTitle};
use crate::scroll::{ScrollHit, ScrollWnd};
use crate::window::{Rect, WindowId};

pub const MESSENGER_ID: WindowId = WindowId::MESSENGER;
pub const SIZE: (i32, i32) = (256, 401);
pub const INITIAL_POSITION_XY: (i32, i32) = (295, 158);
pub const FRAME_RECT: Rect = Rect::new(0, 0, 256, 20);
pub const FRAME_ID: u32 = 0xa03;
pub const TITLE_SYSSTRING: u32 = 999;
pub const TAB_SYSSTRING: u32 = 997;
pub const CHAT_TITLE_SYSSTRING: u32 = 997;
pub const ONLINE_SYSSTRING: u32 = 1006;
pub const OFFLINE_SYSSTRING: u32 = 1007;
pub const MSN_BACK: &str = "L2UI_CH3.Msn.msn_back";
pub const MSN_BACK_RECT: Rect = Rect::new(0, 20, 276, 28);
pub const MSN_BACK_ZERO: &str = "L2UI_CH3.Msn.msn_back0";
pub const PAGE_RECT: Rect = Rect::new(0, 48, 256, 353);
pub const TAB_ACTIVE: &str = "L2UI_CH3.Msn.msn_tab1";
pub const TAB_INACTIVE: &str = "L2UI_CH3.Msn.msn_tab2";
pub const TAB_RECT: Rect = Rect::new(12, 25, 114, 23);
pub const TAB_TEXT_OFFSET: (i32, i32) = (2, 1);
pub const TAB_TEXT: Argb = Argb(0xFFE6DCBE);
pub const MINUS: &str = "L2UI_CH3.QUESTWND.QuestWndMinusBtn";
pub const PLUS: &str = "L2UI_CH3.QUESTWND.QuestWndPlusBtn";
pub const SELECT_LEFT: &str = "L2UI_CH3.ListCtrl.TextSelect";
pub const SELECT_RIGHT: &str = "L2UI_CH3.ListCtrl.TextSelect2";
pub const ITEM_TEXT: Argb = Argb(0xFFDCDCDC);

pub const TREE_RECT: Rect = Rect::new(8, 45, 240, 270);
pub const TREE_WIDTH: i32 = 224;
pub const ROW_HEIGHT: i32 = 19;
pub const ICON_SIZE: i32 = 15;
pub const HIT_LINE: (i32, i32) = (220, 14);
pub const GROUP_ORIGIN: (i32, i32) = (3, 4);
pub const CHILD_INDENT: i32 = 3;
pub const TEXT_PAD: i32 = 3;
pub const TEXT_RIGHT: i32 = 218;
pub const SELECT_TAIL: i32 = 49;
pub const ELLIPSIS_RESERVE: i32 = 12;
pub const SCROLL_STEP: i32 = 15;
pub const WHEEL_STEP: i32 = 45;
pub const WHEEL_DELTA: i32 = 120;
pub const SCROLL_PADDING: i32 = 21;

pub const CHAT_SIZE: (i32, i32) = (256, 401);
pub const CHAT_BACK: &str = "L2UI_CH3.msn_back3";
pub const CHAT_FRAME_ID: u32 = 0xa70;
pub const CHAT_FRAME_RECT: Rect = Rect::new(0, 0, 256, 20);
pub const CHAT_NAME_POS: (i32, i32) = (11, 54);
pub const CHAT_BOX_RECT: Rect = Rect::new(9, 96, 238, 252);
pub const CHAT_EDIT_RECT: Rect = Rect::new(8, 360, 239, 16);
pub const CHAT_EDIT_ID: u32 = 0xa71;
pub const CHAT_EDIT_MAX: i32 = 4800;
pub const CHAT_MAX_LINES: usize = 200;
pub const CHAT_VISIBLE: usize = 14;
pub const CHAT_PITCH: i32 = 18;
pub const CHAT_MARGIN: (i32, i32) = (4, 4);
pub const CHAT_WRAP_MARGIN: i32 = 16 + 6;
pub const CHAT_FLAG_MARGIN: i32 = 12;
pub const CHAT_PREFIX: &str = "   ";
pub const SENDER_COLOR: Argb = Argb(0xFF929292);
pub const FAILURE_COLOR: Argb = Argb(0xFFE37171);
pub const SYSMSG_SENDER_PREFIX: u32 = 1326;
pub const SYSMSG_DELIVERY_FAILED: u32 = 1327;
pub const CHAT_CODE_MESSAGES: [(i32, u32); 4] = [(145, 1356), (147, 1354), (1079, 1355), (620, 1357)];

pub const TASK_SIZE: (i32, i32) = (172, 132);
pub const TASK_MARGIN: (i32, i32) = (172, 414);
pub const TASK_BACKS: [&str; 3] = ["L2UI_CH3.msn6_back1", "L2UI_CH3.msn6_back2", "L2UI_CH3.msn6_back3"];
pub const TASK_BOX_RECT: Rect = Rect::new(14, 3, 155, 126);
pub const TASK_BUTTON: (i32, i32) = (140, 21);
pub const TASK_VISIBLE: usize = 6;
pub const TASK_MAX: usize = 300;
pub const TASK_TEXT_WIDTH: i32 = 116;
pub const TASK_TEXT_POS: (i32, i32) = (7, 4);
pub const TASK_BUTTON_TEXTURES: [&str; 3] = [
    "L2UI_CH3.Msn.msnbutton",
    "L2UI_CH3.Msn.msnbutton_down",
    "L2UI_CH3.Msn.msnbutton_light",
];
pub const TASK_BLINK_MS: i32 = 1000;

pub fn task_rect(viewport: (i32, i32)) -> Rect {
    Rect::new(viewport.0 - TASK_MARGIN.0, viewport.1 - TASK_MARGIN.1, TASK_SIZE.0, TASK_SIZE.1)
}

pub fn chat_position(roll_x: u32, roll_y: u32) -> (i32, i32) {
    ((roll_x % 200) as i32 + 200, (roll_y % 150) as i32 + 150)
}

const MESSENGER_FRAME: FrameContract = FrameContract {
    rect: FRAME_RECT,
    id: FRAME_ID,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(TITLE_SYSSTRING),
    close_button: true,
    minimize_button: true,
};

const CHAT_FRAME: FrameContract = FrameContract {
    rect: CHAT_FRAME_RECT,
    id: CHAT_FRAME_ID,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(CHAT_TITLE_SYSSTRING),
    close_button: true,
    minimize_button: true,
};

const CHAT_EDIT: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: CHAT_EDIT_RECT,
    id: CHAT_EDIT_ID,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x214, 0x1), (0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x254, 0x12c0), (0x2b8, 0x1)],
    textures: &[],
};

pub fn name_filter(text: &str) -> String {
    text.chars().filter(|&c| (0x20..=0x7E).contains(&(c as u32)) || c == '\r' || c == '\n').collect()
}

pub fn truncate_ellipsis(text: &str, limit: i32, m: &dyn Metrics) -> String {
    let mut width = 0;
    let mut out = String::new();
    for ch in text.chars() {
        if ch == '\n' || ch == '\r' {
            out.push(ch);
            continue;
        }
        width += m.char_width(ch as u16);
        if width > limit - ELLIPSIS_RESERVE {
            out.push_str("...");
            return out;
        }
        out.push(ch);
    }
    out
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FriendRow {
    pub name: String,
    pub online: bool,
    pub object_id: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Selection {
    Group(usize),
    Leaf(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowKind {
    Group(usize),
    Leaf(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowBox {
    pub kind: RowKind,
    pub x: i32,
    pub y: i32,
}

impl RowBox {
    pub fn icon_rect(&self) -> Rect {
        Rect::new(self.x, self.y, ICON_SIZE, ICON_SIZE)
    }

    pub fn line_rect(&self) -> Rect {
        let x = match self.kind {
            RowKind::Group(_) => self.x + ICON_SIZE,
            RowKind::Leaf(_) => self.x,
        };
        Rect::new(x, self.y, HIT_LINE.0, HIT_LINE.1)
    }
}

#[derive(Clone, Debug)]
pub struct FriendTree {
    pub friends: Vec<FriendRow>,
    pub groups_expanded: [bool; 2],
    pub groups: [String; 2],
    pub selected: Option<Selection>,
    pub offset: i32,
    bar: ScrollWnd,
}

impl FriendTree {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Self {
            friends: Vec::new(),
            groups_expanded: [false; 2],
            groups: [text(ONLINE_SYSSTRING), text(OFFLINE_SYSSTRING)],
            selected: None,
            offset: 0,
            bar: ScrollWnd::new(TREE_RECT.w, TREE_RECT.h),
        }
    }

    pub fn rebuild(&mut self, friends: Vec<FriendRow>) {
        self.friends = friends;
        self.groups_expanded = [false; 2];
        self.selected = None;
        self.offset = 0;
        self.sync_bar();
    }

    pub fn update(&mut self, friends: Vec<FriendRow>) {
        self.friends = friends;
        self.selected = None;
        self.clamp();
        self.sync_bar();
    }

    fn members(&self, group: usize) -> Vec<usize> {
        (0..self.friends.len()).filter(|&index| usize::from(!self.friends[index].online) == group).collect()
    }

    pub fn layout(&self) -> (Vec<RowBox>, i32) {
        let mut rows = Vec::new();
        let mut y = GROUP_ORIGIN.1;
        for group in 0..2 {
            rows.push(RowBox { kind: RowKind::Group(group), x: GROUP_ORIGIN.0, y });
            y += ROW_HEIGHT;
            if self.groups_expanded[group] {
                for index in self.members(group) {
                    rows.push(RowBox { kind: RowKind::Leaf(index), x: GROUP_ORIGIN.0 + ICON_SIZE + CHILD_INDENT, y });
                    y += ROW_HEIGHT;
                }
            }
        }
        (rows, y)
    }

    pub fn content_height(&self) -> i32 {
        self.layout().1
    }

    pub fn minimum(&self) -> i32 {
        let total = self.content_height() + SCROLL_PADDING;
        if total > TREE_RECT.h { TREE_RECT.h - total } else { 0 }
    }

    fn clamp(&mut self) {
        self.offset = self.offset.clamp(self.minimum(), 0);
    }

    fn sync_bar(&mut self) {
        let total = self.content_height() + SCROLL_PADDING;
        self.bar.slider.set_range(total, TREE_RECT.h);
        self.bar.slider.set_pos(-self.offset);
    }

    pub fn scroll_up(&mut self) {
        if self.offset < 0 {
            self.offset = (self.offset + SCROLL_STEP).min(0);
        }
        self.sync_bar();
    }

    pub fn scroll_down(&mut self) {
        let minimum = self.minimum();
        if self.offset > minimum {
            self.offset = (self.offset - SCROLL_STEP).max(minimum);
        }
        self.sync_bar();
    }

    pub fn wheel(&mut self, delta: i32) {
        let notches = delta / WHEEL_DELTA;
        if notches > 0 {
            self.offset = (self.offset + WHEEL_STEP * notches).min(0);
        } else {
            self.offset = (self.offset + WHEEL_STEP * notches).max(self.minimum());
        }
        self.sync_bar();
    }

    pub fn set_slider(&mut self, position: i32) {
        self.offset = -position;
        self.clamp();
    }

    pub fn row_at(&self, x: i32, y: i32) -> Option<RowBox> {
        let (rows, _) = self.layout();
        let (lx, ly) = (x - TREE_RECT.x, y - TREE_RECT.y - self.offset);
        rows.into_iter().find(|row| match row.kind {
            RowKind::Group(_) => row.icon_rect().contains(lx, ly) || row.line_rect().contains(lx, ly),
            RowKind::Leaf(index) => self.friends.get(index).is_some() && row.line_rect().contains(lx, ly),
        })
    }

    pub fn click(&mut self, x: i32, y: i32) -> Option<RowBox> {
        let row = self.row_at(x, y)?;
        match row.kind {
            RowKind::Group(group) => {
                self.selected = Some(Selection::Group(group));
                self.groups_expanded[group] = !self.groups_expanded[group];
                self.clamp();
                self.sync_bar();
            }
            RowKind::Leaf(index) => self.selected = Some(Selection::Leaf(index)),
        }
        Some(row)
    }

    pub fn double_click(&self, x: i32, y: i32) -> Option<String> {
        let row = self.row_at(x, y)?;
        let RowKind::Leaf(index) = row.kind else { return None };
        let friend = self.friends.get(index)?;
        (friend.online && !friend.name.is_empty()).then(|| friend.name.clone())
    }

    pub fn bar_hit(&self, x: i32, y: i32) -> Option<ScrollHit> {
        self.bar.hit(TREE_RECT.w, TREE_RECT.h, x - TREE_RECT.x, y - TREE_RECT.y)
    }

    pub fn bar_down(&mut self, x: i32, y: i32) -> bool {
        match self.bar_hit(x, y) {
            Some(ScrollHit::Up) => {
                self.bar.up_pressed = true;
                self.scroll_up();
                true
            }
            Some(ScrollHit::Down) => {
                self.bar.down_pressed = true;
                self.scroll_down();
                true
            }
            Some(ScrollHit::Slider(position)) => {
                self.bar.slider.mouse_down(position);
                true
            }
            None => false,
        }
    }

    pub fn bar_move(&mut self, y: i32) {
        if let Some(position) = self.bar.slider.mouse_move(y - TREE_RECT.y - crate::scroll::BUTTON_SIZE) {
            self.set_slider(position);
        }
    }

    pub fn bar_up(&mut self) {
        self.bar.up_pressed = false;
        self.bar.down_pressed = false;
        self.bar.slider.mouse_up();
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let (ox, oy) = (origin.0 + TREE_RECT.x, origin.1 + TREE_RECT.y);
        let start = list.quads.len();
        let metrics = FontMetrics(font);
        let (rows, _) = self.layout();
        for row in rows {
            let (x, y) = (ox + row.x, oy + self.offset + row.y);
            match row.kind {
                RowKind::Group(group) => {
                    let texture = if self.groups_expanded[group] { MINUS } else { PLUS };
                    list.draw_texture(x, y, ICON_SIZE, ICON_SIZE, 0, 0, ICON_SIZE, ICON_SIZE, texture, 1.0);
                    let selected = self.selected == Some(Selection::Group(group));
                    self.paint_select(list, x + ICON_SIZE, y, selected);
                    let text = format!("{} ({})", self.groups[group], self.members(group).len());
                    let limit = TEXT_RIGHT - ICON_SIZE - x + ox;
                    let shown = truncate_ellipsis(&text, limit, &metrics);
                    list.draw_text(x + ICON_SIZE + TEXT_PAD, y + 1 + ((ICON_SIZE - font.line_height as i32) >> 1), ITEM_TEXT, &shown, font);
                }
                RowKind::Leaf(index) => {
                    let Some(friend) = self.friends.get(index) else { continue };
                    let selected = self.selected == Some(Selection::Leaf(index));
                    self.paint_select(list, x, y, selected);
                    let limit = TEXT_RIGHT - (x - ox);
                    let shown = truncate_ellipsis(&friend.name, limit, &metrics);
                    list.draw_text(x + TEXT_PAD, y + 1 + ((ICON_SIZE - font.line_height as i32) >> 1), ITEM_TEXT, &shown, font);
                }
            }
        }
        list.clip_from(start, Rect::new(ox, oy, TREE_WIDTH, TREE_RECT.h));
        self.bar.paint(list, ox, oy, TREE_RECT.w, TREE_RECT.h);
    }

    fn paint_select(&self, list: &mut DrawList, x: i32, y: i32, selected: bool) {
        if !selected {
            return;
        }
        let first = HIT_LINE.0 - SELECT_TAIL;
        list.draw_texture(x, y, first, HIT_LINE.1, 0, 0, 16, 13, SELECT_LEFT, 1.0);
        list.draw_texture(x + first, y, 32, HIT_LINE.1, 0, 0, 32, 13, SELECT_RIGHT, 1.0);
    }
}

#[derive(Clone, Debug)]
pub struct ChatBox {
    lines: Vec<(String, Argb)>,
    pub first: usize,
    scroll: ScrollWnd,
}

impl Default for ChatBox {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatBox {
    pub fn new() -> Self {
        Self { lines: Vec::new(), first: 0, scroll: ScrollWnd::new(CHAT_BOX_RECT.w, CHAT_BOX_RECT.h) }
    }

    pub fn clear(&mut self) {
        self.lines.clear();
        self.first = 0;
        self.update_bar();
    }

    pub fn lines(&self) -> &[(String, Argb)] {
        &self.lines
    }

    fn update_bar(&mut self) {
        self.scroll.slider.set_range(self.lines.len() as i32, CHAT_VISIBLE as i32);
        self.scroll.slider.set_pos(self.first as i32);
    }

    pub fn add_fragment(&mut self, text: &str, color: Argb, indented: bool, m: &dyn Metrics) {
        let width = CHAT_BOX_RECT.w - CHAT_WRAP_MARGIN - if indented { CHAT_FLAG_MARGIN } else { 0 };
        let mut chunks = Vec::new();
        let mut current = String::new();
        let mut used = 0;
        for ch in text.chars() {
            if ch == '\r' {
                continue;
            }
            if ch == '\n' {
                chunks.push(std::mem::take(&mut current));
                used = 0;
                continue;
            }
            if (ch as u32) < 0x20 {
                continue;
            }
            let advance = m.char_width(ch as u16);
            if used + advance > width {
                chunks.push(std::mem::take(&mut current));
                used = 0;
            }
            current.push(ch);
            used += advance;
        }
        if !current.is_empty() || chunks.is_empty() {
            chunks.push(current);
        }
        for chunk in chunks {
            let shown = if indented { format!("{CHAT_PREFIX}{chunk}") } else { chunk };
            if self.lines.len() >= CHAT_MAX_LINES {
                self.lines.remove(0);
            } else if self.lines.len() >= CHAT_VISIBLE {
                self.first += 1;
            }
            self.lines.push((shown, color));
        }
        self.first = self.first.min(self.lines.len().saturating_sub(CHAT_VISIBLE));
        self.update_bar();
    }

    pub fn add_line(&mut self, sender: Option<&str>, text: &str, color: Argb, prefix: &dyn Fn(&str) -> String, m: &dyn Metrics) {
        if text.is_empty() {
            return;
        }
        match sender {
            Some(sender) => {
                self.add_fragment(&prefix(sender), SENDER_COLOR, false, m);
                self.add_fragment(text, color, true, m);
            }
            None => self.add_fragment(text, color, false, m),
        }
    }

    pub fn line_up(&mut self) {
        if self.lines.len() > CHAT_VISIBLE {
            self.first = self.first.saturating_sub(1);
            self.update_bar();
        }
    }

    pub fn line_down(&mut self) {
        if self.lines.len() > CHAT_VISIBLE {
            self.first = (self.first + 1).min(self.lines.len() - CHAT_VISIBLE);
            self.update_bar();
        }
    }

    pub fn wheel(&mut self, delta: i32) {
        let notches = delta / WHEEL_DELTA;
        let maximum = self.lines.len().saturating_sub(CHAT_VISIBLE) as i32;
        self.first = (self.first as i32 - notches).clamp(0, maximum) as usize;
        self.update_bar();
    }

    pub fn bar_down(&mut self, x: i32, y: i32) -> bool {
        match self.scroll.hit(CHAT_BOX_RECT.w, CHAT_BOX_RECT.h, x - CHAT_BOX_RECT.x, y - CHAT_BOX_RECT.y) {
            Some(ScrollHit::Up) => {
                self.scroll.up_pressed = true;
                self.line_up();
                true
            }
            Some(ScrollHit::Down) => {
                self.scroll.down_pressed = true;
                self.line_down();
                true
            }
            Some(ScrollHit::Slider(position)) => {
                self.scroll.slider.mouse_down(position);
                true
            }
            None => false,
        }
    }

    pub fn bar_move(&mut self, y: i32) {
        if let Some(position) = self.scroll.slider.mouse_move(y - CHAT_BOX_RECT.y - crate::scroll::BUTTON_SIZE) {
            self.first = (position.max(0) as usize).min(self.lines.len().saturating_sub(CHAT_VISIBLE));
        }
    }

    pub fn bar_up(&mut self) {
        self.scroll.up_pressed = false;
        self.scroll.down_pressed = false;
        self.scroll.slider.mouse_up();
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let (ox, oy) = (origin.0 + CHAT_BOX_RECT.x, origin.1 + CHAT_BOX_RECT.y);
        let start = list.quads.len();
        for (row, (text, color)) in self.lines.iter().skip(self.first).take(CHAT_VISIBLE).enumerate() {
            list.draw_text(ox + CHAT_MARGIN.0, oy + CHAT_PITCH * row as i32 + CHAT_MARGIN.1, *color, text, font);
        }
        list.clip_from(start, CHAT_BOX_RECT.offset(origin.0, origin.1));
        self.scroll.paint(list, ox, oy, CHAT_BOX_RECT.w, CHAT_BOX_RECT.h);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChatHit {
    Close,
    Minimize,
    Box,
    Edit,
    Page,
}

#[derive(Clone, Debug)]
pub struct ChatWindow {
    pub id: WindowId,
    pub name: String,
    pub in_use: bool,
    pub hidden: bool,
    pub log: ChatBox,
    pub edit: EditBox,
    pub frame: FrameCtrl,
    pressed: Option<ChatHit>,
}

impl ChatWindow {
    pub fn new(id: WindowId, lookup: impl Fn(u32) -> Option<String>) -> Self {
        let mut edit = EditBox::from_contract(&CHAT_EDIT);
        edit.set_max_len(CHAT_EDIT_MAX);
        Self {
            id,
            name: String::new(),
            in_use: false,
            hidden: true,
            log: ChatBox::new(),
            edit,
            frame: CHAT_FRAME.build(|id| lookup(id).unwrap_or_default()),
            pressed: None,
        }
    }

    pub fn assign(&mut self, name: &str) {
        self.name = name.to_string();
        self.in_use = true;
    }

    pub fn release(&mut self) {
        self.in_use = false;
        self.hidden = true;
        self.name.clear();
        self.log.clear();
        self.edit.clear();
        self.edit.kill_focus();
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<ChatHit> {
        if !Rect::new(0, 0, CHAT_SIZE.0, CHAT_SIZE.1).contains(x, y) {
            return None;
        }
        match CHAT_FRAME.hit(&self.frame, (0, 0), (x, y)) {
            Some(FrameHit::Close) => return Some(ChatHit::Close),
            Some(FrameHit::Minimize) => return Some(ChatHit::Minimize),
            _ => {}
        }
        if CHAT_EDIT_RECT.contains(x, y) {
            return Some(ChatHit::Edit);
        }
        if CHAT_BOX_RECT.contains(x, y) {
            return Some(ChatHit::Box);
        }
        Some(ChatHit::Page)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, font: &BitmapFont) -> Option<ChatHit> {
        let hit = self.hit(x, y);
        self.pressed = hit.clone();
        match hit {
            Some(ChatHit::Edit) => {
                if !self.edit.focused {
                    self.edit.focus();
                }
                let metrics = FontMetrics(font);
                self.edit.mouse_down(x, &metrics);
                self.edit.layout(&metrics);
            }
            Some(ChatHit::Box) => {
                self.log.bar_down(x, y);
            }
            _ => {}
        }
        hit
    }

    pub fn mouse_move(&mut self, x: i32, y: i32, font: &BitmapFont) {
        self.log.bar_move(y);
        if self.edit.focused {
            let metrics = FontMetrics(font);
            self.edit.mouse_move(x, &metrics);
            self.edit.layout(&metrics);
        }
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<ChatHit> {
        self.log.bar_up();
        self.edit.mouse_up();
        let pressed = self.pressed.take()?;
        (self.hit(x, y).as_ref() == Some(&pressed) && matches!(pressed, ChatHit::Close | ChatHit::Minimize)).then_some(pressed)
    }

    pub fn key(&mut self, key: crate::chat::InputKey, font: &BitmapFont) -> Option<String> {
        if !self.edit.focused {
            return None;
        }
        if self.edit.input_key(key, &FontMetrics(font)) == EditInput::Escape {
            self.edit.kill_focus();
            return None;
        }
        let entered = self.edit.notices.drain(..).any(|notice| notice.code == NOTICE_ENTER);
        if !entered {
            return None;
        }
        let text = self.edit.get_text();
        self.edit.clear();
        text.filter(|text| !text.is_empty())
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        let start = list.quads.len();
        let (w, h) = (CHAT_SIZE.0, CHAT_SIZE.1 - 20);
        list.draw_texture(x, y + 20, w, h, 0, 0, w, h, CHAT_BACK, 1.0);
        if !self.name.is_empty() {
            list.draw_text(x + CHAT_NAME_POS.0, y + CHAT_NAME_POS.1, ITEM_TEXT, &self.name, font);
        }
        list.clip_from(start, Rect::new(x, y, CHAT_SIZE.0, CHAT_SIZE.1));
        CHAT_FRAME.paint(&self.frame, list, origin, font, alpha);
        self.log.paint(list, origin, font);
        self.edit.paint(list, origin, &FontMetrics(font), font);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Task {
    pub name: String,
    pub pending: bool,
    pub phase: bool,
    pub chat: Option<WindowId>,
    elapsed: i32,
}

#[derive(Clone, Debug, Default)]
pub struct TaskBar {
    pub tasks: Vec<Task>,
    pub pressed: Option<usize>,
    pub first: usize,
}

impl TaskBar {
    pub fn index_of(&self, name: &str) -> Option<usize> {
        self.tasks.iter().position(|task| task.name == name)
    }

    pub fn add(&mut self, chat: WindowId, name: &str, pending: bool, chat_visible: bool) {
        match self.index_of(name) {
            Some(index) => {
                if !chat_visible {
                    self.tasks[index].pending = pending;
                }
            }
            None => {
                if self.tasks.len() < TASK_MAX {
                    self.tasks.push(Task { name: name.to_string(), pending, phase: false, chat: Some(chat), elapsed: 0 });
                }
            }
        }
    }

    pub fn remove(&mut self, name: &str) {
        if let Some(index) = self.index_of(name) {
            self.tasks.remove(index);
            self.first = self.first.min(self.tasks.len().saturating_sub(TASK_VISIBLE));
        }
    }

    pub fn tick(&mut self, elapsed_ms: i32) {
        for task in &mut self.tasks {
            task.elapsed += elapsed_ms;
            while task.elapsed >= TASK_BLINK_MS {
                task.elapsed -= TASK_BLINK_MS;
                task.phase = !task.phase;
            }
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<usize> {
        let (lx, ly) = (x - TASK_BOX_RECT.x, y - TASK_BOX_RECT.y);
        if !(0..=TASK_BOX_RECT.w).contains(&lx) || ly < 0 {
            return None;
        }
        (0..TASK_VISIBLE).find_map(|row| {
            let index = self.first + row;
            (index < self.tasks.len() && ly >= TASK_BUTTON.1 * row as i32 && ly <= TASK_BUTTON.1 * row as i32 + TASK_BUTTON.1).then_some(index)
        })
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        if let Some(index) = self.hit(x, y) {
            self.pressed = Some(index);
        }
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<String> {
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        let task = self.tasks.get_mut(pressed)?;
        task.chat?;
        task.pending = false;
        Some(task.name.clone())
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let (x, y) = origin;
        list.draw_texture(x + 12, y, 180, 8, 0, 0, 180, 8, TASK_BACKS[0], 1.0);
        list.draw_texture(x + 12, y + 8, 180, 116, 0, 0, 180, 8, TASK_BACKS[1], 1.0);
        list.draw_texture(x + 12, y + 124, 180, 8, 0, 0, 180, 8, TASK_BACKS[2], 1.0);
        let (ox, oy) = (x + TASK_BOX_RECT.x, y + TASK_BOX_RECT.y);
        let start = list.quads.len();
        let metrics = FontMetrics(font);
        for (row, task) in self.tasks.iter().skip(self.first).take(TASK_VISIBLE).enumerate() {
            let index = self.first + row;
            let texture = if self.pressed == Some(index) {
                TASK_BUTTON_TEXTURES[1]
            } else if !task.pending || !task.phase {
                TASK_BUTTON_TEXTURES[0]
            } else {
                TASK_BUTTON_TEXTURES[2]
            };
            let top = oy + TASK_BUTTON.1 * row as i32;
            list.draw_texture(ox, top, TASK_BUTTON.0, TASK_BUTTON.1, 0, 0, TASK_BUTTON.0, TASK_BUTTON.1, texture, 1.0);
            let shown = truncate_ellipsis(&task.name, TASK_TEXT_WIDTH, &metrics);
            list.draw_text(ox + TASK_TEXT_POS.0, top + TASK_TEXT_POS.1, ITEM_TEXT, &shown, font);
        }
        list.clip_from(start, TASK_BOX_RECT.offset(x, y));
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessengerHit {
    Close,
    Minimize,
    Tree,
    Page,
}

#[derive(Clone, Debug)]
pub struct MessengerView {
    pub frame: FrameCtrl,
    pub tree: FriendTree,
    pub tab_text: String,
    pressed: Option<MessengerHit>,
}

impl MessengerView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Self { frame: MESSENGER_FRAME.build(text), tree: FriendTree::new(&lookup), tab_text: text(TAB_SYSSTRING), pressed: None }
    }

    pub fn page_to_window(x: i32, y: i32) -> (i32, i32) {
        (x + PAGE_RECT.x, y + PAGE_RECT.y)
    }

    pub fn tree_origin() -> (i32, i32) {
        (PAGE_RECT.x, PAGE_RECT.y)
    }

    fn tree_local(x: i32, y: i32) -> (i32, i32) {
        (x - PAGE_RECT.x, y - PAGE_RECT.y)
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<MessengerHit> {
        if !Rect::new(0, 0, SIZE.0, SIZE.1).contains(x, y) {
            return None;
        }
        match MESSENGER_FRAME.hit(&self.frame, (0, 0), (x, y)) {
            Some(FrameHit::Close) => return Some(MessengerHit::Close),
            Some(FrameHit::Minimize) => return Some(MessengerHit::Minimize),
            _ => {}
        }
        let (px, py) = Self::tree_local(x, y);
        if TREE_RECT.contains(px, py) {
            return Some(MessengerHit::Tree);
        }
        Some(MessengerHit::Page)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<MessengerHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        if hit == Some(MessengerHit::Tree) {
            let (px, py) = Self::tree_local(x, y);
            if !self.tree.bar_down(px, py) {
                self.tree.click(px, py);
            }
        }
        hit
    }

    pub fn mouse_move(&mut self, y: i32) {
        self.tree.bar_move(y - PAGE_RECT.y);
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<MessengerHit> {
        self.tree.bar_up();
        let pressed = self.pressed.take()?;
        (self.hit(x, y) == Some(pressed) && matches!(pressed, MessengerHit::Close | MessengerHit::Minimize)).then_some(pressed)
    }

    pub fn double_click(&self, x: i32, y: i32) -> Option<String> {
        let (px, py) = Self::tree_local(x, y);
        if !TREE_RECT.contains(px, py) || self.tree.bar_hit(px, py).is_some() {
            return None;
        }
        self.tree.double_click(px, py)
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        let (px, py) = Self::tree_local(x, y);
        if !TREE_RECT.contains(px, py) {
            return false;
        }
        self.tree.wheel(delta);
        true
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        let start = list.quads.len();
        let back = MSN_BACK_RECT;
        list.draw_texture(x + back.x, y + back.y, back.w, back.h, 0, 0, back.w, back.h, MSN_BACK, 1.0);
        list.clip_from(start, Rect::new(x, y, SIZE.0, SIZE.1));
        MESSENGER_FRAME.paint(&self.frame, list, origin, font, alpha);
        let tab = TAB_RECT;
        list.draw_texture(x + tab.x, y + tab.y, tab.w, tab.h, 0, 0, tab.w, tab.h, TAB_ACTIVE, 1.0);
        let text_width = font.measure(&self.tab_text) as i32;
        let (dx, dy) = (tab.w / 2 + TAB_TEXT_OFFSET.0 - (text_width >> 1), tab.h / 2 + TAB_TEXT_OFFSET.1 - (font.line_height as i32 >> 1));
        list.draw_text(x + tab.x + dx, y + tab.y + dy, TAB_TEXT, &self.tab_text, font);
        let page = PAGE_RECT;
        list.draw_texture(x + page.x, y + page.y, page.w, page.h, 0, 0, page.w, page.h, MSN_BACK_ZERO, 1.0);
        self.tree.paint(list, (x + page.x, y + page.y), font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    struct Fixed;

    impl Metrics for Fixed {
        fn text_width(&self, text: &[u16]) -> i32 {
            text.len() as i32 * 7
        }

        fn char_width(&self, _: u16) -> i32 {
            7
        }
    }

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn friends(online: usize, offline: usize) -> Vec<FriendRow> {
        let mut rows: Vec<FriendRow> =
            (0..online).map(|i| FriendRow { name: format!("on{i}"), online: true, object_id: 100 + i as i32 }).collect();
        rows.extend((0..offline).map(|i| FriendRow { name: format!("off{i}"), online: false, object_id: 200 + i as i32 }));
        rows
    }

    fn tree(online: usize, offline: usize) -> FriendTree {
        let mut tree = FriendTree::new(|id| Some(format!("s{id}")));
        tree.rebuild(friends(online, offline));
        tree
    }

    #[test]
    fn a_rebuild_shows_both_groups_collapsed_and_a_click_expands_the_clicked_group() {
        let mut tree = tree(2, 1);
        let (rows, total) = tree.layout();
        assert_eq!(rows.len(), 2);
        assert_eq!((rows[0].x, rows[0].y, rows[1].y, total), (3, 4, 23, 42));
        let row = tree.click(TREE_RECT.x + 5, TREE_RECT.y + 6).unwrap();
        assert_eq!(row.kind, RowKind::Group(0));
        assert!(tree.groups_expanded[0]);
        let (rows, total) = tree.layout();
        assert_eq!(rows.iter().map(|row| (row.x, row.y)).collect::<Vec<_>>(), vec![(3, 4), (21, 23), (21, 42), (3, 61)]);
        assert_eq!(total, 80);
        tree.update(friends(1, 1));
        assert!(tree.groups_expanded[0]);
        assert_eq!(tree.selected, None);
    }

    #[test]
    fn the_hit_line_is_220_by_14_and_starts_after_the_group_icon() {
        let tree = tree(1, 0);
        let row = tree.layout().0[0];
        assert_eq!(row.line_rect(), Rect::new(18, 4, 220, 14));
        assert_eq!(row.icon_rect(), Rect::new(3, 4, 15, 15));
        assert!(tree.row_at(TREE_RECT.x + 100, TREE_RECT.y + 10).is_some());
        assert!(tree.row_at(TREE_RECT.x + 100, TREE_RECT.y + 20).is_none());
    }

    #[test]
    fn only_online_leaves_open_a_conversation_on_double_click() {
        let mut tree = tree(1, 1);
        tree.click(TREE_RECT.x + 5, TREE_RECT.y + 6);
        tree.click(TREE_RECT.x + 5, TREE_RECT.y + 6 + 38);
        let (rows, _) = tree.layout();
        let leaf_on = rows.iter().find(|row| matches!(row.kind, RowKind::Leaf(0))).unwrap();
        let leaf_off = rows.iter().find(|row| matches!(row.kind, RowKind::Leaf(1))).unwrap();
        assert_eq!(tree.double_click(TREE_RECT.x + leaf_on.x + 2, TREE_RECT.y + leaf_on.y + 2).as_deref(), Some("on0"));
        assert_eq!(tree.double_click(TREE_RECT.x + leaf_off.x + 2, TREE_RECT.y + leaf_off.y + 2), None);
        assert_eq!(tree.double_click(TREE_RECT.x + 5, TREE_RECT.y + 6), None);
    }

    #[test]
    fn scrolling_moves_by_fifteen_with_buttons_forty_five_by_wheel_and_never_past_the_content() {
        let mut tree = tree(20, 0);
        tree.click(TREE_RECT.x + 5, TREE_RECT.y + 6);
        assert_eq!(tree.minimum(), -173);
        tree.scroll_down();
        assert_eq!(tree.offset, -15);
        tree.wheel(-WHEEL_DELTA);
        assert_eq!(tree.offset, -60);
        tree.wheel(WHEEL_DELTA * 5);
        assert_eq!(tree.offset, 0);
        tree.wheel(-WHEEL_DELTA * 100);
        assert_eq!(tree.offset, tree.minimum());
        tree.scroll_up();
        assert_eq!(tree.offset, tree.minimum() + 15);
    }

    #[test]
    fn long_names_are_cut_with_an_ellipsis_that_reserves_twelve_pixels() {
        assert_eq!(truncate_ellipsis("abc", 100, &Fixed), "abc");
        let cut = truncate_ellipsis(&"x".repeat(30), 100, &Fixed);
        assert_eq!(cut, format!("{}...", "x".repeat(12)));
        assert_eq!(name_filter("a\u{e9}b\r\n\u{1}c"), "ab\r\nc");
    }

    #[test]
    fn the_chat_box_wraps_indents_and_follows_the_end_once_fourteen_lines_are_full() {
        let mut chat = ChatBox::new();
        chat.add_fragment(&"y".repeat(40), ITEM_TEXT, true, &Fixed);
        assert!(chat.lines().len() >= 2);
        assert!(chat.lines().iter().all(|(text, _)| text.starts_with(CHAT_PREFIX)));
        let mut chat = ChatBox::new();
        for i in 0..14 {
            chat.add_fragment(&format!("l{i}"), ITEM_TEXT, false, &Fixed);
        }
        assert_eq!(chat.first, 0);
        chat.add_fragment("l14", ITEM_TEXT, false, &Fixed);
        assert_eq!(chat.first, 1);
        for i in 0..300 {
            chat.add_fragment(&format!("m{i}"), ITEM_TEXT, false, &Fixed);
        }
        assert_eq!(chat.lines().len(), CHAT_MAX_LINES);
        assert_eq!(chat.first, CHAT_MAX_LINES - CHAT_VISIBLE);
        chat.wheel(WHEEL_DELTA * 3);
        assert_eq!(chat.first, CHAT_MAX_LINES - CHAT_VISIBLE - 3);
    }

    #[test]
    fn a_message_is_two_lines_the_gray_sender_and_the_indented_text() {
        let mut chat = ChatBox::new();
        chat.add_line(Some("Ana"), "hi", ITEM_TEXT, &|name| format!("{name}:"), &Fixed);
        assert_eq!(chat.lines(), &[("Ana:".to_string(), SENDER_COLOR), ("   hi".to_string(), ITEM_TEXT)]);
        chat.add_line(None, "", ITEM_TEXT, &|name| name.to_string(), &Fixed);
        assert_eq!(chat.lines().len(), 2);
        chat.add_line(None, "x", FAILURE_COLOR, &|name| name.to_string(), &Fixed);
        assert_eq!(chat.lines()[2], ("x".to_string(), FAILURE_COLOR));
    }

    #[test]
    fn tasks_blink_each_second_while_pending_and_clearing_happens_on_click() {
        let mut bar = TaskBar::default();
        bar.add(WindowId(0x20000), "Ana", true, false);
        bar.add(WindowId(0x20000), "Ana", false, true);
        assert!(bar.tasks[0].pending);
        bar.add(WindowId(0x20000), "Ana", false, false);
        assert!(!bar.tasks[0].pending);
        bar.tasks[0].pending = true;
        bar.tick(1000);
        assert!(bar.tasks[0].phase);
        let (x, y) = (TASK_BOX_RECT.x + 3, TASK_BOX_RECT.y + 3);
        bar.mouse_down(x, y);
        assert_eq!(bar.mouse_up(x, y).as_deref(), Some("Ana"));
        assert!(!bar.tasks[0].pending);
        bar.remove("Ana");
        assert!(bar.tasks.is_empty());
    }

    #[test]
    fn the_task_bar_sits_at_the_bottom_right_and_chat_windows_open_at_a_random_spot() {
        assert_eq!(task_rect((1024, 768)), Rect::new(852, 354, 172, 132));
        assert_eq!(chat_position(0, 0), (200, 150));
        assert_eq!(chat_position(199, 149), (399, 299));
        assert_eq!(chat_position(200, 150), (200, 150));
    }

    #[test]
    fn the_messenger_paints_the_clipped_back_the_active_tab_and_the_tree() {
        let mut view = MessengerView::new(|id| Some(format!("s{id}")));
        view.tree.rebuild(friends(1, 0));
        let mut list = DrawList::default();
        view.paint(&mut list, (0, 0), &font(), 1.0);
        let back = list.quads.iter().find(|q| q.texture == MSN_BACK).unwrap();
        assert_eq!((back.x, back.y, back.w, back.h), (0.0, 20.0, 256.0, 28.0));
        assert!(list.quads.iter().any(|q| q.texture == TAB_ACTIVE && q.x == 12.0 && q.y == 25.0));
        assert!(list.quads.iter().any(|q| q.texture == MSN_BACK_ZERO && q.y == 48.0));
        assert!(list.quads.iter().any(|q| q.texture == PLUS && q.x == 11.0 && q.y == 48.0 + 45.0 + 4.0));
    }
}
