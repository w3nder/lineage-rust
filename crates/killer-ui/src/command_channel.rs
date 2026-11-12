#[path = "command_channel_controls_generated.rs"]
mod controls;
#[path = "command_channel_layout_generated.rs"]
mod layout;

use crate::button::TextButton;
use crate::child_contract::{ChildContract, ChildKind};
use crate::color::{BUTTON_TEXT, GOLD, TEXT, chat_color};
use crate::draw::DrawList;
use crate::edit_box::{EditBox, EditInput, FontMetrics, NOTICE_ENTER};
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::initial_position::InitialPosition;
use crate::list_ctrl::{ColumnContract, ListCell, ListContract, ListCtrl, ListRow, Pressed, TAB_TEXTURES};
use crate::text_list::TextList;
use crate::window::{Rect, WindowId};

pub const SIZE: (i32, i32) = layout::SIZE;
pub const DEFAULT_POSITION: (i32, i32) = (151, 182);
pub const INITIAL_POSITION: InitialPosition = InitialPosition { enabled: true, horizontal_mode: 0, vertical_mode: 3, x: 200, y: 200 };
pub const WINDOW_ID: WindowId = WindowId(75);
pub const PAGE_X: i32 = 12;
pub const BACK: &str = "L2UI_ch3.PartyCommandWnd.Herochat_back";
pub const CHECK_NORMAL: &str = controls::CHILD_FC.texture_str(0x274);
pub const CHECK_CHECKED: &str = controls::CHILD_FC.texture_str(0x278);
pub const TAB_NORMAL: &str = "L2UI_CH3.PartyCommandWnd.herochat_tab2";
pub const TAB_ACTIVE: &str = "L2UI_CH3.PartyCommandWnd.herochat_tab1";
pub const BUTTON_NORMAL: &str = "L2UI_CH3.Button.smallbutton2";
pub const BUTTON_DOWN: &str = "L2UI_CH3.Button.smallbutton2_down";
pub const TAB_RECTS: [Rect; 2] = [Rect::new(24, 141, 64, 23), Rect::new(88, 141, 64, 23)];
pub const INPUT_RECT: Rect = layout::RECT_F8;
pub const CHECK_RECT: Rect = layout::RECT_FC;
pub const CHAT_ID: u32 = 0xf0a0c1;
pub const CHAT_RECT: Rect = Rect::new(PAGE_X + 6, 5, 327, 134);
pub const CHAT_LINES: usize = 200;
pub const CHAT_VISIBLE: i32 = 8;
pub const PAGE_RECT: Rect = Rect::new(PAGE_X, 0, 336, 140);
pub const LIST_RECT: Rect = Rect::new(PAGE_X + 24, 25, 207, 101);
pub const LIST_COLUMNS: [i32; 2] = [160, 47];
pub const HEADER_HEIGHT: i32 = 19;
pub const LIST_SCROLL_RECT: Rect = Rect::new(LIST_RECT.x, LIST_RECT.y + HEADER_HEIGHT, LIST_RECT.w, LIST_RECT.h - HEADER_HEIGHT);
pub const COLUMN_SYSSTRINGS: [i32; 2] = [408, 1036];
pub const BUTTON_RECTS: [Rect; 3] = [
    Rect::new(PAGE_X + 244, 62, 65, 20),
    Rect::new(PAGE_X + 244, 86, 65, 20),
    Rect::new(PAGE_X + 244, 110, 65, 20),
];
pub const OWNER_LABEL_POS: (i32, i32) = (PAGE_X + 22, 9);
pub const PARTIES_LABEL_POS: (i32, i32) = (PAGE_X + 243, 29);
pub const MEMBERS_LABEL_POS: (i32, i32) = (PAGE_X + 243, 44);
pub const VALUE_GAP: i32 = 5;
pub const SYSSTRINGS: [u32; 10] = [444, 491, 938, 681, 1268, 1265, 1266, 1267, 1227, 1228];
pub const TOO_LONG_SYSMSG: u32 = 971;
pub const SAY_ALL_MEMBERS: i32 = 15;
pub const SAY_LEADER: i32 = 16;
pub const SAY_MAX_CHARS: usize = 255;
pub const USER_COMMAND_REFRESH: i32 = 0x61;
pub const USER_COMMAND_QUIT: i32 = 0x60;
pub const REFRESH_TIMER_MS: u32 = 3000;
pub const ROW_HEIGHT: i32 = 17;

