use crate::button::TextButton;
use crate::chat::{CARET, CARET_BACK, InputKey, TextInput, wrap_line};
use crate::color::{Argb, TEXT};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;
use std::collections::HashMap;

pub const SETUP_RECT: Rect = Rect::new(24, 27, 256, 205);
pub const INFO_RECT: Rect = Rect::new(24, 249, 256, 205);
pub const BACK: &str = "L2UI_CH3.LobbyWnd.select2_back";
pub const STRIPE: &str = "L2UI_CH3.Etc.textbackline";
pub const NAME_EDIT_RECT: Rect = Rect::new(66, 17, 128, 17);
pub const LABEL_X: i32 = 0x21;
pub const LABEL_Y: [i32; 7] = [20, 55, 79, 103, 127, 151, 175];
pub const CONTROL_RECTS: [Rect; 6] = [
    Rect::new(86, 52, 136, 17),
    Rect::new(86, 76, 136, 17),
    Rect::new(86, 100, 136, 17),
    Rect::new(86, 124, 136, 17),
    Rect::new(86, 148, 136, 17),
    Rect::new(86, 172, 136, 17),
];
pub const CONTROL_TEXT_POS: (i32, i32) = (3, 3);
pub const CONTROL_BUTTON_SIZE: i32 = 15;
pub const CONTROL_TEXTBOX: [&str; 3] = ["L2ui_ch3.etc.textbox1", "L2ui_ch3.etc.textbox2", "L2ui_ch3.etc.textbox3"];
pub const PREV_TEXTURES: [&str; 2] = ["L2ui_ch3.Button.PREV1", "L2ui_ch3.Button.PREV1_down"];
pub const NEXT_TEXTURES: [&str; 2] = ["L2ui_ch3.Button.NEXT1", "L2ui_ch3.Button.NEXT1_down"];
pub const INFO_STRIPE_ROWS: [i32; 6] = [1, 3, 5, 7, 9, 11];
pub const INFO_TEXT_POS: (i32, i32) = (7, 7);
pub const INFO_TEXT_WIDTH: i32 = 242;
pub const ROUND_BUTTON: i32 = 44;
pub const ROUND_TEXTURES: [[&str; 2]; 4] = [
    ["l2ui.LobbyWnd.left", "l2ui.LobbyWnd.left_click"],
    ["l2ui.LobbyWnd.right", "l2ui.LobbyWnd.right_click"],
    ["l2ui.LobbyWnd.zoomin", "l2ui.LobbyWnd.zoomin_click"],
    ["l2ui.LobbyWnd.zoomout", "l2ui.LobbyWnd.zoomout_click"],
];
pub const FUNCTION_SIZE: (i32, i32) = (120, 138);
pub const FUNCTION_MARGIN: (i32, i32) = (154, 200);
pub const FUNCTION_BACK: &str = "L2UI_CH3.LobbyWnd.lobby_menuback";
pub const CREATE_RECT: Rect = Rect::new(13, 20, 96, 23);
pub const PREVIOUS_RECT: Rect = Rect::new(13, 45, 96, 23);
pub const BIG_BUTTON: &str = "L2UI_CH3.Button.BigButton";
pub const BIG_BUTTON_DOWN: &str = "L2UI_CH3.Button.BigButton_down";
pub const NAME_MAX_BYTES: usize = 16;
pub const NAME_MAX_BYTES_THAI: usize = 12;
pub const LANGUAGE_THAI: i32 = 5;

pub fn name_limit(language_type: i32) -> usize {
    if language_type == LANGUAGE_THAI { NAME_MAX_BYTES_THAI } else { NAME_MAX_BYTES }
}

pub fn ansi_length(text: &str, language_type: i32) -> usize {
    let double_byte = matches!(language_type, 0 | 2 | 3 | 4);
    text.chars()
        .map(|c| match c {
            '\0'..='\u{7f}' => 1,
            '\u{ff61}'..='\u{ff9f}' if language_type == 2 => 1,
            _ if double_byte => 2,
            _ => 1,
        })
        .sum()
}
pub const TURN_RATE: f32 = 2.0;
pub const SYSMSG_TOO_MANY: u32 = 77;
pub const SYSMSG_EXISTS: u32 = 79;
pub const SYSMSG_NAME_LENGTH: u32 = 80;
pub const SYSMSG_NO_RACE: u32 = 81;
pub const SYSMSG_NO_CLASS: u32 = 82;
pub const SYSMSG_NO_SEX: u32 = 83;
pub const SYSMSG_NO_STYLE: u32 = 91;
pub const SYSMSG_FAILED: u32 = 128;
pub const SYSMSG_BAD_NAME: u32 = 204;
pub const SYSMSG_NO_COLOR: u32 = 366;
pub const RACE_ITEMS: [u32; 5] = [170, 171, 172, 173, 174];
pub const CLASS_ITEMS: [u32; 2] = [175, 176];
pub const SEX_ITEMS: [u32; 2] = [177, 178];
pub const MALE_STYLES: [u32; 5] = [179, 180, 181, 182, 186];
pub const FEMALE_STYLES: [u32; 7] = [179, 180, 181, 182, 186, 187, 803];
pub const COLOR_ITEMS: [u32; 4] = [179, 180, 181, 182];
pub const FACE_ITEMS: [u32; 3] = [179, 180, 181];
pub const LABEL_SYSSTRINGS: [u32; 7] = [872, 164, 165, 166, 167, 168, 169];
pub const FUNCTION_SYSSTRINGS: [u32; 2] = [162, 161];
pub const DWARF: usize = 4;
pub const RACE_EVENT: [&str; 5] = ["HUMAN", "ELF", "DARKELF", "ORC", "DWARF"];
pub const CLASS_EVENT: [&str; 2] = ["KNIGHT", "WIZARD"];
pub const SEX_EVENT: [&str; 2] = ["MAN", "WOMAN"];
pub const CLASS_LETTER: [&str; 2] = ["_K", "_W"];
pub const CLASS_INFO_ID: [[u32; 2]; 5] = [[1, 2], [3, 4], [5, 6], [7, 8], [9, 0]];

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CreateTexts {
    pub items: HashMap<u32, String>,
    pub labels: [String; 7],
    pub create: String,
    pub previous: String,
    pub class_info: Vec<String>,
}

