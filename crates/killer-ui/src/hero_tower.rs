#[path = "hero_tower_content_controls_generated.rs"]
mod content_controls;
#[path = "hero_tower_content_layout_generated.rs"]
mod content_layout;
#[path = "hero_tower_controls_generated.rs"]
mod tower_controls;
#[path = "hero_tower_layout_generated.rs"]
mod tower_layout;

use crate::button::TextButton;
use crate::draw::DrawList;
use crate::edit_box::{EditBox, FontMetrics};
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::list_ctrl::{CellIcon, ListCell, ListCtrl, ListRow};
use crate::window::{Rect, WindowId};

pub const TOWER_ID: WindowId = WindowId::HERO_TOWER;
pub const CONTENT_ID: WindowId = WindowId::HERO_TOWER_CONTENT;
pub const TOWER_SIZE: (i32, i32) = tower_layout::SIZE;
pub const CONTENT_SIZE: (i32, i32) = content_layout::SIZE;
pub const TOWER_INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 2, x: TOWER_SIZE.0, y: TOWER_SIZE.1 };
pub const CONTENT_INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 2, x: CONTENT_SIZE.0, y: CONTENT_SIZE.1 };

pub fn tower_default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 * 0.5 - 300.0) as i32, (viewport.1 as f32 * 0.5 - 163.0) as i32)
}

pub fn content_default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 * 0.5 - 155.0) as i32, (viewport.1 as f32 * 0.5 - 200.0) as i32)
}

pub fn tower_grip() -> Rect {
    tower_layout::RECT_7C
}

pub fn content_grip() -> Rect {
    content_layout::RECT_7C
}

pub const TOWER_BACK: &str = "L2UI_CH3.HeroTower.herotower1_back";
pub const CONTENT_BACK: &str = "L2UI_CH3.HeroTower.herotower2_back";
pub const CELL_COUNT: usize = 7;
pub const CELL_CLASS: usize = 1;
pub const CELL_CLAN: usize = 2;
pub const DIARY_COMMAND: &str = "_diary?class=";
pub const DIARY_PAGE: &str = "&page=1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HeroRow {
    pub name: String,
    pub class_id: i32,
    pub class_name: String,
    pub clan: String,
    pub clan_crest: Option<String>,
    pub ally: String,
    pub ally_crest: Option<String>,
    pub count: i32,
}

