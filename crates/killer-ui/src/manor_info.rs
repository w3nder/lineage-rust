#[path = "manor_crop_controls_generated.rs"]
mod crop_controls;
#[path = "manor_default_controls_generated.rs"]
mod default_controls;
#[path = "manor_info_controls_generated.rs"]
mod frame_controls;
#[path = "manor_info_layout_generated.rs"]
mod layout;
#[path = "manor_seed_controls_generated.rs"]
mod seed_controls;

use crate::button::TextButton;
use crate::color::Argb;
use crate::combo::ComboBox;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::list_ctrl::{ListCell, ListCtrl, ListRow};
use crate::tab_control::{TabEvent, TabEvents};
use crate::tab_strip::{TabSpec, TabStrip};
use crate::window::{Rect, WindowId};

pub const WINDOW_ID: WindowId = WindowId::MANOR_INFO;
pub const SIZE: (i32, i32) = layout::SIZE;
pub const INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 2, x: SIZE.0, y: SIZE.1 };

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 * 0.5 - 300.0) as i32, (viewport.1 as f32 * 0.5 - 163.0) as i32)
}

pub fn grip() -> Rect {
    frame_controls::FRAME_7C.rect_at((0, 0))
}

pub const UP_BACK: &str = "L2UI_CH3.Manor.manor1_back1";
pub const PAGE_BACK: &str = "L2UI_CH3.Manor.manor1_back2";
pub const TAB_NORMAL: &str = "L2UI_CH3.Manor.manor1_tab2";
pub const TAB_PRESSED: &str = "L2UI_CH3.Manor.manor1_tab1";
pub const TAB_SIZE: (i32, i32) = (0x5e, 0x17);
pub const TAB_SPACING: i32 = 1;
pub const TAB_ID: u32 = 0xff00_0001;
pub const TAB_LABELS: [u32; 3] = [0x459, 0x45a, 0x45b];
pub const PAGE_TITLES: [u32; 3] = [0x459, 0x45a, 0x46b];
pub const TIME_LABELS: [u32; 2] = [0x45c, 0x45d];
pub const TITLE_POS: (i32, i32) = (16, 15);
pub const BOTTOM_POS: (i32, i32) = (16, 0xfa);
pub const BOTTOM_MESSAGE: u32 = 0x645;
pub const BOTTOM_ARGUMENTS: [&str; 2] = ["$s1", "$c1"];
pub const TEXT_COLOR: Argb = Argb(0xffdcdcdc);
pub const CASTLE_NAMES: [(i32, u32); 7] = [(1, 1049), (2, 1052), (3, 1053), (4, 1057), (5, 1060), (6, 1059), (7, 1248)];

pub const ASK_MENU_1: &str = "manor_menu_select?ask=1&state=-1&time=0";
pub const ASK_MENU_2: &str = "manor_menu_select?ask=2&state=-1&time=0";
pub const ASK_TAB_DEFAULT: &str = "manor_menu_select?ask=5&state=-1&time=0";

const UP_RECT: Rect = layout::RECT_E8;
const BUTTON_MENU_1_DEFAULT: u32 = default_controls::BUTTON_F0.id;
const BUTTON_MENU_1_SEED: u32 = seed_controls::BUTTON_F4.id;
const TABS_POS: (i32, i32) = (12, UP_RECT.y + 5);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ManorPage {
    Seed,
    Crop,
    Default,
}

impl ManorPage {
    pub const fn index(self) -> usize {
        match self {
            ManorPage::Seed => 0,
            ManorPage::Crop => 1,
            ManorPage::Default => 2,
        }
    }

    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(ManorPage::Seed),
            1 => Some(ManorPage::Crop),
            2 => Some(ManorPage::Default),
            _ => None,
        }
    }
}

pub fn castle_name(id: i32, lookup: &dyn Fn(u32) -> Option<String>) -> Option<String> {
    CASTLE_NAMES.iter().find(|(castle, _)| *castle == id).and_then(|(_, sysstring)| lookup(*sysstring)).filter(|name| !name.is_empty())
}

