use crate::button::TextButton;
use crate::color::{Argb, GOLD, TEXT};
use crate::combo::ComboBox;
use crate::draw::DrawList;
use crate::exp::{exp_fill, exp_percent};
use crate::font::BitmapFont;
use crate::window::Rect;

pub const MAX_CHARACTERS: usize = 7;
pub const VOICE_FADE: f32 = 1.5;
pub const SELECT_RECT: Rect = Rect::new(24, 27, 256, 108);
pub const SELECT_BACK: &str = "L2UI_CH3.LobbyWnd.select1_back";
pub const COMBO_RECT: Rect = Rect::new(46, 13, 138, 17);
pub const HP_BAR: &str = "L2UI_ch3.playerstatuswnd.ps_hpbar";
pub const MP_BAR: &str = "L2UI_ch3.PlayerStatusWnd.ps_mpbar";
pub const EXP_BAR: &str = "L2UI_ch3.PlayerStatusWnd.ps_expbar";
pub const BAR_MAX: [i32; 3] = [85, 85, 206];
pub const BAR_POSITIONS: [(i32, i32); 3] = [(0x23, 0x37), (0x23, 0x45), (0x23, 0x53)];
pub const NAME_LABEL_POS: (i32, i32) = (15, 16);
pub const LEVEL_LABEL_POS: (i32, i32) = (15, 39);
pub const KARMA_LABEL_POS: (i32, i32) = (131, 69);
pub const LEVEL_VALUE_POS: (i32, i32) = (0x20, 0x27);
pub const VALUE_RIGHT: i32 = 0xec;
pub const SP_VALUE_Y: i32 = 0x38;
pub const KARMA_VALUE_Y: i32 = 0x46;
pub const KARMA_PLUS_FROM: i32 = 999_999;
pub const START_SIZE: (i32, i32) = (140, 49);
pub const START_BACK: &str = "L2UI_CH3.LobbyWnd.lobby_startbtnback";
pub const START_BUTTON_RECT: Rect = Rect::new(13, 10, 116, 31);
pub const START_NORMAL: &str = "L2UI_CH3.Button.BigButton2";
pub const START_DOWN: &str = "L2UI_CH3.Button.BigButton2_down";
pub const START_ORIGIN: (i32, i32) = (71, 111);
pub const FUNCTION_SIZE: (i32, i32) = (120, 138);
pub const FUNCTION_MARGIN: (i32, i32) = (154, 200);
pub const FUNCTION_BACK: &str = "L2UI_CH3.LobbyWnd.lobby_menuback";
pub const CREATE_RECT: Rect = Rect::new(13, 20, 96, 23);
pub const DELETE_RECT: Rect = Rect::new(13, 45, 96, 23);
pub const EXIT_RECT: Rect = Rect::new(13, 95, 96, 23);
pub const BIG_BUTTON: &str = "L2UI_CH3.Button.BigButton";
pub const BIG_BUTTON_DOWN: &str = "L2UI_CH3.Button.BigButton_down";
pub const DIALOG_CREATE_SIZE: (i32, i32) = (203, 87);
pub const SYSMSG_TOO_MANY: u32 = 77;
pub const SYSMSG_DELETE_CONFIRM: u32 = 78;
pub const SYSMSG_BAD_NAME: u32 = 205;
pub const SYSMSG_RESTORE_CONFIRM: u32 = 1555;
pub const SYSSTRINGS: [u32; 12] = [157, 50, 88, 101, 158, 159, 884, 160, 184, 185, 1108, 0];
pub const NAME_FONT_OFFSET: i32 = 0;

#[derive(Debug, Clone, PartialEq)]
pub struct CharCard {
    pub name: String,
    pub level: i32,
    pub class_name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub exp: i64,
    pub sp: i32,
    pub karma: i32,
    pub delete_seconds: i32,
    pub dead: bool,
    pub last_used: bool,
}

