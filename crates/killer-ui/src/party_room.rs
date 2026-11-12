use crate::button::TextButton;
use crate::chat::{InputKey, TextInput};
use crate::color::TEXT;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::list_ctrl::{BigIcon, ColumnContract, ListCell, ListContract, ListCtrl, ListRow, Pressed, TAB_TEXTURES, wtoi};
use crate::window::{Rect, WindowId};

pub const MAKING_SIZE: (i32, i32) = (356, 141);
pub const MAKING_INITIAL_POSITION: crate::initial_position::InitialPosition = crate::initial_position::InitialPosition {
    enabled: true, horizontal_mode: 2, vertical_mode: 2, x: 356, y: 141,
};

pub fn making_default_position(viewport: (i32, i32)) -> (i32, i32) {
    (
        crate::window_restore::x87_to_int(f64::from(viewport.0 as f32) * 0.5 - 178.0),
        crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) * 0.5 - 70.0),
    )
}
pub const MAKING_BACK: &str = "L2UI_CH3.PartyMatchWnd.PartyMatch3_Back";
pub const BUTTON_NORMAL: &str = "L2UI_CH3.Button.Btn1_normal";
pub const BUTTON_DOWN: &str = "L2UI_CH3.Button.Btn1_normalOn";
pub const TITLE_RECT: Rect = Rect::new(103, 17, 236, 17);
pub const MEMBERS_RECT: Rect = Rect::new(103, 37, 40, 17);
pub const MIN_LEVEL_RECT: Rect = Rect::new(131, 57, 26, 17);
pub const MAX_LEVEL_RECT: Rect = Rect::new(201, 57, 26, 17);
pub const OK_RECT: Rect = Rect::new(101, 110, 76, 23);
pub const CANCEL_RECT: Rect = Rect::new(181, 110, 76, 23);
pub const TITLE_MAX_LENGTH: usize = 300;
pub const LABEL_Y: [i32; 5] = [20, 40, 60, 60, 60];
pub const LABEL_X: [i32; 5] = [94, 94, 94, 164, 288];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MakingHit {
    Title,
    Members,
    MinLevel,
    MaxLevel,
    Ok,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MakingData {
    pub room_id: i32,
    pub loot_type: i32,
    pub location: i32,
    pub title: String,
    pub max_members: i32,
    pub min_level: i32,
    pub max_level: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MakingView {
    pub id: WindowId,
    pub room_id: i32,
    pub loot_type: i32,
    pub location: i32,
    pub title: TextInput,
    pub members: crate::combo::ComboBox,
    pub min_level: TextInput,
    pub max_level: TextInput,
    pub labels: [String; 5],
    pub ok: TextButton,
    pub cancel: TextButton,
    pub pressed: Option<MakingHit>,
    pub focus: Option<MakingHit>,
}

impl MakingView {
    pub fn new(labels: [String; 5], ok: &str, cancel: &str) -> Self {
        Self {
            id: WindowId::PARTY_ROOM_MAKING,
            room_id: 0,
            loot_type: 0,
            location: 0,
            title: TextInput::default(),
            members: crate::combo::ComboBox::new(MEMBERS_RECT, (2..=12).map(|count| count.to_string()).collect(), 13),
            min_level: TextInput { text: "1".into(), cursor: 1 },
            max_level: TextInput { text: "78".into(), cursor: 2 },
            labels,
            ok: TextButton::new(OK_RECT, BUTTON_NORMAL, BUTTON_DOWN, ok),
            cancel: TextButton::new(CANCEL_RECT, BUTTON_NORMAL, BUTTON_DOWN, cancel),
            pressed: None,
            focus: None,
        }
    }

    pub fn set_data(&mut self, data: &MakingData) {
        self.room_id = data.room_id;
        self.loot_type = data.loot_type;
        self.location = data.location;
        self.title.text = data.title.clone();
        self.title.cursor = self.title.text.chars().count();
        self.set_members_index(data.max_members.saturating_sub(2));
        Self::set_number(&mut self.min_level, data.min_level);
        Self::set_number(&mut self.max_level, data.max_level);
    }

    pub fn set_members_index(&mut self, index: i32) {
        if index >= 0 && (index as usize) < self.members.entries.len() {
            self.members.selected = index;
        }
    }

    pub fn lose_combo_focus(&mut self) {
        self.members.lose_focus();
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        self.members.mouse_move(x, y);
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        let hit = self.hit(x, y);
        self.pressed = hit;
        if matches!(hit, Some(MakingHit::Title | MakingHit::MinLevel | MakingHit::MaxLevel)) {
            self.focus = hit;
        }
        if hit != Some(MakingHit::Members) {
            self.members.lose_focus();
            return;
        }
        self.members.mouse_move(x, y);
        self.members.mouse_down();
        self.focus = None;
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<MakingHit> {
        if self.members.open && self.members.expanded_rect().contains(x, y) { return Some(MakingHit::Members); }
        if TITLE_RECT.contains(x, y) { return Some(MakingHit::Title); }
        if MEMBERS_RECT.contains(x, y) { return Some(MakingHit::Members); }
        if MIN_LEVEL_RECT.contains(x, y) { return Some(MakingHit::MinLevel); }
        if MAX_LEVEL_RECT.contains(x, y) { return Some(MakingHit::MaxLevel); }
        if self.ok.contains(x, y) { return Some(MakingHit::Ok); }
        if self.cancel.contains(x, y) { return Some(MakingHit::Cancel); }
        None
    }

    pub fn key(&mut self, key: InputKey) -> bool {
        let Some(hit) = self.focus else { return false };
        let input = match hit {
            MakingHit::Title => &mut self.title,
            MakingHit::Members => return false,
            MakingHit::MinLevel => &mut self.min_level,
            MakingHit::MaxLevel => &mut self.max_level,
            MakingHit::Ok | MakingHit::Cancel => return false,
        };
        if let InputKey::Char(c) = key {
            if matches!(hit, MakingHit::MinLevel | MakingHit::MaxLevel) && !c.is_ascii_digit() {
                return true;
            }
            if hit == MakingHit::Title && input.text.chars().count() >= TITLE_MAX_LENGTH {
                return true;
            }
        }
        input.key(key);
        true
    }

    pub fn data(&self) -> MakingData {
        let min_level = self.number(&self.min_level).clamp(1, 78);
        let max_level = if self.max_level.text.is_empty() { 78 } else { self.number(&self.max_level).clamp(min_level, 78) };
        MakingData {
            room_id: self.room_id,
            loot_type: self.loot_type,
            location: self.location,
            title: self.title.text.clone(),
            max_members: self.members.selected.saturating_add(2),
            min_level,
            max_level,
        }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        list.draw_texture(x, y, MAKING_SIZE.0, MAKING_SIZE.1, 0, 0, MAKING_SIZE.0, MAKING_SIZE.1, MAKING_BACK, alpha);
        for i in 0..self.labels.len() {
            if !self.labels[i].is_empty() {
                list.draw_text(x + LABEL_X[i] - font.measure(&self.labels[i]) as i32, y + LABEL_Y[i], TEXT, &self.labels[i], font);
            }
        }
        list.draw_text_clipped(x + TITLE_RECT.x, y + TITLE_RECT.y, TITLE_RECT.w, TEXT, &self.title.text, font);
        list.draw_text_clipped(x + MIN_LEVEL_RECT.x, y + MIN_LEVEL_RECT.y, MIN_LEVEL_RECT.w, TEXT, &self.min_level.text, font);
        list.draw_text_clipped(x + MAX_LEVEL_RECT.x, y + MAX_LEVEL_RECT.y, MAX_LEVEL_RECT.w, TEXT, &self.max_level.text, font);
        self.ok.paint(list, x, y, self.pressed == Some(MakingHit::Ok), font);
        self.cancel.paint(list, x, y, self.pressed == Some(MakingHit::Cancel), font);
        self.members.paint(list, (x, y), font, self.members.open, self.pressed == Some(MakingHit::Members));
    }

    fn set_number(input: &mut TextInput, value: i32) {
        input.text = value.to_string();
        input.cursor = input.text.chars().count();
    }

    fn number(&self, input: &TextInput) -> i32 {
        input.text.parse().unwrap_or(0)
    }
}

pub fn grip() -> Rect {
    Rect::new(0, 0, MAKING_SIZE.0, 17)
}

pub fn room_grip() -> Rect {
    Rect::new(0, 0, ROOM_SIZE.0, 20)
}

pub const ROOM_SIZE: (i32, i32) = (600, 431);
pub const ROOM_POS: (i32, i32) = (0, 76);
pub const ROOM_INITIAL_POSITION: crate::initial_position::InitialPosition = crate::initial_position::InitialPosition {
    enabled: true, horizontal_mode: 0, vertical_mode: 3, x: ROOM_POS.0, y: ROOM_POS.1,
};
pub const ROOM_FRAME: crate::frame_contract::FrameContract = crate::frame_contract::FrameContract {
    rect: Rect::new(0, 0, 600, 20),
    id: 0xf7,
    mode: crate::frame::FrameMode::Title,
    title: crate::frame_contract::FrameTitle::Sysstring(389),
    close_button: true,
    minimize_button: true,
};
pub const ROOM_BACK: &str = "L2UI_CH3.PartyMatchWnd.PartyMatch2_Back";
pub const ROOM_BACK_RECT: Rect = Rect::new(0, 20, 600, 411);
pub const ROOM_LIST_RECT: Rect = Rect::new(7, 123, 586, 172);
pub const ROOM_LIST_CAPACITY: usize = 20;
pub const ROOM_VISIBLE_ROWS: usize = 9;
pub const ROOM_HEADER_HEIGHT: i32 = 19;
pub const ROOM_ROW_HEIGHT: i32 = 17;
pub const ROOM_COLUMN_WIDTHS: [i32; 5] = [204, 54, 74, 132, 121];
pub const ROOM_COLUMN_SYSSTRINGS: [u32; 5] = [393, 391, 392, 1029, 1031];
pub const ROOM_CHAT_ID: u32 = 0xffa0_f020;
pub const ROOM_CHAT_RECT: Rect = Rect::new(7, 302, 586, 72);
pub const ROOM_CHAT_MAX_LINES: usize = 40;
pub const ROOM_CHAT_VISIBLE: i32 = 4;
pub const ROOM_EDIT: crate::child_contract::ChildContract = crate::child_contract::ChildContract {
    kind: crate::child_contract::ChildKind::EditBox,
    rect: Rect::new(7, 377, 436, 17),
    id: 0xffa0_f021,
    vtable: 0x101a_2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x214, 0x1), (0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x83), (0x2b8, 0x1)],
    textures: &[],
};
pub const ROOM_BUTTON_IDS: [u32; 4] = [0xffa0_f023, 0xffa0_f024, 0xffa0_f025, 0xffa0_f026];
pub const ROOM_BUTTON_RECTS: [Rect; 4] = [
    Rect::new(298, 402, 66, 21),
    Rect::new(373, 402, 66, 21),
    Rect::new(448, 402, 66, 21),
    Rect::new(523, 402, 66, 21),
];
pub const ROOM_BUTTON_SYSSTRINGS: [u32; 4] = [1032, 1033, 396, 908];
pub const ROOM_BUTTON_NORMAL: &str = "L2UI_CH3.Button.SmallButton2";
pub const ROOM_BUTTON_DOWN: &str = "L2UI_CH3.Button.SmallButton2_down";
pub const ROOM_INFO_SYSSTRINGS: [u32; 6] = [416, 413, 1029, 1036, 489, 1030];
pub const ROOM_INFO_LABEL_RIGHT: [(i32, i32); 6] = [(59, 41), (59, 56), (59, 71), (254, 71), (59, 86), (254, 86)];
pub const ROOM_INFO_VALUE_X: [i32; 6] = [68, 68, 68, 263, 68, 263];
pub const ROOM_LABEL_COLOR: crate::color::Argb = crate::color::Argb(0xffdc_dcdc);
pub const ROOM_VALUE_COLOR: crate::color::Argb = crate::color::Argb(0xffb0_9b79);
pub const ROLE_SYSSTRINGS: [u32; 3] = [1061, 1062, 1063];
pub const LOOT_SYSSTRINGS: [u32; 5] = [487, 488, 798, 799, 800];
pub const LOCATION_SYSSTRINGS: [u32; 15] = [1048, 1049, 1050, 1051, 1052, 1053, 1054, 1055, 1056, 1057, 1058, 1059, 1060, 1247, 1248];
pub const ROLE_LEADER: i32 = 1;
pub const SAY_PARTY_ROOM: i32 = 14;
pub const CLASS_NAME_COUNT: u32 = 128;
pub const COLUMN_NUMERIC: [bool; 5] = [false, true, true, false, false];
pub const ROOM_LIST_CHILD: crate::child_contract::ChildContract = crate::child_contract::ChildContract {
    kind: crate::child_contract::ChildKind::ListCtrl,
    rect: ROOM_LIST_RECT,
    id: 0x0,
    vtable: 0x101b_d220,
    registration: (0x22, 0x0, 0x0),
    handles: &[0x230, 0x234, 0x238],
    state: &[
        (0x23c, ROOM_VISIBLE_ROWS as u32),
        (0x240, ROOM_LIST_CAPACITY as u32),
        (0x244, 0x5),
        (0x250, 0xffff_ffff),
        (0x254, 0xffff_ffff),
        (0x258, 0xf),
        (0x25c, ROOM_HEADER_HEIGHT as u32),
        (0x264, 0x3),
        (0x268, 0x1),
        (0x26c, ROOM_ROW_HEIGHT as u32),
    ],
    textures: &[],
};
const fn room_column(index: usize) -> ColumnContract {
    ColumnContract {
        width: ROOM_COLUMN_WIDTHS[index],
        height: ROOM_HEADER_HEIGHT,
        sysstring: ROOM_COLUMN_SYSSTRINGS[index] as i32,
        flag_c: 1,
        flag_10: COLUMN_NUMERIC[index] as u32,
        textures: TAB_TEXTURES,
    }
}
pub const ROOM_COLUMNS: [ColumnContract; 5] = [room_column(0), room_column(1), room_column(2), room_column(3), room_column(4)];
pub const ROOM_LIST: ListContract = ListContract { child: ROOM_LIST_CHILD, columns: &ROOM_COLUMNS };
pub const ROW_TIP_CLASS_COLUMN: usize = 1;
pub const NULL_TEXT: &str = "(null)";
pub const CLASS_ICON_SIZE: i32 = 11;
pub const MODE_ADD: i32 = 0;
pub const MODE_UPDATE: i32 = 1;
pub const MODE_REMOVE: i32 = 2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomData {
    pub id: i32,
    pub title: String,
    pub location: i32,
    pub min_level: i32,
    pub max_level: i32,
    pub members: i32,
    pub max_members: i32,
    pub owner_name: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoomLabels {
    pub title: String,
    pub info: [String; 6],
    pub columns: [String; 5],
    pub buttons: [String; 4],
    pub roles: [String; 3],
    pub loot: [String; 5],
    pub locations: [String; 15],
    pub class_names: Vec<String>,
}

impl RoomLabels {
    pub fn from_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id| lookup(id).unwrap_or_default();
        Self {
            title: text(389),
            info: ROOM_INFO_SYSSTRINGS.map(text),
            columns: ROOM_COLUMN_SYSSTRINGS.map(text),
            buttons: ROOM_BUTTON_SYSSTRINGS.map(text),
            roles: ROLE_SYSSTRINGS.map(text),
            loot: LOOT_SYSSTRINGS.map(text),
            locations: LOCATION_SYSSTRINGS.map(text),
            class_names: (0..CLASS_NAME_COUNT).map(|class| text(crate::detail_status::class_name_sysstring(class))).collect(),
        }
    }

    pub fn class_name(&self, class_id: i32) -> Option<&str> {
        usize::try_from(class_id).ok().and_then(|index| self.class_names.get(index)).map(String::as_str)
    }

    pub fn location(&self, location: i32) -> Option<&str> {
        usize::try_from(location).ok().and_then(|index| index.checked_sub(1)).and_then(|index| self.locations.get(index)).map(String::as_str)
    }

    pub fn loot(&self, loot_type: i32) -> Option<&str> {
        usize::try_from(loot_type).ok().and_then(|index| self.loot.get(index)).map(String::as_str)
    }

    pub fn role(&self, role: i32) -> Option<&str> {
        usize::try_from(role).ok().and_then(|index| self.roles.get(index)).map(String::as_str)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoomInfo {
    pub room_id: i32,
    pub max_members: i32,
    pub min_level: i32,
    pub max_level: i32,
    pub loot_type: i32,
    pub location: i32,
    pub title: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RoomMember {
    pub id: i32,
    pub name: String,
    pub class_id: i32,
    pub class_text: String,
    pub level: String,
    pub location: String,
    pub role: String,
}

impl RoomMember {
    pub fn cell(&self, column: usize) -> &str {
        match column {
            0 => &self.name,
            1 => &self.class_text,
            2 => &self.level,
            3 => &self.location,
            4 => &self.role,
            _ => "",
        }
    }
}

impl RoomMember {
    pub fn list_row(&self) -> ListRow {
        let class = ListCell {
            text: self.class_text.clone(),
            text_enabled: false,
            big_icon: crate::class_icon::texture(self.class_id)
                .map(|texture| BigIcon { texture: texture.to_string(), w: CLASS_ICON_SIZE, h: CLASS_ICON_SIZE }),
            ..ListCell::default()
        };
        ListRow {
            cells: vec![ListCell::text(&self.name), class, ListCell::text(&self.level), ListCell::text(&self.location), ListCell::text(&self.role)],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberValues {
    pub id: i32,
    pub name: String,
    pub class_id: i32,
    pub level: i32,
    pub location: i32,
    pub role: i32,
}

pub fn level_band(level: i32) -> String {
    if level < 10 {
        "1 ~ 9".to_string()
    } else if level < 70 {
        let low = level / 10 * 10;
        format!("{} ~ {}", low, low + 9)
    } else {
        format!("70 ~ {}", 78)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoomHit {
    Close,
    Minimize,
    Header(usize),
    Row(usize),
    List,
    Scroll,
    Chat,
    Edit,
    Manage,
    Ban,
    Invite,
    Leave,
}

const BUTTON_HITS: [RoomHit; 4] = [RoomHit::Manage, RoomHit::Ban, RoomHit::Invite, RoomHit::Leave];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoomAction {
    Manage { room_id: i32, title: String, max_members: i32, min_level: i32, max_level: i32 },
    Oust { object_id: i32 },
    Invite { name: String, loot_type: i32 },
    Dismiss { room_id: i32 },
    Withdraw { room_id: i32 },
    Say { text: String },
    Bypass(String),
    Hide,
    Minimize,
}

#[derive(Debug, Clone)]
pub struct PartyRoomView {
    pub id: WindowId,
    pub labels: RoomLabels,
    pub frame: crate::frame::FrameCtrl,
    pub info: RoomInfo,
    pub role: i32,
    pub members: Vec<RoomMember>,
    pub list: ListCtrl,
    pub chat: crate::text_list::TextList,
    pub edit: crate::edit_box::EditBox,
    pub buttons: [TextButton; 4],
    pub pressed: Option<RoomHit>,
}

impl PartyRoomView {
    pub fn new(labels: RoomLabels) -> Self {
        Self {
            id: WindowId::PARTY_ROOM,
            frame: ROOM_FRAME.build(|_| labels.title.clone()),
            buttons: std::array::from_fn(|i| TextButton::new(ROOM_BUTTON_RECTS[i], ROOM_BUTTON_NORMAL, ROOM_BUTTON_DOWN, &labels.buttons[i])),
            labels,
            info: RoomInfo::default(),
            role: 0,
            members: Vec::new(),
            list: ListCtrl::from_contract(&ROOM_LIST),
            chat: crate::text_list::TextList::new(ROOM_CHAT_ID, ROOM_CHAT_RECT, ROOM_CHAT_MAX_LINES, ROOM_CHAT_VISIBLE, true),
            edit: crate::edit_box::EditBox::from_contract(&ROOM_EDIT),
            pressed: None,
        }
    }

    pub fn set_labels(&mut self, labels: RoomLabels) {
        self.frame.title = labels.title.clone();
        for (button, label) in self.buttons.iter_mut().zip(&labels.buttons) {
            button.label = label.clone();
        }
        self.labels = labels;
    }

    pub fn set_info(&mut self, info: RoomInfo) {
        self.info = info;
    }

    pub fn apply_role(&mut self, role: i32) {
        let enabled = match role {
            ROLE_LEADER => true,
            0 | 2 => false,
            _ => return,
        };
        for button in &mut self.buttons[..3] {
            button.enabled = enabled;
        }
        self.buttons[3].enabled = true;
    }

    pub fn set_members(&mut self, role: i32, members: &[MemberValues]) {
        self.members.clear();
        self.list.clear();
        self.role = role;
        self.apply_role(role);
        for member in members {
            let row = self.row(member);
            self.add_row(row);
        }
        self.rebuild();
    }

    pub fn manage_member(&mut self, mode: i32, member: &MemberValues, me: Option<i32>) {
        let mut role = 0;
        match mode {
            MODE_ADD => {
                let row = self.row(member);
                self.add_row(row);
                role = member.role;
            }
            MODE_UPDATE => {
                let fresh = self.row(member);
                if let Some(row) = self.members.iter_mut().find(|row| row.id == member.id) {
                    row.name = fresh.name;
                    row.class_id = fresh.class_id;
                    row.class_text = fresh.class_text;
                    row.level = fresh.level;
                    row.location = fresh.location;
                    if self.labels.role(member.role).is_some() {
                        row.role = fresh.role;
                    }
                    role = member.role;
                }
            }
            MODE_REMOVE => {
                if let Some(index) = self.members.iter().position(|row| row.id == member.id) {
                    self.members.remove(index);
                }
            }
            _ => {}
        }
        if me == Some(member.id) {
            self.role = role;
            self.apply_role(role);
        }
        self.rebuild();
    }

    pub fn selected(&self) -> Option<usize> {
        self.list.selected()
    }

    pub fn sort_column(&mut self, column: usize) {
        self.list.sort(column);
        self.follow_sort();
    }

    fn follow_sort(&mut self) {
        let order = self.list.origins();
        if order.iter().enumerate().all(|(at, &origin)| at == origin) {
            return;
        }
        self.members = order.iter().map(|&origin| self.members[origin].clone()).collect();
        self.list.reset_origins();
    }

    pub fn row_tooltip(&self) -> Option<crate::tooltip::Tooltip> {
        let index = usize::try_from(self.list.hovered()).ok()?;
        if self.list.selected() != Some(index) {
            return None;
        }
        let member = self.members.get(index)?;
        let class_id = wtoi(member.cell(ROW_TIP_CLASS_COLUMN));
        Some(crate::tooltip::Tooltip {
            row: true,
            row_style: 0x800000,
            lines: vec![crate::tooltip::TooltipLine {
                label: self.labels.columns[ROW_TIP_CLASS_COLUMN].clone(),
                value: self.labels.class_name(class_id).unwrap_or_default().to_string(),
                color: crate::tooltip::VALUE_COLOR,
            }],
            ..Default::default()
        })
    }

    pub fn close_packet(&mut self) {
        self.chat.clear();
        self.hide();
    }

    pub fn hide(&mut self) {
        self.edit.kill_focus();
        self.pressed = None;
    }

    pub fn add_chat(&mut self, text: &str, color: crate::color::Argb, font: &BitmapFont) {
        self.chat.add_string(text, color, &crate::edit_box::FontMetrics(font));
    }

    pub fn selected_member(&self) -> Option<&RoomMember> {
        self.list.selected().and_then(|index| self.members.get(index))
    }

    pub fn leave_request(&self) -> Option<RoomAction> {
        match self.role {
            ROLE_LEADER => Some(RoomAction::Dismiss { room_id: self.info.room_id }),
            0 | 2 => Some(RoomAction::Withdraw { room_id: self.info.room_id }),
            _ => None,
        }
    }

    pub fn close_actions(&self) -> Vec<RoomAction> {
        self.leave_request().into_iter().chain(std::iter::once(RoomAction::Hide)).collect()
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<RoomHit> {
        match ROOM_FRAME.hit(&self.frame, (0, 0), (x, y)) {
            Some(crate::frame_contract::FrameHit::Close) => return Some(RoomHit::Close),
            Some(crate::frame_contract::FrameHit::Minimize) => return Some(RoomHit::Minimize),
            _ => {}
        }
        for (index, button) in self.buttons.iter().enumerate() {
            if button.enabled && button.contains(x, y) {
                return Some(BUTTON_HITS[index]);
            }
        }
        if self.edit.rect.contains(x, y) {
            return Some(RoomHit::Edit);
        }
        if self.chat.rect.contains(x, y) {
            return Some(RoomHit::Chat);
        }
        if ROOM_LIST_RECT.contains(x, y) {
            match self.list.child_at(x, y) {
                Some(Pressed::Header(column)) => return Some(RoomHit::Header(column)),
                Some(Pressed::Up | Pressed::Down | Pressed::Slider) => return Some(RoomHit::Scroll),
                None => {}
            }
            let row = self.list.hit(x, y);
            return Some(if row >= 0 { RoomHit::Row(row as usize) } else { RoomHit::List });
        }
        None
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, font: &BitmapFont) -> Option<RoomHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        match hit {
            Some(RoomHit::Edit) => {
                if !self.edit.focused {
                    self.edit.focus();
                }
                let metrics = crate::edit_box::FontMetrics(font);
                self.edit.mouse_down(x, &metrics);
                self.edit.layout(&metrics);
            }
            Some(RoomHit::Chat) => {
                self.chat.mouse_down(x, y);
            }
            Some(RoomHit::Row(_) | RoomHit::List | RoomHit::Header(_) | RoomHit::Scroll) => {
                self.list.mouse_down(x, y);
            }
            _ => {}
        }
        hit
    }

    pub fn mouse_move(&mut self, x: i32, y: i32, font: &BitmapFont) {
        if self.edit.focused {
            let metrics = crate::edit_box::FontMetrics(font);
            self.edit.mouse_move(x, &metrics);
            self.edit.layout(&metrics);
        }
        self.chat.mouse_move(y);
        self.list.mouse_move(x, y);
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Vec<RoomAction> {
        self.edit.mouse_up();
        self.chat.mouse_up();
        let pressed = self.pressed.take();
        if self.list.left_held {
            self.list.mouse_up(x, y);
            self.follow_sort();
        }
        let Some(pressed) = pressed else { return Vec::new() };
        if self.hit(x, y) != Some(pressed) {
            return Vec::new();
        }
        match pressed {
            RoomHit::Minimize => vec![RoomAction::Minimize],
            RoomHit::Close | RoomHit::Leave => self.close_actions(),
            RoomHit::Manage => vec![RoomAction::Manage {
                room_id: self.info.room_id,
                title: self.info.title.clone(),
                max_members: self.info.max_members,
                min_level: self.info.min_level,
                max_level: self.info.max_level,
            }],
            RoomHit::Ban => self.selected_member().map(|member| RoomAction::Oust { object_id: member.id }).into_iter().collect(),
            RoomHit::Invite => self
                .selected_member()
                .map(|member| RoomAction::Invite { name: member.name.clone(), loot_type: self.info.loot_type })
                .into_iter()
                .collect(),
            _ => Vec::new(),
        }
    }

    pub fn key(&mut self, key: crate::chat::InputKey, font: &BitmapFont) -> Option<Vec<RoomAction>> {
        if !self.edit.focused {
            return None;
        }
        if self.edit.input_key(key, &crate::edit_box::FontMetrics(font)) == crate::edit_box::EditInput::Escape {
            self.edit.kill_focus();
            return Some(Vec::new());
        }
        let entered = self.edit.notices.drain(..).any(|notice| notice.code == crate::edit_box::NOTICE_ENTER);
        Some(if entered { self.enter() } else { Vec::new() })
    }

    pub fn enter(&mut self) -> Vec<RoomAction> {
        let mut actions = Vec::new();
        if let Some(text) = self.edit.get_text() {
            match crate::petition_chat::classify(&text) {
                crate::petition_chat::Command::Say => actions.push(RoomAction::Say { text }),
                crate::petition_chat::Command::Build => actions.push(RoomAction::Bypass(text.chars().skip(2).collect())),
                crate::petition_chat::Command::Ignored => {}
            }
            self.edit.clear();
        }
        actions
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        if ROOM_LIST_RECT.contains(x, y) {
            self.list.wheel(delta as i16);
            return true;
        }
        if self.chat.rect.contains(x, y) {
            self.chat.wheel(delta);
            return true;
        }
        false
    }

    pub fn tick(&mut self, elapsed_ms: u32) {
        self.edit.tick(elapsed_ms);
    }

    pub fn info_values(&self) -> [String; 6] {
        let text = |value: Option<&str>| value.unwrap_or(NULL_TEXT).to_string();
        [
            self.info.room_id.to_string(),
            self.info.title.clone(),
            text(self.labels.location(self.info.location)),
            format!("{}/{}", self.members.len(), self.info.max_members),
            text(self.labels.loot(self.info.loot_type)),
            format!("{}-{}", self.info.min_level, self.info.max_level),
        ]
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        ROOM_FRAME.paint(&self.frame, list, (x, y), font, alpha);
        let start = list.quads.len();
        list.draw_texture(
            x + ROOM_BACK_RECT.x,
            y + ROOM_BACK_RECT.y,
            ROOM_BACK_RECT.w,
            ROOM_BACK_RECT.h,
            0,
            0,
            ROOM_BACK_RECT.w,
            ROOM_BACK_RECT.h,
            ROOM_BACK,
            alpha,
        );
        let values = self.info_values();
        for index in 0..values.len() {
            let (right, row_y) = ROOM_INFO_LABEL_RIGHT[index];
            let label = &self.labels.info[index];
            list.draw_text(x + right - font.measure(label) as i32, y + row_y, ROOM_LABEL_COLOR, label, font);
            list.draw_text(x + ROOM_INFO_VALUE_X[index], y + row_y, ROOM_VALUE_COLOR, &values[index], font);
        }
        list.clip_from(start, Rect::new(x, y, ROOM_SIZE.0, ROOM_SIZE.1));
        self.paint_list(list, x, y, font);
        self.chat.paint(list, (x, y), font);
        self.edit.paint(list, (x, y), &crate::edit_box::FontMetrics(font), font);
        for (index, button) in self.buttons.iter().enumerate() {
            button.paint(list, x, y, self.pressed == Some(BUTTON_HITS[index]), font);
        }
    }

    fn paint_list(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        self.list.paint(&mut body, (x, y), font);
        self.list.paint_header(&mut body, (x, y), font, &|id| {
            ROOM_COLUMN_SYSSTRINGS.iter().position(|column| *column as i32 == id).map(|index| self.labels.columns[index].clone())
        });
        list.append_clipped(body, Rect::new(x + ROOM_LIST_RECT.x, y + ROOM_LIST_RECT.y, ROOM_LIST_RECT.w, ROOM_LIST_RECT.h));
    }

    fn row(&self, member: &MemberValues) -> RoomMember {
        RoomMember {
            id: member.id,
            name: member.name.clone(),
            class_id: member.class_id,
            class_text: member.class_id.to_string(),
            level: level_band(member.level),
            location: self.labels.location(member.location).unwrap_or_default().to_string(),
            role: self.labels.role(member.role).unwrap_or_default().to_string(),
        }
    }

    fn add_row(&mut self, row: RoomMember) {
        if self.members.len() < ROOM_LIST_CAPACITY {
            self.members.push(row);
        }
    }

    fn rebuild(&mut self) {
        let rows = self.members.iter().map(RoomMember::list_row).collect();
        self.list.replace_rows(rows);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view() -> MakingView {
        MakingView::new(["".into(), "".into(), "".into(), "".into(), "".into()], "OK", "Cancel")
    }

    #[test]
    fn empty_level_boxes_default_to_1_and_78_and_the_second_never_goes_below_the_first() {
        let mut view = view();
        view.min_level.text.clear();
        view.max_level.text.clear();
        let data = view.data();
        assert_eq!((data.min_level, data.max_level), (1, 78));
        view.min_level.text = "90".into();
        view.max_level.text = "10".into();
        let data = view.data();
        assert_eq!((data.min_level, data.max_level), (78, 78));
        view.min_level.text = "20".into();
        view.max_level.text = "0".into();
        let data = view.data();
        assert_eq!((data.min_level, data.max_level), (20, 20));
    }

    #[test]
    fn geometry_matches_c4_controls() {
        let view = view();
        assert_eq!(MAKING_SIZE, (356, 141));
        assert_eq!(view.hit(103, 17), Some(MakingHit::Title));
        assert_eq!(view.hit(101, 110), Some(MakingHit::Ok));
        assert_eq!(view.hit(181, 110), Some(MakingHit::Cancel));
    }

    #[test]
    fn levels_are_clamped_like_on_ok() {
        let mut view = view();
        view.min_level.text = "0".into();
        view.max_level.text = "99".into();
        let data = view.data();
        assert_eq!((data.min_level, data.max_level), (1, 78));
        view.min_level.text = "60".into();
        view.max_level.text = "20".into();
        assert_eq!((view.data().min_level, view.data().max_level), (60, 60));
    }

    #[test]
    fn members_follow_the_c4_display_offset() {
        let mut view = view();
        view.set_data(&MakingData {
            room_id: 7,
            loot_type: 0,
            location: 0,
            title: "Room".into(),
            max_members: 9,
            min_level: 1,
            max_level: 78,
        });
        assert_eq!(view.members.selected_text(), Some("9"));
        assert_eq!(view.data().max_members, 9);
    }

    #[test]
    fn the_members_box_is_a_combo_with_the_items_two_to_twelve() {
        let mut view = view();
        assert_eq!(view.members.entries.len(), 11);
        assert_eq!((view.members.entries[0].as_str(), view.members.entries[10].as_str()), ("2", "12"));
        assert_eq!(view.members.selected, -1);
        let rect = view.members.rect;
        view.mouse_down(rect.x + 2, rect.y + 2);
        assert!(view.members.open);
        view.mouse_move(rect.x + 2, rect.y + rect.h + 2 * view.members.row_height + 1);
        view.mouse_down(rect.x + 2, rect.y + rect.h + 2 * view.members.row_height + 1);
        assert!(!view.members.open);
        assert_eq!(view.data().max_members, 4);
        view.mouse_down(rect.x + 2, rect.y + 2);
        view.mouse_down(10, 100);
        assert!(!view.members.open);
    }

    fn room() -> PartyRoomView {
        PartyRoomView::new(RoomLabels::from_sysstrings(|id| Some(format!("s{id}"))))
    }

    fn font() -> BitmapFont {
        use crate::font::{Glyphs, gly};
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("font".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn member(id: i32, role: i32) -> MemberValues {
        MemberValues { id, name: format!("m{id}"), class_id: id, level: 40 + id, location: 1, role }
    }

    fn info() -> RoomInfo {
        RoomInfo { room_id: 7, max_members: 12, min_level: 20, max_level: 60, loot_type: 2, location: 3, title: "Sala".into() }
    }

    fn enabled(view: &PartyRoomView) -> [bool; 4] {
        view.buttons.each_ref().map(|button| button.enabled)
    }

    fn click(view: &mut PartyRoomView, x: i32, y: i32) -> Vec<RoomAction> {
        view.mouse_down(x, y, &font());
        view.mouse_up(x, y)
    }

    #[test]
    fn on_create_100b55d0_controls_and_ids() {
        let view = room();
        assert_eq!(ROOM_SIZE, (600, 431));
        assert_eq!(ROOM_FRAME.rect, Rect::new(0, 0, 600, 20));
        assert_eq!(view.frame.title, "s389");
        assert_eq!(ROOM_LIST_RECT, Rect::new(7, 123, 586, 172));
        assert_eq!(ROOM_HEADER_HEIGHT + ROOM_VISIBLE_ROWS as i32 * ROOM_ROW_HEIGHT, ROOM_LIST_RECT.h);
        assert_eq!(view.chat.rect, Rect::new(7, 302, 586, 72));
        assert_eq!((view.chat.id, view.chat.max_lines, view.chat.visible, view.chat.bar), (0xffa0_f020, 40, 4, true));
        assert_eq!((view.edit.id, view.edit.rect, view.edit.kind, view.edit.extra_width, view.edit.max_len), (0xffa0_f021, Rect::new(7, 377, 436, 17), 1, 0x83, 0));
        assert_eq!(view.labels.columns, ["s393", "s391", "s392", "s1029", "s1031"]);
        assert_eq!(view.labels.buttons, ["s1032", "s1033", "s396", "s908"]);
        assert_eq!(view.buttons.each_ref().map(|button| button.rect), ROOM_BUTTON_RECTS);
        assert_eq!(view.hit(530, 422), Some(RoomHit::Leave));
        assert_eq!(view.hit(298, 402), Some(RoomHit::Manage));
        assert_eq!(view.hit(373, 402), Some(RoomHit::Ban));
        assert_eq!(view.hit(448, 402), Some(RoomHit::Invite));
        assert_eq!(view.hit(580, 6), Some(RoomHit::Close));
        assert_eq!(view.hit(8, 124), Some(RoomHit::Header(0)));
        assert_eq!(view.hit(8, 150), Some(RoomHit::List));
        assert_eq!(view.hit(10, 380), Some(RoomHit::Edit));
        assert_eq!(view.hit(10, 310), Some(RoomHit::Chat));
        assert_eq!(enabled(&view), [true; 4]);
    }

    #[test]
    fn level_band_follows_100b1cd0_for_every_level() {
        for level in -5..=120 {
            let expected = if level < 10 {
                "1 ~ 9".to_string()
            } else if level < 70 {
                format!("{} ~ {}", level / 10 * 10, level / 10 * 10 + 9)
            } else {
                "70 ~ 78".to_string()
            };
            assert_eq!(level_band(level), expected);
        }
        assert_eq!((level_band(10).as_str(), level_band(69).as_str()), ("10 ~ 19", "60 ~ 69"));
    }

    #[test]
    fn labels_resolve_location_loot_and_role_tables() {
        let labels = room().labels;
        for location in -1..=17 {
            let expected = match location {
                1..=13 => Some(format!("s{}", 1047 + location)),
                14 => Some("s1247".to_string()),
                15 => Some("s1248".to_string()),
                _ => None,
            };
            assert_eq!(labels.location(location).map(str::to_string), expected);
        }
        for loot in -1..=6 {
            let expected = usize::try_from(loot).ok().and_then(|i| LOOT_SYSSTRINGS.get(i)).map(|id| format!("s{id}"));
            assert_eq!(labels.loot(loot).map(str::to_string), expected);
        }
        for role in -1..=4 {
            let expected = usize::try_from(role).ok().and_then(|i| ROLE_SYSSTRINGS.get(i)).map(|id| format!("s{id}"));
            assert_eq!(labels.role(role).map(str::to_string), expected);
        }
    }

    #[test]
    fn member_list_role_rule_enables_buttons_by_role() {
        let mut view = room();
        view.set_members(1, &[]);
        assert_eq!(enabled(&view), [true; 4]);
        for role in [0, 2] {
            view.set_members(role, &[]);
            assert_eq!(enabled(&view), [false, false, false, true]);
            assert_eq!(view.hit(298, 402), None);
            assert_eq!(view.hit(530, 422), Some(RoomHit::Leave));
        }
        view.set_members(5, &[]);
        assert_eq!(enabled(&view), [false, false, false, true]);
        assert_eq!(view.role, 5);
    }

    #[test]
    fn member_list_fills_five_cells_resets_selection_and_respects_capacity() {
        let mut view = room();
        view.set_members(1, &[member(1, 0)]);
        view.list.select(0);
        let members = (0..25).map(|id| member(id, id % 4)).collect::<Vec<_>>();
        view.set_members(1, &members);
        assert_eq!(view.selected(), None);
        assert_eq!(view.members.len(), ROOM_LIST_CAPACITY);
        let row = &view.members[2];
        assert_eq!((row.id, row.name.as_str(), row.class_id, row.level.as_str(), row.location.as_str(), row.role.as_str()), (2, "m2", 2, "40 ~ 49", "s1048", "s1063"));
        assert_eq!(view.members[3].role, "");
        assert_eq!(view.members[1].role, "s1062");
        assert_eq!(view.members[0].role, "s1061");
    }

    #[test]
    fn manage_member_100b3ec0_add_update_remove_and_own_role() {
        let mut view = room();
        view.set_members(1, &[member(1, 1), member(2, 0)]);
        view.manage_member(MODE_ADD, &member(3, 2), Some(1));
        assert_eq!(view.members.iter().map(|row| row.id).collect::<Vec<_>>(), [1, 2, 3]);
        assert_eq!(enabled(&view), [true; 4]);
        view.manage_member(MODE_UPDATE, &MemberValues { level: 75, location: 15, ..member(2, 7) }, Some(1));
        assert_eq!((view.members[1].level.as_str(), view.members[1].location.as_str(), view.members[1].role.as_str()), ("70 ~ 78", "s1248", "s1061"));
        view.manage_member(MODE_UPDATE, &member(2, 2), Some(1));
        assert_eq!(view.members[1].role, "s1063");
        view.manage_member(MODE_UPDATE, &member(1, 2), Some(1));
        assert_eq!((view.role, enabled(&view)), (2, [false, false, false, true]));
        view.manage_member(MODE_UPDATE, &member(1, 1), Some(1));
        assert_eq!((view.role, enabled(&view)), (1, [true; 4]));
        view.manage_member(MODE_UPDATE, &member(9, 1), Some(9));
        assert_eq!((view.role, enabled(&view)), (0, [false, false, false, true]));
        view.manage_member(MODE_ADD, &member(1, 1), Some(1));
        assert_eq!(view.role, 1);
        view.manage_member(MODE_REMOVE, &member(3, 1), Some(1));
        assert_eq!(view.members.iter().map(|row| row.id).collect::<Vec<_>>(), [1, 2, 1]);
        assert_eq!((view.role, enabled(&view)), (1, [true; 4]));
        view.manage_member(MODE_REMOVE, &member(9, 1), Some(9));
        assert_eq!((view.role, enabled(&view)), (0, [false, false, false, true]));
        view.manage_member(MODE_ADD, &member(1, 1), Some(1));
        view.manage_member(9, &member(1, 1), Some(1));
        assert_eq!(view.role, 0);
        view.manage_member(MODE_ADD, &member(1, 1), Some(1));
        view.manage_member(MODE_REMOVE, &member(2, 0), None);
        view.manage_member(MODE_UPDATE, &member(4, 0), Some(5));
        assert_eq!(view.role, 1);
        assert_eq!(view.members.iter().filter(|row| row.id == 2).count(), 0);
    }

    #[test]
    fn manage_add_keeps_role_update_when_the_list_is_full() {
        let mut view = room();
        let members = (0..20).map(|id| member(id, 0)).collect::<Vec<_>>();
        view.set_members(0, &members);
        view.manage_member(MODE_ADD, &member(99, 1), Some(99));
        assert_eq!(view.members.len(), 20);
        assert_eq!((view.role, enabled(&view)), (1, [true; 4]));
    }

    #[test]
    fn remove_keeps_the_selected_index_like_the_native_array() {
        let mut view = room();
        view.set_members(1, &[member(1, 0), member(2, 0), member(3, 0)]);
        view.list.select(2);
        view.manage_member(MODE_REMOVE, &member(1, 0), None);
        assert_eq!(view.list.selected_raw(), 2);
        assert!(view.selected_member().is_none());
    }

    #[test]
    fn ban_invite_and_manage_buttons_follow_100b19c0_100b1a80_100b18d0() {
        let mut view = room();
        view.set_info(info());
        view.set_members(1, &[member(5, 0), member(6, 2)]);
        assert!(click(&mut view, 380, 410).is_empty());
        assert!(click(&mut view, 450, 410).is_empty());
        let row_y = ROOM_LIST_RECT.y + ROOM_HEADER_HEIGHT + ROOM_ROW_HEIGHT + 2;
        assert!(click(&mut view, 20, row_y).is_empty());
        assert_eq!(view.selected(), Some(1));
        assert_eq!(click(&mut view, 380, 410), vec![RoomAction::Oust { object_id: 6 }]);
        assert_eq!(click(&mut view, 450, 410), vec![RoomAction::Invite { name: "m6".into(), loot_type: 2 }]);
        assert_eq!(
            click(&mut view, 300, 410),
            vec![RoomAction::Manage { room_id: 7, title: "Sala".into(), max_members: 12, min_level: 20, max_level: 60 }]
        );
        assert!(click(&mut view, 20, ROOM_LIST_RECT.y + ROOM_LIST_RECT.h - 2).is_empty());
        assert_eq!(view.selected(), None);
        view.mouse_down(300, 410, &font());
        assert!(view.mouse_up(380, 410).is_empty());
    }

    #[test]
    fn close_helper_100b1b50_depends_on_the_role() {
        let mut view = room();
        view.set_info(info());
        for (role, expected) in [
            (1, vec![RoomAction::Dismiss { room_id: 7 }, RoomAction::Hide]),
            (0, vec![RoomAction::Withdraw { room_id: 7 }, RoomAction::Hide]),
            (2, vec![RoomAction::Withdraw { room_id: 7 }, RoomAction::Hide]),
            (3, vec![RoomAction::Hide]),
            (-1, vec![RoomAction::Hide]),
        ] {
            view.set_members(role, &[]);
            assert_eq!(click(&mut view, 530, 410), expected);
            assert_eq!(click(&mut view, 580, 6), expected);
            assert_eq!(view.close_actions(), expected);
            assert_eq!(view.leave_request(), expected.first().filter(|action| **action != RoomAction::Hide).cloned());
        }
    }

    #[test]
    fn chat_edit_enter_100b1fb0_routes_say_and_build_commands() {
        let font = font();
        let mut view = room();
        assert!(view.key(crate::chat::InputKey::Char('a'), &font).is_none());
        view.mouse_down(10, 380, &font);
        assert!(view.edit.focused);
        for c in "oi".chars() {
            view.key(crate::chat::InputKey::Char(c), &font);
        }
        assert_eq!(view.key(crate::chat::InputKey::Enter, &font), Some(vec![RoomAction::Say { text: "oi".into() }]));
        assert_eq!(view.edit.get_text(), None);
        view.edit.set_text("//admin");
        assert_eq!(view.enter(), vec![RoomAction::Bypass("admin".into())]);
        view.edit.set_text("/target x");
        assert!(view.enter().is_empty());
        assert_eq!(view.edit.get_text(), None);
        assert!(view.enter().is_empty());
        assert_eq!(view.key(crate::chat::InputKey::Escape, &font), Some(Vec::new()));
        assert!(!view.edit.focused);
    }

    #[test]
    fn chat_list_receives_lines_and_close_packet_clears_it() {
        let font = font();
        let mut view = room();
        view.add_chat("a: b", crate::color::TEXT, &font);
        assert_eq!(view.chat.lines().len(), 1);
        view.mouse_down(10, 380, &font);
        view.close_packet();
        assert!(view.chat.lines().is_empty());
        assert!(!view.edit.focused);
    }

    #[test]
    fn on_paint_100b3490_info_texts_and_null_names() {
        let mut view = room();
        view.set_info(info());
        view.set_members(1, &[member(1, 1)]);
        assert_eq!(view.info_values(), ["7", "Sala", "s1050", "1/12", "s798", "20-60"].map(String::from));
        view.set_info(RoomInfo { location: 16, loot_type: 5, ..info() });
        assert_eq!(view.info_values()[2], NULL_TEXT);
        assert_eq!(view.info_values()[4], NULL_TEXT);
        let font = font();
        let mut list = DrawList { capture_text: true, ..Default::default() };
        view.set_info(info());
        view.list.select(0);
        view.paint(&mut list, 100, 50, &font, 1.0);
        let back = list.quads.iter().find(|q| q.texture == ROOM_BACK).unwrap();
        assert_eq!((back.x, back.y, back.w, back.h), (100.0, 70.0, 600.0, 411.0));
        let value = list.text_runs.iter().find(|t| t.text == "Sala").unwrap();
        assert_eq!((value.rect[0], value.rect[1]), (168.0, 106.0));
        let label = list.text_runs.iter().find(|t| t.text == "s1036").unwrap();
        assert_eq!(label.rect[0] as i32 + font.measure("s1036") as i32, 354);
        for text in ["m1", "40 ~ 49", "s1048", "s1062", "s393"] {
            assert!(list.text_runs.iter().any(|t| t.text == text), "{text}");
        }
        assert!(list.quads.iter().any(|q| q.texture.contains("party_styleicon")));
        let in_list = |list: &DrawList| list.quads.iter().filter(|q| q.texture.contains("ScrollBar") && q.y < 50.0 + ROOM_CHAT_RECT.y as f32).count();
        assert_eq!(in_list(&list), 2);
        assert!(!list.quads.iter().any(|q| q.texture == crate::scroll::SLIDER_CENTER));
        view.set_members(1, &(0..12).map(|id| member(id, 0)).collect::<Vec<_>>());
        let mut list = DrawList { capture_text: true, ..Default::default() };
        view.paint(&mut list, 100, 50, &font, 1.0);
        assert!(list.quads.iter().any(|q| q.texture == crate::scroll::SLIDER_CENTER));
    }

    fn sorted_names(view: &PartyRoomView) -> Vec<String> {
        view.members.iter().map(|member| member.name.clone()).collect()
    }

    #[test]
    fn header_click_sorts_by_the_column_cells_toggling_the_direction_each_time() {
        let mut view = room();
        let named = |id: i32, name: &str, class: i32, level: i32| MemberValues { id, name: name.into(), class_id: class, level, location: 1 + id % 3, role: 0 };
        view.set_members(
            1,
            &[named(1, "bob", 20, 45), named(2, "Alice", 5, 9), named(3, "carl", 100, 70), named(4, "Dan", 5, 33)],
        );
        view.list.select(2);
        view.sort_column(0);
        assert_eq!(sorted_names(&view), ["Alice", "bob", "carl", "Dan"]);
        assert_eq!(view.selected(), Some(2));
        view.sort_column(0);
        assert_eq!(sorted_names(&view), ["Dan", "carl", "bob", "Alice"]);
        assert_eq!(view.selected(), Some(1));
        view.sort_column(1);
        assert_eq!(view.members.iter().map(|m| m.class_id).collect::<Vec<_>>(), [5, 5, 20, 100]);
        view.sort_column(1);
        assert_eq!(view.members.iter().map(|m| m.class_id).collect::<Vec<_>>(), [100, 20, 5, 5]);
        view.sort_column(2);
        assert_eq!(view.members.iter().map(|m| m.level.as_str()).collect::<Vec<_>>(), ["1 ~ 9", "30 ~ 39", "40 ~ 49", "70 ~ 78"]);
        view.sort_column(2);
        assert_eq!(view.members.iter().map(|m| m.level.as_str()).collect::<Vec<_>>(), ["70 ~ 78", "40 ~ 49", "30 ~ 39", "1 ~ 9"]);
    }

    #[test]
    fn header_mouse_up_on_the_pressed_header_sorts_and_sends_nothing() {
        let font = font();
        let mut view = room();
        view.set_members(1, &[member(3, 0), member(1, 0), member(2, 0)]);
        let x = ROOM_LIST_RECT.x + 10;
        let y = ROOM_LIST_RECT.y + 5;
        assert_eq!(view.mouse_down(x, y, &font), Some(RoomHit::Header(0)));
        assert!(view.mouse_up(x, y).is_empty());
        assert_eq!(sorted_names(&view), ["m1", "m2", "m3"]);
    }

    #[test]
    fn row_tip_800000_shows_the_class_name_only_when_hovered_row_is_selected() {
        let font = font();
        let mut view = room();
        view.set_members(1, &[member(1, 0), member(2, 0)]);
        assert!(view.row_tooltip().is_none());
        view.list.select(1);
        let row_at = |index: i32| (ROOM_LIST_RECT.x + 20, ROOM_LIST_RECT.y + ROOM_HEADER_HEIGHT + ROOM_ROW_HEIGHT * index + 5);
        view.mouse_move(row_at(0).0, row_at(0).1, &font);
        assert!(view.row_tooltip().is_none());
        view.mouse_move(row_at(1).0, row_at(1).1, &font);
        let tip = view.row_tooltip().unwrap();
        assert_eq!(tip.lines.len(), 1);
        assert_eq!((tip.lines[0].label.as_str(), tip.lines[0].value.as_str()), ("s391", "s249"));
        assert_eq!(tip.size(&font), (crate::tooltip::ROW_WIDTH + crate::tooltip::BOX_MARGIN, font.line_height as i32 + crate::tooltip::LINE_GAP + crate::tooltip::BOX_MARGIN));
    }

    #[test]
    fn minimize_button_emits_the_minimize_action() {
        let font = font();
        let mut view = room();
        let hit = ROOM_FRAME.hit(&view.frame, (0, 0), (560, 10));
        assert!(matches!(hit, Some(crate::frame_contract::FrameHit::Minimize)));
        assert_eq!(view.mouse_down(560, 10, &font), Some(RoomHit::Minimize));
        assert_eq!(view.mouse_up(560, 10), vec![RoomAction::Minimize]);
    }

    #[test]
    fn list_wheel_and_scrollbar_follow_the_nclistctrl_rows() {
        let mut view = room();
        view.set_members(1, &(0..15).map(|id| member(id, 0)).collect::<Vec<_>>());
        assert!(view.wheel(20, 200, -120));
        assert_eq!(view.list.first_row(), 1);
        view.wheel(20, 200, -120 * 20);
        assert_eq!(view.list.first_row(), 6);
        view.wheel(20, 200, 120 * 20);
        assert_eq!(view.list.first_row(), 0);
        assert!(view.wheel(20, 310, -120));
        assert!(!view.wheel(20, 20, -120));
        let x = ROOM_LIST_RECT.x + ROOM_LIST_RECT.w - 5;
        let down_y = ROOM_LIST_RECT.y + ROOM_LIST_RECT.h - 5;
        assert_eq!(view.hit(x, down_y), Some(RoomHit::Scroll));
        view.mouse_down(x, down_y, &font());
        view.mouse_up(x, down_y);
        assert_eq!(view.list.first_row(), 1);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font(), 1.0);
        assert!(list.quads.iter().any(|q| q.texture.contains("ScrollBar") && q.y < ROOM_CHAT_RECT.y as f32));
    }
}
