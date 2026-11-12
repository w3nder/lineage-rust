#[path = "gm_wnd_controls_generated.rs"]
mod controls;
#[path = "gm_wnd_layout_generated.rs"]
mod layout;

use crate::button::TextButton;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::edit_box::{EditBox, FontMetrics};
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::frame_contract::{FrameContract, FrameHit, FrameTitle};
use crate::window::{Rect, WindowId};

pub const ID: WindowId = WindowId::GM;
pub const SIZE: (i32, i32) = layout::SIZE;
pub const BACK: &str = "l2ui_ch3.siegewnd.siege_back31";
pub const BACK_ORIGIN: (i32, i32) = (0, 20);
pub const BUTTON_NORMAL: &str = "sek.cbui91";
pub const BUTTON_DOWN: &str = "sek.cbui93";
pub const NAME_LABEL_SYSSTRING: u32 = 50;
pub const NAME_LABEL_POSITION: (i32, i32) = (7, 55);
pub const NAME_LABEL_COLOR: Argb = Argb(0xFFDCDCDC);
pub const FRAME_ID: u32 = 0xf0f015;
pub const ENTER_NAME_SYSMSG: u32 = 364;
pub const SUMMON_DIALOG_TYPE: i32 = 0x4b;
pub const SUMMON_SYSMSG: u32 = 1220;
pub const ESCAPE_DIALOG_TYPE: i32 = 0x4a;
pub const ESCAPE_SYSMSG: u32 = 1221;
pub const NPC_ID_OFFSET: i32 = 1_000_000;
pub const INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 1, vertical_mode: 4, x: 257, y: 327 };

const FRAME: FrameContract = FrameContract {
    rect: layout::RECT_7C,
    id: FRAME_ID,
    mode: FrameMode::Title,
    title: FrameTitle::None,
    close_button: true,
    minimize_button: false,
};

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 - 256.0 - 1.0) as i32, (viewport.1 as f32 - 26.0 - 1.0 - 300.0) as i32)
}