pub fn clan_icons(clan: Option<&str>, ally: Option<&str>) -> Vec<CellIcon> {
    let icon = |texture: &str, x: i32, w: i32| CellIcon { texture: texture.to_string(), x, y: 0, w, h: 12, u: 0, v: 4, uw: 0, vh: 0 };
    match (ally, clan) {
        (Some(ally), Some(clan)) => vec![icon(ally, 10, 8), icon(clan, 18, 16)],
        (Some(ally), None) => vec![icon(ally, 10, 8)],
        (None, Some(clan)) => vec![icon(clan, 10, 16)],
        (None, None) => Vec::new(),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TowerHit {
    Close,
    Message,
    Record,
    Words,
    List,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TowerAction {
    Close,
    Words(String),
    Diary(String),
}

#[derive(Clone, Debug)]
pub struct HeroTowerView {
    pub frame: FrameCtrl,
    pub list: ListCtrl,
    pub words: EditBox,
    pub buttons: [TextButton; 2],
    pub classes: Vec<i32>,
    pub words_visible: bool,
    pub font: Option<BitmapFont>,
    pressed: Option<TowerHit>,
}

impl HeroTowerView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        let mut words = EditBox::from_contract(&tower_controls::CHILD_F0);
        words.set_max_len(64);
        Self {
            frame: tower_controls::FRAME_7C.build(text),
            list: ListCtrl::from_contract(&tower_controls::LIST_EC),
            words,
            buttons: [tower_controls::BUTTON_F4.build((0, 0), text), tower_controls::BUTTON_F8.build((0, 0), text)],
            classes: Vec::new(),
            words_visible: true,
            font: None,
            pressed: None,
        }
    }

    pub fn set_font(&mut self, font: &BitmapFont) {
        self.font = Some(font.clone());
    }

    pub fn set_language(&mut self, language: crate::ime::Language) {
        self.words.set_language(language);
    }

    pub fn set_heroes(&mut self, rows: &[HeroRow]) {
        self.list.clear();
        self.classes.clear();
        for row in rows {
            let mut cells: Vec<ListCell> = [&row.name, &row.class_name, &row.clan, &row.ally, &row.count.to_string()]
                .into_iter()
                .map(|text| ListCell::text(text))
                .collect();
            cells.resize(CELL_COUNT, ListCell::default());
            cells[CELL_CLAN].icons = clan_icons(row.clan_crest.as_deref(), row.ally_crest.as_deref());
            if self.list.add_item(ListRow { cells }) {
                self.classes.push(row.class_id);
            }
        }
    }

    pub fn set_owner_hero(&mut self, hero: Option<bool>) {
        if let Some(hero) = hero {
            self.words_visible = hero;
            if !hero && self.words.focused {
                self.words.kill_focus();
            }
        }
    }

    pub fn diary(&self) -> Option<String> {
        let row = self.list.selected_origin()?;
        let class = self.classes.get(row)?;
        Some(format!("{DIARY_COMMAND}{class}{DIARY_PAGE}"))
    }

    pub fn words_text(&mut self) -> String {
        let text = crate::edit_box::to_string(self.words.left());
        self.words.clear_left();
        match &self.font {
            Some(font) => font.strip_missing_glyphs(&text),
            None => text,
        }
    }

    fn button_hit(&self, x: i32, y: i32) -> Option<TowerHit> {
        if self.words_visible && tower_controls::BUTTON_F4.hit(&self.buttons[0], x, y).is_some() {
            return Some(TowerHit::Message);
        }
        tower_controls::BUTTON_F8.hit(&self.buttons[1], x, y).map(|_| TowerHit::Record)
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<TowerHit> {
        if tower_controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(TowerHit::Close);
        }
        if let Some(hit) = self.button_hit(x, y) {
            return Some(hit);
        }
        if self.words_visible && self.words.rect.contains(x, y) {
            return Some(TowerHit::Words);
        }
        self.list.owns(x, y).then_some(TowerHit::List)
    }

    pub fn mouse_move(&mut self, x: i32, y: i32, font: &BitmapFont) {
        self.list.mouse_move(x, y);
        if self.words.focused {
            let metrics = FontMetrics(font);
            self.words.mouse_move(x, &metrics);
            self.words.layout(&metrics);
        }
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, font: &BitmapFont) -> Option<TowerHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        match hit {
            Some(TowerHit::Words) => {
                let metrics = FontMetrics(font);
                if !self.words.focused {
                    self.words.focus();
                }
                self.words.mouse_down(x, &metrics);
                self.words.layout(&metrics);
            }
            Some(TowerHit::List) => {
                self.blur();
                self.list.mouse_down(x, y);
            }
            Some(TowerHit::Message | TowerHit::Record) => self.blur(),
            None if y >= crate::frame::TITLE_HEIGHT && self.words_visible => {
                if !self.words.focused {
                    self.words.focus();
                }
            }
            _ => self.blur(),
        }
        hit
    }

    pub fn blur(&mut self) {
        if self.words.focused {
            self.words.kill_focus();
        }
    }

    pub fn double_click(&mut self, x: i32, y: i32) -> Option<TowerAction> {
        if self.hit(x, y) != Some(TowerHit::List) || self.list.hit(x, y) < 0 {
            return None;
        }
        self.list.double_click(x, y);
        self.list.notices.clear();
        Some(TowerAction::Diary(self.diary()?))
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<TowerAction> {
        self.words.mouse_up();
        self.list.mouse_up(x, y);
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            TowerHit::Close => Some(TowerAction::Close),
            TowerHit::Message => Some(TowerAction::Words(self.words_text())),
            TowerHit::Record => self.diary().map(TowerAction::Diary),
            TowerHit::Words | TowerHit::List => None,
        }
    }

    pub fn key(&mut self, key: crate::chat::InputKey, font: &BitmapFont) -> bool {
        if !self.words_visible || !self.words.focused {
            return false;
        }
        if self.words.input_key(key, &FontMetrics(font)) == crate::edit_box::EditInput::Escape {
            self.words.kill_focus();
        }
        true
    }

    pub fn shift(&mut self, pressed: bool) -> bool {
        if !self.words_visible || !self.words.focused {
            return false;
        }
        if pressed {
            self.words.key_down_shift();
        } else {
            self.words.key_up(crate::edit_box::KEY_SHIFT);
        }
        true
    }

    pub fn tick(&mut self, elapsed_ms: u32) {
        self.words.tick(elapsed_ms);
    }

    pub fn wheel(&mut self, delta: i16) {
        self.list.wheel(delta);
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32, lookup: &dyn Fn(u32) -> Option<String>) {
        let (x, y) = origin;
        self.frame.paint(list, x, y, TOWER_SIZE.0, TOWER_SIZE.1, font, alpha);
        let start = list.quads.len();
        list.draw_texture(x, y + 20, 600, 306, 0, 0, 600, 306, TOWER_BACK, alpha);
        list.clip_from(start, Rect::new(x, y, TOWER_SIZE.0, TOWER_SIZE.1));
        self.list.paint_header(list, origin, font, &|id| lookup(id as u32));
        self.list.paint(list, origin, font);
        if self.words_visible {
            self.words.paint(list, origin, &FontMetrics(font), font);
            tower_controls::BUTTON_F4.paint(&self.buttons[0], list, origin, self.pressed == Some(TowerHit::Message), font);
        }
        tower_controls::BUTTON_F8.paint(&self.buttons[1], list, origin, self.pressed == Some(TowerHit::Record), font);
    }
}