impl CreateTexts {
    pub fn from_tables(sysstring: impl Fn(u32) -> Option<String>, class_info: Vec<String>) -> CreateTexts {
        let text = |id: u32| sysstring(id).unwrap_or_default();
        let mut items = HashMap::new();
        for id in RACE_ITEMS.into_iter().chain(CLASS_ITEMS).chain(SEX_ITEMS).chain(FEMALE_STYLES) {
            items.insert(id, text(id));
        }
        CreateTexts {
            items,
            labels: LABEL_SYSSTRINGS.map(text),
            create: text(FUNCTION_SYSSTRINGS[0]),
            previous: text(FUNCTION_SYSSTRINGS[1]),
            class_info,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectCtrl {
    pub rect: Rect,
    pub items: Vec<u32>,
    pub index: Option<usize>,
    pub enabled: bool,
}

impl SelectCtrl {
    pub fn new(rect: Rect, items: &[u32], enabled: bool) -> SelectCtrl {
        SelectCtrl { rect, items: items.to_vec(), index: None, enabled }
    }

    pub fn step_back(&mut self) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }
        let next = match self.index {
            Some(index) if index > 0 => index - 1,
            _ => self.items.len() - 1,
        };
        self.index = Some(next);
        self.index
    }

    pub fn step_forward(&mut self) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }
        let next = match self.index {
            Some(index) if index + 1 < self.items.len() => index + 1,
            _ => 0,
        };
        self.index = Some(next);
        self.index
    }

    pub fn prev_rect(&self) -> Rect {
        Rect::new(self.rect.w - 33, 1, CONTROL_BUTTON_SIZE, CONTROL_BUTTON_SIZE)
    }

    pub fn next_rect(&self) -> Rect {
        Rect::new(self.rect.w - 15, 1, CONTROL_BUTTON_SIZE, CONTROL_BUTTON_SIZE)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    Name,
    Prev(usize),
    Next(usize),
    Turn(bool),
    Zoom,
    Create,
    Previous,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CreateAction {
    Message(u32),
    Changed,
    Camera { tag: String, reverse: bool },
    Look,
    Zoom,
    Turn(f32),
    Stop,
    Previous,
    Create { name: String, race: usize, sex: usize, class_id: i32, style: usize, color: usize, face: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateScreen {
    pub screen: (i32, i32),
    pub texts: CreateTexts,
    pub name: TextInput,
    pub name_focus: bool,
    pub controls: [SelectCtrl; 6],
    pub description: String,
    pub rotate_visible: bool,
    pub zoomed: bool,
    pub pressed: Option<Hit>,
    pub language_type: i32,
    pub camera_step: i32,
}

pub fn base_class_id(race: usize, class: Option<usize>) -> Option<i32> {
    let pick = |fighter: i32, mystic: i32| match class {
        Some(0) => Some(fighter),
        Some(1) => Some(mystic),
        _ => None,
    };
    match race {
        0 => pick(0, 10),
        1 => pick(18, 25),
        2 => pick(31, 38).or_else(|| pick(44, 49)).or(Some(53)),
        3 => pick(44, 49).or(Some(53)),
        4 => Some(53),
        _ => None,
    }
}

impl CreateScreen {
    pub fn new(screen: (i32, i32), texts: CreateTexts) -> CreateScreen {
        let controls = [
            SelectCtrl::new(CONTROL_RECTS[0], &RACE_ITEMS, true),
            SelectCtrl::new(CONTROL_RECTS[1], &CLASS_ITEMS, false),
            SelectCtrl::new(CONTROL_RECTS[2], &SEX_ITEMS, false),
            SelectCtrl::new(CONTROL_RECTS[3], &MALE_STYLES, false),
            SelectCtrl::new(CONTROL_RECTS[4], &COLOR_ITEMS, false),
            SelectCtrl::new(CONTROL_RECTS[5], &FACE_ITEMS, false),
        ];
        let description = texts.class_info.first().cloned().unwrap_or_default();
        CreateScreen { screen, texts, name: TextInput::default(), name_focus: true, controls, description, rotate_visible: false, zoomed: false, pressed: None, language_type: 1, camera_step: 0 }
    }

    pub fn round_height(&self) -> i32 {
        self.screen.1 - 52
    }

    pub fn rotate_rect(&self, right: bool) -> Rect {
        let x = (self.screen.0 - ROUND_BUTTON) / 2 + if right { 66 } else { -66 };
        Rect::new(x, self.round_height() - ROUND_BUTTON - 56, ROUND_BUTTON, ROUND_BUTTON)
    }

    pub fn zoom_rect(&self) -> Rect {
        Rect::new((self.screen.0 - ROUND_BUTTON) / 2, self.round_height() - 100, ROUND_BUTTON, ROUND_BUTTON)
    }

    pub fn function_origin(&self) -> (i32, i32) {
        (self.screen.0 - FUNCTION_MARGIN.0, self.screen.1 - FUNCTION_MARGIN.1)
    }

    pub fn clear_all(&mut self) {
        self.name = TextInput::default();
        for control in &mut self.controls {
            control.index = None;
        }
        self.rotate_visible = false;
        self.zoomed = false;
        self.camera_step = 0;
    }

    pub fn reset(&mut self) {
        self.clear_all();
        self.description = self.texts.class_info.first().cloned().unwrap_or_default();
    }

    pub fn race(&self) -> Option<usize> {
        self.controls[0].index
    }

    pub fn class(&self) -> Option<usize> {
        self.controls[1].index
    }

    pub fn sex(&self) -> Option<usize> {
        self.controls[2].index
    }

    fn set_enabled(&mut self, count: usize) {
        for (index, control) in self.controls.iter_mut().enumerate() {
            control.enabled = index < count;
        }
    }

    fn set_description(&mut self) {
        let text = match (self.race(), self.class()) {
            (Some(race), Some(class)) => CLASS_INFO_ID.get(race).and_then(|row| row.get(class)).copied().filter(|id| *id != 0).and_then(|id| self.texts.class_info.get(id as usize)),
            _ => None,
        };
        self.description = text.cloned().unwrap_or_default();
    }

    fn camera_event(&mut self, step: i32) -> Option<CreateAction> {
        let (race, class, sex) = (self.race(), self.class(), self.sex());
        let letter = |class: Option<usize>| class.map_or("", |class| CLASS_LETTER[class]);
        if step > self.camera_step {
            let mut event = String::new();
            if let Some(race) = race {
                event.push_str(RACE_EVENT[race]);
                self.camera_step = 1;
            }
            if let Some(class) = class {
                event.push('_');
                event.push_str(CLASS_EVENT[class]);
                self.camera_step = 2;
            }
            if let Some(sex) = sex {
                event.push_str(letter(class));
                event.push_str(SEX_EVENT[sex]);
                self.camera_step = 3;
            }
            return (!event.is_empty()).then_some(CreateAction::Camera { tag: event, reverse: false });
        }
        let race = race?;
        if step == self.camera_step {
            let tag = match step {
                1 => RACE_EVENT[race].to_string(),
                2 if race != DWARF => {
                    let class = class?;
                    format!("{}_{}_{}", RACE_EVENT[race], CLASS_EVENT[1 - class], CLASS_EVENT[class])
                }
                3 => {
                    let sex = sex?;
                    format!("{}{}{}{}{}", RACE_EVENT[race], letter(class), SEX_EVENT[1 - sex], letter(class), SEX_EVENT[sex])
                }
                _ => return None,
            };
            return Some(CreateAction::Camera { tag, reverse: false });
        }
        match step {
            1 => {
                self.camera_step = 1;
                Some(CreateAction::Camera { tag: RACE_EVENT[race].to_string(), reverse: false })
            }
            2 if race != DWARF => {
                let (class, sex) = (class?, sex?);
                self.camera_step = 2;
                Some(CreateAction::Camera { tag: format!("{}_{}{}{}", RACE_EVENT[race], CLASS_EVENT[class], CLASS_LETTER[1 - class], SEX_EVENT[sex]), reverse: true })
            }
            _ => None,
        }
    }

    pub fn choose_race(&mut self, index: usize) -> Vec<CreateAction> {
        self.controls[0].index = Some(index);
        for control in &mut self.controls[1..] {
            control.index = None;
        }
        self.controls[1].items = if index == DWARF { vec![CLASS_ITEMS[0]] } else { CLASS_ITEMS.to_vec() };
        self.set_enabled(2);
        self.rotate_visible = false;
        self.zoomed = false;
        self.description.clear();
        let mut actions: Vec<CreateAction> = self.camera_event(1).into_iter().collect();
        actions.push(CreateAction::Changed);
        actions
    }

    pub fn choose_class(&mut self, index: usize) -> Vec<CreateAction> {
        self.controls[1].index = Some(index);
        let camera = self.camera_event(2);
        for control in &mut self.controls[2..] {
            control.index = None;
        }
        if self.race().is_none() {
            self.controls[1].index = None;
            return camera.into_iter().chain([CreateAction::Message(SYSMSG_NO_RACE)]).collect();
        }
        self.set_enabled(3);
        self.rotate_visible = false;
        self.zoomed = false;
        self.set_description();
        camera.into_iter().chain([CreateAction::Changed]).collect()
    }

    pub fn choose_sex(&mut self, index: usize) -> Vec<CreateAction> {
        self.rotate_visible = false;
        self.zoomed = false;
        self.controls[3].items = if index == 0 { MALE_STYLES.to_vec() } else { FEMALE_STYLES.to_vec() };
        self.controls[2].index = Some(index);
        for control in &mut self.controls[3..] {
            control.index = Some(0);
        }
        if self.race().is_none() {
            self.clear_after_race_error();
            return vec![CreateAction::Message(SYSMSG_NO_RACE)];
        }
        if self.class().is_none() {
            for control in &mut self.controls[2..] {
                control.index = None;
            }
            return vec![CreateAction::Message(SYSMSG_NO_CLASS)];
        }
        self.set_enabled(6);
        self.rotate_visible = true;
        let mut actions: Vec<CreateAction> = self.camera_event(3).into_iter().collect();
        actions.push(CreateAction::Changed);
        actions
    }

    fn clear_after_race_error(&mut self) {
        for control in &mut self.controls {
            control.index = None;
        }
    }

    pub fn choose_look(&mut self, which: usize, index: usize) -> Vec<CreateAction> {
        self.controls[which].index = Some(index);
        let checks = [(self.race(), SYSMSG_NO_RACE), (self.class(), SYSMSG_NO_CLASS), (self.sex(), SYSMSG_NO_SEX)];
        for (value, message) in checks {
            if value.is_none() {
                return vec![CreateAction::Message(message)];
            }
        }
        if which >= 4 && self.controls[3].index.is_none() {
            return vec![CreateAction::Message(SYSMSG_NO_STYLE)];
        }
        if which == 5 && self.controls[4].index.is_none() {
            return vec![CreateAction::Message(SYSMSG_NO_COLOR)];
        }
        vec![CreateAction::Look]
    }

    fn picked(&mut self, which: usize, forward: bool) -> Vec<CreateAction> {
        let control = &mut self.controls[which];
        let Some(index) = (if forward { control.step_forward() } else { control.step_back() }) else { return Vec::new() };
        match which {
            0 => self.choose_race(index),
            1 => self.choose_class(index),
            2 => self.choose_sex(index),
            _ => self.choose_look(which, index),
        }
    }

    pub fn model_index(&self) -> Option<usize> {
        Some(self.race()? * 4 + self.class()? * 2 + self.sex()?)
    }

    pub fn tick_description(&mut self) {}

    pub fn fail(&mut self, code: i32, name_blocked: bool) -> Option<u32> {
        let _ = name_blocked;
        match code {
            0 => Some(SYSMSG_FAILED),
            1 => {
                self.clear_all();
                Some(SYSMSG_TOO_MANY)
            }
            2 => self.reject_name(SYSMSG_EXISTS),
            3 => self.reject_name(SYSMSG_NAME_LENGTH),
            4 => self.reject_name(SYSMSG_BAD_NAME),
            _ => None,
        }
    }

    fn reject_name(&mut self, message: u32) -> Option<u32> {
        self.name = TextInput::default();
        self.name_focus = true;
        Some(message)
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<Hit> {
        if self.rotate_visible {
            for (right, hit) in [(false, Hit::Turn(false)), (true, Hit::Turn(true))] {
                if self.rotate_rect(right).contains(x, y) {
                    return Some(hit);
                }
            }
            if self.zoom_rect().contains(x, y) {
                return Some(Hit::Zoom);
            }
        }
        let function = self.function_origin();
        if CREATE_RECT.offset(function.0, function.1).contains(x, y) {
            return Some(Hit::Create);
        }
        if PREVIOUS_RECT.offset(function.0, function.1).contains(x, y) {
            return Some(Hit::Previous);
        }
        let (lx, ly) = (x - SETUP_RECT.x, y - SETUP_RECT.y);
        if NAME_EDIT_RECT.contains(lx, ly) {
            return Some(Hit::Name);
        }
        for (index, control) in self.controls.iter().enumerate() {
            if !control.enabled {
                continue;
            }
            let (cx, cy) = (lx - control.rect.x, ly - control.rect.y);
            if control.prev_rect().contains(cx, cy) {
                return Some(Hit::Prev(index));
            }
            if control.next_rect().contains(cx, cy) {
                return Some(Hit::Next(index));
            }
        }
        None
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Vec<CreateAction> {
        self.pressed = self.hit(x, y);
        match self.pressed {
            Some(Hit::Name) => self.name_focus = true,
            Some(Hit::Turn(right)) => return vec![CreateAction::Turn(if right { -TURN_RATE } else { TURN_RATE })],
            _ => {}
        }
        Vec::new()
    }

    pub fn mouse_up(&mut self, x: i32, y: i32, name_blocked: impl Fn(&str) -> bool) -> Vec<CreateAction> {
        let Some(pressed) = self.pressed.take() else { return Vec::new() };
        if let Hit::Turn(_) = pressed {
            return vec![CreateAction::Stop];
        }
        if self.hit(x, y) != Some(pressed) {
            return Vec::new();
        }
        match pressed {
            Hit::Name | Hit::Turn(_) => Vec::new(),
            Hit::Prev(index) => self.picked(index, false),
            Hit::Next(index) => self.picked(index, true),
            Hit::Zoom => {
                self.zoomed = !self.zoomed;
                vec![CreateAction::Zoom]
            }
            Hit::Previous => vec![CreateAction::Previous],
            Hit::Create => self.create(name_blocked),
        }
    }

    pub fn key(&mut self, key: InputKey) {
        if self.name_focus {
            self.name.key(key);
        }
    }

    pub fn create(&mut self, name_blocked: impl Fn(&str) -> bool) -> Vec<CreateAction> {
        let length = ansi_length(&self.name.text, self.language_type);
        if length == 0 || length > name_limit(self.language_type) {
            self.name_focus = true;
            return vec![CreateAction::Message(SYSMSG_NAME_LENGTH)];
        }
        let Some(race) = self.race() else { return vec![CreateAction::Message(SYSMSG_NO_RACE)] };
        let Some(class) = base_class_id(race, self.class()) else { return vec![CreateAction::Message(SYSMSG_NO_CLASS)] };
        let Some(sex) = self.sex() else { return vec![CreateAction::Message(SYSMSG_NO_SEX)] };
        let Some(style) = self.controls[3].index else { return vec![CreateAction::Message(SYSMSG_NO_STYLE)] };
        if name_blocked(&self.name.text) {
            self.name = TextInput::default();
            self.name_focus = true;
            return vec![CreateAction::Message(SYSMSG_BAD_NAME)];
        }
        let color = self.controls[4].index.unwrap_or(0);
        let face = self.controls[5].index.unwrap_or(0);
        vec![CreateAction::Create { name: self.name.text.clone(), race, sex, class_id: class, style, color, face }]
    }

    pub fn paint(&self, list: &mut DrawList, font: &BitmapFont) {
        self.paint_setup(list, font);
        self.paint_info(list, font);
        if self.rotate_visible {
            let pressed = |hit: Hit| self.pressed == Some(hit);
            for (right, textures) in [(false, ROUND_TEXTURES[0]), (true, ROUND_TEXTURES[1])] {
                let rect = self.rotate_rect(right);
                let texture = textures[usize::from(pressed(Hit::Turn(right)))];
                list.draw_texture(rect.x, rect.y, rect.w, rect.h, 0, 0, 64, 64, texture, 1.0);
            }
            let rect = self.zoom_rect();
            let textures = if self.zoomed { ROUND_TEXTURES[3] } else { ROUND_TEXTURES[2] };
            list.draw_texture(rect.x, rect.y, rect.w, rect.h, 0, 0, 64, 64, textures[usize::from(pressed(Hit::Zoom))], 1.0);
        }
        let (fx, fy) = self.function_origin();
        list.draw_texture(fx, fy, FUNCTION_SIZE.0, FUNCTION_SIZE.1, 0, 0, FUNCTION_SIZE.0, FUNCTION_SIZE.1, FUNCTION_BACK, 1.0);
        TextButton::new(CREATE_RECT, BIG_BUTTON, BIG_BUTTON_DOWN, &self.texts.create).paint(list, fx, fy, self.pressed == Some(Hit::Create), font);
        TextButton::new(PREVIOUS_RECT, BIG_BUTTON, BIG_BUTTON_DOWN, &self.texts.previous).paint(list, fx, fy, self.pressed == Some(Hit::Previous), font);
    }

    fn paint_setup(&self, list: &mut DrawList, font: &BitmapFont) {
        let (ox, oy) = (SETUP_RECT.x, SETUP_RECT.y);
        list.draw_texture(ox, oy, SETUP_RECT.w, SETUP_RECT.h, 0, 0, SETUP_RECT.w, SETUP_RECT.h, BACK, 1.0);
        for (index, y) in LABEL_Y.iter().enumerate() {
            list.draw_text(ox + LABEL_X, oy + y, TEXT, &self.texts.labels[index], font);
        }
        crate::lobby_login::paint_edit_back(list, Rect::new(ox + NAME_EDIT_RECT.x, oy + NAME_EDIT_RECT.y, NAME_EDIT_RECT.w, NAME_EDIT_RECT.h));
        let (nx, ny) = (ox + NAME_EDIT_RECT.x + 2, oy + NAME_EDIT_RECT.y + 2);
        list.draw_text_clipped(nx, ny, NAME_EDIT_RECT.w - 2, TEXT, &self.name.text, font);
        if self.name_focus {
            let caret = nx + font.measure(self.name.before_cursor()) as i32 - CARET_BACK;
            list.draw_text(caret, ny, TEXT, &CARET.to_string(), font);
        }
        for (index, control) in self.controls.iter().enumerate() {
            self.paint_control(list, font, control, (ox, oy), index);
        }
    }

    fn paint_control(&self, list: &mut DrawList, font: &BitmapFont, control: &SelectCtrl, origin: (i32, i32), index: usize) {
        let (x, y) = (origin.0 + control.rect.x, origin.1 + control.rect.y);
        let w = control.rect.w;
        list.draw_texture(x, y, 8, 17, 0, 0, 8, 17, CONTROL_TEXTBOX[0], 1.0);
        list.draw_texture(x + 8, y, w - 38 - 8, 17, 0, 0, 8, 17, CONTROL_TEXTBOX[1], 1.0);
        list.draw_texture(x + w - 38, y, 8, 17, 0, 0, 8, 17, CONTROL_TEXTBOX[2], 1.0);
        if let Some(item) = control.index.and_then(|i| control.items.get(i)) {
            let text = self.texts.items.get(item).cloned().unwrap_or_default();
            list.draw_text(x + CONTROL_TEXT_POS.0, y + CONTROL_TEXT_POS.1, TEXT, &text, font);
        }
        for (rect, textures, hit) in [(control.prev_rect(), PREV_TEXTURES, Hit::Prev(index)), (control.next_rect(), NEXT_TEXTURES, Hit::Next(index))] {
            let texture = textures[usize::from(self.pressed == Some(hit))];
            list.draw_texture(x + rect.x, y + rect.y, rect.w, rect.h, 0, 0, 16, 16, texture, 1.0);
        }
    }

    fn paint_info(&self, list: &mut DrawList, font: &BitmapFont) {
        let (ox, oy) = (INFO_RECT.x, INFO_RECT.y);
        list.draw_texture(ox, oy, INFO_RECT.w, INFO_RECT.h, 0, 0, INFO_RECT.w, INFO_RECT.h, BACK, 1.0);
        for row in INFO_STRIPE_ROWS {
            list.draw_texture(ox + 5, oy + row * 15 + 5, INFO_RECT.w - 10, 15, 0, 0, 8, 15, STRIPE, 1.0);
        }
        let color: Argb = TEXT;
        let rows = self.description.split('\n').flat_map(|paragraph| wrap_line(paragraph.trim_end_matches('\r'), INFO_TEXT_WIDTH, 0, font));
        for (index, (_, line)) in rows.enumerate() {
            list.draw_text(ox + INFO_TEXT_POS.0, oy + INFO_TEXT_POS.1 + index as i32 * font.line_height as i32, color, &line, font);
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

    #[test]
    fn a_description_with_line_breaks_draws_each_line_on_its_own_row() {
        let mut screen = screen();
        screen.description = "primeira\nsegunda\n\nquarta".into();
        let mut list = DrawList { capture_text: true, ..Default::default() };
        screen.paint_info(&mut list, &font());
        let texts: Vec<&str> = list.text_runs.iter().map(|run| run.text.as_str()).collect();
        assert_eq!(texts, ["primeira", "segunda", "quarta"]);
        let ys: Vec<f32> = list.text_runs.iter().map(|run| run.rect[1]).collect();
        assert!(ys[1] > ys[0] && ys[2] > ys[1] + font().line_height as f32);
        assert!(list.text_runs.iter().all(|run| !run.text.contains('\n')));
    }

    fn screen() -> CreateScreen {
        let info: Vec<String> = (0..10).map(|i| format!("classe {i}")).collect();
        CreateScreen::new((1024, 768), CreateTexts::from_tables(|id| Some(format!("s{id}")), info))
    }

    fn camera(tag: &str, reverse: bool) -> CreateAction {
        CreateAction::Camera { tag: tag.into(), reverse }
    }

    fn cameras(actions: Vec<CreateAction>) -> Vec<(String, bool)> {
        actions.into_iter().filter_map(|action| match action { CreateAction::Camera { tag, reverse } => Some((tag, reverse)), _ => None }).collect()
    }

    #[test]
    fn the_camera_events_follow_execcameraevent_100cdee0_with_its_camera_step() {
        let mut screen = screen();
        assert_eq!(cameras(screen.choose_race(1)), [("ELF".to_string(), false)]);
        assert_eq!(cameras(screen.choose_class(0)), [("ELF_KNIGHT".to_string(), false)]);
        assert_eq!(cameras(screen.choose_class(1)), [("ELF_KNIGHT_WIZARD".to_string(), false)]);
        assert_eq!(cameras(screen.choose_sex(1)), [("ELF_WIZARD_WWOMAN".to_string(), false)]);
        assert_eq!(screen.camera_step, 3);
        assert_eq!(cameras(screen.choose_sex(0)), [("ELF_WWOMAN_WMAN".to_string(), false)]);
        assert_eq!(cameras(screen.choose_class(0)), [("ELF_KNIGHT_WMAN".to_string(), true)]);
        assert_eq!(screen.camera_step, 2);
        assert_eq!(cameras(screen.choose_sex(1)), [("ELF_KNIGHT_KWOMAN".to_string(), false)]);
        assert_eq!(cameras(screen.choose_race(0)), [("HUMAN".to_string(), false)]);
        assert_eq!(screen.camera_step, 1);
    }

    #[test]
    fn the_dwarf_class_step_plays_nothing_and_keeps_the_camera_step_above_it() {
        let mut screen = screen();
        screen.choose_race(DWARF);
        assert_eq!(cameras(screen.choose_class(0)), [("DWARF_KNIGHT".to_string(), false)]);
        assert_eq!(cameras(screen.choose_sex(0)), [("DWARF_KNIGHT_KMAN".to_string(), false)]);
        assert_eq!(cameras(screen.choose_class(0)), []);
        assert_eq!(screen.camera_step, 3);
        assert_eq!(cameras(screen.choose_sex(1)), [("DWARF_KMAN_KWOMAN".to_string(), false)]);
    }

    fn press(screen: &mut CreateScreen, control: usize, forward: bool) -> Vec<CreateAction> {
        let rect = screen.controls[control].rect;
        let button = if forward { screen.controls[control].next_rect() } else { screen.controls[control].prev_rect() };
        let (x, y) = (SETUP_RECT.x + rect.x + button.x + 1, SETUP_RECT.y + rect.y + button.y + 1);
        screen.mouse_down(x, y);
        screen.mouse_up(x, y, |_| false)
    }

    #[test]
    fn controls_are_revealed_one_step_at_a_time_and_the_race_resets_the_rest() {
        let mut screen = screen();
        assert_eq!(screen.controls.iter().filter(|c| c.enabled).count(), 1);
        assert_eq!(press(&mut screen, 0, true), vec![camera("HUMAN", false), CreateAction::Changed]);
        assert_eq!(screen.race(), Some(0));
        assert_eq!(screen.controls.iter().filter(|c| c.enabled).count(), 2);
        press(&mut screen, 1, true);
        assert_eq!(screen.controls.iter().filter(|c| c.enabled).count(), 3);
        assert_eq!(screen.description, "classe 1");
        press(&mut screen, 2, true);
        assert_eq!(screen.controls.iter().filter(|c| c.enabled).count(), 6);
        assert!(screen.rotate_visible);
        assert_eq!(screen.model_index(), Some(0));
        press(&mut screen, 0, true);
        assert_eq!(screen.race(), Some(1));
        assert_eq!(screen.class(), None);
        assert_eq!(screen.controls.iter().filter(|c| c.enabled).count(), 2);
        assert!(!screen.rotate_visible);
    }

    #[test]
    fn the_dwarf_only_has_the_fighter_and_the_sex_picks_five_or_seven_styles() {
        let mut screen = screen();
        screen.choose_race(DWARF);
        assert_eq!(screen.controls[1].items, vec![175]);
        screen.choose_class(0);
        screen.choose_sex(0);
        assert_eq!(screen.controls[3].items.len(), 5);
        assert_eq!((screen.controls[3].index, screen.controls[4].index, screen.controls[5].index), (Some(0), Some(0), Some(0)));
        screen.choose_sex(1);
        assert_eq!(screen.controls[3].items.len(), 7);
        assert_eq!(screen.description, "classe 9");
    }

    #[test]
    fn select_controls_wrap_around_and_start_from_the_ends_when_empty() {
        let mut control = SelectCtrl::new(CONTROL_RECTS[0], &RACE_ITEMS, true);
        assert_eq!(control.step_back(), Some(4));
        control.index = None;
        assert_eq!(control.step_forward(), Some(0));
        assert_eq!(control.step_back(), Some(4));
        assert_eq!(control.step_forward(), Some(0));
    }

    #[test]
    fn choosing_a_class_without_a_race_asks_for_the_race_with_message_81() {
        let mut screen = screen();
        assert_eq!(screen.choose_class(1), vec![camera("_WIZARD", false), CreateAction::Message(SYSMSG_NO_RACE)]);
        assert_eq!(screen.class(), None);
    }

    #[test]
    fn create_validates_in_the_order_of_cdc30_and_sends_the_class_id() {
        let mut screen = screen();
        assert_eq!(screen.create(|_| false), vec![CreateAction::Message(SYSMSG_NAME_LENGTH)]);
        for c in "Heroi".chars() {
            screen.key(InputKey::Char(c));
        }
        assert_eq!(screen.create(|_| false), vec![CreateAction::Message(SYSMSG_NO_RACE)]);
        screen.choose_race(1);
        assert_eq!(screen.create(|_| false), vec![CreateAction::Message(SYSMSG_NO_CLASS)]);
        screen.choose_class(1);
        assert_eq!(screen.create(|_| false), vec![CreateAction::Message(SYSMSG_NO_SEX)]);
        screen.choose_sex(1);
        screen.choose_look(3, 2);
        assert_eq!(screen.create(|_| true), vec![CreateAction::Message(SYSMSG_BAD_NAME)]);
        assert!(screen.name.text.is_empty());
        for c in "Heroi".chars() {
            screen.key(InputKey::Char(c));
        }
        assert_eq!(
            screen.create(|_| false),
            vec![CreateAction::Create { name: "Heroi".into(), race: 1, sex: 1, class_id: 25, style: 2, color: 0, face: 0 }]
        );
        screen.name = TextInput { text: "x".repeat(17), cursor: 17 };
        assert_eq!(screen.create(|_| false), vec![CreateAction::Message(SYSMSG_NAME_LENGTH)]);
    }

    #[test]
    fn the_name_is_measured_in_ansi_bytes_with_the_thai_limit_of_12() {
        assert_eq!(ansi_length("Heroi", 1), 5);
        assert_eq!(ansi_length("\u{c9}a", 1), 2);
        assert_eq!(ansi_length("\u{4e2d}a", 4), 3);
        assert_eq!(ansi_length("\u{ff76}", 2), 1);
        assert_eq!((name_limit(1), name_limit(5)), (16, 12));
        let mut screen = screen();
        screen.language_type = LANGUAGE_THAI;
        screen.name = TextInput { text: "x".repeat(13), cursor: 13 };
        assert_eq!(screen.create(|_| false), vec![CreateAction::Message(SYSMSG_NAME_LENGTH)]);
        screen.language_type = 4;
        screen.name = TextInput { text: "\u{4e2d}".repeat(9), cursor: 9 };
        assert_eq!(screen.create(|_| false), vec![CreateAction::Message(SYSMSG_NAME_LENGTH)]);
    }

    #[test]
    fn base_class_id_keeps_the_fall_through_of_100cc585_dark_elves_orcs_and_dwarves_without_a_class_type_get_53() {
        assert_eq!(base_class_id(0, None), None);
        assert_eq!(base_class_id(1, None), None);
        assert_eq!(base_class_id(2, None), Some(53));
        assert_eq!(base_class_id(3, None), Some(53));
        assert_eq!(base_class_id(4, None), Some(53));
        assert_eq!(base_class_id(2, Some(1)), Some(38));
        assert_eq!(base_class_id(3, Some(0)), Some(44));
        assert_eq!(base_class_id(5, Some(0)), None);
        let mut screen = screen();
        for c in "Heroi".chars() {
            screen.key(InputKey::Char(c));
        }
        screen.choose_race(2);
        assert_eq!(screen.create(|_| false), vec![CreateAction::Message(SYSMSG_NO_SEX)]);
        screen.choose_race(1);
        assert_eq!(screen.create(|_| false), vec![CreateAction::Message(SYSMSG_NO_CLASS)]);
    }

    #[test]
    fn creation_failures_map_to_the_messages_of_10065a50_and_clear_what_the_original_clears() {
        let mut screen = screen();
        for c in "Nome".chars() {
            screen.key(InputKey::Char(c));
        }
        screen.choose_race(0);
        assert_eq!(screen.fail(0, false), Some(SYSMSG_FAILED));
        assert_eq!(screen.fail(2, false), Some(SYSMSG_EXISTS));
        assert!(screen.name.text.is_empty());
        assert_eq!(screen.fail(1, false), Some(SYSMSG_TOO_MANY));
        assert_eq!(screen.race(), None);
        assert_eq!(screen.fail(3, false), Some(SYSMSG_NAME_LENGTH));
        assert_eq!(screen.fail(4, false), Some(SYSMSG_BAD_NAME));
        assert_eq!(screen.fail(9, false), None);
    }

    #[test]
    fn rotate_zoom_and_function_buttons_follow_the_anchor_positions() {
        let screen = screen();
        assert_eq!(screen.rotate_rect(false), Rect::new(424, 616, 44, 44));
        assert_eq!(screen.zoom_rect(), Rect::new(490, 616, 44, 44));
        assert_eq!(screen.rotate_rect(true), Rect::new(556, 616, 44, 44));
        assert_eq!(screen.function_origin(), (870, 568));
    }

    #[test]
    fn turning_starts_on_press_and_stops_on_release_and_zoom_toggles() {
        let mut screen = screen();
        screen.choose_race(0);
        screen.choose_class(0);
        screen.choose_sex(0);
        let rect = screen.rotate_rect(false);
        let point = (rect.x + 5, rect.y + 5);
        assert_eq!(screen.mouse_down(point.0, point.1), vec![CreateAction::Turn(TURN_RATE)]);
        assert_eq!(screen.mouse_up(point.0, point.1, |_| false), vec![CreateAction::Stop]);
        let zoom = screen.zoom_rect();
        screen.mouse_down(zoom.x + 5, zoom.y + 5);
        assert_eq!(screen.mouse_up(zoom.x + 5, zoom.y + 5, |_| false), vec![CreateAction::Zoom]);
        assert!(screen.zoomed);
    }

    #[test]
    fn painting_shows_the_labels_the_wrapped_class_text_and_the_selected_item_names() {
        let mut screen = screen();
        screen.choose_race(2);
        screen.controls[0].index = Some(2);
        let mut list = DrawList { capture_text: true, ..Default::default() };
        screen.paint(&mut list, &font());
        let texts: Vec<&str> = list.text_runs.iter().map(|run| run.text.as_str()).collect();
        assert!(texts.contains(&"s872") && texts.contains(&"s164"));
        assert!(texts.contains(&"s172"));
        assert!(texts.contains(&"s162") && texts.contains(&"s161"));
    }
}