pub fn grip() -> Rect {
    layout::RECT_7C
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GmButton {
    Go,
    Move,
    Summon,
    Status,
    Inventory,
    Skills,
    Quest,
    Information,
    Warehouse,
    Clan,
    PetitionInvitation,
    Escape,
    NpcSearch,
    ItemSearch,
    SkillSearch,
    ForcedPetition,
    ServerTransfer,
}

pub const BUTTONS: [(Rect, u32, GmButton); 17] = [
    (layout::RECT_108, 899, GmButton::Go),
    (layout::RECT_114, 900, GmButton::Move),
    (layout::RECT_10C, 688, GmButton::Summon),
    (layout::RECT_F8, 433, GmButton::Status),
    (layout::RECT_F0, 138, GmButton::Inventory),
    (layout::RECT_F4, 119, GmButton::Skills),
    (layout::RECT_FC, 118, GmButton::Quest),
    (layout::RECT_118, 901, GmButton::Information),
    (layout::RECT_100, 131, GmButton::Warehouse),
    (layout::RECT_104, 439, GmButton::Clan),
    (layout::RECT_110, 902, GmButton::PetitionInvitation),
    (layout::RECT_11C, 903, GmButton::Escape),
    (layout::RECT_120, 690, GmButton::NpcSearch),
    (layout::RECT_124, 691, GmButton::ItemSearch),
    (layout::RECT_128, 692, GmButton::SkillSearch),
    (layout::RECT_12C, 1274, GmButton::ForcedPetition),
    (layout::RECT_130, 1275, GmButton::ServerTransfer),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewKind {
    Status = 1,
    Clan = 2,
    Skills = 3,
    Quest = 4,
    Inventory = 5,
    Warehouse = 6,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchKind {
    Npc,
    Item,
    Skill,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GmAction {
    Close,
    Bypass(String),
    ForcePeti { name: String },
    Confirm { dialog_type: i32, message_id: u32, name: String, command: String },
    View { kind: ViewKind, name: String },
    Notice(u32),
    Search { kind: SearchKind, name: String },
    ServerTransfer { id: i32 },
}

pub fn atoi(text: &str) -> i32 {
    let mut chars = text.trim_start().chars().peekable();
    let negative = chars.peek() == Some(&'-');
    if negative || chars.peek() == Some(&'+') {
        chars.next();
    }
    let mut value = 0_i32;
    for ch in chars {
        let Some(digit) = ch.to_digit(10) else { break };
        value = value.wrapping_mul(10).wrapping_add(digit as i32);
    }
    if negative { value.wrapping_neg() } else { value }
}

pub fn search_result(id: i32, name: &str) -> String {
    format!("ClassID:{id} Name:{name}")
}

#[derive(Clone, Debug)]
pub struct GmView {
    pub frame: FrameCtrl,
    pub edit: EditBox,
    pub buttons: Vec<TextButton>,
    pub name_label: String,
    pressed: Option<GmHit>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GmHit {
    Close,
    Button(usize),
    Edit,
}

impl GmView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Self {
            frame: FRAME.build(text),
            edit: EditBox::from_contract(&controls::CHILD_EC),
            buttons: BUTTONS.iter().map(|(rect, sysstring, _)| TextButton::new(*rect, BUTTON_NORMAL, BUTTON_DOWN, &text(*sysstring))).collect(),
            name_label: text(NAME_LABEL_SYSSTRING),
            pressed: None,
        }
    }

    pub fn name(&self) -> String {
        self.edit.get_text().unwrap_or_default()
    }

    pub fn set_name(&mut self, name: &str, font: &BitmapFont) {
        self.edit.clear();
        self.edit.insert_text(name, true, &FontMetrics(font));
    }

    pub fn press(&self, button: GmButton) -> Option<GmAction> {
        let name = self.name();
        let named = |action: GmAction| (!name.is_empty()).then_some(action);
        let view = |kind: ViewKind| Some(if name.is_empty() { GmAction::Notice(ENTER_NAME_SYSMSG) } else { GmAction::View { kind, name: name.clone() } });
        match button {
            GmButton::Go => named(GmAction::Bypass(format!("teleportto {name}"))),
            GmButton::Move => Some(GmAction::Bypass("instant_move".into())),
            GmButton::ForcedPetition => named(GmAction::ForcePeti { name: name.clone() }),
            GmButton::PetitionInvitation => named(GmAction::Bypass(format!("add_peti_chat {name}"))),
            GmButton::Information => named(GmAction::Bypass(format!("debug {name}"))),
            GmButton::Summon => named(GmAction::Confirm {
                dialog_type: SUMMON_DIALOG_TYPE,
                message_id: SUMMON_SYSMSG,
                name: name.clone(),
                command: format!("recall {name}"),
            }),
            GmButton::Escape => named(GmAction::Confirm {
                dialog_type: ESCAPE_DIALOG_TYPE,
                message_id: ESCAPE_SYSMSG,
                name: name.clone(),
                command: format!("sendhome {name}"),
            }),
            GmButton::Inventory => view(ViewKind::Inventory),
            GmButton::Skills => view(ViewKind::Skills),
            GmButton::Status => view(ViewKind::Status),
            GmButton::Quest => view(ViewKind::Quest),
            GmButton::Warehouse => view(ViewKind::Warehouse),
            GmButton::Clan => view(ViewKind::Clan),
            GmButton::NpcSearch => named(GmAction::Search { kind: SearchKind::Npc, name: name.clone() }),
            GmButton::ItemSearch => named(GmAction::Search { kind: SearchKind::Item, name: name.clone() }),
            GmButton::SkillSearch => named(GmAction::Search { kind: SearchKind::Skill, name: name.clone() }),
            GmButton::ServerTransfer => Some(GmAction::ServerTransfer { id: atoi(&name) }),
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<GmHit> {
        if FRAME.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(GmHit::Close);
        }
        if let Some(index) = self.buttons.iter().position(|button| button.contains(x, y)) {
            return Some(GmHit::Button(index));
        }
        layout::RECT_EC.contains(x, y).then_some(GmHit::Edit)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, font: &BitmapFont) -> Option<GmHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        if hit == Some(GmHit::Edit) {
            let metrics = FontMetrics(font);
            self.edit.focus();
            self.edit.mouse_down(x, &metrics);
            self.edit.layout(&metrics);
        }
        hit
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<GmAction> {
        self.edit.mouse_up();
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            GmHit::Close => Some(GmAction::Close),
            GmHit::Button(index) => self.press(BUTTONS[index].2),
            GmHit::Edit => None,
        }
    }

    pub fn key(&mut self, key: crate::chat::InputKey, font: &BitmapFont) -> bool {
        if !self.edit.focused {
            return false;
        }
        self.edit.input_key(key, &FontMetrics(font));
        self.edit.notices.clear();
        true
    }

    pub fn tick(&mut self, elapsed_ms: u32) {
        self.edit.tick(elapsed_ms);
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        FRAME.paint(&self.frame, list, origin, font, alpha);
        let start = list.quads.len();
        list.draw_texture(x + BACK_ORIGIN.0, y + BACK_ORIGIN.1, SIZE.0, SIZE.1 - BACK_ORIGIN.1, 0, 0, SIZE.0, SIZE.1 - BACK_ORIGIN.1, BACK, alpha);
        list.clip_from(start, Rect::new(x, y, SIZE.0, SIZE.1));
        list.draw_text(x + NAME_LABEL_POSITION.0, y + NAME_LABEL_POSITION.1, NAME_LABEL_COLOR, &self.name_label, font);
        self.edit.paint(list, origin, &FontMetrics(font), font);
        for (index, button) in self.buttons.iter().enumerate() {
            button.paint(list, x, y, self.pressed == Some(GmHit::Button(index)), font);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn view() -> GmView {
        GmView::new(|id| Some(format!("s{id}")))
    }

    #[test]
    fn the_layout_matches_the_on_create_snapshot() {
        let view = view();
        assert_eq!(SIZE, (256, 276));
        assert_eq!(view.edit.rect, Rect::new(35, 55, 112, 14));
        assert_eq!(view.buttons.len(), 17);
        assert_eq!(view.buttons[0].rect, Rect::new(10, 75, 54, 18));
        assert_eq!(view.buttons[0].label, "s899");
        assert_eq!(view.buttons[16].rect, Rect::new(74, 180, 54, 18));
        assert_eq!(view.name_label, "s50");
        assert_eq!(default_position((1280, 800)), (1023, 473));
    }

    #[test]
    fn the_initial_position_restores_the_creation_coordinates() {
        use crate::initial_position::InitialAxis;
        let viewport = (1280, 800);
        let x = InitialAxis::horizontal(1, INITIAL_POSITION.x).unwrap().coordinate(viewport.0);
        let y = InitialAxis::vertical(4, INITIAL_POSITION.y).unwrap().coordinate(viewport.1);
        assert_eq!((x, y), default_position(viewport));
    }

    #[test]
    fn text_commands_have_no_double_slash_prefix_and_need_a_name_except_move() {
        let font = font();
        let mut view = view();
        assert_eq!(view.press(GmButton::Go), None);
        assert_eq!(view.press(GmButton::Move), Some(GmAction::Bypass("instant_move".into())));
        view.set_name("Ana", &font);
        assert_eq!(view.press(GmButton::Go), Some(GmAction::Bypass("teleportto Ana".into())));
        assert_eq!(view.press(GmButton::Information), Some(GmAction::Bypass("debug Ana".into())));
        assert_eq!(view.press(GmButton::PetitionInvitation), Some(GmAction::Bypass("add_peti_chat Ana".into())));
        assert_eq!(view.press(GmButton::ForcedPetition), Some(GmAction::ForcePeti { name: "Ana".into() }));
    }

    #[test]
    fn summon_and_escape_ask_for_confirmation_first() {
        let font = font();
        let mut view = view();
        assert_eq!(view.press(GmButton::Summon), None);
        view.set_name("Ana", &font);
        assert_eq!(
            view.press(GmButton::Summon),
            Some(GmAction::Confirm { dialog_type: 0x4b, message_id: 1220, name: "Ana".into(), command: "recall Ana".into() })
        );
        assert_eq!(
            view.press(GmButton::Escape),
            Some(GmAction::Confirm { dialog_type: 0x4a, message_id: 1221, name: "Ana".into(), command: "sendhome Ana".into() })
        );
    }

    #[test]
    fn the_view_buttons_warn_without_a_name_and_carry_the_gm_command_type() {
        let font = font();
        let mut view = view();
        for button in [GmButton::Inventory, GmButton::Skills, GmButton::Status, GmButton::Quest, GmButton::Warehouse, GmButton::Clan] {
            assert_eq!(view.press(button), Some(GmAction::Notice(364)));
        }
        view.set_name("Ana", &font);
        let kind = |button| match view.press(button) {
            Some(GmAction::View { kind, .. }) => kind as i32,
            other => panic!("{other:?}"),
        };
        assert_eq!(
            [GmButton::Status, GmButton::Clan, GmButton::Skills, GmButton::Quest, GmButton::Inventory, GmButton::Warehouse].map(kind),
            [1, 2, 3, 4, 5, 6]
        );
    }

    #[test]
    fn searches_are_local_and_server_transfer_reads_the_id_with_atoi() {
        let font = font();
        let mut view = view();
        assert_eq!(view.press(GmButton::NpcSearch), None);
        assert_eq!(view.press(GmButton::ServerTransfer), Some(GmAction::ServerTransfer { id: 0 }));
        view.set_name("12", &font);
        assert_eq!(view.press(GmButton::ServerTransfer), Some(GmAction::ServerTransfer { id: 12 }));
        view.set_name("Gremlin", &font);
        assert_eq!(view.press(GmButton::ItemSearch), Some(GmAction::Search { kind: SearchKind::Item, name: "Gremlin".into() }));
        assert_eq!(search_result(7, "x"), "ClassID:7 Name:x");
        assert_eq!(atoi("-5x"), -5);
    }

    #[test]
    fn releasing_on_a_button_runs_its_handler_and_the_close_button_closes() {
        let font = font();
        let mut view = view();
        let rect = view.buttons[1].rect;
        view.mouse_down(rect.x + 2, rect.y + 2, &font);
        assert_eq!(view.mouse_up(rect.x + 2, rect.y + 2), Some(GmAction::Bypass("instant_move".into())));
        view.mouse_down(rect.x + 2, rect.y + 2, &font);
        assert_eq!(view.mouse_up(0, 100), None);
    }
}