pub fn join_names(ids: &[i32], name: &dyn Fn(i32) -> String) -> String {
    ids.iter().map(|id| name(*id)).collect::<Vec<_>>().join(",")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManorHit {
    Close,
    Tab(usize),
    Button(ManorPage, u32),
    List(ManorPage),
    Combo(ManorPage, usize),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManorRow {
    pub item: i32,
    pub numbers: Vec<i32>,
    pub first: Vec<i32>,
    pub second: Vec<i32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ManorTable {
    pub flag: bool,
    pub state: i32,
    pub rows: Vec<ManorRow>,
}

#[derive(Debug, Clone)]
pub struct DefaultPage {
    pub list: ListCtrl,
    pub flag: bool,
    pub buttons: [TextButton; 2],
}

#[derive(Debug, Clone)]
pub struct StatePage {
    pub list: ListCtrl,
    pub flag: bool,
    pub state: i32,
    pub committed: i32,
    pub castle: ComboBox,
    pub castle_ids: Vec<i32>,
    pub time: ComboBox,
    pub button: TextButton,
}

impl StatePage {
    fn state_id(&self) -> i32 {
        usize::try_from(self.castle.selected).ok().and_then(|index| self.castle_ids.get(index).copied()).unwrap_or(-1)
    }

    fn time_index(&self) -> i32 {
        self.time.selected
    }

    fn request(&self, ask: u32) -> String {
        format!("manor_menu_select?ask={ask}&state={}&time={}", self.state_id(), self.time_index())
    }
}

#[derive(Debug, Clone)]
pub struct ManorInfoView {
    pub frame: FrameCtrl,
    pub tabs: TabStrip,
    pub seed: StatePage,
    pub crop: StatePage,
    pub default: DefaultPage,
    pub castles: Vec<i32>,
    pub requests: Vec<String>,
    pub bottom_template: String,
    pressed: Option<ManorHit>,
}

fn state_page(
    list: crate::list_ctrl::ListContract,
    castle: &crate::child_contract::ChildContract,
    time: &crate::child_contract::ChildContract,
    button: TextButton,
    text: &dyn Fn(u32) -> String,
) -> StatePage {
    let mut time = ComboBox::from_contract(time, TIME_LABELS.iter().map(|id| text(*id)).collect(), 13);
    time.selected = 0;
    StatePage {
        list: ListCtrl::from_contract(&list),
        flag: false,
        state: -1,
        committed: -1,
        castle: ComboBox::from_contract(castle, Vec::new(), 13),
        castle_ids: Vec::new(),
        time,
        button,
    }
}

impl ManorInfoView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        let mut tabs = TabStrip::new(TAB_ID, 0);
        for (index, label) in TAB_LABELS.into_iter().enumerate() {
            tabs.add_fixed(TabSpec {
                label: Some(text(label)),
                page: Some(index),
                size: TAB_SIZE,
                textures: [Some(TAB_NORMAL.into()), Some(TAB_PRESSED.into())],
                spacing: TAB_SPACING,
                tooltip: -1,
                special_attachment: false,
            });
        }
        for button in &mut tabs.buttons {
            button.visible = true;
        }
        let seed_button = seed_controls::BUTTON_F4.build((0, 0), text);
        let crop_button = crop_controls::BUTTON_F4.build((0, 0), text);
        Self {
            frame: frame_controls::FRAME_7C.build(text),
            tabs,
            seed: state_page(seed_controls::LIST_E8, &seed_controls::CHILD_F0, &seed_controls::CHILD_F8, seed_button, &text),
            crop: state_page(crop_controls::LIST_E8, &crop_controls::CHILD_F0, &crop_controls::CHILD_F8, crop_button, &text),
            default: DefaultPage {
                list: ListCtrl::from_contract(&default_controls::LIST_E8),
                flag: false,
                buttons: [default_controls::BUTTON_F0.build((0, 0), text), default_controls::BUTTON_F4.build((0, 0), text)],
            },
            castles: Vec::new(),
            requests: Vec::new(),
            bottom_template: String::new(),
            pressed: None,
        }
    }

    pub fn set_castles(&mut self, ids: Vec<i32>) {
        self.castles = ids;
    }

    pub fn page(&self) -> Option<ManorPage> {
        self.tabs.control.current.and_then(ManorPage::from_index)
    }

    fn select_page(&mut self, page: ManorPage) {
        let mut events = TabEvents::default();
        let _ = self.tabs.control.select(page.index(), false, &mut events);
    }

    fn refill_castles(page: &mut StatePage, castles: &[i32], lookup: &dyn Fn(u32) -> Option<String>) {
        page.castle.clear_entries();
        page.castle_ids.clear();
        for id in castles {
            if let Some(name) = castle_name(*id, lookup) {
                page.castle.entries.push(name);
                page.castle_ids.push(*id);
            }
        }
        if let Some(index) = page.castle_ids.iter().position(|id| *id == page.state) {
            page.castle.selected = index as i32;
        }
    }

    fn fill_rows(list: &mut ListCtrl, table: &ManorTable, item_name: &dyn Fn(i32) -> String) {
        list.clear();
        for row in &table.rows {
            let mut cells = vec![ListCell::text(&item_name(row.item))];
            cells.extend(row.numbers.iter().map(|value| ListCell::text(&value.to_string())));
            cells.push(ListCell::text(&join_names(&row.first, item_name)));
            cells.push(ListCell::text(&join_names(&row.second, item_name)));
            list.add_item(ListRow { cells });
        }
    }

    pub fn show_seed(&mut self, table: &ManorTable, item_name: &dyn Fn(i32) -> String, lookup: &dyn Fn(u32) -> Option<String>, was_visible: bool) {
        Self::fill_rows(&mut self.seed.list, table, item_name);
        self.seed.flag = table.flag;
        self.seed.state = table.state;
        Self::refill_castles(&mut self.seed, &self.castles, lookup);
        if !was_visible {
            self.seed.committed = self.seed.state;
        }
        self.select_page(ManorPage::Seed);
    }

    pub fn show_crop(&mut self, table: &ManorTable, item_name: &dyn Fn(i32) -> String, lookup: &dyn Fn(u32) -> Option<String>) {
        Self::fill_rows(&mut self.crop.list, table, item_name);
        self.crop.flag = table.flag;
        self.crop.state = table.state;
        Self::refill_castles(&mut self.crop, &self.castles, lookup);
        self.select_page(ManorPage::Crop);
    }

    pub fn show_default(&mut self, table: &ManorTable, item_name: &dyn Fn(i32) -> String) {
        Self::fill_rows(&mut self.default.list, table, item_name);
        self.default.flag = table.flag;
        self.select_page(ManorPage::Default);
    }

    fn pressed_button(&self, id: u32) -> bool {
        matches!(self.pressed, Some(ManorHit::Button(_, pressed)) if pressed == id)
    }

    fn tab_rect(&self, index: usize) -> Option<Rect> {
        self.tabs.buttons.get(index).filter(|button| button.visible).map(|button| button.paint.rect.offset(TABS_POS.0, TABS_POS.1))
    }

    fn page_rect(page: ManorPage) -> Rect {
        match page {
            ManorPage::Seed => layout::RECT_EC,
            ManorPage::Crop => layout::RECT_F0,
            ManorPage::Default => layout::RECT_F4,
        }
    }

    fn page_point(page: ManorPage, x: i32, y: i32) -> (i32, i32) {
        let rect = Self::page_rect(page);
        (x - rect.x, y - rect.y)
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<ManorHit> {
        if frame_controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(ManorHit::Close);
        }
        if let Some(index) = (0..self.tabs.buttons.len()).find(|index| self.tab_rect(*index).is_some_and(|rect| rect.contains(x, y))) {
            return Some(ManorHit::Tab(index));
        }
        let page = self.page()?;
        let (px, py) = Self::page_point(page, x, y);
        match page {
            ManorPage::Default => {
                if !self.default.flag {
                    let first = default_controls::BUTTON_F0.hit(&self.default.buttons[0], px, py);
                    if let Some(id) = first.or_else(|| default_controls::BUTTON_F4.hit(&self.default.buttons[1], px, py)) {
                        return Some(ManorHit::Button(ManorPage::Default, id));
                    }
                }
                self.default.list.owns(px, py).then_some(ManorHit::List(ManorPage::Default))
            }
            page => {
                let state = if page == ManorPage::Seed { &self.seed } else { &self.crop };
                for (index, combo) in [&state.castle, &state.time].into_iter().enumerate() {
                    if combo.expanded_rect().contains(px, py) {
                        return Some(ManorHit::Combo(page, index));
                    }
                }
                if !state.flag {
                    let id = if page == ManorPage::Seed {
                        seed_controls::BUTTON_F4.hit(&state.button, px, py)
                    } else {
                        crop_controls::BUTTON_F4.hit(&state.button, px, py)
                    };
                    if let Some(id) = id {
                        return Some(ManorHit::Button(page, id));
                    }
                }
                state.list.owns(px, py).then_some(ManorHit::List(page))
            }
        }
    }

    fn state_page_mut(&mut self, page: ManorPage) -> &mut StatePage {
        if page == ManorPage::Seed { &mut self.seed } else { &mut self.crop }
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        let Some(page) = self.page() else { return };
        let (px, py) = Self::page_point(page, x, y);
        match page {
            ManorPage::Default => {
                self.default.list.mouse_move(px, py);
            }
            page => {
                let state = self.state_page_mut(page);
                state.castle.mouse_move(px, py);
                state.time.mouse_move(px, py);
                state.list.mouse_move(px, py);
            }
        }
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<ManorHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        match hit {
            Some(ManorHit::Tab(index)) => {
                let mut events = TabEvents::default();
                self.tabs.control.press(index, true, &mut events);
                for event in events.0 {
                    if let TabEvent::NotifyParent(tab) = event {
                        self.tab_request(tab);
                    }
                }
            }
            Some(ManorHit::Combo(page, which)) => {
                let (px, py) = Self::page_point(page, x, y);
                let state = self.state_page_mut(page);
                let combo = if which == 0 { &mut state.castle } else { &mut state.time };
                combo.mouse_move(px, py);
                if combo.mouse_down().is_some() {
                    let ask = if page == ManorPage::Seed { 3 } else { 4 };
                    let request = self.state_page_mut(page).request(ask);
                    self.requests.push(request);
                }
            }
            Some(ManorHit::List(page)) => {
                let (px, py) = Self::page_point(page, x, y);
                match page {
                    ManorPage::Default => {
                        self.default.list.mouse_down(px, py);
                    }
                    _ => {
                        self.state_page_mut(page).list.mouse_down(px, py);
                    }
                }
            }
            _ => {}
        }
        hit
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<ManorHit> {
        if let Some(page) = self.page() {
            let (px, py) = Self::page_point(page, x, y);
            match page {
                ManorPage::Default => self.default.list.mouse_up(px, py),
                page => self.state_page_mut(page).list.mouse_up(px, py),
            };
        }
        let pressed = self.pressed.take()?;
        let released = self.hit(x, y);
        if released != Some(pressed) {
            return None;
        }
        if let ManorHit::Button(_, id) = pressed {
            let command = match id {
                BUTTON_MENU_1_DEFAULT | BUTTON_MENU_1_SEED => ASK_MENU_1,
                _ => ASK_MENU_2,
            };
            self.requests.push(command.to_string());
        }
        Some(pressed)
    }

    fn tab_request(&mut self, tab: usize) {
        let request = match ManorPage::from_index(tab) {
            Some(ManorPage::Seed) => self.seed.request(3),
            Some(ManorPage::Crop) => self.crop.request(4),
            Some(ManorPage::Default) => ASK_TAB_DEFAULT.to_string(),
            None => return,
        };
        self.requests.push(request);
    }

    pub fn wheel(&mut self, delta: i16) {
        match self.page() {
            Some(ManorPage::Default) => {
                self.default.list.wheel(delta);
            }
            Some(page @ (ManorPage::Seed | ManorPage::Crop)) => {
                self.state_page_mut(page).list.wheel(delta);
            }
            None => {}
        }
    }

    pub fn paint(
        &self,
        list: &mut DrawList,
        origin: (i32, i32),
        font: &BitmapFont,
        alpha: f32,
        lookup: &dyn Fn(u32) -> Option<String>,
    ) {
        let (x, y) = origin;
        self.frame.paint(list, x, y, SIZE.0, SIZE.1, font, alpha);
        let start = list.quads.len();
        list.draw_texture(x + UP_RECT.x, y + UP_RECT.y, UP_RECT.w, UP_RECT.h, 0, 0, UP_RECT.w, UP_RECT.h, UP_BACK, alpha);
        list.clip_from(start, Rect::new(x + UP_RECT.x, y + UP_RECT.y, UP_RECT.w, UP_RECT.h));
        self.tabs.paint(list, (x + TABS_POS.0, y + TABS_POS.1), font);
        let Some(page) = self.page() else { return };
        let rect = Self::page_rect(page);
        let (px, py) = (x + rect.x, y + rect.y);
        let start = list.quads.len();
        list.draw_texture(px, py, rect.w, rect.h, 0, 0, 600, 278, PAGE_BACK, alpha);
        let title = lookup(PAGE_TITLES[page.index()]).unwrap_or_default();
        list.draw_text(px + TITLE_POS.0, py + TITLE_POS.1, TEXT_COLOR, &title, font);
        list.clip_from(start, Rect::new(px, py, rect.w, rect.h));
        match page {
            ManorPage::Default => {
                self.default.list.paint_header(list, (px, py), font, &|id| lookup(id as u32));
                self.default.list.paint(list, (px, py), font);
                if !self.default.flag {
                    default_controls::BUTTON_F0.paint(&self.default.buttons[0], list, (px, py), self.pressed_button(default_controls::BUTTON_F0.id), font);
                    default_controls::BUTTON_F4.paint(&self.default.buttons[1], list, (px, py), self.pressed_button(default_controls::BUTTON_F4.id), font);
                }
            }
            page => {
                let state = if page == ManorPage::Seed { &self.seed } else { &self.crop };
                state.list.paint_header(list, (px, py), font, &|id| lookup(id as u32));
                state.list.paint(list, (px, py), font);
                if !state.flag {
                    if page == ManorPage::Seed {
                        seed_controls::BUTTON_F4.paint(&state.button, list, (px, py), self.pressed_button(seed_controls::BUTTON_F4.id), font);
                    } else {
                        crop_controls::BUTTON_F4.paint(&state.button, list, (px, py), self.pressed_button(crop_controls::BUTTON_F4.id), font);
                    }
                }
                if page == ManorPage::Seed
                    && !state.flag
                    && let Some(name) = castle_name(state.committed, lookup)
                {
                    let text = BOTTOM_ARGUMENTS.iter().fold(self.bottom_template.clone(), |text, token| text.replace(token, &name));
                    let start = list.quads.len();
                    list.draw_text(px + BOTTOM_POS.0, py + BOTTOM_POS.1, TEXT_COLOR, &text, font);
                    list.clip_from(start, Rect::new(px, py, rect.w, rect.h));
                }
                state.time.paint(list, (px, py), font, true, false);
                state.castle.paint(list, (px, py), font, true, false);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("font".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn lookup(id: u32) -> Option<String> {
        Some(format!("s{id}"))
    }

    fn item(id: i32) -> String {
        format!("item{id}")
    }

    fn table(flag: bool, state: i32, rows: usize) -> ManorTable {
        ManorTable {
            flag,
            state,
            rows: (0..rows as i32)
                .map(|index| ManorRow { item: 100 + index, numbers: vec![1, 2, 3], first: vec![5, 6], second: vec![7] })
                .collect(),
        }
    }

    fn view() -> ManorInfoView {
        let mut view = ManorInfoView::new(lookup);
        view.set_castles(vec![1, 2, 9, 7]);
        view
    }

    fn page_point(view: &ManorInfoView, rect: Rect) -> (i32, i32) {
        let page = ManorInfoView::page_rect(view.page().unwrap());
        (page.x + rect.x + 1, page.y + rect.y + 1)
    }

    #[test]
    fn castle_names_come_from_the_native_switch_and_unknown_ids_have_none() {
        assert_eq!(castle_name(1, &lookup).as_deref(), Some("s1049"));
        assert_eq!(castle_name(7, &lookup).as_deref(), Some("s1248"));
        assert_eq!(castle_name(0, &lookup), None);
        assert_eq!(castle_name(8, &lookup), None);
        assert_eq!(castle_name(2, &|_| None), None);
    }

    #[test]
    fn view_starts_on_the_first_tab_page_and_the_tabs_are_the_three_labels() {
        let view = view();
        assert_eq!(view.page(), Some(ManorPage::Seed));
        assert_eq!(view.tabs.buttons.iter().map(|b| b.paint.label.clone()).collect::<Vec<_>>(), vec!["s1113", "s1114", "s1115"]);
        assert_eq!(SIZE, (600, 326));
        assert_eq!(view.seed.time.entries, vec!["s1116", "s1117"]);
        assert_eq!(view.seed.time.selected, 0);
        assert_eq!((view.seed.state, view.seed.committed), (-1, -1));
    }

    #[test]
    fn default_info_rows_use_item_names_numbers_and_comma_joined_lists() {
        let mut view = view();
        view.show_default(&table(false, 0, 2), &item);
        assert_eq!(view.page(), Some(ManorPage::Default));
        assert_eq!(view.default.list.count(), 2);
        let row = &view.default.list.rows[0];
        let texts: Vec<_> = row.cells.iter().map(|cell| cell.text.as_str()).collect();
        assert_eq!(texts, vec!["item100", "1", "2", "3", "item5,item6", "item7"]);
        view.show_default(&table(true, 0, 1), &item);
        assert_eq!(view.default.list.count(), 1);
        assert!(view.default.flag);
    }

    #[test]
    fn state_pages_fill_the_castle_combo_with_known_castles_and_select_the_state() {
        let mut view = view();
        view.show_seed(&table(false, 7, 1), &item, &lookup, false);
        assert_eq!(view.page(), Some(ManorPage::Seed));
        assert_eq!(view.seed.castle_ids, vec![1, 2, 7]);
        assert_eq!(view.seed.castle.entries, vec!["s1049", "s1052", "s1248"]);
        assert_eq!(view.seed.castle.selected, 2);
        assert_eq!((view.seed.state, view.seed.committed), (7, 7));
        view.show_seed(&table(false, 2, 1), &item, &lookup, true);
        assert_eq!((view.seed.state, view.seed.committed), (2, 7));
        assert_eq!(view.seed.castle.selected, 1);
        view.show_crop(&table(false, 99, 1), &item, &lookup);
        assert_eq!(view.page(), Some(ManorPage::Crop));
        assert_eq!(view.crop.castle.selected, -1);
        assert_eq!(view.crop.list.rows[0].cells.len(), 6);
    }

    #[test]
    fn tabs_press_the_page_and_request_it_with_the_current_combo_selection() {
        let mut view = view();
        view.show_seed(&table(false, 2, 0), &item, &lookup, false);
        view.show_default(&table(false, 0, 0), &item);
        let tab = |view: &ManorInfoView, index: usize| {
            let rect = view.tab_rect(index).unwrap();
            (rect.x + 2, rect.y + 2)
        };
        let (x, y) = tab(&view, 0);
        assert_eq!(view.mouse_down(x, y), Some(ManorHit::Tab(0)));
        assert_eq!(view.page(), Some(ManorPage::Seed));
        assert_eq!(view.requests, vec!["manor_menu_select?ask=3&state=2&time=0"]);
        let (x, y) = tab(&view, 1);
        view.mouse_down(x, y);
        assert_eq!(view.page(), Some(ManorPage::Crop));
        assert_eq!(view.requests[1], "manor_menu_select?ask=4&state=-1&time=0");
        let (x, y) = tab(&view, 2);
        view.mouse_down(x, y);
        assert_eq!(view.requests[2], ASK_TAB_DEFAULT);
        assert_eq!(view.page(), Some(ManorPage::Default));
    }

    #[test]
    fn combo_selection_sends_the_state_page_request_after_the_second_click() {
        let mut view = view();
        view.show_seed(&table(false, 1, 0), &item, &lookup, false);
        let castle = view.seed.castle.rect;
        let (x, y) = page_point(&view, castle);
        view.mouse_down(x, y);
        assert!(view.seed.castle.open);
        assert!(view.requests.is_empty());
        let row = y + castle.h + 17;
        view.mouse_move(x, row);
        view.mouse_down(x, row);
        assert_eq!(view.requests, vec!["manor_menu_select?ask=3&state=2&time=0"]);
        view.show_crop(&table(false, 1, 0), &item, &lookup);
        let time = view.crop.time.rect;
        let (x, y) = page_point(&view, time);
        view.mouse_down(x, y);
        let row = y + time.h + 17;
        view.mouse_move(x, row);
        view.mouse_down(x, row);
        assert_eq!(view.requests[1], "manor_menu_select?ask=4&state=1&time=1");
    }

    #[test]
    fn page_buttons_send_their_menu_request_on_release_and_hide_with_the_flag() {
        let mut view = view();
        view.show_default(&table(false, 0, 0), &item);
        let first = default_controls::BUTTON_F0.rect;
        let (x, y) = page_point(&view, first);
        assert_eq!(view.mouse_down(x, y), Some(ManorHit::Button(ManorPage::Default, 0xfaddcca0)));
        assert!(view.requests.is_empty());
        view.mouse_up(x, y);
        assert_eq!(view.requests, vec![ASK_MENU_1]);
        let second = default_controls::BUTTON_F4.rect;
        let (x, y) = page_point(&view, second);
        view.mouse_down(x, y);
        view.mouse_up(x + 200, y);
        assert_eq!(view.requests.len(), 1);
        view.mouse_down(x, y);
        view.mouse_up(x, y);
        assert_eq!(view.requests[1], ASK_MENU_2);
        view.show_default(&table(true, 0, 0), &item);
        assert_eq!(view.hit(x + ManorInfoView::page_rect(ManorPage::Default).x, y + ManorInfoView::page_rect(ManorPage::Default).y), None);
        view.show_seed(&table(false, 1, 0), &item, &lookup, false);
        let button = seed_controls::BUTTON_F4.rect;
        let (x, y) = page_point(&view, button);
        view.mouse_down(x, y);
        view.mouse_up(x, y);
        assert_eq!(view.requests[2], ASK_MENU_1);
        view.show_crop(&table(false, 1, 0), &item, &lookup);
        let (x, y) = page_point(&view, crop_controls::BUTTON_F4.rect);
        view.mouse_down(x, y);
        view.mouse_up(x, y);
        assert_eq!(view.requests[3], ASK_MENU_2);
    }

    #[test]
    fn list_rows_are_selected_by_click_and_scrolled_by_the_wheel() {
        let mut view = view();
        view.show_default(&table(false, 0, 30), &item);
        let page = ManorInfoView::page_rect(ManorPage::Default);
        let list = default_controls::CHILD_E8.rect;
        let (x, y) = (page.x + list.x + 5, page.y + list.y + 19 + 18 * 2 + 1);
        assert_eq!(view.mouse_down(x, y), Some(ManorHit::List(ManorPage::Default)));
        assert_eq!(view.default.list.selected(), Some(2));
        view.wheel(-120 * 3);
        assert_eq!(view.default.list.first_row(), 3);
        view.mouse_move(x, y);
        assert_eq!(view.default.list.hovered(), 5);
    }

    #[test]
    fn list_ctrl_scroll_buttons_and_header_sort_work_through_the_view() {
        let mut view = view();
        view.show_seed(&table(false, 2, 30), &item, &lookup, false);
        let page = ManorInfoView::page_rect(ManorPage::Seed);
        let rects = view.seed.list.scroll_rects();
        let origin = (page.x + view.seed.list.rect.x, page.y + view.seed.list.rect.y);
        let down = (origin.0 + rects.down.x + 3, origin.1 + rects.down.y + 3);
        assert_eq!(view.mouse_down(down.0, down.1), Some(ManorHit::List(ManorPage::Seed)));
        assert_eq!(view.seed.list.first_row(), 1);
        assert_eq!(view.seed.list.pressed, Some(crate::list_ctrl::Pressed::Down));
        view.mouse_up(down.0, down.1);
        assert_eq!(view.seed.list.pressed, None);
        let header = (origin.0 + 3, origin.1 + 3);
        view.mouse_move(header.0, header.1);
        assert_eq!(view.seed.list.headers[0].state, 2);
        let before = view.seed.list.rows[0].clone();
        assert_eq!(view.mouse_down(header.0, header.1), Some(ManorHit::List(ManorPage::Seed)));
        assert_eq!(view.seed.list.headers[0].state, 1);
        view.mouse_up(header.0, header.1);
        assert_eq!(view.seed.list.rows[0], before);
        view.mouse_down(header.0, header.1);
        view.mouse_up(header.0, header.1);
        assert_ne!(view.seed.list.rows[0], before);
    }

    #[test]
    fn close_button_of_the_frame_is_hit_at_the_top_right() {
        let view = view();
        let rect = SIZE.0 - 23 + 2;
        assert_eq!(view.hit(rect, 5), Some(ManorHit::Close));
        assert_eq!(view.hit(5, 5), None);
    }

    #[test]
    fn paint_draws_the_title_columns_and_the_seed_bottom_message_only_without_the_flag() {
        let mut view = view();
        view.show_seed(&table(false, 2, 1), &item, &lookup, false);
        view.bottom_template = "Seeds of $s1".into();
        let mut list = DrawList::default();
        view.paint(&mut list, (10, 20), &font(), 1.0, &lookup);
        assert!(list.quads.iter().any(|q| q.texture == PAGE_BACK && (q.x, q.y) == (10.0, 20.0 + 48.0)));
        assert!(list.quads.iter().any(|q| q.texture == UP_BACK));
        let glyphs = |list: &DrawList| list.quads.iter().filter(|q| q.texture == "font").count();
        let with_flag_off = glyphs(&list);
        view.show_seed(&table(true, 2, 1), &item, &lookup, true);
        let mut hidden = DrawList::default();
        view.paint(&mut hidden, (10, 20), &font(), 1.0, &lookup);
        assert!(glyphs(&hidden) < with_flag_off);
        let mut empty = DrawList::default();
        ManorInfoView::new(lookup).paint(&mut empty, (0, 0), &font(), 1.0, &lookup);
        assert!(empty.quads.iter().any(|q| q.texture == PAGE_BACK));
    }
}