impl CharCard {
    pub fn pending_delete(&self) -> bool {
        self.delete_seconds > 0 && !self.dead
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NameTag {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub color: Argb,
    pub second_line: Option<SecondLine>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SecondLine {
    pub x: i32,
    pub y: i32,
    pub text: String,
}

pub fn karma_color(karma: i32) -> Argb {
    let shade = if karma <= 0 { 0 } else { karma.min(255).max((karma >> 4) + 100).min(255) };
    let channel = (0xff - shade) as u32;
    Argb(0xff00_0000 | 0xff << 16 | channel << 8 | channel)
}

pub fn remaining_text(seconds: i32, units: [&str; 3]) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3600;
    let minutes = (seconds % 3600) / 60;
    format!("{days}{} {hours}{} {minutes}{}", units[0], units[1], units[2])
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelectTexts {
    pub title: String,
    pub name: String,
    pub level: String,
    pub karma: String,
    pub create: String,
    pub delete: String,
    pub exit: String,
    pub start: String,
    pub yes: String,
    pub no: String,
    pub delete_line: String,
    pub units: [String; 3],
    pub delete_confirm: String,
    pub restore_confirm: String,
}

impl SelectTexts {
    pub fn from_tables(sysstring: impl Fn(u32) -> Option<String>, systemmsg: impl Fn(u32) -> Option<String>) -> SelectTexts {
        let text = |id: u32| sysstring(id).unwrap_or_default();
        SelectTexts {
            title: text(SYSSTRINGS[0]),
            name: text(SYSSTRINGS[1]),
            level: text(SYSSTRINGS[2]),
            karma: text(SYSSTRINGS[3]),
            create: text(SYSSTRINGS[4]),
            delete: text(SYSSTRINGS[5]),
            exit: text(SYSSTRINGS[6]),
            start: text(SYSSTRINGS[7]),
            yes: text(SYSSTRINGS[8]),
            no: text(SYSSTRINGS[9]),
            delete_line: text(SYSSTRINGS[10]),
            units: [text(1109), text(1110), text(1111)],
            delete_confirm: systemmsg(SYSMSG_DELETE_CONFIRM).unwrap_or_default(),
            restore_confirm: systemmsg(SYSMSG_RESTORE_CONFIRM).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    Combo,
    Start,
    Create,
    Delete,
    Exit,
    DialogYes,
    DialogNo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    Delete,
    Restore,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dialog {
    pub kind: DialogKind,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectAction {
    Select(Option<usize>),
    Start(usize),
    Create,
    Delete(usize),
    Restore(usize),
    Exit,
    Message(u32),
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectScreen {
    pub screen: (i32, i32),
    pub texts: SelectTexts,
    pub cards: Vec<CharCard>,
    pub combo: ComboBox,
    pub selected: Option<usize>,
    pub start_enabled: bool,
    pub delete_enabled: bool,
    pub locked: bool,
    pub dialog: Option<Dialog>,
    pub pressed: Option<Hit>,
    pub stop_voices: bool,
}

impl SelectScreen {
    pub fn new(screen: (i32, i32), texts: SelectTexts) -> SelectScreen {
        SelectScreen {
            screen,
            texts,
            cards: Vec::new(),
            combo: ComboBox::new(COMBO_RECT, Vec::new(), 13),
            selected: None,
            start_enabled: false,
            delete_enabled: false,
            locked: false,
            dialog: None,
            pressed: None,
            stop_voices: false,
        }
    }

    pub fn start_origin(&self) -> (i32, i32) {
        (self.screen.0 / 2 - START_ORIGIN.0, self.screen.1 - START_ORIGIN.1)
    }

    pub fn function_origin(&self) -> (i32, i32) {
        (self.screen.0 - FUNCTION_MARGIN.0, self.screen.1 - FUNCTION_MARGIN.1)
    }

    pub fn dialog_origin(&self) -> (i32, i32) {
        let height = self.screen.1 - 52;
        ((self.screen.0 as f32 * 0.5 - 101.0) as i32, (height as f32 * 0.5 - 44.0) as i32)
    }

    pub fn reset(&mut self) {
        self.cards.clear();
        self.combo.clear_entries();
        self.selected = None;
        self.dialog = None;
        self.enable_all();
    }

    pub fn show(&mut self) {
        self.combo.selected = -1;
        self.selected = None;
        self.dialog = None;
        self.enable_all();
    }

    fn enable_all(&mut self) {
        self.start_enabled = true;
        self.delete_enabled = true;
    }

    pub fn set_cards(&mut self, cards: Vec<CharCard>) -> Option<SelectAction> {
        self.reset();
        let mut auto = None;
        for card in cards.into_iter().take(MAX_CHARACTERS) {
            let index = self.cards.len();
            self.combo.entries.push(card.name.clone());
            if card.last_used && !card.pending_delete() {
                auto = Some(index);
            }
            self.cards.push(card);
        }
        auto.and_then(|index| self.choose(index))
    }

    pub fn choose(&mut self, index: usize) -> Option<SelectAction> {
        let Some(card) = self.cards.get(index) else {
            self.selected = None;
            return Some(SelectAction::Select(None));
        };
        self.combo.selected = index as i32;
        if card.pending_delete() {
            self.selected = Some(index);
            self.start_enabled = false;
            self.delete_enabled = false;
            self.dialog = Some(Dialog { kind: DialogKind::Restore, text: self.texts.restore_confirm.clone() });
            return Some(SelectAction::Select(None));
        }
        if card.dead {
            self.selected = None;
            self.start_enabled = false;
            self.delete_enabled = false;
            return Some(SelectAction::Select(None));
        }
        self.selected = Some(index);
        self.start_enabled = true;
        self.delete_enabled = true;
        Some(SelectAction::Select(Some(index)))
    }

    pub fn start(&mut self, name_ok: impl Fn(&str) -> bool) -> Option<SelectAction> {
        let selected = self.selected?;
        let card = self.cards.get(selected)?;
        if card.pending_delete() || card.dead || self.cards.is_empty() {
            return None;
        }
        if !card.name.is_empty() && !name_ok(&card.name) {
            return Some(SelectAction::Message(SYSMSG_BAD_NAME));
        }
        Some(SelectAction::Start(selected))
    }

    pub fn create(&mut self) -> SelectAction {
        if self.cards.len() >= MAX_CHARACTERS { SelectAction::Message(SYSMSG_TOO_MANY) } else { SelectAction::Create }
    }

    pub fn delete(&mut self) {
        let Some(selected) = self.selected else { return };
        let Some(card) = self.cards.get(selected) else { return };
        if card.pending_delete() || card.dead {
            return;
        }
        self.dialog = Some(Dialog { kind: DialogKind::Delete, text: self.texts.delete_confirm.replace("$s1", &card.name) });
    }

    fn answer(&mut self, yes: bool) -> Option<SelectAction> {
        let dialog = self.dialog.take()?;
        let selected = self.selected;
        let action = match (dialog.kind, yes, selected) {
            (DialogKind::Delete, true, Some(index)) => Some(SelectAction::Delete(index)),
            (DialogKind::Restore, true, _) => self.combo.selected.try_into().ok().map(SelectAction::Restore),
            _ => None,
        };
        if action.is_none() || dialog.kind == DialogKind::Restore {
            self.combo.selected = -1;
            self.selected = None;
            self.enable_all();
        }
        action.or(Some(SelectAction::Select(None)))
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<Hit> {
        let origin = self.dialog_origin();
        if self.dialog.is_some() {
            let (dx, dy) = (x - origin.0, y - origin.1);
            return match self.dialog_view().and_then(|view| view.button_at(dx, dy)) {
                Some(crate::dialog::DialogButton::First) => Some(Hit::DialogYes),
                Some(crate::dialog::DialogButton::Second) => Some(Hit::DialogNo),
                _ => None,
            };
        }
        let combo = self.combo.expanded_rect();
        if combo.offset(SELECT_RECT.x, SELECT_RECT.y).contains(x, y) {
            return Some(Hit::Combo);
        }
        let start = self.start_origin();
        if START_BUTTON_RECT.offset(start.0, start.1).contains(x, y) && self.start_enabled {
            return Some(Hit::Start);
        }
        let function = self.function_origin();
        for (rect, hit, enabled) in [(CREATE_RECT, Hit::Create, !self.locked), (DELETE_RECT, Hit::Delete, self.delete_enabled), (EXIT_RECT, Hit::Exit, !self.locked)] {
            if rect.offset(function.0, function.1).contains(x, y) && enabled {
                return Some(hit);
            }
        }
        None
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        self.combo.mouse_move(x - SELECT_RECT.x, y - SELECT_RECT.y);
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<SelectAction> {
        self.pressed = self.hit(x, y);
        if self.dialog.is_none() {
            if self.pressed == Some(Hit::Combo) {
                self.combo.mouse_move(x - SELECT_RECT.x, y - SELECT_RECT.y);
                if let Some(index) = self.combo.mouse_down() {
                    return self.choose(index as usize);
                }
            } else {
                self.combo.lose_focus();
            }
        }
        None
    }

    pub fn mouse_up(&mut self, x: i32, y: i32, name_ok: impl Fn(&str) -> bool) -> Option<SelectAction> {
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            Hit::Combo => None,
            Hit::Start => {
                self.stop_voices = true;
                self.start(name_ok)
            }
            Hit::Create => Some(self.create()),
            Hit::Delete => {
                self.delete();
                None
            }
            Hit::Exit => {
                self.stop_voices = true;
                Some(SelectAction::Exit)
            }
            Hit::DialogYes => self.answer(true),
            Hit::DialogNo => self.answer(false),
        }
    }

    pub fn enter(&mut self, name_ok: impl Fn(&str) -> bool) -> Option<SelectAction> {
        if self.dialog.is_some() {
            return None;
        }
        self.stop_voices = true;
        self.start(name_ok)
    }

    pub fn paint(&self, list: &mut DrawList, font: &BitmapFont, tags: &[NameTag]) {
        for tag in tags {
            let width = font.measure(&tag.name) as i32;
            list.draw_text(tag.x - width / 2, tag.y + NAME_FONT_OFFSET, tag.color, &tag.name, font);
            if let Some(line) = &tag.second_line {
                let width = font.measure(&line.text) as i32;
                list.draw_text(line.x - width / 2, line.y, TEXT, &line.text, font);
            }
        }
        self.paint_select(list, font);
        self.paint_start(list, font);
        self.paint_function(list, font);
        if self.dialog.is_some() {
            self.paint_dialog(list, font);
        }
    }

    fn paint_select(&self, list: &mut DrawList, font: &BitmapFont) {
        let (ox, oy) = (SELECT_RECT.x, SELECT_RECT.y);
        list.draw_texture(ox, oy, SELECT_RECT.w, SELECT_RECT.h, 0, 0, SELECT_RECT.w, SELECT_RECT.h, SELECT_BACK, 1.0);
        list.draw_text(ox + NAME_LABEL_POS.0, oy + NAME_LABEL_POS.1, Argb(0xffa3a3a3), &self.texts.name, font);
        list.draw_text(ox + LEVEL_LABEL_POS.0, oy + LEVEL_LABEL_POS.1, Argb(0xffa3a3a3), &self.texts.level, font);
        list.draw_text(ox + KARMA_LABEL_POS.0, oy + KARMA_LABEL_POS.1, TEXT, &self.texts.karma, font);
        if let Some(card) = self.selected.and_then(|index| self.cards.get(index)) {
            self.paint_card(list, font, card, ox, oy);
        }
        self.combo.paint(list, (ox, oy), font, true, false);
    }

    fn paint_card(&self, list: &mut DrawList, font: &BitmapFont, card: &CharCard, ox: i32, oy: i32) {
        let hp = if card.max_hp != 0 { card.hp * BAR_MAX[0] / card.max_hp } else { 0 };
        let mp = if card.max_mp != 0 { card.mp * BAR_MAX[1] / card.max_mp } else { 0 };
        let exp = exp_fill(card.level, card.exp, BAR_MAX[2]);
        for (index, (value, texture)) in [(hp, HP_BAR), (mp, MP_BAR), (exp, EXP_BAR)].into_iter().enumerate() {
            let (x, y) = BAR_POSITIONS[index];
            let max = BAR_MAX[index];
            if value > max {
                list.draw_texture(ox + x, oy + y, max, 10, 0, 0, 8, 12, texture, 1.0);
            } else if value >= 1 {
                list.draw_texture(ox + x, oy + y, value, 12, 0, 0, 8, 12, texture, 1.0);
            }
        }
        let percent = format!("{:.2}%", exp_percent(card.level, card.exp));
        list.draw_digits(ox + 0x8a - font.measure(&percent) as i32 / 2, oy + 0x55, &percent, TEXT);
        list.draw_digits(ox + 0x4d - font.measure("/") as i32 / 2, oy + 0x39, "/", TEXT);
        list.draw_digits(ox + 0x43 - font.measure(&card.hp.to_string()) as i32, oy + 0x39, &card.hp.to_string(), TEXT);
        list.draw_digits(ox + 0x53, oy + 0x39, &card.max_hp.to_string(), TEXT);
        list.draw_digits(ox + 0x4d - font.measure("/") as i32 / 2, oy + 0x47, "/", TEXT);
        list.draw_digits(ox + 0x43 - font.measure(&card.mp.to_string()) as i32, oy + 0x47, &card.mp.to_string(), TEXT);
        list.draw_digits(ox + 0x53, oy + 0x47, &card.max_mp.to_string(), TEXT);
        let sp = card.sp.to_string();
        list.draw_text(ox + VALUE_RIGHT - font.measure(&sp) as i32, oy + SP_VALUE_Y, GOLD, &sp, font);
        let karma = if card.karma >= KARMA_PLUS_FROM { format!("{}+", card.karma) } else { card.karma.to_string() };
        list.draw_text(ox + VALUE_RIGHT - font.measure(&karma) as i32, oy + KARMA_VALUE_Y, GOLD, &karma, font);
        list.draw_text(ox + LEVEL_VALUE_POS.0, oy + LEVEL_VALUE_POS.1, GOLD, &format!("{} {}", card.level, card.class_name), font);
    }

    fn paint_start(&self, list: &mut DrawList, font: &BitmapFont) {
        let (ox, oy) = self.start_origin();
        list.draw_texture(ox, oy, START_SIZE.0, START_SIZE.1, 0, 0, START_SIZE.0, START_SIZE.1, START_BACK, 1.0);
        let mut button = TextButton::new(START_BUTTON_RECT, START_NORMAL, START_DOWN, &self.texts.start);
        button.enabled = self.start_enabled;
        button.paint(list, ox, oy, self.pressed == Some(Hit::Start), font);
    }

    fn paint_function(&self, list: &mut DrawList, font: &BitmapFont) {
        let (ox, oy) = self.function_origin();
        list.draw_texture(ox, oy, FUNCTION_SIZE.0, FUNCTION_SIZE.1, 0, 0, FUNCTION_SIZE.0, FUNCTION_SIZE.1, FUNCTION_BACK, 1.0);
        for (rect, label, hit, enabled) in [
            (CREATE_RECT, &self.texts.create, Hit::Create, !self.locked),
            (DELETE_RECT, &self.texts.delete, Hit::Delete, self.delete_enabled),
            (EXIT_RECT, &self.texts.exit, Hit::Exit, !self.locked),
        ] {
            let mut button = TextButton::new(rect, BIG_BUTTON, BIG_BUTTON_DOWN, label);
            button.enabled = enabled;
            button.paint(list, ox, oy, self.pressed == Some(hit), font);
        }
    }

    pub fn dialog_view(&self) -> Option<crate::dialog::DialogView> {
        let dialog = self.dialog.as_ref()?;
        let mut view = crate::dialog::DialogView::new(crate::dialog::DialogKind::YesNo, &dialog.text);
        view.labels = (self.texts.yes.clone(), self.texts.no.clone());
        view.dialog_type = if dialog.kind == DialogKind::Restore { 1 } else { 0 };
        Some(view)
    }

    fn paint_dialog(&self, list: &mut DrawList, font: &BitmapFont) {
        let (ox, oy) = self.dialog_origin();
        let Some(view) = self.dialog_view() else { return };
        let pressed = match self.pressed {
            Some(Hit::DialogYes) => Some(crate::dialog::DialogButton::First),
            Some(Hit::DialogNo) => Some(crate::dialog::DialogButton::Second),
            _ => None,
        };
        crate::dialog::paint_dialog(list, ox, oy, &view, pressed, font);
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

    fn texts() -> SelectTexts {
        SelectTexts::from_tables(|id| Some(format!("s{id}")), |id| Some(format!("m{id} $s1")))
    }

    fn card(name: &str, delete_seconds: i32, dead: bool, last_used: bool) -> CharCard {
        CharCard { name: name.into(), level: 20, class_name: "Knight".into(), hp: 50, max_hp: 100, mp: 10, max_mp: 20, exp: 5000, sp: 7, karma: 0, delete_seconds, dead, last_used }
    }

    fn screen() -> SelectScreen {
        SelectScreen::new((1024, 768), texts())
    }

    #[test]
    fn the_auto_log_lock_disables_create_and_the_exit_button_like_1006e_autolog() {
        let mut screen = SelectScreen::new((1024, 768), SelectTexts::default());
        let function = screen.function_origin();
        let create = (function.0 + CREATE_RECT.x + 2, function.1 + CREATE_RECT.y + 2);
        let exit = (function.0 + EXIT_RECT.x + 2, function.1 + EXIT_RECT.y + 2);
        assert_eq!((screen.hit(create.0, create.1), screen.hit(exit.0, exit.1)), (Some(Hit::Create), Some(Hit::Exit)));
        screen.locked = true;
        assert_eq!((screen.hit(create.0, create.1), screen.hit(exit.0, exit.1)), (None, None));
    }

    #[test]
    fn windows_sit_where_createlobbywindows_puts_them() {
        let screen = screen();
        assert_eq!(screen.start_origin(), (1024 / 2 - 71, 768 - 111));
        assert_eq!(screen.function_origin(), (1024 - 154, 768 - 200));
        assert_eq!(screen.dialog_origin(), (411, (716.0f32 * 0.5 - 44.0) as i32));
        assert_eq!(SELECT_RECT, Rect::new(24, 27, 256, 108));
    }

    #[test]
    fn dialog_show_10009710_resizes_to_256x108_with_buttons_at_51_and_131_by_75() {
        let mut screen = screen();
        screen.dialog = Some(Dialog { kind: DialogKind::Delete, text: "x".into() });
        let view = screen.dialog_view().unwrap();
        let [yes, no] = view.buttons();
        assert_eq!((yes.rect, no.rect), (Rect::new(51, 75, 76, 23), Rect::new(131, 75, 76, 23)));
        let (ox, oy) = screen.dialog_origin();
        assert_eq!(screen.hit(ox + 60, oy + 80), Some(Hit::DialogYes));
        assert_eq!(screen.hit(ox + 140, oy + 80), Some(Hit::DialogNo));
        assert_eq!(screen.hit(ox + 30, oy + 60), None);
        let mut list = DrawList::default();
        screen.paint(&mut list, &font(), &[]);
        let back = list.quads.iter().find(|q| q.texture == crate::dialog::SYSTEM_BACK).unwrap();
        assert_eq!((back.x, back.y, back.w, back.h), (ox as f32, oy as f32, 256.0, 108.0));
        assert_eq!(DIALOG_CREATE_SIZE, (203, 87));
    }

    #[test]
    fn the_last_used_character_comes_selected_unless_it_is_pending_deletion() {
        let mut screen = screen();
        let action = screen.set_cards(vec![card("A", 0, false, false), card("B", 0, false, true)]);
        assert_eq!(action, Some(SelectAction::Select(Some(1))));
        assert_eq!(screen.selected, Some(1));
        let action = screen.set_cards(vec![card("A", 0, false, false), card("B", 3600, false, true)]);
        assert_eq!(action, None);
        assert_eq!(screen.selected, None);
        assert!(screen.dialog.is_none());
    }

    #[test]
    fn choosing_a_pending_or_dead_character_selects_nobody_and_disables_start_and_delete() {
        let mut screen = screen();
        screen.set_cards(vec![card("A", 0, false, false), card("P", 3600, false, false), card("D", 0, true, false)]);
        assert_eq!(screen.choose(0), Some(SelectAction::Select(Some(0))));
        assert!(screen.start_enabled && screen.delete_enabled);
        assert_eq!(screen.choose(1), Some(SelectAction::Select(None)));
        assert_eq!(screen.selected, Some(1));
        assert!(!screen.start_enabled && !screen.delete_enabled);
        assert_eq!(screen.dialog.as_ref().map(|d| d.kind), Some(DialogKind::Restore));
        assert_eq!(screen.dialog.as_ref().map(|d| d.text.as_str()), Some("m1555 $s1"));
        screen.dialog = None;
        assert_eq!(screen.choose(2), Some(SelectAction::Select(None)));
        assert!(screen.dialog.is_none() && !screen.start_enabled);
    }

    #[test]
    fn start_checks_the_selection_the_state_and_the_name_glyphs() {
        let mut screen = screen();
        screen.set_cards(vec![card("A", 0, false, false)]);
        assert_eq!(screen.start(|_| true), None);
        screen.choose(0);
        assert_eq!(screen.start(|_| false), Some(SelectAction::Message(SYSMSG_BAD_NAME)));
        assert_eq!(screen.start(|_| true), Some(SelectAction::Start(0)));
    }

    #[test]
    fn create_is_refused_with_seven_characters_by_message_77() {
        let mut screen = screen();
        assert_eq!(screen.create(), SelectAction::Create);
        screen.set_cards((0..9).map(|i| card(&format!("C{i}"), 0, false, false)).collect());
        assert_eq!(screen.cards.len(), MAX_CHARACTERS);
        assert_eq!(screen.create(), SelectAction::Message(SYSMSG_TOO_MANY));
    }

    #[test]
    fn delete_asks_with_the_name_and_yes_sends_it_while_no_deselects_everything() {
        let mut screen = screen();
        screen.set_cards(vec![card("Hero", 0, false, false)]);
        screen.choose(0);
        screen.delete();
        assert_eq!(screen.dialog.as_ref().map(|d| d.text.as_str()), Some("m78 Hero"));
        assert_eq!(screen.answer(true), Some(SelectAction::Delete(0)));
        screen.choose(0);
        screen.delete();
        assert_eq!(screen.answer(false), Some(SelectAction::Select(None)));
        assert_eq!(screen.selected, None);
        assert!(screen.start_enabled && screen.delete_enabled);
    }

    #[test]
    fn restore_yes_sends_the_request_for_the_pending_slot_and_always_clears_the_selection() {
        let mut screen = screen();
        screen.set_cards(vec![card("A", 0, false, false), card("P", 3600, false, false)]);
        screen.choose(1);
        assert_eq!(screen.answer(true), Some(SelectAction::Restore(1)));
        assert_eq!(screen.selected, None);
        assert!(screen.start_enabled && screen.delete_enabled);
    }

    #[test]
    fn karma_turns_the_name_red_and_the_deletion_time_is_split_in_days_hours_minutes() {
        assert_eq!(karma_color(0), Argb(0xffffffff));
        assert_eq!(karma_color(255), Argb(0xffff0000));
        assert_eq!(karma_color(100), Argb(0xffff9595));
        assert_eq!(remaining_text(86_400 + 3600 * 2 + 60 * 5, ["d", "h", "m"]), "1d 2h 5m");
    }

    #[test]
    fn the_card_paints_bars_digits_and_the_class_line_only_for_a_valid_selection() {
        let mut screen = screen();
        screen.set_cards(vec![card("A", 0, false, true)]);
        let mut list = DrawList { capture_text: true, ..Default::default() };
        screen.paint(&mut list, &font(), &[]);
        let texts: Vec<&str> = list.text_runs.iter().map(|run| run.text.as_str()).collect();
        assert!(texts.contains(&"20 Knight"));
        assert!(texts.iter().any(|t| t.ends_with('%')));
        let mut empty = DrawList { capture_text: true, ..Default::default() };
        screen.reset();
        screen.paint(&mut empty, &font(), &[]);
        assert!(!empty.text_runs.iter().any(|run| run.text == "20 Knight"));
    }

    #[test]
    fn a_pending_character_stays_selected_so_its_card_is_painted_while_the_restore_question_waits() {
        let mut screen = screen();
        screen.set_cards(vec![card("P", 3600, false, false)]);
        screen.choose(0);
        assert_eq!(screen.selected, Some(0));
        assert!(screen.dialog.is_some());
        let mut list = DrawList { capture_text: true, ..Default::default() };
        screen.paint(&mut list, &font(), &[]);
        assert!(list.text_runs.iter().any(|run| run.text == "20 Knight"));
    }

    #[test]
    fn the_slashes_of_the_bars_are_centered_on_x_77_by_the_extent_of_the_slash() {
        let mut screen = screen();
        screen.set_cards(vec![card("A", 0, false, true)]);
        let mut list = DrawList { capture_text: true, ..Default::default() };
        screen.paint(&mut list, &font(), &[]);
        let slash = screen.selected.map(|_| font().measure("/") as i32 / 2).unwrap();
        assert!(slash > 0);
        let digits: Vec<f32> = list.quads.iter().filter(|q| &*q.texture == crate::draw::NUMBER_STRIP.texture).map(|q| q.x).collect();
        assert!(digits.contains(&((SELECT_RECT.x + 0x4d - slash) as f32)));
    }

    #[test]
    fn start_and_exit_clicks_and_enter_ask_for_the_voices_to_fade_out_even_when_the_start_is_refused() {
        let mut screen = screen();
        screen.set_cards(vec![card("A", 0, false, false)]);
        assert_eq!(screen.enter(|_| true), None);
        assert!(std::mem::take(&mut screen.stop_voices));
        let (ox, oy) = screen.function_origin();
        let (x, y) = (ox + EXIT_RECT.x + 1, oy + EXIT_RECT.y + 1);
        screen.mouse_down(x, y);
        assert_eq!(screen.mouse_up(x, y, |_| true), Some(SelectAction::Exit));
        assert!(screen.stop_voices);
    }

    #[test]
    fn the_combo_selects_a_character_by_clicking_the_box_and_then_a_row() {
        let mut screen = screen();
        screen.set_cards(vec![card("A", 0, false, false), card("B", 0, false, false)]);
        let box_point = (SELECT_RECT.x + COMBO_RECT.x + 5, SELECT_RECT.y + COMBO_RECT.y + 5);
        assert_eq!(screen.mouse_down(box_point.0, box_point.1), None);
        assert!(screen.combo.open);
        let row = (box_point.0, SELECT_RECT.y + COMBO_RECT.y + COMBO_RECT.h + crate::combo::ROW_HEIGHT + 3);
        screen.mouse_move(row.0, row.1);
        assert_eq!(screen.mouse_down(row.0, row.1), Some(SelectAction::Select(Some(1))));
        assert_eq!(screen.selected, Some(1));
    }
}