#[derive(Clone, Debug)]
pub struct HeroContentView {
    pub frame: FrameCtrl,
    pub buttons: [TextButton; 2],
    pressed: Option<usize>,
}

impl HeroContentView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Self {
            frame: content_controls::FRAME_7C.build(text),
            buttons: [content_controls::BUTTON_EC.build((0, 0), text), content_controls::BUTTON_F0.build((0, 0), text)],
            pressed: None,
        }
    }

    fn button_at(&self, x: i32, y: i32) -> Option<usize> {
        if content_controls::BUTTON_EC.hit(&self.buttons[0], x, y).is_some() {
            return Some(0);
        }
        content_controls::BUTTON_F0.hit(&self.buttons[1], x, y).map(|_| 1)
    }

    pub fn close_hit(&self, x: i32, y: i32) -> bool {
        content_controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> bool {
        self.pressed = self.button_at(x, y);
        self.close_hit(x, y)
    }

    pub fn mouse_up(&mut self) {
        self.pressed = None;
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        self.frame.paint(list, x, y, CONTENT_SIZE.0, CONTENT_SIZE.1, font, alpha);
        let start = list.quads.len();
        list.draw_texture(x, y + 20, 310, 381, 0, 0, 310, 381, CONTENT_BACK, alpha);
        list.clip_from(start, Rect::new(x, y, CONTENT_SIZE.0, CONTENT_SIZE.1));
        content_controls::BUTTON_EC.paint(&self.buttons[0], list, origin, self.pressed == Some(0), font);
        content_controls::BUTTON_F0.paint(&self.buttons[1], list, origin, self.pressed == Some(1), font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::InputKey;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("font".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn lookup(id: u32) -> Option<String> {
        Some(format!("s{id}"))
    }

    fn row(name: &str, class_id: i32, clan: Option<&str>, ally: Option<&str>) -> HeroRow {
        HeroRow {
            name: name.into(),
            class_id,
            class_name: format!("class{class_id}"),
            clan: "Clan".into(),
            clan_crest: clan.map(Into::into),
            ally: "Ally".into(),
            ally_crest: ally.map(Into::into),
            count: 3,
        }
    }

    fn view() -> HeroTowerView {
        let mut view = HeroTowerView::new(lookup);
        view.set_heroes(&[row("A", 88, Some("crest.crest_1"), None), row("B", 90, Some("crest.crest_2"), Some("crest.ally_3"))]);
        view
    }

    #[test]
    fn windows_use_the_native_sizes_positions_and_columns() {
        assert_eq!((TOWER_SIZE, CONTENT_SIZE), ((600, 326), (310, 401)));
        assert_eq!((TOWER_ID, CONTENT_ID), (WindowId(76), WindowId(77)));
        assert_eq!(tower_default_position((1558, 933)), (479, 303));
        assert_eq!(content_default_position((1558, 933)), (624, 266));
        let view = view();
        assert_eq!(view.list.columns.iter().map(|c| (c.width, c.sysstring)).collect::<Vec<_>>(), [(169, 393), (120, 391), (128, 604), (119, 605), (50, 1191)]);
        assert_eq!(view.words.rect, Rect::new(25, 300, 405, 16));
        assert_eq!(view.words.max_len, 64);
    }

    #[test]
    fn rows_have_seven_cells_and_the_clan_cell_carries_the_crest_icons() {
        let view = view();
        assert_eq!(view.list.rows[0].cells.len(), CELL_COUNT);
        let texts: Vec<_> = view.list.rows[1].cells.iter().take(5).map(|cell| cell.text.clone()).collect();
        assert_eq!(texts, ["B", "class90", "Clan", "Ally", "3"]);
        let icons = &view.list.rows[1].cells[CELL_CLAN].icons;
        assert_eq!(icons.iter().map(|i| (i.texture.as_str(), i.x, i.w)).collect::<Vec<_>>(), [("crest.ally_3", 10, 8), ("crest.crest_2", 18, 16)]);
        assert_eq!(view.list.rows[0].cells[CELL_CLAN].icons.iter().map(|i| (i.x, i.w, i.v, i.h)).collect::<Vec<_>>(), [(10, 16, 4, 12)]);
        assert!(clan_icons(None, None).is_empty());
        assert_eq!(view.classes, vec![88, 90]);
    }

    #[test]
    fn view_record_and_double_click_ask_the_diary_of_the_selected_class() {
        let mut view = view();
        assert_eq!(view.diary(), None);
        let record = tower_controls::BUTTON_F8.rect;
        view.mouse_down(record.x + 2, record.y + 2, &font());
        assert_eq!(view.mouse_up(record.x + 2, record.y + 2), None);
        view.list.select(1);
        view.mouse_down(record.x + 2, record.y + 2, &font());
        assert_eq!(view.mouse_up(record.x + 2, record.y + 2), Some(TowerAction::Diary("_diary?class=90&page=1".into())));
        let rect = view.list.rect;
        let y = rect.y + view.list.header_height + 2;
        assert_eq!(view.double_click(rect.x + 3, y), Some(TowerAction::Diary("_diary?class=88&page=1".into())));
    }

    #[test]
    fn pressing_the_message_button_takes_the_focus_from_the_field_so_the_whole_text_goes_out_10038750() {
        let font = font();
        let mut view = view();
        view.words.set_text("hello world");
        view.words.focus();
        for _ in 0..6 {
            assert!(view.key(InputKey::Left, &font));
        }
        assert_eq!(crate::edit_box::to_string(view.words.right()), " world");
        let message = tower_controls::BUTTON_F4.rect;
        view.mouse_down(message.x + 2, message.y + 2, &font);
        assert!(!view.words.focused);
        assert_eq!(view.mouse_up(message.x + 2, message.y + 2), Some(TowerAction::Words("hello world".into())));
        assert!(view.words.left().is_empty());
        assert!(view.words.right().is_empty());
    }

    #[test]
    fn the_message_of_an_unfocused_field_with_a_cursor_in_the_middle_is_the_left_half_1013c350() {
        let mut view = view();
        view.words.set_text("hello world");
        view.words.focus();
        let font = font();
        for _ in 0..6 {
            view.key(InputKey::Left, &font);
        }
        assert_eq!(view.words_text(), "hello");
        assert_eq!(crate::edit_box::to_string(view.words.right()), " world");
    }

    #[test]
    fn view_record_also_takes_the_focus_from_the_field() {
        let font = font();
        let mut view = view();
        view.words.focus();
        let record = tower_controls::BUTTON_F8.rect;
        view.mouse_down(record.x + 2, record.y + 2, &font);
        assert!(!view.words.focused);
    }

    #[test]
    fn the_message_field_and_button_follow_the_hero_flag_of_the_own_character() {
        let mut view = view();
        view.set_owner_hero(Some(false));
        let message = tower_controls::BUTTON_F4.rect;
        assert_eq!(view.hit(message.x + 2, message.y + 2), None);
        assert!(!view.key(InputKey::Char('a'), &font()));
        view.set_owner_hero(None);
        assert!(!view.words_visible);
        view.set_owner_hero(Some(true));
        assert_eq!(view.hit(message.x + 2, message.y + 2), Some(TowerHit::Message));
    }

    #[test]
    fn content_window_has_two_inert_buttons_and_closes_from_the_frame() {
        let mut content = HeroContentView::new(lookup);
        let close = Rect::new(287, 3, 15, 15);
        assert!(content.mouse_down(close.x + 2, close.y + 2));
        let first = content_controls::BUTTON_EC.rect;
        assert!(!content.mouse_down(first.x + 2, first.y + 2));
        content.mouse_up();
        assert_eq!(content.buttons[0].label, "s1");
    }

    #[test]
    fn paints_draw_the_native_backgrounds() {
        let font = font();
        let mut list = DrawList::default();
        view().paint(&mut list, (10, 20), &font, 1.0, &lookup);
        assert!(list.quads.iter().any(|quad| quad.texture == TOWER_BACK));
        let mut list = DrawList::default();
        HeroContentView::new(lookup).paint(&mut list, (10, 20), &font, 1.0);
        assert!(list.quads.iter().any(|quad| quad.texture == CONTENT_BACK));
    }

    #[test]
    fn the_filter_drops_characters_without_a_glyph_and_keeps_line_breaks() {
        let mut view = view();
        view.set_font(&font());
        view.words.set_text("a\u{7f}b\u{81}c\u{4e2d}");
        assert_eq!(view.words_text(), "abc");
    }

    #[test]
    fn a_click_on_the_window_background_focuses_the_visible_edit_but_the_frame_and_buttons_take_the_focus() {
        let mut view = HeroTowerView::new(|id| Some(format!("s{id}")));
        let font = font();
        assert!(view.hit(2, 300).is_none());
        view.mouse_down(2, 300, &font);
        assert!(view.words.focused);
        view.mouse_down(100, 5, &font);
        assert!(!view.words.focused);
        view.mouse_down(2, 300, &font);
        view.words_visible = false;
        view.words.kill_focus();
        view.mouse_down(2, 300, &font);
        assert!(!view.words.focused);
    }

    #[test]
    fn the_hero_list_has_the_ncscrollbar_of_the_list_ctrl_and_its_down_button_rolls_one_row() {
        let font = font();
        let mut view = HeroTowerView::new(lookup);
        let rows: Vec<HeroRow> = (0..20).map(|n| row(&format!("H{n}"), 88, None, None)).collect();
        view.set_heroes(&rows);
        let rect = view.list.rect;
        let down = (rect.x + rect.w - 8, rect.y + rect.h - 8);
        assert_eq!(view.hit(down.0, down.1), Some(TowerHit::List));
        view.mouse_down(down.0, down.1, &font);
        assert_eq!(view.mouse_up(down.0, down.1), None);
        assert_eq!(view.list.first_row(), 1);
        assert_eq!(view.list.selected(), None);
        let mut list = DrawList::default();
        view.paint(&mut list, (0, 0), &font, 1.0, &lookup);
        assert!(list.quads.iter().any(|q| q.texture == crate::scroll::SLIDER_TOP));
    }

    #[test]
    fn both_windows_paint_the_back_piece_below_the_title_bar_at_full_alpha() {
        let font = font();
        let find = |list: &DrawList, name: &str| list.quads.iter().find(|q| q.texture == name).map(|q| (q.x, q.y, q.w, q.h, q.uw, q.vh, q.color[3]));
        let tower = HeroTowerView::new(lookup);
        let mut list = DrawList::default();
        tower.paint(&mut list, (0, 0), &font, 1.0, &lookup);
        let (w, h) = (TOWER_SIZE.0 as f32, TOWER_SIZE.1 as f32 - 20.0);
        assert_eq!(find(&list, TOWER_BACK), Some((0.0, 20.0, w, h, w, h, 1.0)));
        let content = HeroContentView::new(lookup);
        let mut list = DrawList::default();
        content.paint(&mut list, (0, 0), &font, 1.0);
        let (w, h) = (CONTENT_SIZE.0 as f32, CONTENT_SIZE.1 as f32 - 20.0);
        assert_eq!(find(&list, CONTENT_BACK), Some((0.0, 20.0, w, h, w, h, 1.0)));
    }
}