pub const EDIT: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: INPUT_RECT,
    id: 0xf0a0c2,
    vtable: 0x101a_2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x214, 0x1), (0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0xa28), (0x2b8, 0x1)],
    textures: &[],
};

pub const LIST_CHILD: ChildContract = ChildContract {
    kind: ChildKind::ListCtrl,
    rect: LIST_RECT,
    id: 0x0,
    vtable: 0x101a6fa8,
    registration: (0x22, 0x0, 0x0),
    handles: &[0x230, 0x234, 0x238],
    state: &[(0x23c, 0x5), (0x240, 0x3e8), (0x244, 0x2), (0x250, 0xffffffff), (0x254, 0xffffffff), (0x258, 0xf), (0x25c, 0x13), (0x264, 0x3), (0x268, 0x1), (0x26c, 0x11)],
    textures: &[],
};

pub const LIST_COLUMN_CONTRACTS: &[ColumnContract] = &[
    ColumnContract { width: LIST_COLUMNS[0], height: HEADER_HEIGHT, sysstring: COLUMN_SYSSTRINGS[0], flag_c: 1, flag_10: 0, textures: TAB_TEXTURES },
    ColumnContract { width: LIST_COLUMNS[1], height: HEADER_HEIGHT, sysstring: COLUMN_SYSSTRINGS[1], flag_c: 1, flag_10: 1, textures: TAB_TEXTURES },
];

pub const LIST: ListContract = ListContract { child: LIST_CHILD, columns: LIST_COLUMN_CONTRACTS };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    Tab(usize),
    Check,
    Input,
    Header(usize),
    Row(usize),
    Scroll,
    Chat,
    Refresh,
    Dismiss,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Say { kind: i32, text: String },
    UserCommand(i32),
    OustFromChannel(String),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Labels {
    pub tabs: [String; 2],
    pub buttons: [String; 3],
    pub owner: String,
    pub parties: String,
    pub members: String,
    pub all_members: String,
    pub leader: String,
    pub columns: [String; 2],
}

