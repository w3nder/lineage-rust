#[path = "gm_event_controls_generated.rs"]
mod controls;
#[path = "gm_event_layout_generated.rs"]
mod layout;

use std::collections::HashMap;

use crate::button::TextButton;
use crate::chat::InputKey;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::edit_box::{EditBox, FontMetrics};
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::ini::IniFile;
use crate::list_ctrl::{ListCell, ListCtrl, ListRow};
use crate::window::{Rect, WindowId};

pub const ID: WindowId = WindowId::GM_EVENT;
pub const SIZE: (i32, i32) = layout::SIZE;
pub const BACK: &str = "L2UI_CH3.Minimap.MapBack";
pub const BACK_ORIGIN: (i32, i32) = (0, 20);
pub const BACK_SOURCE: (i32, i32) = (0x100, 0x6c);
pub const TEXT_COLOR: Argb = Argb(0xFFDCDCDC);
pub const COMMAND_PREFIX: &str = "eventmatch";
pub const NO_PLAYER: i32 = -1;
pub const SLOTS: usize = 9;
pub const CLASS_NAME_BASE: u32 = 247;
pub const TIMER_MS: u32 = 1000;
pub const COUNTDOWN_START: i32 = 4;
pub const INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 2, x: 700, y: 500 };

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 * 0.5 - 350.0) as i32, (viewport.1 as f32 * 0.5 - 250.0) as i32)
}

pub fn grip() -> Rect {
    layout::RECT_7C
}

pub const L_CREATE: u32 = 1068;
pub const L_REMOVE: u32 = 1069;
pub const L_LOCK: u32 = 1071;
pub const L_LIFT: u32 = 1072;
pub const L_PAUSE: u32 = 1073;
pub const L_RESUME: u32 = 1074;
pub const L_START: u32 = 1075;
pub const L_END: u32 = 1076;
pub const L_FENCE_UP: u32 = 1081;
pub const L_FENCE_DOWN: u32 = 1082;
pub const L_LOCKING: u32 = 1097;
pub const L_REMOVING: u32 = 1098;
pub const L_CREATING: u32 = 1099;
pub const L_STOPPING: u32 = 1100;
pub const L_RESUMING: u32 = 1101;
pub const L_STARTING: u32 = 1102;
pub const L_ENDING: u32 = 1103;

pub const SYSSTRINGS: [u32; 36] = [
    1068, 1069, 1070, 1071, 1072, 1073, 1074, 1075, 1076, 1077, 1078, 1079, 1080, 1081, 1082, 1083, 1084, 1085, 1086, 1087, 1088, 1089,
    1090, 1091, 1092, 1093, 1094, 1095, 1096, 1097, 1098, 1099, 1100, 1101, 1102, 1103,
];
pub const TEXTS: [(u32, (i32, i32)); 12] = [
    (1085, (25, 35)),
    (1086, (265, 35)),
    (1087, (25, 90)),
    (1088, (265, 90)),
    (1089, (25, 150)),
    (1090, (350, 150)),
    (1091, (25, 235)),
    (1092, (355, 235)),
    (1093, (25, 420)),
    (1094, (80, 420)),
    (1095, (150, 420)),
    (1096, (25, 450)),
];
pub const GAME_ID_LABEL: u32 = 1107;
pub const GAME_ID_POSITION: (i32, i32) = (495, 55);
pub const COLON_POSITION: (i32, i32) = (140, 420);
pub const MESSAGE_COLON_POSITION: (i32, i32) = (25, 450);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Btn {
    Command,
    Select1,
    Lock1,
    Select2,
    Lock2,
    Forward,
    Start,
    Create,
    Pause,
    Apply,
    Ending,
    Fence,
    Fire1,
    Fire2,
}

pub const BUTTONS: [Btn; 14] = [
    Btn::Command,
    Btn::Select1,
    Btn::Lock1,
    Btn::Select2,
    Btn::Lock2,
    Btn::Forward,
    Btn::Start,
    Btn::Create,
    Btn::Pause,
    Btn::Apply,
    Btn::Ending,
    Btn::Fence,
    Btn::Fire1,
    Btn::Fire2,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ed {
    Team1,
    Team2,
    Score1,
    Score2,
    Config,
    Commands,
    Leader1,
    Leader2,
    Message,
}

pub const EDITS: [Ed; 9] = [Ed::Team1, Ed::Team2, Ed::Score1, Ed::Score2, Ed::Config, Ed::Commands, Ed::Leader1, Ed::Leader2, Ed::Message];

fn button_contract(button: Btn) -> &'static controls::TextButtonContract {
    match button {
        Btn::Command => &controls::BUTTON_110,
        Btn::Select1 => &controls::BUTTON_114,
        Btn::Lock1 => &controls::BUTTON_118,
        Btn::Select2 => &controls::BUTTON_11C,
        Btn::Lock2 => &controls::BUTTON_120,
        Btn::Forward => &controls::BUTTON_124,
        Btn::Start => &controls::BUTTON_128,
        Btn::Create => &controls::BUTTON_12C,
        Btn::Pause => &controls::BUTTON_130,
        Btn::Apply => &controls::BUTTON_134,
        Btn::Ending => &controls::BUTTON_138,
        Btn::Fence => &controls::BUTTON_13C,
        Btn::Fire1 => &controls::BUTTON_140,
        Btn::Fire2 => &controls::BUTTON_144,
    }
}

fn edit_contract(edit: Ed) -> &'static crate::child_contract::ChildContract {
    match edit {
        Ed::Team1 => &controls::CHILD_EC,
        Ed::Team2 => &controls::CHILD_F0,
        Ed::Score1 => &controls::CHILD_F4,
        Ed::Score2 => &controls::CHILD_F8,
        Ed::Config => &controls::CHILD_FC,
        Ed::Commands => &controls::CHILD_100,
        Ed::Leader1 => &controls::CHILD_104,
        Ed::Leader2 => &controls::CHILD_108,
        Ed::Message => &controls::CHILD_10C,
    }
}

