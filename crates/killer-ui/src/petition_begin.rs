#[path = "petition_begin_controls_generated.rs"]
mod controls;
#[path = "petition_begin_layout_generated.rs"]
mod layout;

use crate::button::TextButton;
use crate::color::Argb;
use crate::combo::ComboBox;
use crate::draw::DrawList;
use crate::edit_box::FontMetrics;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::html::{self, Line};
use crate::ini::IniFile;
use crate::multi_edit::MultiEdit;
use crate::npc_viewer::paint_html_viewer;
use crate::window::{Rect, WindowId};

pub const ID: WindowId = WindowId::PETITION_BEGIN;
pub const SIZE: (i32, i32) = layout::SIZE;
pub const BACK: &str = "L2UI_ch3.PetitionWnd.petition1_back";
pub const BACK_ORIGIN: (i32, i32) = (0, 20);
pub const TEXT_COLOR: Argb = Argb(0xFFDCDCDC);
pub const TEXT_POSITIONS: [(i32, i32); 3] = [(22, 30), (22, 53), (22, 226)];
pub const TEXT_SYSSTRINGS: [u32; 3] = [705, 412, 707];
pub const SELECT_TYPE_SYSSTRING: u32 = 695;
pub const MULTI_EDIT_ID: u32 = 0xf0e0a4;
pub const PAGE_PREFIX: &str = "..\\L2text\\";
pub const SELECT_TYPE_SYSMSG: u32 = 804;
pub const TOO_SHORT_SYSMSG: u32 = 386;
pub const TOO_LONG_SYSMSG: u32 = 971;
pub const MIN_CHARS: usize = 5;
pub const MAX_CHARS: usize = 255;
pub const LOCAL_LINE_SYSSTRING: u32 = 708;
pub const LOCAL_LINE_COLOR: Argb = Argb(0xFFDCDCDC);
pub const CATEGORY_SECTION: &str = "CategoryNum";
pub const CATEGORY_COUNT_KEY: &str = "Num";
pub const CATEGORY_KEY_TITLE: &str = "Title";
pub const CATEGORY_KEY_LINK: &str = "Link";
pub const CATEGORY_LINK_CHARS: usize = 0xff;
pub const WHEEL_STEP: i32 = crate::npc_viewer::HTML_WHEEL_STEP;

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    (0, (viewport.1 as f32 * 0.5 - 252.0) as i32)
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, y) = default_position(viewport);
    crate::initial_position::InitialPosition {
        enabled: true,
        horizontal_mode: 0,
        vertical_mode: 5,
        x,
        y: (viewport.1 / 2).wrapping_sub(y),
    }
}

pub fn grip() -> Rect {
    layout::RECT_7C
}

pub fn viewer_rect() -> Rect {
    layout::RECT_F0
}