impl Labels {
    pub fn from_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> Labels {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Labels {
            tabs: [text(SYSSTRINGS[0]), text(SYSSTRINGS[1])],
            buttons: [text(SYSSTRINGS[2]), text(SYSSTRINGS[3]), text(SYSSTRINGS[4])],
            owner: text(SYSSTRINGS[5]),
            parties: text(SYSSTRINGS[6]),
            members: text(SYSSTRINGS[7]),
            all_members: text(SYSSTRINGS[8]),
            leader: text(SYSSTRINGS[9]),
            columns: COLUMN_SYSSTRINGS.map(|id| text(id as u32)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChannelInfo {
    pub owner: String,
    pub members: i32,
    pub parties: i32,
    pub rows: Vec<(String, i32)>,
}

#[derive(Debug, Clone)]
pub struct CommandChannelView {
    pub id: WindowId,
    pub tab: usize,
    pub labels: Labels,
    pub chat: TextList,
    pub edit: EditBox,
    pub all_members: bool,
    pub owner: String,
    pub parties: i32,
    pub members: i32,
    pub list: ListCtrl,
    pub refresh: TextButton,
    pub dismiss: TextButton,
    pub quit: TextButton,
    pub can_refresh: bool,
    pub timer_seconds: f32,
    pub chat_visible: bool,
    pub info_visible: bool,
    pub too_long: String,
    pub pressed: Option<Hit>,
}

impl CommandChannelView {
    pub fn new(labels: Labels) -> Self {
        let mut view = CommandChannelView {
            id: WINDOW_ID,
            tab: 0,
            labels: Labels::default(),
            chat: TextList::new(CHAT_ID, CHAT_RECT, CHAT_LINES, CHAT_VISIBLE, true),
            edit: EditBox::from_contract(&EDIT),
            all_members: false,
            owner: String::new(),
            parties: 0,
            members: 0,
            list: ListCtrl::from_contract(&LIST),
            refresh: TextButton::new(BUTTON_RECTS[0], BUTTON_NORMAL, BUTTON_DOWN, ""),
            dismiss: TextButton::new(BUTTON_RECTS[1], BUTTON_NORMAL, BUTTON_DOWN, ""),
            quit: TextButton::new(BUTTON_RECTS[2], BUTTON_NORMAL, BUTTON_DOWN, ""),
            can_refresh: true,
            timer_seconds: 0.0,
            chat_visible: true,
            info_visible: false,
            too_long: String::new(),
            pressed: None,
        };
        view.set_labels(labels);
        view
    }

    pub fn set_labels(&mut self, labels: Labels) {
        self.refresh.label = labels.buttons[0].clone();
        self.dismiss.label = labels.buttons[1].clone();
        self.quit.label = labels.buttons[2].clone();
        self.labels = labels;
    }

    pub fn grip() -> Rect {
        FrameCtrl::new(FrameMode::VerticalGrip).grip(SIZE.0, SIZE.1)
    }

    pub fn show(&mut self) {
        self.chat_visible = true;
        self.info_visible = self.tab == 1;
    }

    pub fn reset_info(&mut self) {
        self.owner.clear();
        self.parties = 0;
        self.members = 0;
        self.list.clear();
        self.can_refresh = true;
        self.refresh.enabled = true;
    }

    pub fn set_info(&mut self, info: &ChannelInfo) {
        self.reset_info();
        self.owner = info.owner.clone();
        self.members = info.members;
        self.parties = info.parties;
        for (name, count) in &info.rows {
            self.list.add_item(ListRow { cells: vec![ListCell::text(name), ListCell::text(&count.to_string())] });
        }
    }

    pub fn row_name(&self, index: usize) -> Option<&str> {
        self.list.rows.get(index).and_then(|row| row.cells.first()).map(|cell| cell.text.as_str())
    }

    pub fn row_count(&self) -> usize {
        self.list.rows.len()
    }

    pub fn selected(&self) -> Option<usize> {
        self.list.selected()
    }

    pub fn receive_chat(&mut self, kind: i32, line: &str, font: &BitmapFont) {
        self.chat.add_string(line, chat_color(kind), &FontMetrics(font));
    }

    pub fn close(&mut self) {
        self.edit.clear();
        self.chat.clear();
    }

    pub fn tick(&mut self, delta_ms: u32) {
        self.edit.tick(delta_ms);
        let period = REFRESH_TIMER_MS as f32 * 0.001;
        self.timer_seconds += delta_ms as f32 * 0.001;
        if period < self.timer_seconds {
            self.timer_seconds -= period;
            if !self.can_refresh {
                self.can_refresh = true;
                self.refresh.enabled = true;
            }
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<Hit> {
        for (index, rect) in TAB_RECTS.iter().enumerate() {
            if rect.contains(x, y) {
                return Some(Hit::Tab(index));
            }
        }
        if self.info_visible && PAGE_RECT.contains(x, y) {
            return self.info_hit(x, y);
        }
        if self.chat_visible && CHAT_RECT.contains(x, y) {
            return Some(Hit::Chat);
        }
        if INPUT_RECT.contains(x, y) {
            return Some(Hit::Input);
        }
        if CHECK_RECT.contains(x, y) {
            return Some(Hit::Check);
        }
        None
    }

    fn info_hit(&self, x: i32, y: i32) -> Option<Hit> {
        for (button, hit) in [(&self.refresh, Hit::Refresh), (&self.dismiss, Hit::Dismiss), (&self.quit, Hit::Quit)] {
            if button.contains(x, y) {
                return Some(hit);
            }
        }
        if LIST_RECT.contains(x, y) {
            match self.list.child_at(x, y) {
                Some(Pressed::Header(column)) => return Some(Hit::Header(column)),
                Some(Pressed::Up | Pressed::Down | Pressed::Slider) => return Some(Hit::Scroll),
                None => {}
            }
            let row = self.list.hit(x, y);
            if row >= 0 {
                return Some(Hit::Row(row as usize));
            }
        }
        None
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, font: &BitmapFont) {
        self.pressed = self.hit(x, y);
        let focus_edit = matches!(self.pressed, Some(Hit::Input));
        if !focus_edit && self.edit.focused {
            self.edit.kill_focus();
        }
        match self.pressed {
            Some(Hit::Input) => {
                if !self.edit.focused {
                    self.edit.focus();
                }
                let metrics = FontMetrics(font);
                self.edit.mouse_down(x, &metrics);
                self.edit.layout(&metrics);
            }
            Some(Hit::Check) => self.all_members = !self.all_members,
            Some(Hit::Row(_) | Hit::Header(_) | Hit::Scroll) => {
                self.list.mouse_down(x, y);
            }
            Some(Hit::Chat) => {
                self.chat.mouse_down(x, y);
            }
            _ => {}
        }
    }

    pub fn mouse_move(&mut self, x: i32, y: i32, font: &BitmapFont) {
        if self.edit.focused {
            let metrics = FontMetrics(font);
            self.edit.mouse_move(x, &metrics);
            self.edit.layout(&metrics);
        }
        self.chat.mouse_move(y);
        if self.info_visible || self.list.pressed.is_some() {
            self.list.mouse_move(x, y);
        }
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        if self.info_visible && PAGE_RECT.contains(x, y) {
            if LIST_RECT.contains(x, y) {
                self.list.wheel(delta as i16);
            }
            return true;
        }
        if self.chat_visible && CHAT_RECT.contains(x, y) {
            self.chat.wheel(delta);
            return true;
        }
        false
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<Action> {
        self.edit.mouse_up();
        self.chat.mouse_up();
        let pressed = self.pressed.take();
        if self.list.left_held {
            self.list.mouse_up(x, y);
        }
        if pressed == Some(Hit::Scroll) {
            return None;
        }
        let pressed = pressed?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            Hit::Tab(index) => {
                let action = self.select_tab(index);
                self.tab = index;
                action
            }
            Hit::Refresh => self.request_refresh(),
            Hit::Dismiss => {
                let name = self.row_name(self.list.selected()?)?.to_string();
                Some(Action::OustFromChannel(name))
            }
            Hit::Quit => Some(Action::UserCommand(USER_COMMAND_QUIT)),
            Hit::Check | Hit::Input | Hit::Header(_) | Hit::Row(_) | Hit::Scroll | Hit::Chat => None,
        }
    }

    pub fn key(&mut self, key: crate::chat::InputKey, font: &BitmapFont) -> (bool, Option<Action>) {
        if !self.edit.focused {
            return (false, None);
        }
        let metrics = FontMetrics(font);
        if self.edit.input_key(key, &metrics) == EditInput::Escape {
            self.edit.kill_focus();
            return (true, None);
        }
        let entered = self.edit.notices.drain(..).any(|notice| notice.code == NOTICE_ENTER);
        if !entered {
            return (true, None);
        }
        let Some(text) = self.edit.get_text() else { return (true, None) };
        let action = if text.encode_utf16().count() >= SAY_MAX_CHARS {
            let line = self.too_long.clone();
            self.chat.add_string(&line, TEXT, &metrics);
            None
        } else {
            let kind = if self.all_members { SAY_ALL_MEMBERS } else { SAY_LEADER };
            Some(Action::Say { kind, text })
        };
        self.edit.clear();
        (true, action)
    }

    pub fn check_hint(&self) -> &str {
        if self.all_members { &self.labels.all_members } else { &self.labels.leader }
    }

    fn select_tab(&mut self, index: usize) -> Option<Action> {
        match index {
            0 => {
                self.info_visible = false;
                self.chat_visible = true;
                None
            }
            _ => {
                self.info_visible = true;
                self.chat_visible = false;
                self.request_refresh()
            }
        }
    }

    fn request_refresh(&mut self) -> Option<Action> {
        if !self.can_refresh {
            return None;
        }
        self.can_refresh = false;
        self.refresh.enabled = false;
        Some(Action::UserCommand(USER_COMMAND_REFRESH))
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        FrameCtrl::new(FrameMode::VerticalGrip).paint(list, x, y, SIZE.0, SIZE.1, font, alpha);
        let back_w = SIZE.0 - PAGE_X;
        list.draw_texture(x + PAGE_X, y, back_w, SIZE.1, 0, 0, back_w, SIZE.1, BACK, alpha);
        if self.chat_visible {
            self.chat.paint(list, (x, y), font);
        }
        if self.info_visible {
            self.paint_info(list, x, y, font);
        }
        for (index, rect) in TAB_RECTS.iter().enumerate() {
            let texture = if self.tab == index { TAB_ACTIVE } else { TAB_NORMAL };
            list.draw_texture(x + rect.x, y + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, texture, 1.0);
            let label = &self.labels.tabs[index];
            let tx = rect.x + rect.w / 2 - font.measure(label) as i32 / 2;
            let ty = rect.y + rect.h / 2 - font.line_height as i32 / 2;
            list.draw_text(x + tx, y + ty, BUTTON_TEXT, label, font);
        }
        let check_texture = if self.all_members { CHECK_CHECKED } else { CHECK_NORMAL };
        list.draw_texture(x + CHECK_RECT.x, y + CHECK_RECT.y, CHECK_RECT.w, CHECK_RECT.h, 0, 0, CHECK_RECT.w, CHECK_RECT.h, check_texture, 1.0);
        self.edit.paint(list, (x, y), &FontMetrics(font), font);
    }

    fn paint_info(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let owner_end = list.draw_text(x + OWNER_LABEL_POS.0, y + OWNER_LABEL_POS.1, TEXT, &self.labels.owner, font);
        list.draw_text(owner_end + VALUE_GAP, y + OWNER_LABEL_POS.1, GOLD, &self.owner, font);
        let parties_end = list.draw_text(x + PARTIES_LABEL_POS.0, y + PARTIES_LABEL_POS.1, TEXT, &self.labels.parties, font);
        list.draw_text(parties_end + VALUE_GAP, y + PARTIES_LABEL_POS.1, GOLD, &self.parties.to_string(), font);
        let members_end = list.draw_text(x + MEMBERS_LABEL_POS.0, y + MEMBERS_LABEL_POS.1, TEXT, &self.labels.members, font);
        list.draw_text(members_end + VALUE_GAP, y + MEMBERS_LABEL_POS.1, GOLD, &self.members.to_string(), font);
        let clip_start = list.quads.len();
        self.list.paint_header(list, (x, y), font, &|id| {
            COLUMN_SYSSTRINGS.iter().position(|column| *column == id).map(|index| self.labels.columns[index].clone())
        });
        self.list.paint(list, (x, y), font);
        list.clip_from(clip_start, Rect::new(x + PAGE_RECT.x, y + PAGE_RECT.y, PAGE_RECT.w, PAGE_RECT.h));
        for (button, hit) in [(&self.refresh, Hit::Refresh), (&self.dismiss, Hit::Dismiss), (&self.quit, Hit::Quit)] {
            button.paint(list, x, y, self.pressed == Some(hit), font);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::InputKey;
    use crate::font::{Glyphs, gly};

    #[test]
    fn check_box_comes_from_the_native_child_contract() {
        assert_eq!(controls::CHILD_FC.kind, ChildKind::CheckBox);
        assert_eq!(controls::CHILD_FC.rect, CHECK_RECT);
        assert_eq!((controls::CHILD_FC.field(0x2a4), controls::CHILD_FC.field(0x2a8)), (CHECK_RECT.w as u32, CHECK_RECT.h as u32));
        assert_eq!(CHECK_NORMAL, "L2UI_ch3.PartyCommandWnd.Herochat_btn1");
        assert_eq!(CHECK_CHECKED, "L2UI_ch3.PartyCommandWnd.Herochat_btn2");
    }

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn labels() -> Labels {
        Labels::from_sysstrings(|id| Some(format!("s{id}")))
    }

    fn info() -> ChannelInfo {
        ChannelInfo { owner: "Chefe".into(), members: 11, parties: 2, rows: vec![("Ana".into(), 9), ("Bia".into(), 2)] }
    }

    fn info_view() -> CommandChannelView {
        let mut view = CommandChannelView::new(labels());
        view.tab = 1;
        view.show();
        view.set_info(&info());
        view
    }

    #[test]
    fn on_paint_10051660_moves_labels_and_values_together() {
        let view = info_view();
        let mut origin = DrawList::default();
        let mut moved = DrawList::default();
        view.paint(&mut origin, 0, 0, &font(), 1.0);
        view.paint(&mut moved, 391, 127, &font(), 1.0);
        assert!(!origin.quads.is_empty());
        assert_eq!(origin.quads.len(), moved.quads.len());
        for (mut expected, actual) in origin.quads.into_iter().zip(moved.quads) {
            expected.x += 391.0;
            expected.y += 127.0;
            assert_eq!(expected, actual);
        }
    }

    fn click(view: &mut CommandChannelView, x: i32, y: i32) -> Option<Action> {
        view.mouse_down(x, y, &font());
        view.mouse_up(x, y)
    }

    fn type_text(view: &mut CommandChannelView, text: &str) {
        for c in text.chars() {
            assert_eq!(view.key(InputKey::Char(c), &font()), (true, None));
        }
    }

    #[test]
    fn layout_follows_the_oncreate_of_10052480_with_the_page_origin_at_x_12() {
        assert_eq!(SIZE, (348, 187));
        assert_eq!(TAB_RECTS, [Rect::new(24, 141, 64, 23), Rect::new(88, 141, 64, 23)]);
        assert_eq!(INPUT_RECT, Rect::new(35, 165, 308, 17));
        assert_eq!(CHECK_RECT, Rect::new(18, 166, 15, 15));
        assert_eq!(CHAT_RECT, Rect::new(18, 5, 327, 134));
        assert_eq!(LIST_RECT, Rect::new(36, 25, 207, 101));
        assert_eq!(BUTTON_RECTS, [Rect::new(256, 62, 65, 20), Rect::new(256, 86, 65, 20), Rect::new(256, 110, 65, 20)]);
        assert_eq!(LIST_COLUMNS, [160, 47]);
        assert_eq!(CommandChannelView::grip(), Rect::new(0, 0, 12, 187));
        assert_eq!(INITIAL_POSITION, InitialPosition { enabled: true, horizontal_mode: 0, vertical_mode: 3, x: 200, y: 200 });
    }

    #[test]
    fn the_list_columns_follow_the_two_addcolumn_calls_of_10051ca0() {
        let list = ListCtrl::from_contract(&LIST);
        assert_eq!((list.rows_visible, list.capacity, list.max_columns, list.header_height, list.row_height), (5, 1000, 2, 19, 17));
        assert_eq!(list.mode, (0, 3));
        let columns: Vec<_> = list.columns.iter().map(|c| (c.width, c.height, c.sysstring, c.flag_c, c.flag_10)).collect();
        assert_eq!(columns, [(160, 19, 408, 1, 0), (47, 19, 1036, 1, 1)]);
        assert_eq!(list.columns[0].textures.len(), 6);
    }

    #[test]
    fn the_info_packet_puts_the_second_d_in_parties_and_the_first_in_members() {
        let mut view = CommandChannelView::new(labels());
        view.set_info(&info());
        assert_eq!((view.owner.as_str(), view.members, view.parties), ("Chefe", 11, 2));
        assert_eq!(view.row_count(), 2);
        assert_eq!(view.list.rows[1].cells[1].text, "2");
        assert!(view.can_refresh && view.refresh.enabled);
    }

    #[test]
    fn enter_sends_type_16_unchecked_and_15_checked_and_clears_the_box() {
        let mut view = CommandChannelView::new(labels());
        view.mouse_down(INPUT_RECT.x + 1, INPUT_RECT.y + 1, &font());
        type_text(&mut view, "oi");
        assert_eq!(view.key(InputKey::Enter, &font()), (true, Some(Action::Say { kind: SAY_LEADER, text: "oi".into() })));
        assert_eq!(view.edit.get_text(), None);
        click(&mut view, CHECK_RECT.x + 1, CHECK_RECT.y + 1);
        assert!(view.all_members);
        view.mouse_down(INPUT_RECT.x + 1, INPUT_RECT.y + 1, &font());
        type_text(&mut view, "a");
        assert_eq!(view.key(InputKey::Enter, &font()), (true, Some(Action::Say { kind: SAY_ALL_MEMBERS, text: "a".into() })));
        assert_eq!(view.check_hint(), "s1227");
    }

    #[test]
    fn enter_with_255_characters_or_more_adds_the_systemmsg_971_line_and_clears_the_box() {
        let mut view = CommandChannelView::new(labels());
        view.too_long = "muito longo".into();
        view.mouse_down(INPUT_RECT.x + 1, INPUT_RECT.y + 1, &font());
        assert_eq!(view.key(InputKey::Enter, &font()), (true, None));
        assert!(view.chat.lines().is_empty());
        view.edit.set_text(&"x".repeat(254));
        assert!(matches!(view.key(InputKey::Enter, &font()), (true, Some(Action::Say { .. }))));
        view.edit.set_text(&"x".repeat(255));
        assert_eq!(view.key(InputKey::Enter, &font()), (true, None));
        assert_eq!(view.edit.get_text(), None);
        assert_eq!(view.chat.lines()[0].text, "muito longo");
        assert_eq!(view.chat.lines()[0].color, TEXT);
    }

    #[test]
    fn keys_are_ignored_while_the_box_is_not_focused_and_escape_drops_the_focus() {
        let mut view = CommandChannelView::new(labels());
        assert_eq!(view.key(InputKey::Char('a'), &font()), (false, None));
        assert_eq!(view.edit.get_text(), None);
        view.mouse_down(INPUT_RECT.x + 1, INPUT_RECT.y + 1, &font());
        assert!(view.edit.focused);
        assert_eq!(view.key(InputKey::Escape, &font()), (true, None));
        assert!(!view.edit.focused);
    }

    #[test]
    fn selecting_the_information_tab_asks_for_a_refresh_once_until_the_info_or_the_timer_returns() {
        let mut view = CommandChannelView::new(labels());
        assert_eq!(click(&mut view, TAB_RECTS[1].x + 1, TAB_RECTS[1].y + 1), Some(Action::UserCommand(USER_COMMAND_REFRESH)));
        assert_eq!(view.tab, 1);
        assert!(!view.can_refresh && !view.refresh.enabled);
        assert_eq!(click(&mut view, BUTTON_RECTS[0].x + 1, BUTTON_RECTS[0].y + 1), None);
        view.set_info(&info());
        assert_eq!(click(&mut view, BUTTON_RECTS[0].x + 1, BUTTON_RECTS[0].y + 1), Some(Action::UserCommand(USER_COMMAND_REFRESH)));
        view.tick(REFRESH_TIMER_MS - 1);
        assert!(!view.can_refresh);
        view.tick(1);
        assert!(!view.can_refresh);
        view.tick(1);
        assert!(view.can_refresh && view.refresh.enabled);
        view.can_refresh = false;
        view.refresh.enabled = false;
        view.tick(REFRESH_TIMER_MS + 1);
        assert!(view.can_refresh && view.refresh.enabled);
    }

    #[test]
    fn the_timer_is_periodic_and_keeps_the_remainder_like_1007fa90() {
        let mut view = CommandChannelView::new(labels());
        view.tick(3500);
        assert!(view.timer_seconds > 0.49 && view.timer_seconds < 0.51);
        view.can_refresh = false;
        view.tick(2400);
        assert!(!view.can_refresh);
        view.tick(200);
        assert!(view.can_refresh);
    }

    #[test]
    fn dismiss_needs_a_selected_row_and_quit_sends_command_0x60() {
        let mut view = info_view();
        assert_eq!(click(&mut view, BUTTON_RECTS[1].x + 1, BUTTON_RECTS[1].y + 1), None);
        click(&mut view, LIST_RECT.x + 1, LIST_RECT.y + HEADER_HEIGHT + ROW_HEIGHT + 1);
        assert_eq!(view.selected(), Some(1));
        assert_eq!(click(&mut view, BUTTON_RECTS[1].x + 1, BUTTON_RECTS[1].y + 1), Some(Action::OustFromChannel("Bia".into())));
        assert_eq!(click(&mut view, BUTTON_RECTS[2].x + 1, BUTTON_RECTS[2].y + 1), Some(Action::UserCommand(USER_COMMAND_QUIT)));
    }

    #[test]
    fn information_controls_do_not_answer_on_the_chat_tab() {
        let mut view = CommandChannelView::new(labels());
        view.set_info(&info());
        assert_eq!(view.hit(BUTTON_RECTS[2].x + 1, BUTTON_RECTS[2].y + 1), Some(Hit::Chat));
        assert_eq!(view.hit(LIST_RECT.x + 1, LIST_RECT.y + HEADER_HEIGHT + 1), Some(Hit::Chat));
    }

    #[test]
    fn show_with_the_information_tab_current_leaves_both_pages_visible_like_1002e380() {
        let mut view = CommandChannelView::new(labels());
        click(&mut view, TAB_RECTS[1].x + 1, TAB_RECTS[1].y + 1);
        assert_eq!((view.info_visible, view.chat_visible), (true, false));
        view.show();
        assert_eq!((view.info_visible, view.chat_visible), (true, true));
        click(&mut view, TAB_RECTS[0].x + 1, TAB_RECTS[0].y + 1);
        assert_eq!((view.info_visible, view.chat_visible), (false, true));
        view.show();
        assert_eq!((view.info_visible, view.chat_visible), (false, true));
        view.tab = 1;
        view.show();
        view.set_info(&info());
        let mut list = DrawList { capture_text: true, ..Default::default() };
        view.receive_chat(15, "Ana: oi", &font());
        view.paint(&mut list, 0, 0, &font(), 1.0);
        assert!(list.text_runs.iter().any(|t| t.text == "Ana: oi"));
        assert!(list.text_runs.iter().any(|t| t.text == "Chefe"));
    }

    #[test]
    fn closing_clears_the_chat_and_the_box_and_chat_lines_use_the_type_color() {
        let mut view = CommandChannelView::new(labels());
        view.receive_chat(15, "Ana: oi", &font());
        view.receive_chat(16, "Bia: ola", &font());
        let lines: Vec<_> = view.chat.lines().iter().map(|line| (line.text.as_str(), line.color)).collect();
        assert_eq!(lines, [("Ana: oi", chat_color(15)), ("Bia: ola", chat_color(16))]);
        view.edit.set_text("x");
        view.close();
        assert!(view.chat.lines().is_empty() && view.edit.get_text().is_none());
    }

    #[test]
    fn long_chat_lines_wrap_and_the_wheel_scrolls_the_chat_list() {
        let mut view = CommandChannelView::new(labels());
        view.receive_chat(15, &"palavra ".repeat(80), &font());
        assert!(view.chat.lines().len() > 1);
        for index in 0..20 {
            view.receive_chat(16, &format!("linha {index}"), &font());
        }
        let before = view.chat.first();
        assert!(view.wheel(CHAT_RECT.x + 20, CHAT_RECT.y + 20, 120));
        assert!(view.chat.first() <= before);
    }

    #[test]
    fn header_click_sorts_the_info_list_toggling_the_direction_and_following_the_selection() {
        let mut view = info_view();
        view.set_info(&ChannelInfo {
            owner: "Chefe".into(),
            members: 20,
            parties: 3,
            rows: vec![("bia".into(), 10), ("Ana".into(), 9), ("carl".into(), 100)],
        });
        view.list.select(2);
        let name_header = (LIST_RECT.x + 5, LIST_RECT.y + 5);
        let count_header = (LIST_RECT.x + LIST_COLUMNS[0] + 5, LIST_RECT.y + 5);
        assert_eq!(view.hit(name_header.0, name_header.1), Some(Hit::Header(0)));
        assert_eq!(click(&mut view, name_header.0, name_header.1), None);
        let names = |view: &CommandChannelView| (0..3).map(|i| view.row_name(i).unwrap().to_string()).collect::<Vec<_>>();
        assert_eq!(names(&view), ["Ana", "bia", "carl"]);
        assert_eq!(view.selected(), Some(2));
        click(&mut view, name_header.0, name_header.1);
        assert_eq!(names(&view), ["carl", "bia", "Ana"]);
        assert_eq!(view.selected(), Some(0));
        click(&mut view, count_header.0, count_header.1);
        assert_eq!(names(&view), ["Ana", "bia", "carl"]);
        click(&mut view, count_header.0, count_header.1);
        assert_eq!(names(&view), ["carl", "bia", "Ana"]);
    }

    #[test]
    fn the_scrollbar_buttons_page_the_list_and_the_thumb_shows_only_above_five_rows() {
        let mut view = info_view();
        assert!(!view.list.slider.shown);
        view.set_info(&ChannelInfo { owner: "o".into(), members: 1, parties: 1, rows: (0..9).map(|i| (format!("p{i}"), i)).collect() });
        assert!(view.list.slider.shown);
        let down = (LIST_SCROLL_RECT.x + LIST_SCROLL_RECT.w - 8, LIST_SCROLL_RECT.y + LIST_SCROLL_RECT.h - 8);
        assert_eq!(view.hit(down.0, down.1), Some(Hit::Scroll));
        click(&mut view, down.0, down.1);
        assert_eq!(view.list.first_row(), 1);
        view.wheel(LIST_RECT.x + 5, LIST_RECT.y + 40, -120 * 10);
        assert_eq!(view.list.first_row(), 4);
        let up = (LIST_SCROLL_RECT.x + LIST_SCROLL_RECT.w - 8, LIST_SCROLL_RECT.y + 8);
        click(&mut view, up.0, up.1);
        assert_eq!(view.list.first_row(), 3);
    }

    #[test]
    fn painting_the_info_tab_places_the_values_after_the_labels_and_draws_the_header() {
        let view = info_view();
        let mut list = DrawList { capture_text: true, ..Default::default() };
        view.paint(&mut list, 100, 50, &font(), 1.0);
        assert!(!list.quads.is_empty());
        for text in ["s1265", "Chefe", "s408", "s1036", "Ana", "9"] {
            assert!(list.text_runs.iter().any(|t| t.text == text), "{text}");
        }
        assert!(list.quads.iter().any(|q| q.texture == "L2UI_CH3.ListCtrl.tab1"));
        let mut chat = DrawList::default();
        let mut idle = view.clone();
        idle.tab = 0;
        idle.info_visible = false;
        idle.paint(&mut chat, 100, 50, &font(), 1.0);
        assert!(chat.quads.len() < list.quads.len());
    }
}