fn edit_limit(edit: Ed) -> Option<i32> {
    match edit {
        Ed::Team1 | Ed::Team2 => Some(10),
        Ed::Score1 | Ed::Score2 => Some(1),
        Ed::Message => Some(0x32),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Slot {
    pub object_id: i32,
    pub name: String,
    pub level: i32,
    pub class_id: i32,
    pub values: [i32; 10],
}

impl Default for Slot {
    fn default() -> Self {
        Self { object_id: NO_PLAYER, name: String::new(), level: 0, class_id: 0, values: [0; 10] }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct RaceOptions {
    pub buffs: Vec<(i32, i32)>,
    pub skill_default_allow: bool,
    pub skills: Vec<i32>,
    pub item_default_allow: bool,
    pub items: Vec<i32>,
    pub teleport_delay: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GmEventAction {
    Command(String),
    Notice(u32, Option<i32>),
    SelectTarget(usize),
    ReadOptions(String),
    ReadCommandFile(String),
    OpenFence,
    Close,
    Minimize,
}

pub fn option_error(id: u32, index: Option<i32>) -> GmEventAction {
    GmEventAction::Notice(id, index)
}

fn config_space(unit: u16) -> bool {
    matches!(unit, 9..=13 | 32 | 160)
}

fn config_int(ini: &IniFile, section: &str, key: &str) -> Option<i32> {
    let units: Vec<u16> = ini.get(section, key)?.encode_utf16().collect();
    Some(crate::ini_value::wide_atoi(&units, config_space))
}

fn config_bool(ini: &IniFile, section: &str, key: &str) -> Option<bool> {
    let units: Option<Vec<u16>> = ini.get(section, key).map(|value| value.encode_utf16().collect());
    crate::ini_value::config_bool(
        units.as_deref(),
        |value| value.iter().copied().map(|unit| if (65..=90).contains(&unit) { unit + 32 } else { unit }).eq("true".encode_utf16()),
        config_space,
    )
}

pub fn read_options(ini: &IniFile) -> Result<RaceOptions, GmEventAction> {
    let int = |section: &str, key: &str, error: u32, index: Option<i32>| -> Result<i32, GmEventAction> {
        config_int(ini, section, key).ok_or(GmEventAction::Notice(error, index))
    };
    let boolean = |section: &str, key: &str, error: u32| -> Result<bool, GmEventAction> {
        config_bool(ini, section, key).ok_or(GmEventAction::Notice(error, None))
    };
    let mut options = RaceOptions::default();
    let count = int("Buff", "BuffCnt", 1422, None)?;
    for index in 0..count {
        let id = int("Buff", &format!("BuffID{index}"), 1423, Some(index))?;
        let level = int("Buff", &format!("BuffLv{index}"), 1424, Some(index))?;
        options.buffs.push((id, level));
    }
    options.skill_default_allow = boolean("Skill", "DefaultAllow", 1425)?;
    let count = int("Skill", "ExpSkillCnt", 1426, None)?;
    for index in 0..count {
        options.skills.push(int("Skill", &format!("ExpSkillID{index}"), 1427, Some(index))?);
    }
    options.item_default_allow = boolean("Item", "DefaultAllow", 1425)?;
    let count = int("Item", "ExpItemCnt", 1428, None)?;
    for index in 0..count {
        options.items.push(int("Item", &format!("ExpItemID{index}"), 1429, Some(index))?);
    }
    options.teleport_delay = int("Etc", "TeleportDelay", 1430, None)?;
    Ok(options)
}

pub fn command_html(ini: &IniFile) -> String {
    let count = config_int(ini, "Cmd", "CmdCnt").unwrap_or(0);
    let mut html = String::from("<html><body>");
    for index in 0..count {
        let command = ini.get("Cmd", &format!("Cmd{index}")).unwrap_or_default();
        html.push_str(&format!("<a cmd = \"{command}\">{command}</a><br1>"));
    }
    html.push_str("</body></html>");
    html
}

#[derive(Clone, Debug)]
pub struct GmEventView {
    pub frame: FrameCtrl,
    pub edits: Vec<EditBox>,
    pub buttons: Vec<TextButton>,
    pub label_ids: [u32; 14],
    pub lists: [ListCtrl; 2],
    pub game_id: String,
    pub slots: [[Slot; SLOTS]; 2],
    pub current_team: Option<usize>,
    pub options: RaceOptions,
    pub countdown: Option<(i32, u32)>,
    labels: HashMap<u32, String>,
    game_id_label: String,
    texts: Vec<String>,
    pressed: Option<GmHit>,
    focus: Option<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GmHit {
    Close,
    Minimize,
    Button(usize),
    Edit(usize),
    List,
}

fn button_index(button: Btn) -> usize {
    BUTTONS.iter().position(|&known| known == button).unwrap_or(0)
}

fn edit_index(edit: Ed) -> usize {
    EDITS.iter().position(|&known| known == edit).unwrap_or(0)
}

impl GmEventView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        let labels = SYSSTRINGS.iter().map(|&id| (id, text(id))).collect();
        let buttons: Vec<TextButton> =
            BUTTONS.iter().map(|&button| button_contract(button).build((0, 0), text)).collect();
        let label_ids = BUTTONS.map(|button| button_contract(button).sysstring);
        let mut view = Self {
            frame: controls::FRAME_7C.build(text),
            edits: EDITS
                .iter()
                .map(|&edit| {
                    let mut box_ = EditBox::from_contract(edit_contract(edit));
                    if let Some(limit) = edit_limit(edit) {
                        box_.set_max_len(limit);
                    }
                    box_
                })
                .collect(),
            buttons,
            label_ids,
            lists: [ListCtrl::from_contract(&controls::LIST_148), ListCtrl::from_contract(&controls::LIST_14C)],
            game_id: String::new(),
            slots: Default::default(),
            current_team: None,
            options: RaceOptions::default(),
            countdown: None,
            labels,
            game_id_label: text(GAME_ID_LABEL),
            texts: TEXTS.iter().map(|(id, _)| text(*id)).collect(),
            pressed: None,
            focus: None,
        };
        for button in BUTTONS {
            if !matches!(button, Btn::Command | Btn::Create) {
                view.set_enabled(button, false);
            }
        }
        view
    }

    pub fn label(&self, id: u32) -> &str {
        self.labels.get(&id).map_or("", String::as_str)
    }

    pub fn set_label(&mut self, button: Btn, id: u32) {
        let index = button_index(button);
        self.label_ids[index] = id;
        self.buttons[index].label = self.label(id).to_string();
    }

    pub fn label_of(&self, button: Btn) -> u32 {
        self.label_ids[button_index(button)]
    }

    pub fn set_enabled(&mut self, button: Btn, enabled: bool) {
        self.buttons[button_index(button)].enabled = enabled;
    }

    pub fn enabled(&self, button: Btn) -> bool {
        self.buttons[button_index(button)].enabled
    }

    pub fn edit_text(&self, edit: Ed) -> String {
        self.edits[edit_index(edit)].get_text().unwrap_or_default()
    }

    pub fn set_edit(&mut self, edit: Ed, text: &str, font: &BitmapFont) {
        let target = &mut self.edits[edit_index(edit)];
        target.clear();
        target.insert_text(text, true, &FontMetrics(font));
    }

    #[cfg(test)]
    fn clear_edit(&mut self, edit: Ed) {
        self.edits[edit_index(edit)].clear();
    }

    pub fn clear_slots(&mut self) {
        self.slots = Default::default();
    }

    fn command(&self, rest: &str) -> GmEventAction {
        GmEventAction::Command(format!("{COMMAND_PREFIX} {rest}"))
    }

    fn game_command(&self, rest: &str) -> GmEventAction {
        self.command(rest)
    }

    pub fn press(&mut self, button: Btn, _font: &BitmapFont) -> Vec<GmEventAction> {
        let id = self.game_id.clone();
        match button {
            Btn::Command => {
                let file = self.edit_text(Ed::Commands);
                if file.is_empty() { vec![GmEventAction::Notice(1415, None)] } else { vec![GmEventAction::ReadCommandFile(file)] }
            }
            Btn::Select1 => vec![GmEventAction::SelectTarget(0)],
            Btn::Select2 => vec![GmEventAction::SelectTarget(1)],
            Btn::Lock1 => self.lock(0),
            Btn::Lock2 => self.lock(1),
            Btn::Forward => vec![self.game_command(&format!("msg {id} 0 \"{}\"", self.edit_text(Ed::Message)))],
            Btn::Ending => vec![self.game_command(&format!("msg {id} 3 NULL"))],
            Btn::Apply => {
                vec![self.game_command(&format!("score {id} {} {}", self.edit_text(Ed::Score1), self.edit_text(Ed::Score2)))]
            }
            Btn::Fence => {
                if self.label_of(Btn::Fence) == L_FENCE_UP {
                    self.set_label(Btn::Fence, L_FENCE_DOWN);
                    vec![self.game_command(&format!("fence {id} 2"))]
                } else if self.label_of(Btn::Fence) == L_FENCE_DOWN {
                    self.set_label(Btn::Fence, L_FENCE_UP);
                    vec![self.game_command(&format!("fence {id} 1"))]
                } else {
                    Vec::new()
                }
            }
            Btn::Fire1 => self.firecrackers(0),
            Btn::Fire2 => self.firecrackers(1),
            Btn::Pause => {
                if self.label_of(Btn::Pause) == L_PAUSE {
                    self.set_label(Btn::Pause, L_STOPPING);
                    self.set_enabled(Btn::Pause, false);
                    let action = self.game_command(&format!("pause {id}"));
                    self.set_enabled(Btn::Pause, true);
                    self.set_label(Btn::Pause, L_RESUME);
                    vec![action]
                } else {
                    self.set_label(Btn::Pause, L_RESUMING);
                    let action = self.game_command(&format!("start {id}"));
                    self.set_enabled(Btn::Pause, true);
                    self.set_label(Btn::Pause, L_PAUSE);
                    vec![action]
                }
            }
            Btn::Start => {
                if self.label_of(Btn::Start) == L_START {
                    let file = self.edit_text(Ed::Config);
                    if file.is_empty() { vec![GmEventAction::Notice(1421, None)] } else { vec![GmEventAction::ReadOptions(file)] }
                } else {
                    self.set_label(Btn::Start, L_ENDING);
                    let actions = vec![
                        self.game_command(&format!("dispelall {id}")),
                        self.game_command(&format!("end {id}")),
                        self.game_command(&format!("msg {id} 1 NULL")),
                    ];
                    self.ended_ui();
                    actions
                }
            }
            Btn::Create => {
                if self.label_of(Btn::Create) == L_CREATE {
                    let (first, second) = (self.edit_text(Ed::Team1), self.edit_text(Ed::Team2));
                    if first.is_empty() {
                        vec![GmEventAction::Notice(1418, None)]
                    } else if second.is_empty() {
                        vec![GmEventAction::Notice(1419, None)]
                    } else if first == second {
                        vec![GmEventAction::Notice(1420, None)]
                    } else {
                        vec![GmEventAction::OpenFence]
                    }
                } else {
                    self.set_label(Btn::Create, L_REMOVING);
                    let action = self.game_command(&format!("remove {id}"));
                    self.notify_remove();
                    vec![action]
                }
            }
        }
    }

    fn lock(&mut self, team: usize) -> Vec<GmEventAction> {
        let id = self.game_id.clone();
        let (lock, leader, number) = if team == 0 { (Btn::Lock1, Ed::Leader1, 1) } else { (Btn::Lock2, Ed::Leader2, 2) };
        if self.label_of(lock) == L_LOCK {
            let name = self.edit_text(leader);
            if name.is_empty() {
                return vec![GmEventAction::Notice(if team == 0 { 1416 } else { 1417 }, None)];
            }
            let actions = vec![
                self.game_command(&format!("leader {id} {number} {name}")),
                self.game_command(&format!("lock {id} {number}")),
            ];
            self.set_enabled(lock, false);
            self.set_label(lock, L_LOCKING);
            actions
        } else {
            let action = self.game_command(&format!("unlock {id} {number}"));
            self.set_enabled(lock, false);
            self.set_label(lock, L_REMOVING);
            self.notify_unlock(team);
            vec![action]
        }
    }

    fn firecrackers(&self, team: usize) -> Vec<GmEventAction> {
        self.slots[team]
            .iter()
            .filter(|slot| slot.object_id != NO_PLAYER)
            .map(|slot| self.game_command(&format!("firecracker {}", slot.name)))
            .collect()
    }

    pub fn notify_lock(&mut self, team: usize) {
        let (lock, select, other) = if team == 0 { (Btn::Lock1, Btn::Select1, Btn::Lock2) } else { (Btn::Lock2, Btn::Select2, Btn::Lock1) };
        self.set_enabled(lock, true);
        self.set_label(lock, L_LIFT);
        self.set_enabled(select, false);
        if self.label_of(other) == L_LIFT {
            self.set_enabled(Btn::Start, true);
        }
    }

    pub fn notify_unlock(&mut self, team: usize) {
        let (lock, select) = if team == 0 { (Btn::Lock1, Btn::Select1) } else { (Btn::Lock2, Btn::Select2) };
        self.set_enabled(lock, true);
        self.set_enabled(Btn::Start, false);
        self.set_enabled(select, true);
        self.set_label(lock, L_LOCK);
        self.lists[team].clear();
    }

    pub fn started_ui(&mut self) -> GmEventAction {
        for button in [Btn::Select1, Btn::Select2, Btn::Lock1, Btn::Lock2] {
            self.set_enabled(button, false);
        }
        self.set_enabled(Btn::Pause, true);
        self.set_enabled(Btn::Start, true);
        self.set_label(Btn::Start, L_END);
        self.game_command(&format!("msg {} 2 NULL", self.game_id))
    }

    pub fn ended_ui(&mut self) {
        for button in [Btn::Select1, Btn::Select2, Btn::Lock1, Btn::Lock2] {
            self.set_enabled(button, true);
        }
        self.set_enabled(Btn::Pause, false);
        self.set_enabled(Btn::Start, true);
        self.set_label(Btn::Start, L_START);
    }

    pub fn notify_remove(&mut self) {
        self.game_id.clear();
        for button in BUTTONS {
            if !matches!(button, Btn::Command | Btn::Create) {
                self.set_enabled(button, false);
            }
        }
        self.set_enabled(Btn::Create, true);
        self.set_label(Btn::Create, L_CREATE);
        self.set_label(Btn::Lock1, L_LOCK);
        self.set_label(Btn::Lock2, L_LOCK);
        self.set_label(Btn::Start, L_START);
        self.set_label(Btn::Fence, L_FENCE_UP);
        for list in &mut self.lists {
            list.clear();
        }
    }

    const GAME_BUTTONS: [Btn; 11] = [
        Btn::Create,
        Btn::Select1,
        Btn::Select2,
        Btn::Lock1,
        Btn::Lock2,
        Btn::Forward,
        Btn::Ending,
        Btn::Apply,
        Btn::Fence,
        Btn::Fire1,
        Btn::Fire2,
    ];

    pub fn game_created(&mut self, game_id: i32) {
        if self.label_of(Btn::Create) == L_CREATING {
            self.game_id = game_id.to_string();
            self.set_label(Btn::Create, L_REMOVE);
            for button in Self::GAME_BUTTONS {
                self.set_enabled(button, true);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn manage_header(&mut self, game_id: i32, flags: [u8; 2], first: &str, middle: u8, second: &str, last: u8, font: &BitmapFont) {
        self.game_id = game_id.to_string();
        self.set_label(Btn::Create, L_REMOVE);
        for button in Self::GAME_BUTTONS {
            self.set_enabled(button, true);
        }
        match flags[0] {
            1 => {
                self.set_label(Btn::Start, L_START);
                self.set_label(Btn::Pause, L_PAUSE);
                self.set_enabled(Btn::Pause, false);
                self.set_enabled(Btn::Start, middle == 1 && last == 1);
                self.set_enabled(Btn::Lock1, true);
                self.set_enabled(Btn::Lock2, true);
            }
            2 => {
                self.set_label(Btn::Start, L_END);
                self.set_label(Btn::Pause, L_PAUSE);
                self.set_enabled(Btn::Start, true);
                self.set_enabled(Btn::Pause, true);
                self.set_enabled(Btn::Lock1, false);
                self.set_enabled(Btn::Lock2, false);
            }
            3 => {
                self.set_label(Btn::Start, L_END);
                self.set_label(Btn::Pause, L_RESUME);
                self.set_enabled(Btn::Start, true);
                self.set_enabled(Btn::Pause, true);
                self.set_enabled(Btn::Lock1, false);
                self.set_enabled(Btn::Lock2, false);
            }
            _ => {}
        }
        match flags[1] {
            0 | 1 => self.set_label(Btn::Fence, L_FENCE_UP),
            2 => self.set_label(Btn::Fence, L_FENCE_DOWN),
            _ => {}
        }
        for (lock, state) in [(Btn::Lock1, middle), (Btn::Lock2, last)] {
            match state {
                0 => self.set_label(lock, L_LOCK),
                1 => self.set_label(lock, L_LIFT),
                _ => {}
            }
        }
        self.set_edit(Ed::Team1, first, font);
        self.set_edit(Ed::Team2, second, font);
        for list in &mut self.lists {
            list.clear();
        }
        self.clear_slots();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn manage_entry(
        &mut self,
        game_id: i32,
        index: usize,
        time: u8,
        leader: u8,
        object_id: i32,
        name: &str,
        class_id: i32,
        level: i32,
        class_name: &str,
        font: &BitmapFont,
    ) {
        if self.game_id != game_id.to_string() || !(1..=2).contains(&time) {
            return;
        }
        let team = usize::from(time) - 1;
        let Some(slot_index) = (index + SLOTS).checked_sub(SLOTS * usize::from(time)).filter(|slot| *slot < SLOTS) else {
            return;
        };
        let slot = &mut self.slots[team][slot_index];
        slot.object_id = object_id;
        slot.name = name.to_string();
        slot.level = level;
        slot.class_id = class_id;
        self.lists[team].add_item(ListRow {
            cells: vec![ListCell::text(name), ListCell::text(&level.to_string()), ListCell::text(class_name)],
        });
        if leader == 1 {
            self.set_edit(if team == 0 { Ed::Leader1 } else { Ed::Leader2 }, name, font);
        }
    }

    pub fn team_info_start(&mut self, game_id: i32, team: i32) {
        if self.game_id != game_id.to_string() {
            return;
        }
        let team = match team {
            1 => 0,
            2 => 1,
            _ => return,
        };
        self.current_team = Some(team);
        for slot in &mut self.slots[team] {
            slot.object_id = NO_PLAYER;
        }
    }

    pub fn team_info_entry(&mut self, index: usize, object_id: i32, name: &str, values: [i32; 10]) {
        let Some(team) = self.current_team else { return };
        if index >= SLOTS {
            return;
        }
        let slot = &mut self.slots[team][index];
        slot.object_id = object_id;
        slot.name = name.to_string();
        slot.values = values;
        slot.level = values[6];
        slot.class_id = values[7];
    }

    pub fn team_info_end(&mut self, class_name: &dyn Fn(i32) -> String) {
        let Some(team) = self.current_team.take() else { return };
        self.lists[team].clear();
        self.notify_lock(team);
        for slot in self.slots[team].iter().take_while(|slot| slot.object_id != NO_PLAYER) {
            self.lists[team].add_item(ListRow {
                cells: vec![ListCell::text(&slot.name), ListCell::text(&slot.level.to_string()), ListCell::text(&class_name(slot.class_id))],
            });
        }
    }

    pub fn start_countdown(&mut self) -> GmEventAction {
        self.countdown = Some((COUNTDOWN_START, 0));
        self.game_command(&format!("msg {} 8 NULL", self.game_id))
    }

    pub fn tick(&mut self, elapsed_ms: u32) -> Vec<GmEventAction> {
        for edit in &mut self.edits {
            edit.tick(elapsed_ms);
        }
        let Some((state, mut waited)) = self.countdown else { return Vec::new() };
        waited += elapsed_ms;
        if waited < TIMER_MS {
            self.countdown = Some((state, waited));
            return Vec::new();
        }
        waited -= TIMER_MS;
        let id = self.game_id.clone();
        let mut actions = Vec::new();
        match state {
            4 => actions.push(self.game_command(&format!("msg {id} 7 NULL"))),
            3 => actions.push(self.game_command(&format!("msg {id} 6 NULL"))),
            2 => actions.push(self.game_command(&format!("msg {id} 5 NULL"))),
            1 => actions.push(self.game_command(&format!("msg {id} 4 NULL"))),
            0 => {
                actions.push(self.game_command(&format!("msg {id} 2 NULL")));
                let mut skills = format!("useskill {id}");
                for (buff, level) in &self.options.buffs {
                    skills.push_str(&format!(" {buff} {level}"));
                }
                actions.push(self.game_command(&skills));
                actions.push(self.game_command(&format!("start {id}")));
                actions.push(self.started_ui());
                self.countdown = None;
                return actions;
            }
            _ => {}
        }
        self.countdown = Some((state - 1, waited));
        actions
    }

    pub fn start_race(&mut self, options: RaceOptions) -> Vec<GmEventAction> {
        let id = self.game_id.clone();
        self.options = options;
        let mut item = format!("item_rule {id} ");
        if self.options.item_default_allow {
            item.push_str("allow_all");
            for entry in &self.options.items {
                item.push_str(&format!(" D{entry}"));
            }
        } else {
            item.push_str("deny_all");
            for entry in &self.options.items {
                item.push_str(&format!(" A{entry}"));
            }
        }
        let skill = if self.options.skill_default_allow {
            let mut rules = format!("{COMMAND_PREFIX} skill_rule {id} allow_all");
            for entry in &self.options.skills {
                rules.push_str(&format!(" D{entry}"));
            }
            rules
        } else {
            let mut rules = String::new();
            for entry in &self.options.skills {
                rules.push_str(&format!(" A{entry}"));
            }
            rules
        };
        let countdown = self.start_countdown();
        self.set_enabled(Btn::Start, false);
        self.set_label(Btn::Start, L_STARTING);
        vec![self.game_command(&item), GmEventAction::Command(skill), countdown]
    }

    pub fn fence_created(&mut self) {
        self.set_enabled(Btn::Create, false);
        self.set_label(Btn::Create, L_CREATING);
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<GmHit> {
        match controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) {
            Some(FrameHit::Close) => return Some(GmHit::Close),
            Some(FrameHit::Minimize) => return Some(GmHit::Minimize),
            _ => {}
        }
        if let Some(index) = BUTTONS.iter().position(|&button| button_contract(button).hit(&self.buttons[button_index(button)], x, y).is_some()) {
            return Some(GmHit::Button(index));
        }
        if let Some(index) = EDITS.iter().position(|&edit| edit_contract(edit).rect.contains(x, y)) {
            return Some(GmHit::Edit(index));
        }
        (controls::CHILD_148.rect.contains(x, y) || controls::CHILD_14C.rect.contains(x, y)).then_some(GmHit::List)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, font: &BitmapFont) -> Option<GmHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        if let Some(GmHit::Edit(index)) = hit {
            if let Some(previous) = self.focus.replace(index).filter(|previous| *previous != index) {
                self.edits[previous].kill_focus();
            }
            let metrics = FontMetrics(font);
            self.edits[index].focus();
            self.edits[index].mouse_down(x, &metrics);
            self.edits[index].layout(&metrics);
        }
        if hit == Some(GmHit::List) {
            for list in &mut self.lists {
                if list.rect.contains(x, y) {
                    list.mouse_down(x, y);
                }
            }
        }
        hit
    }

    pub fn mouse_up(&mut self, x: i32, y: i32, font: &BitmapFont) -> Vec<GmEventAction> {
        for edit in &mut self.edits {
            edit.mouse_up();
        }
        for list in &mut self.lists {
            list.mouse_up(x, y);
        }
        let Some(pressed) = self.pressed.take() else { return Vec::new() };
        if self.hit(x, y) != Some(pressed) {
            return Vec::new();
        }
        match pressed {
            GmHit::Close => vec![GmEventAction::Close],
            GmHit::Minimize => vec![GmEventAction::Minimize],
            GmHit::Button(index) if self.buttons[index].enabled => self.press(BUTTONS[index], font),
            _ => Vec::new(),
        }
    }

    pub fn key(&mut self, key: InputKey, font: &BitmapFont) -> bool {
        let Some(index) = self.focus else { return false };
        if !self.edits[index].focused {
            return false;
        }
        self.edits[index].input_key(key, &FontMetrics(font));
        self.edits[index].notices.clear();
        true
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32, lookup: &dyn Fn(u32) -> Option<String>) {
        let (x, y) = origin;
        controls::FRAME_7C.paint(&self.frame, list, origin, font, alpha);
        let start = list.quads.len();
        list.draw_texture(
            x + BACK_ORIGIN.0,
            y + BACK_ORIGIN.1,
            SIZE.0,
            SIZE.1 - BACK_ORIGIN.1,
            0,
            0,
            BACK_SOURCE.0,
            BACK_SOURCE.1,
            BACK,
            alpha,
        );
        list.clip_from(start, Rect::new(x, y, SIZE.0, SIZE.1));
        for (text, (_, position)) in self.texts.iter().zip(TEXTS.iter()) {
            list.draw_text(x + position.0, y + position.1, TEXT_COLOR, text, font);
        }
        let game = format!("{}: {}", self.label_of_text(GAME_ID_LABEL, lookup), self.game_id);
        list.draw_text(x + GAME_ID_POSITION.0, y + GAME_ID_POSITION.1, TEXT_COLOR, &game, font);
        list.draw_text(x + COLON_POSITION.0, y + COLON_POSITION.1, TEXT_COLOR, ":", font);
        let message = format!("{}:", self.label_of_text(1096, lookup));
        list.draw_text(x + MESSAGE_COLON_POSITION.0, y + MESSAGE_COLON_POSITION.1, TEXT_COLOR, &message, font);
        for edit in &self.edits {
            edit.paint(list, origin, &FontMetrics(font), font);
        }
        for column in 0..2 {
            self.lists[column].paint_header(list, origin, font, &|id| lookup(id as u32));
            self.lists[column].paint(list, origin, font);
        }
        for (index, button) in BUTTONS.iter().enumerate() {
            let state = &self.buttons[index];
            button_contract(*button).paint(state, list, origin, self.pressed == Some(GmHit::Button(index)) && state.enabled, font);
        }
    }

    fn label_of_text(&self, id: u32, lookup: &dyn Fn(u32) -> Option<String>) -> String {
        if id == GAME_ID_LABEL { self.game_id_label.clone() } else { lookup(id).unwrap_or_default() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ini::parse_ini;

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn view() -> GmEventView {
        GmEventView::new(|id| Some(format!("s{id}")))
    }

    fn created(view: &mut GmEventView, id: i32) {
        view.fence_created();
        view.game_created(id);
    }

    fn commands(actions: &[GmEventAction]) -> Vec<&str> {
        actions
            .iter()
            .filter_map(|action| if let GmEventAction::Command(text) = action { Some(text.as_str()) } else { None })
            .collect()
    }

    #[test]
    fn the_frame_minimize_button_asks_to_minimize_the_window() {
        let font = font();
        let mut view = view();
        let rect = controls::FRAME_7C.button_rect(&view.frame, (0, 0), FrameHit::Minimize).unwrap();
        let (x, y) = (rect.x + 2, rect.y + 2);
        assert_eq!(view.mouse_down(x, y, &font), Some(GmHit::Minimize));
        assert_eq!(view.mouse_up(x, y, &font), vec![GmEventAction::Minimize]);
    }

    #[test]
    fn the_layout_and_the_initial_enable_state_match_on_create() {
        let view = view();
        assert_eq!(SIZE, (700, 500));
        assert_eq!(view.buttons.len(), 14);
        assert_eq!(view.buttons[0].rect, Rect::new(25, 470, 76, 23));
        assert_eq!(view.buttons[0].label, "s1079");
        assert_eq!(view.edits.len(), 9);
        assert_eq!(view.lists[0].rect, Rect::new(25, 250, 300, 150));
        assert_eq!(view.lists[1].rect, Rect::new(355, 250, 300, 150));
        let enabled: Vec<bool> = BUTTONS.iter().map(|&button| view.enabled(button)).collect();
        assert_eq!(enabled.iter().filter(|&&on| on).count(), 2);
        assert!(view.enabled(Btn::Command) && view.enabled(Btn::Create));
        assert_eq!(view.edits[0].max_len, 10);
        assert_eq!(view.edits[2].max_len, 1);
        assert_eq!(view.edits[8].max_len, 0x32);
    }

    #[test]
    fn creating_needs_two_different_team_names_before_opening_the_fence_window() {
        let font = font();
        let mut view = view();
        assert_eq!(view.press(Btn::Create, &font), vec![GmEventAction::Notice(1418, None)]);
        view.set_edit(Ed::Team1, "Red", &font);
        assert_eq!(view.press(Btn::Create, &font), vec![GmEventAction::Notice(1419, None)]);
        view.set_edit(Ed::Team2, "Red", &font);
        assert_eq!(view.press(Btn::Create, &font), vec![GmEventAction::Notice(1420, None)]);
        view.clear_edit(Ed::Team2);
        view.set_edit(Ed::Team2, "Blue", &font);
        assert_eq!(view.press(Btn::Create, &font), vec![GmEventAction::OpenFence]);
    }

    #[test]
    fn a_created_game_enables_eleven_buttons_and_remove_restores_the_idle_state() {
        let font = font();
        let mut view = view();
        created(&mut view, 55);
        assert_eq!(view.game_id, "55");
        assert_eq!(view.label_of(Btn::Create), L_REMOVE);
        assert_eq!(BUTTONS.iter().filter(|&&button| view.enabled(button)).count(), 12);
        assert!(!view.enabled(Btn::Start) && !view.enabled(Btn::Pause));
        let actions = view.press(Btn::Create, &font);
        assert_eq!(commands(&actions), vec!["eventmatch remove 55"]);
        assert_eq!(view.game_id, "");
        assert_eq!(view.label_of(Btn::Create), L_CREATE);
        assert_eq!(BUTTONS.iter().filter(|&&button| view.enabled(button)).count(), 2);
    }

    #[test]
    fn the_created_packet_is_only_taken_while_the_creation_waits_for_it() {
        let mut view = view();
        view.game_created(9);
        assert_eq!(view.game_id, "");
        assert_eq!(view.label_of(Btn::Create), L_CREATE);
        view.fence_created();
        assert!(!view.enabled(Btn::Create));
        view.game_created(9);
        assert_eq!(view.game_id, "9");
        assert_eq!(view.label_of(Btn::Create), L_REMOVE);
        assert!(view.enabled(Btn::Create));
    }

    #[test]
    fn locking_sends_leader_then_lock_and_lifting_sends_unlock_and_clears_the_list() {
        let font = font();
        let mut view = view();
        created(&mut view, 3);
        assert_eq!(view.press(Btn::Lock1, &font), vec![GmEventAction::Notice(1416, None)]);
        view.set_edit(Ed::Leader1, "Ana", &font);
        let actions = view.press(Btn::Lock1, &font);
        assert_eq!(commands(&actions), vec!["eventmatch leader 3 1 Ana", "eventmatch lock 3 1"]);
        assert!(!view.enabled(Btn::Lock1));
        view.notify_lock(0);
        assert_eq!((view.label_of(Btn::Lock1), view.enabled(Btn::Lock1), view.enabled(Btn::Select1)), (L_LIFT, true, false));
        assert!(!view.enabled(Btn::Start));
        view.notify_lock(1);
        assert!(view.enabled(Btn::Start));
        let actions = view.press(Btn::Lock1, &font);
        assert_eq!(commands(&actions), vec!["eventmatch unlock 3 1"]);
        assert_eq!((view.label_of(Btn::Lock1), view.enabled(Btn::Select1), view.enabled(Btn::Start)), (L_LOCK, true, false));
        assert_eq!(view.lists[0].count(), 0);
    }

    #[test]
    fn messages_score_ending_and_fence_build_the_exact_commands() {
        let font = font();
        let mut view = view();
        created(&mut view, 3);
        view.set_edit(Ed::Message, "hello", &font);
        view.set_edit(Ed::Score1, "2", &font);
        view.set_edit(Ed::Score2, "1", &font);
        assert_eq!(commands(&view.press(Btn::Forward, &font)), vec!["eventmatch msg 3 0 \"hello\""]);
        assert_eq!(commands(&view.press(Btn::Ending, &font)), vec!["eventmatch msg 3 3 NULL"]);
        assert_eq!(commands(&view.press(Btn::Apply, &font)), vec!["eventmatch score 3 2 1"]);
        assert_eq!(view.label_of(Btn::Fence), L_FENCE_UP);
        assert_eq!(commands(&view.press(Btn::Fence, &font)), vec!["eventmatch fence 3 2"]);
        assert_eq!(view.label_of(Btn::Fence), L_FENCE_DOWN);
        assert_eq!(commands(&view.press(Btn::Fence, &font)), vec!["eventmatch fence 3 1"]);
    }

    #[test]
    fn firecrackers_go_to_every_occupied_slot_of_the_team() {
        let font = font();
        let mut view = view();
        created(&mut view, 3);
        view.team_info_start(3, 2);
        view.team_info_entry(0, 10, "Ana", [0, 0, 0, 0, 0, 0, 5, 3, 0, 0]);
        view.team_info_entry(4, 11, "Bo", [0; 10]);
        let actions = view.press(Btn::Fire2, &font);
        assert_eq!(commands(&actions), vec!["eventmatch firecracker Ana", "eventmatch firecracker Bo"]);
        assert!(commands(&view.press(Btn::Fire1, &font)).is_empty());
    }

    #[test]
    fn team_info_fills_the_team_list_and_locks_the_team() {
        let mut view = view();
        created(&mut view, 3);
        view.team_info_start(3, 1);
        view.team_info_entry(0, 10, "Ana", [0, 0, 0, 0, 0, 0, 40, 18, 0, 0]);
        view.team_info_end(&|class| format!("class{class}"));
        assert_eq!(view.lists[0].count(), 1);
        assert_eq!(view.lists[0].rows[0].cells[2].text, "class18");
        assert_eq!(view.label_of(Btn::Lock1), L_LIFT);
        view.team_info_start(99, 2);
        assert_eq!(view.current_team, None);
    }

    #[test]
    fn manage_entries_fill_the_slots_by_team_and_the_leader_edit() {
        let font = font();
        let mut view = view();
        view.manage_header(7, [1, 1], "Red", 1, "Blue", 0, &font);
        assert_eq!(view.edit_text(Ed::Team1), "Red");
        assert_eq!(view.label_of(Btn::Lock1), L_LIFT);
        assert_eq!(view.label_of(Btn::Lock2), L_LOCK);
        assert!(!view.enabled(Btn::Start));
        view.manage_entry(7, 0, 1, 1, 100, "Ana", 18, 40, "Mage", &font);
        view.manage_entry(7, 9, 2, 0, 101, "Bo", 5, 30, "Fighter", &font);
        view.manage_entry(8, 1, 1, 0, 102, "Other", 5, 30, "Fighter", &font);
        assert_eq!(view.slots[0][0].name, "Ana");
        assert_eq!(view.slots[1][0].name, "Bo");
        assert_eq!(view.edit_text(Ed::Leader1), "Ana");
        assert_eq!((view.lists[0].count(), view.lists[1].count()), (1, 1));
        view.manage_header(7, [3, 2], "", 1, "", 1, &font);
        assert_eq!((view.label_of(Btn::Start), view.label_of(Btn::Pause), view.label_of(Btn::Fence)), (L_END, L_RESUME, L_FENCE_DOWN));
        assert!(view.enabled(Btn::Start) && view.enabled(Btn::Pause) && !view.enabled(Btn::Lock1));
        assert_eq!(view.edit_text(Ed::Team1), "");
        view.manage_header(7, [9, 9], "", 7, "", 7, &font);
        assert_eq!((view.label_of(Btn::Fence), view.label_of(Btn::Lock1), view.label_of(Btn::Lock2)), (L_FENCE_DOWN, L_LIFT, L_LIFT));
    }

    #[test]
    fn the_option_file_requires_every_key_and_reports_the_matching_message() {
        let good = "[Buff]\nBuffCnt=1\nBuffID0=1204\nBuffLv0=2\n[Skill]\nDefaultAllow=True\nExpSkillCnt=1\nExpSkillID0=10\n[Item]\nDefaultAllow=0\nExpItemCnt=2\nExpItemID0=57\nExpItemID1=58\n[Etc]\nTeleportDelay=5\n";
        let options = read_options(&parse_ini(good)).unwrap();
        assert_eq!(options.buffs, vec![(1204, 2)]);
        assert!(options.skill_default_allow && !options.item_default_allow);
        assert_eq!((options.skills.clone(), options.items.clone(), options.teleport_delay), (vec![10], vec![57, 58], 5));
        assert_eq!(read_options(&parse_ini("")).unwrap_err(), GmEventAction::Notice(1422, None));
        let missing = good.replace("BuffLv0=2\n", "");
        assert_eq!(read_options(&parse_ini(&missing)).unwrap_err(), GmEventAction::Notice(1424, Some(0)));
        let loose = good.replace("BuffCnt=1", "BuffCnt=1x").replace("DefaultAllow=0", "DefaultAllow=1tail").replace("DefaultAllow=True", "DefaultAllow=tRuE");
        let options = read_options(&parse_ini(&loose)).unwrap();
        assert!(options.skill_default_allow && options.item_default_allow);
        assert_eq!(options.buffs.len(), 1);
        let no_delay = good.replace("TeleportDelay=5\n", "");
        assert_eq!(read_options(&parse_ini(&no_delay)).unwrap_err(), GmEventAction::Notice(1430, None));
    }

    #[test]
    fn starting_sends_the_rules_then_counts_down_and_starts_the_race() {
        let font = font();
        let mut view = view();
        created(&mut view, 3);
        assert_eq!(view.press(Btn::Start, &font), vec![GmEventAction::Notice(1421, None)]);
        view.set_edit(Ed::Config, "race.ini", &font);
        assert_eq!(view.press(Btn::Start, &font), vec![GmEventAction::ReadOptions("race.ini".into())]);
        let options = RaceOptions {
            buffs: vec![(1204, 2)],
            skill_default_allow: false,
            skills: vec![10],
            item_default_allow: true,
            items: vec![57],
            teleport_delay: 0,
        };
        let actions = view.start_race(options);
        assert_eq!(
            commands(&actions),
            vec!["eventmatch item_rule 3 allow_all D57", " A10", "eventmatch msg 3 8 NULL"]
        );
        assert_eq!((view.label_of(Btn::Start), view.enabled(Btn::Start)), (L_STARTING, false));
        let mut sent = Vec::new();
        for _ in 0..6 {
            sent.extend(view.tick(1000));
        }
        assert_eq!(
            commands(&sent),
            vec![
                "eventmatch msg 3 7 NULL",
                "eventmatch msg 3 6 NULL",
                "eventmatch msg 3 5 NULL",
                "eventmatch msg 3 4 NULL",
                "eventmatch msg 3 2 NULL",
                "eventmatch useskill 3 1204 2",
                "eventmatch start 3",
                "eventmatch msg 3 2 NULL",
            ]
        );
        assert!(view.countdown.is_none());
        assert_eq!((view.label_of(Btn::Start), view.enabled(Btn::Pause), view.enabled(Btn::Lock1)), (L_END, true, false));
    }

    #[test]
    fn ending_the_race_sends_three_commands_and_restores_the_buttons() {
        let font = font();
        let mut view = view();
        created(&mut view, 3);
        let _ = view.started_ui();
        let actions = view.press(Btn::Start, &font);
        assert_eq!(commands(&actions), vec!["eventmatch dispelall 3", "eventmatch end 3", "eventmatch msg 3 1 NULL"]);
        assert_eq!((view.label_of(Btn::Start), view.enabled(Btn::Pause), view.enabled(Btn::Lock1)), (L_START, false, true));
    }

    #[test]
    fn pause_and_resume_alternate_between_pause_and_start_commands() {
        let font = font();
        let mut view = view();
        created(&mut view, 3);
        let _ = view.started_ui();
        assert_eq!(commands(&view.press(Btn::Pause, &font)), vec!["eventmatch pause 3"]);
        assert_eq!(view.label_of(Btn::Pause), L_RESUME);
        assert_eq!(commands(&view.press(Btn::Pause, &font)), vec!["eventmatch start 3"]);
        assert_eq!(view.label_of(Btn::Pause), L_PAUSE);
    }

    #[test]
    fn the_command_html_lists_one_link_per_command() {
        let ini = parse_ini("[Cmd]\nCmdCnt=2\nCmd0=a b\nCmd1=c\n");
        assert_eq!(command_html(&ini), "<html><body><a cmd = \"a b\">a b</a><br1><a cmd = \"c\">c</a><br1></body></html>");
        assert_eq!(command_html(&parse_ini("")), "<html><body></body></html>");
        let sparse = parse_ini("[Cmd]\nCmdCnt=2x\nCmd1=c\n");
        assert_eq!(command_html(&sparse), "<html><body><a cmd = \"\"></a><br1><a cmd = \"c\">c</a><br1></body></html>");
    }
}