pub fn edit_rect() -> Rect {
    layout::RECT_F8
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Category {
    pub title: u32,
    pub link: String,
}

pub fn categories(localization: &IniFile) -> Vec<Category> {
    let count = localization.int(CATEGORY_SECTION, CATEGORY_COUNT_KEY).unwrap_or(0);
    let mut list = Vec::new();
    for index in 1..=count {
        let section = format!("Category{index}");
        let Some(title) = localization.int(&section, CATEGORY_KEY_TITLE) else { break };
        let Some(link) = localization.get(&section, CATEGORY_KEY_LINK) else { break };
        list.push(Category { title: u32::try_from(title).unwrap_or(0), link: link.chars().take(CATEGORY_LINK_CHARS).collect() });
    }
    list
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BeginAction {
    Close,
    Submit { text: String, kind: i32 },
    Cancel,
    Notice(u32),
    LoadPage(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeginHit {
    Close,
    Combo,
    Submit,
    Cancel,
    Viewer,
    Edit,
}

#[derive(Clone, Debug)]
pub struct PetitionBegin {
    pub frame: FrameCtrl,
    pub combo: ComboBox,
    pub edit: MultiEdit,
    pub buttons: [TextButton; 2],
    pub categories: Vec<Category>,
    pub lines: Vec<Line>,
    pub page_height: i32,
    pub page_scroll: i32,
    pub texts: [String; 3],
    pub local_line_label: String,
    select_type: String,
    pressed: Option<BeginHit>,
    arrow_pressed: bool,
}

impl PetitionBegin {
    pub fn new(lookup: impl Fn(u32) -> Option<String>, line_height: i32, categories: Vec<Category>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        let select_type = text(SELECT_TYPE_SYSSTRING);
        let mut entries = vec![select_type.clone()];
        entries.extend(categories.iter().map(|category| text(category.title)));
        Self {
            frame: controls::FRAME_7C.build(text),
            combo: ComboBox::from_contract(&controls::CHILD_F4, entries, line_height),
            edit: MultiEdit::new(MULTI_EDIT_ID, layout::RECT_F8, line_height),
            buttons: [controls::BUTTON_E8.build((0, 0), text), controls::BUTTON_EC.build((0, 0), text)],
            categories,
            lines: Vec::new(),
            page_height: 0,
            page_scroll: 0,
            texts: TEXT_SYSSTRINGS.map(text),
            local_line_label: text(LOCAL_LINE_SYSSTRING),
            select_type,
            pressed: None,
            arrow_pressed: false,
        }
    }

    pub fn reset(&mut self) {
        self.combo.selected = 0;
        self.edit.clear();
        self.lines.clear();
        self.page_height = 0;
        self.page_scroll = 0;
    }

    pub fn open(&mut self, preset: Option<(&str, &BitmapFont)>) {
        self.reset();
        if let Some((text, font)) = preset.filter(|(text, _)| !text.is_empty()) {
            self.edit.set_text(text, &FontMetrics(font));
        }
        self.edit.focus();
    }

    pub fn close(&mut self) {
        self.edit.kill_focus();
        self.combo.lose_focus();
        self.pressed = None;
        self.arrow_pressed = false;
    }

    pub fn select_type_label(&self) -> &str {
        &self.select_type
    }

    pub fn combo_changed(&self, index: i32) -> Option<BeginAction> {
        let position = usize::try_from(index).ok().filter(|position| (1..=self.categories.len()).contains(position))?;
        Some(BeginAction::LoadPage(format!("{PAGE_PREFIX}{}", self.categories[position - 1].link)))
    }

    pub fn set_page(&mut self, html_text: &str, font: &BitmapFont) {
        self.set_page_with(html_text, font, &|_, _| None);
    }

    pub fn set_page_with(&mut self, html_text: &str, font: &BitmapFont, resolve: &dyn Fn(html::Marker, i32) -> Option<String>) {
        self.lines = html::layout(&html::parse_with(html_text, resolve), layout::RECT_F0.w - crate::npc_viewer::HTML_WRAP_MARGIN, font);
        self.page_height = html::height(&self.lines, font);
        self.page_scroll = 0;
    }

    pub fn wheel_page(&mut self, ticks: i32) {
        let room = (self.page_height - layout::RECT_F0.h).max(0);
        self.page_scroll = (self.page_scroll - ticks * WHEEL_STEP).clamp(0, room);
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        if layout::RECT_F0.contains(x, y) {
            self.wheel_page(delta / crate::multi_edit::WHEEL_STEP);
            true
        } else if layout::RECT_F8.contains(x, y) {
            self.edit.wheel(delta);
            true
        } else {
            false
        }
    }

    pub fn submit(&mut self) -> BeginAction {
        let kind = self.combo.selected;
        if kind == 0 {
            return BeginAction::Notice(SELECT_TYPE_SYSMSG);
        }
        let Some(text) = self.edit.get_text() else {
            return BeginAction::Notice(TOO_SHORT_SYSMSG);
        };
        let length = text.encode_utf16().count();
        if length < MIN_CHARS {
            return BeginAction::Notice(TOO_SHORT_SYSMSG);
        }
        if length >= MAX_CHARS {
            return BeginAction::Notice(TOO_LONG_SYSMSG);
        }
        self.reset();
        BeginAction::Submit { text, kind }
    }

    pub fn cancel(&mut self) -> BeginAction {
        self.reset();
        BeginAction::Cancel
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<BeginHit> {
        if self.combo.open && self.combo.expanded_rect().contains(x, y) {
            return Some(BeginHit::Combo);
        }
        if controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(BeginHit::Close);
        }
        if controls::BUTTON_E8.hit(&self.buttons[0], x, y).is_some() {
            return Some(BeginHit::Submit);
        }
        if controls::BUTTON_EC.hit(&self.buttons[1], x, y).is_some() {
            return Some(BeginHit::Cancel);
        }
        if self.combo.rect.contains(x, y) {
            return Some(BeginHit::Combo);
        }
        if layout::RECT_F8.contains(x, y) {
            return Some(BeginHit::Edit);
        }
        layout::RECT_F0.contains(x, y).then_some(BeginHit::Viewer)
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        self.combo.mouse_move(x, y);
        self.edit.mouse_move(y);
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<BeginAction> {
        let hit = self.hit(x, y);
        if self.combo.open && hit != Some(BeginHit::Combo) {
            self.combo.lose_focus();
        }
        self.pressed = hit;
        match hit {
            Some(BeginHit::Combo) => {
                self.combo.mouse_move(x, y);
                self.arrow_pressed = self.combo.arrow_rect().contains(x, y);
                let index = self.combo.mouse_down()?;
                self.combo_changed(index)
            }
            Some(BeginHit::Edit) => {
                self.edit.focus();
                self.edit.mouse_down(x, y);
                None
            }
            _ => None,
        }
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<BeginAction> {
        self.arrow_pressed = false;
        self.edit.mouse_up();
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            BeginHit::Close => {
                self.close();
                Some(BeginAction::Close)
            }
            BeginHit::Submit => Some(self.submit()),
            BeginHit::Cancel => Some(self.cancel()),
            _ => None,
        }
    }

    pub fn key(&mut self, key: crate::chat::InputKey, font: &BitmapFont) -> bool {
        if !self.edit.focused {
            return false;
        }
        self.edit.input_key(key, &FontMetrics(font));
        true
    }

    pub fn tick(&mut self, elapsed_ms: u32) {
        self.edit.tick(elapsed_ms);
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        self.frame.paint(list, x, y, SIZE.0, SIZE.1, font, alpha);
        let start = list.quads.len();
        list.draw_texture(x + BACK_ORIGIN.0, y + BACK_ORIGIN.1, SIZE.0, SIZE.1 - BACK_ORIGIN.1, 0, 0, SIZE.0, SIZE.1 - BACK_ORIGIN.1, BACK, alpha);
        list.clip_from(start, Rect::new(x, y, SIZE.0, SIZE.1));
        for (position, text) in TEXT_POSITIONS.iter().zip(&self.texts) {
            list.draw_text(x + position.0, y + position.1, TEXT_COLOR, text, font);
        }
        let viewer = layout::RECT_F0;
        paint_html_viewer(list, x + viewer.x, y + viewer.y, (viewer.w, viewer.h), &self.lines, self.page_scroll, font, alpha);
        self.edit.paint(list, origin, &FontMetrics(font), font);
        controls::BUTTON_E8.paint(&self.buttons[0], list, origin, self.pressed == Some(BeginHit::Submit), font);
        controls::BUTTON_EC.paint(&self.buttons[1], list, origin, self.pressed == Some(BeginHit::Cancel), font);
        self.combo.paint(list, origin, font, true, self.arrow_pressed);
    }
}

pub fn local_line(label: &str, text: &str) -> String {
    format!("{label} : {text}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit_box::Metrics;
    use crate::ini::parse_ini;

    struct Fixed;

    impl Metrics for Fixed {
        fn text_width(&self, text: &[u16]) -> i32 {
            text.len() as i32 * 7
        }

        fn char_width(&self, _: u16) -> i32 {
            7
        }
    }

    fn view() -> PetitionBegin {
        let ini = parse_ini("[CategoryNum]\nNum=2\n[Category1]\nTitle=696\nLINK=pet_help_move.htm\n[Category2]\nTitle=697\nLINK=pet_help_recover.htm\n");
        PetitionBegin::new(|id| Some(format!("s{id}")), 13, categories(&ini))
    }

    #[test]
    fn the_categories_come_from_localization_with_case_insensitive_keys() {
        let view = view();
        assert_eq!(
            view.categories,
            vec![
                Category { title: 696, link: "pet_help_move.htm".into() },
                Category { title: 697, link: "pet_help_recover.htm".into() }
            ]
        );
        assert_eq!(view.combo.entries, vec!["s695", "s696", "s697"]);
    }

    #[test]
    fn loadcategories_100e22a0_stops_at_the_first_category_without_title_or_link() {
        let ini = parse_ini("[CategoryNum]\nNum=4\n[Category1]\nTitle=696\nLink=a.htm\n[Category2]\nTitle=697\n[Category3]\nTitle=698\nLink=c.htm\n");
        assert_eq!(categories(&ini), vec![Category { title: 696, link: "a.htm".into() }]);
    }

    #[test]
    fn the_layout_matches_the_on_create_snapshot() {
        let view = view();
        assert_eq!(SIZE, (256, 401));
        assert_eq!(view.combo.rect, Rect::new(51, 50, 145, 17));
        assert_eq!(viewer_rect(), Rect::new(8, 79, 240, 141));
        assert_eq!(edit_rect(), Rect::new(8, 241, 240, 124));
        assert_eq!(view.buttons[0].rect, Rect::new(51, 372, 76, 23));
        assert_eq!(view.buttons[1].rect, Rect::new(131, 372, 76, 23));
        assert_eq!(view.buttons[0].label, "s694");
        assert_eq!(view.buttons[1].label, "s473");
        assert_eq!(view.combo.selected, -1);
    }

    #[test]
    fn a_missing_type_is_the_804_notice_before_the_text_is_read() {
        let mut view = view();
        view.reset();
        assert_eq!(view.submit(), BeginAction::Notice(SELECT_TYPE_SYSMSG));
    }

    #[test]
    fn short_and_long_texts_are_refused_and_the_window_keeps_its_state() {
        let mut view = view();
        view.reset();
        view.combo.selected = 1;
        assert_eq!(view.submit(), BeginAction::Notice(TOO_SHORT_SYSMSG));
        view.edit.set_text("abcd", &Fixed);
        assert_eq!(view.submit(), BeginAction::Notice(TOO_SHORT_SYSMSG));
        assert_eq!(view.combo.selected, 1);
        view.edit.clear();
        view.edit.set_text(&"a".repeat(255), &Fixed);
        assert_eq!(view.submit(), BeginAction::Notice(TOO_LONG_SYSMSG));
        assert_eq!(view.combo.selected, 1);
    }

    #[test]
    fn a_valid_petition_sends_kind_and_text_and_resets_the_window() {
        let mut view = view();
        view.reset();
        view.combo.selected = 2;
        view.edit.set_text("help me", &Fixed);
        assert_eq!(view.submit(), BeginAction::Submit { text: "help me".into(), kind: 2 });
        assert_eq!(view.combo.selected, 0);
        assert_eq!(view.edit.get_text(), None);
    }

    #[test]
    fn cancel_resets_and_always_reports_the_cancel_request() {
        let mut view = view();
        view.combo.selected = 2;
        assert_eq!(view.cancel(), BeginAction::Cancel);
        assert_eq!(view.combo.selected, 0);
    }

    #[test]
    fn picking_a_category_requests_its_page_and_the_first_item_requests_nothing() {
        let view = view();
        assert_eq!(view.combo_changed(0), None);
        assert_eq!(view.combo_changed(2), Some(BeginAction::LoadPage("..\\L2text\\pet_help_recover.htm".into())));
        assert_eq!(view.combo_changed(3), None);
        assert_eq!(view.combo_changed(-1), None);
    }

    #[test]
    fn the_local_line_joins_label_and_text() {
        assert_eq!(local_line("Contents", "text"), "Contents : text");
    }
}
