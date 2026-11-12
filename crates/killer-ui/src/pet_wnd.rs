#[path = "summon_layout_generated.rs"]
mod summon_layout;

#[path = "pet_layout_generated.rs"]
mod layout;

use crate::button::TextButton;
use crate::color::Argb;
use crate::crt_float::div_f32;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, TITLE_HEIGHT};
use crate::item_grid::{GridEntry, GridHit, ItemGrid};
use crate::tab_control::TabEvents;
use crate::tab_strip::{TabSpec, TabStrip};
use crate::window::Rect;

pub const SIZE: (i32, i32) = layout::SIZE;
pub const DEFAULT_POSITION: (i32, i32) = (0xac, 0x4c);
pub const STYLE: u32 = 0x3002;
pub const TITLE_SYSSTRING: u32 = 0x1ee;
pub const TAB_SYSSTRINGS: [u32; 3] = [0x1eb, 0x1ec, 0x1ed];
pub const NAME_SYSSTRING: u32 = 0x1f1;
pub const LEVEL_SYSSTRING: u32 = 88;
pub const BACK1: &str = "L2UI_CH3.PetWnd.Petinterface_back1";
pub const BACK2: &str = "L2UI_CH3.PetWnd.Petinterface_back2";
pub const PAGE_BACKS: [&str; 3] =
    ["L2UI_ch3.PetWnd.petinterface_back31", "L2UI_ch3.PetWnd.petinterface_back32", "L2UI_CH3.PetWnd.Petinterface_back33"];
pub const TAB_NORMAL: &str = "L2UI_Ch3.PetWnd.petinterface_tab2";
pub const TAB_PRESSED: &str = "L2UI_Ch3.PetWnd.petinterface_tab1";
pub const HP_BAR: &str = "L2UI_CH3.PlayerStatusWnd.ps_HPbar";
pub const MP_BAR: &str = "L2UI_CH3.PlayerStatusWnd.ps_MPbar";
pub const EXP_BAR: &str = "L2UI_CH3.PlayerStatusWnd.ps_ExPbar";
pub const WEIGHT_BARS: [&str; 4] = [
    "L2UI_CH3.PlayerStatusWnd.ps_weightbar1",
    "L2UI_CH3.PlayerStatusWnd.ps_weightbar2",
    "L2UI_CH3.PlayerStatusWnd.ps_weightbar3",
    "L2UI_CH3.PlayerStatusWnd.ps_weightbar4",
];
pub const FOOD_BAR: &str = "L2UI_CH3.PlayerStatusWnd.ps_foodbar";
pub const BUTTON_NORMAL: &str = "L2UI_Ch3.BUTTON.btn1_normal";
pub const BUTTON_DOWN: &str = "L2UI_Ch3.BUTTON.btn1_normalon";
pub const NAME_BUTTON: Rect = layout::RECT_1D4;
pub const TABS_POS: (i32, i32) = (layout::RECT_1D0.x, layout::RECT_1D0.y);
pub const TAB_SIZE: (i32, i32) = (0x4a, 0x17);
pub const PAGE_RECT: Rect = layout::RECT_1C4;
pub const SHORT_BAR: i32 = 0x55;
pub const LONG_BAR: i32 = 0xce;
pub const LABEL: Argb = Argb(0xffa3a3a3);
pub const VALUE: Argb = Argb(0xffb09b79);
pub const NUMBER: Argb = Argb(0xffdcdcdc);
pub const PERCENT: Argb = Argb(0xffb4b4b4);
pub const DETAIL_LABELS: [[u32; 6]; 2] = [[94, 95, 96, 113, 111, 404], [98, 99, 97, 432, 112, 496]];
pub const DETAIL_LABEL_X: [i32; 2] = [0x11, 0x84];
pub const DETAIL_VALUE_CENTER: [i32; 2] = [0x59, 0xd6];
pub const DETAIL_ROW_Y: [i32; 6] = [3, 0x11, 0x1f, 0x2d, 0x3b, 0x49];
pub const ITEMS_RECT: Rect = Rect::new(18, 9, 235, 69);
pub const ITEMS_COLS: usize = 6;
pub const ITEMS_ROWS: usize = 2;
pub const ITEM_EQUIPPED: &str = "l2ui_ch3.PetWnd.petitem_click";
pub const PAGE_ITEMS: usize = 2;
pub const PAGE_ACTIONS: usize = 1;
pub const ACTIONS_RECT: Rect = Rect::new(18, 9, 219, 69);
pub const ACTIONS_COLS: usize = 6;
pub const ACTIONS_ROWS: usize = 2;
pub const ACTIONS_MAX: usize = 15;
pub const PET_ACTION_CATEGORY: u32 = 4;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PetStats {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub sp: i32,
    pub level: i32,
    pub exp: i32,
    pub exp_this_level: i32,
    pub exp_next_level: i32,
    pub fed: i32,
    pub max_fed: i32,
    pub load: i32,
    pub max_load: i32,
    pub detail: [[i32; 6]; 2],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PetHit {
    Tab(usize),
    Name,
    Item(usize),
    ItemScroll(crate::scroll::ScrollHit),
    Action(i32),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PetItem {
    pub grid: GridEntry,
    pub equipped: bool,
}

#[derive(Debug, Clone)]
pub struct PetWndView {
    pub frame: FrameCtrl,
    pub tabs: TabStrip,
    pub name_button: TextButton,
    pub level_label: String,
    pub detail_labels: [[String; 6]; 2],
    pub stats: PetStats,
    pub name_pressed: bool,
    pub name_button_visible: bool,
    pub items: Vec<PetItem>,
    pub grid: ItemGrid,
    pub actions: Vec<crate::actions::ActionEntry>,
    pub action_pressed: Option<i32>,
}

pub fn grip() -> Rect {
    Rect::new(0, 0, SIZE.0, TITLE_HEIGHT)
}

pub fn initial_position() -> crate::initial_position::InitialPosition {
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 0, vertical_mode: 3, x: DEFAULT_POSITION.0, y: DEFAULT_POSITION.1 }
}

fn ratio(value: i32, max: i32, width: i32) -> i32 {
    if max == 0 { 0 } else { value.wrapping_mul(width) / max }
}

fn percent(value: f32) -> String {
    format!("{}%", crate::crt_float::format_f2(f64::from(value)))
}

fn action_at(actions: &[crate::actions::ActionEntry], px: i32, py: i32) -> Option<i32> {
    if !ACTIONS_RECT.contains(px, py) {
        return None;
    }
    let slot = crate::inventory::grid_slot_at(px - ACTIONS_RECT.x, py - ACTIONS_RECT.y, ACTIONS_COLS, ACTIONS_ROWS, 0)?;
    actions.get(slot).map(|a| a.id)
}

fn paint_actions(list: &mut DrawList, x: i32, y: i32, actions: &[crate::actions::ActionEntry], pressed: Option<i32>, alpha: f32) {
    let outline = crate::inventory::OUTLINE_SIZE;
    for (slot, action) in actions.iter().take(ACTIONS_COLS * ACTIONS_ROWS).enumerate() {
        let (sx, sy) = crate::inventory::grid_slot_pos_cols(slot, ACTIONS_COLS);
        let (sx, sy) = (x + ACTIONS_RECT.x + sx, y + ACTIONS_RECT.y + sy);
        list.draw_texture(sx, sy, 32, 32, 0, 0, 32, 32, action.icon(), alpha);
        list.draw_texture(sx - 1, sy - 1, outline, outline, 0, 0, outline, outline, crate::inventory::OUTLINE, alpha);
        if pressed == Some(action.id) {
            list.draw_texture(sx - 1, sy - 1, outline, outline, 0, 0, outline, outline, crate::inventory::OUTLINE_DOWN, alpha);
        }
    }
}

fn paint_detail(list: &mut DrawList, x: i32, y: i32, labels: &[[String; 6]; 2], values: &[[i32; 6]; 2], font: &BitmapFont) {
    for column in 0..2 {
        for (row, ry) in DETAIL_ROW_Y.into_iter().enumerate() {
            list.draw_text(x + DETAIL_LABEL_X[column], y + ry, LABEL, &labels[column][row], font);
            let value = values[column][row].to_string();
            list.draw_text(x + DETAIL_VALUE_CENTER[column] - font.measure(&value) as i32 / 2, y + ry, VALUE, &value, font);
        }
    }
}

fn paint_hp_mp(list: &mut DrawList, x: i32, y: i32, s: &PetStats, font: &BitmapFont, alpha: f32) {
    let hp = ratio(s.hp, s.max_hp, SHORT_BAR);
    if hp > SHORT_BAR {
        list.draw_texture(x + 0x23, y + 0x33, SHORT_BAR, 0xc, 0, 0, 8, 0xc, HP_BAR, alpha);
    } else if hp > 0 {
        list.draw_texture(x + 0x23, y + 0x33, hp, 0xc, 0, 0, 8, 0xc, HP_BAR, alpha);
    }
    let mp = ratio(s.mp, s.max_mp, SHORT_BAR);
    if mp > SHORT_BAR {
        list.draw_texture(x + 0x23, y + 0x41, SHORT_BAR, 10, 0, 0, 0x54, 10, MP_BAR, alpha);
    } else if mp > 0 {
        list.draw_texture(x + 0x23, y + 0x41, mp, 0xc, 0, 0, 8, 0xc, MP_BAR, alpha);
    }
    let width = |value: &str| font.measure(value) as i32;
    for (row, current, max) in [(0x35, s.hp, s.max_hp), (0x43, s.mp, s.max_mp)] {
        list.draw_text(x + 0x4d - width("/") / 2, y + row, NUMBER, "/", font);
        let current = current.to_string();
        list.draw_text(x + 0x43 - width(&current), y + row, NUMBER, &current, font);
        list.draw_text(x + 0x53, y + row, NUMBER, &max.to_string(), font);
    }
}

fn paint_level(list: &mut DrawList, x: i32, y: i32, label: &str, s: &PetStats, font: &BitmapFont) {
    list.draw_text(x + 0xf, y + 0x23, LABEL, label, font);
    let name = if s.name.is_empty() { "(null)" } else { s.name.as_str() };
    list.draw_text(x + 0x21, y + 0x23, VALUE, &format!("{} {}", s.level, name), font);
}

impl PetWndView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> PetWndView {
        let text = |id: u32| lookup(id).unwrap_or_default();
        let mut tabs = TabStrip::new(0xfe001b, 0);
        for (page, label) in TAB_SYSSTRINGS.into_iter().enumerate() {
            tabs.add_fixed(TabSpec {
                label: Some(text(label)), page: Some(page), size: TAB_SIZE,
                textures: [Some(TAB_NORMAL.into()), Some(TAB_PRESSED.into())],
                spacing: i32::from(page < 2), tooltip: -1, special_attachment: false,
            });
        }
        for button in &mut tabs.buttons { button.visible = true; }
        let mut frame = FrameCtrl::titled(&text(TITLE_SYSSTRING), true);
        frame.minimize_button = true;
        PetWndView {
            frame,
            tabs,
            name_button: TextButton::new(NAME_BUTTON, BUTTON_NORMAL, BUTTON_DOWN, &text(NAME_SYSSTRING)),
            level_label: text(LEVEL_SYSSTRING),
            detail_labels: DETAIL_LABELS.map(|column| column.map(text)),
            stats: PetStats::default(),
            name_pressed: false,
            name_button_visible: false,
            items: Vec::new(),
            grid: ItemGrid { outline: Some(crate::inventory::OUTLINE), click_first: true, ..ItemGrid::new(ITEMS_RECT, ITEMS_COLS, ITEMS_ROWS) },
            actions: Vec::new(),
            action_pressed: None,
        }
    }

    pub fn page(&self) -> usize {
        self.tabs.control.current.unwrap_or(0)
    }

    pub fn press_tab(&mut self, index: usize) {
        let mut events = TabEvents::default();
        self.tabs.control.press(index, true, &mut events);
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<PetHit> {
        if self.name_button_visible && self.name_button.contains(x, y) {
            return Some(PetHit::Name);
        }
        if let Some(tab) = self.tabs.buttons.iter().position(|b| b.paint.rect.offset(TABS_POS.0, TABS_POS.1).contains(x, y)) {
            return Some(PetHit::Tab(tab));
        }
        if self.page() == PAGE_ACTIONS {
            return action_at(&self.actions, x - PAGE_RECT.x, y - PAGE_RECT.y).map(PetHit::Action);
        }
        if self.page() == PAGE_ITEMS {
            return match self.grid.hit(x - PAGE_RECT.x, y - PAGE_RECT.y)? {
                GridHit::Slot(index) => (index < self.items.len()).then_some(PetHit::Item(index)),
                GridHit::Scroll(hit) => Some(PetHit::ItemScroll(hit)),
            };
        }
        None
    }

    pub fn slot_at(&self, x: i32, y: i32) -> Option<usize> {
        if self.page() != PAGE_ITEMS {
            return None;
        }
        match self.grid.hit(x - PAGE_RECT.x, y - PAGE_RECT.y)? {
            GridHit::Slot(index) => Some(index),
            GridHit::Scroll(_) => None,
        }
    }

    pub fn move_item(&mut self, object_id: i32, target: usize) -> bool {
        let Some(current) = self.items.iter().position(|item| item.grid.object_id == object_id) else { return false };
        if target == current {
            return false;
        }
        let item = self.items.remove(current);
        let at = target.min(self.items.len());
        self.items.insert(at, item);
        self.grid.selected = Some(at);
        true
    }

    pub fn set_actions(&mut self, all: &[crate::actions::ActionEntry]) {
        self.actions = all.iter().filter(|a| a.category == PET_ACTION_CATEGORY).take(ACTIONS_MAX).cloned().collect();
    }

    pub fn set_items(&mut self, items: Vec<PetItem>) {
        self.items = items;
        if self.grid.selected.is_some_and(|index| index >= self.items.len()) {
            self.grid.selected = None;
        }
        self.grid.update_rows(self.items.len());
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        let s = &self.stats;
        self.frame.paint(list, x, y, SIZE.0, TITLE_HEIGHT, font, alpha);
        list.draw_texture(x, y + 0x14, 0x100, 0x5f, 0, 0, 0x100, 0x5f, BACK1, alpha);
        list.draw_texture(x, y + 0x73, 0x100, 0x17, 0, 0, 0x100, 0x17, BACK2, alpha);
        paint_hp_mp(list, x, y, s, font, alpha);
        paint_level(list, x, y, &self.level_label, s, font);
        let mut bar = |bx: i32, by: i32, w: i32, h: i32, uw: i32, vh: i32, texture: &str| {
            list.draw_texture(x + bx, y + by, w, h, 0, 0, uw, vh, texture, alpha);
        };
        let fed = ratio(s.fed, s.max_fed, LONG_BAR);
        if fed > LONG_BAR {
            bar(0x23, 0x5d, LONG_BAR, 10, 8, 10, FOOD_BAR);
        } else if fed > 0 {
            bar(0x23, 0x5d, fed, 0xc, 8, 0xc, FOOD_BAR);
        }
        let range = s.exp_next_level.wrapping_sub(s.exp_this_level);
        let exp = if s.exp_next_level == 0 || range == 0 {
            0
        } else {
            (s.exp.wrapping_sub(s.exp_this_level) as f32 * LONG_BAR as f32 / range as f32) as i32
        };
        if exp > LONG_BAR {
            bar(0x23, 0x4f, LONG_BAR, 0xc, 8, 0xc, EXP_BAR);
        } else if exp > 0 {
            bar(0x23, 0x4f, exp, 0xc, exp, 0xc, EXP_BAR);
        }
        let load = ratio(s.load, s.max_load, SHORT_BAR);
        let mut weight = 0.0;
        if load > 0 {
            weight = s.load as f32 / s.max_load as f32 * 100.0;
            let texture = if weight <= 50.0 {
                Some(WEIGHT_BARS[0])
            } else if weight <= 66.66 {
                Some(WEIGHT_BARS[1])
            } else if weight <= 80.0 {
                Some(WEIGHT_BARS[2])
            } else {
                Some(WEIGHT_BARS[3])
            };
            if let Some(texture) = texture {
                bar(0x9c, 0x33, load, 0xc, 8, 0xc, texture);
            }
        }
        let mut text = |tx: i32, ty: i32, color: Argb, value: &str| list.draw_text(x + tx, y + ty, color, value, font);
        let width = |value: &str| font.measure(value) as i32;
        let fed_text = percent(div_f32(s.fed as f32 * 100.0, s.max_fed as f32));
        text(0x8a - width(&fed_text) / 2, 0x5f, PERCENT, &fed_text);
        let exp_text = percent(div_f32(s.exp.wrapping_sub(s.exp_this_level) as f32 * 100.0, range as f32));
        text(0x8a - width(&exp_text) / 2, 0x51, PERCENT, &exp_text);
        let sp = s.sp.to_string();
        text(0xc6 - width(&sp) / 2, 0x42, VALUE, &sp);
        let weight_text = percent(weight);
        text(0xc6 - width(&weight_text) / 2, 0x35, NUMBER, &weight_text);
        if self.name_button_visible {
            self.name_button.paint(list, x, y, self.name_pressed, font);
        }
        self.tabs.paint(list, (x + TABS_POS.0, y + TABS_POS.1), font);
        self.paint_page(list, x + PAGE_RECT.x, y + PAGE_RECT.y, font, alpha);
    }

    fn paint_page(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        let page = self.page().min(2);
        list.draw_texture(x, y, PAGE_RECT.w, PAGE_RECT.h, 0, 0, PAGE_RECT.w, PAGE_RECT.h, PAGE_BACKS[page], alpha);
        if page == PAGE_ACTIONS {
            paint_actions(list, x, y, &self.actions, self.action_pressed, alpha);
        }
        if page == PAGE_ITEMS {
            let entries: Vec<GridEntry> = self.items.iter().map(|item| item.grid.clone()).collect();
            self.grid.paint(list, x, y, &entries, font);
            let first = self.grid.scroll_row * ITEMS_COLS;
            for (index, item) in self.items.iter().enumerate().skip(first).take(self.grid.visible()) {
                if item.equipped {
                    let (sx, sy) = crate::inventory::grid_slot_pos_cols(index - first, ITEMS_COLS);
                    let (sx, sy) = (x + ITEMS_RECT.x + sx, y + ITEMS_RECT.y + sy);
                    list.draw_texture(sx, sy, 32, 32, 0, 0, 32, 32, ITEM_EQUIPPED, alpha);
                }
            }
        }
        if page != 0 {
            return;
        }
        paint_detail(list, x, y, &self.detail_labels, &self.stats.detail, font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;
    use crate::font::Glyphs;

    const SMALL: &[u8] = include_bytes!("../tests/fixtures/smallfont-e.gly");

    fn font() -> BitmapFont {
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(SMALL).unwrap())
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    fn view() -> PetWndView {
        let mut view = PetWndView::new(|id| Some(format!("s{id}")));
        view.name_button_visible = true;
        view.stats = PetStats {
            name: "Wolf".into(), hp: 50, max_hp: 100, mp: 200, max_mp: 100, sp: 7, level: 15,
            exp: 150, exp_this_level: 100, exp_next_level: 200, fed: 30, max_fed: 60, load: 70, max_load: 100,
            detail: [[1, 2, 3, 4, 5, 6], [7, 8, 9, 10, 11, 12]],
        };
        view
    }

    #[test]
    fn the_status_bars_follow_the_original_widths_and_saturation() {
        let font = font();
        let mut list = DrawList::default();
        view().paint(&mut list, 0, 0, &font, 1.0);
        let hp = with_texture(&list.quads, HP_BAR);
        assert_eq!((hp[0].x, hp[0].y, hp[0].w, hp[0].h, hp[0].uw), (35.0, 51.0, 42.0, 12.0, 8.0));
        let mp = with_texture(&list.quads, MP_BAR);
        assert_eq!((mp[0].w, mp[0].h, mp[0].uw, mp[0].vh), (85.0, 10.0, 84.0, 10.0));
        let exp = with_texture(&list.quads, EXP_BAR);
        assert_eq!((exp[0].y, exp[0].w, exp[0].uw), (79.0, 103.0, 103.0));
        let food = with_texture(&list.quads, FOOD_BAR);
        assert_eq!((food[0].y, food[0].w), (93.0, 103.0));
        let weight = with_texture(&list.quads, WEIGHT_BARS[2]);
        assert_eq!((weight[0].x, weight[0].w), (156.0, 59.0));
    }

    #[test]
    fn level_row_0x100dde50_prints_null_for_a_pet_without_a_name_like_the_crt() {
        let font = font();
        let mut view = view();
        view.stats.name.clear();
        let mut named = DrawList::default();
        let mut unnamed = DrawList::default();
        super::paint_level(&mut named, 0, 0, "Lv", &view.stats, &font);
        let mut with_null = view.stats.clone();
        with_null.name = "(null)".into();
        super::paint_level(&mut unnamed, 0, 0, "Lv", &with_null, &font);
        assert_eq!(named.quads, unnamed.quads);
    }

    #[test]
    fn zero_denominators_print_what_the_crt_prints_for_infinity_and_the_indefinite_nan() {
        let font = font();
        let mut view = view();
        view.stats.max_fed = 0;
        view.stats.fed = 5;
        view.stats.exp_next_level = 100;
        view.stats.exp_this_level = 100;
        view.stats.exp = 100;
        let mut list = DrawList { capture_text: true, ..Default::default() };
        view.paint(&mut list, 0, 0, &font, 1.0);
        let lines: Vec<String> = list.text_runs.iter().map(|run| run.text.clone()).collect();
        assert!(lines.contains(&"1.#J%".to_string()), "{lines:?}");
        assert!(lines.contains(&"-1.#J%".to_string()), "{lines:?}");
    }

    #[test]
    fn the_weight_bar_changes_texture_at_50_66_and_80_percent() {
        let font = font();
        for (load, texture) in [(50, 0), (60, 1), (80, 2), (90, 3)] {
            let mut view = view();
            view.stats.load = load;
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font, 1.0);
            assert_eq!(with_texture(&list.quads, WEIGHT_BARS[texture]).len(), 1, "{load}");
        }
    }

    #[test]
    fn the_items_page_shows_the_pet_inventory_with_the_equipped_mark() {
        let font = font();
        let mut view = view();
        view.press_tab(PAGE_ITEMS);
        view.set_items(vec![
            PetItem { grid: GridEntry { object_id: 1, icon: "icon.a".into(), count: 1, ..Default::default() }, equipped: true },
            PetItem { grid: GridEntry { object_id: 2, icon: "icon.b".into(), count: 5, ..Default::default() }, equipped: false },
        ]);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font, 1.0);
        let a = with_texture(&list.quads, "icon.a");
        assert_eq!((a[0].x, a[0].y), (19.0, 148.0));
        assert_eq!((with_texture(&list.quads, "icon.b")[0].x), 19.0 + 37.0);
        let mark = with_texture(&list.quads, ITEM_EQUIPPED);
        assert_eq!((mark.len(), mark[0].x, mark[0].y), (1, 19.0, 148.0));
        assert_eq!(view.hit(20, 150), Some(PetHit::Item(0)));
        assert_eq!(view.hit(20 + 37 * 2, 150), None);
    }

    #[test]
    fn paint_page_draws_the_inventory_page_back_and_the_grid_at_0x100ded40_offsets() {
        let font = font();
        let mut view = view();
        view.press_tab(PAGE_ITEMS);
        view.set_items(vec![PetItem { grid: GridEntry { object_id: 1, icon: "icon.a".into(), count: 1, ..Default::default() }, equipped: false }]);
        let mut list = DrawList::default();
        view.paint_page(&mut list, 0, 138, &font, 1.0);
        let back = with_texture(&list.quads, PAGE_BACKS[2]);
        assert_eq!((back.len(), back[0].y, back[0].w, back[0].h), (1, 138.0, 256.0, 91.0));
        assert_eq!((ITEMS_RECT.x, ITEMS_RECT.y, ITEMS_RECT.w, ITEMS_RECT.h), (18, 9, 235, 69));
        let icon = with_texture(&list.quads, "icon.a");
        assert_eq!((icon[0].x, icon[0].y), (19.0, 148.0));
    }

    #[test]
    fn the_action_page_lists_the_pet_actions_from_actionname() {
        let font = font();
        let mut view = view();
        let all: Vec<_> = (0..20).map(|id| crate::actions::ActionEntry { id, category: if id % 2 == 0 { 4 } else { 1 }, icon: format!("icon.a{id}"), name: String::new(), ..Default::default() }).collect();
        view.set_actions(&all);
        assert_eq!(view.actions.len(), 10);
        view.press_tab(PAGE_ACTIONS);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font, 1.0);
        let first = with_texture(&list.quads, "icon.a0");
        assert_eq!((first[0].x, first[0].y), (19.0, 148.0));
        assert_eq!(view.hit(20 + 37, 150), Some(PetHit::Action(2)));
    }

    #[test]
    fn tabs_switch_the_page_and_the_info_page_lists_the_stats() {
        let font = font();
        let mut view = view();
        assert_eq!(view.page(), 0);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font, 1.0);
        assert_eq!(with_texture(&list.quads, PAGE_BACKS[0])[0].y, 138.0);
        assert_eq!(view.hit(12 + 74 + 1 + 5, 120), Some(PetHit::Tab(1)));
        assert_eq!(view.hit(170, 30), Some(PetHit::Name));
        view.press_tab(2);
        assert_eq!(view.page(), 2);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font, 1.0);
        assert_eq!(with_texture(&list.quads, PAGE_BACKS[2]).len(), 1);
        assert!(with_texture(&list.quads, PAGE_BACKS[0]).is_empty());
    }
}

pub mod summoned {
    use super::*;

    pub const SIZE: (i32, i32) = super::summon_layout::SIZE;
    pub const STYLE: u32 = 0x3002;
    pub const TITLE_SYSSTRING: u32 = 0x1ef;
    pub const BACK1: &str = "L2UI_CH3.SummonWnd.summon_back1";
    pub const BACK2: &str = "L2UI_CH3.SummonWnd.summon_back2";
    pub const PAGE_BACKS: [&str; 2] = ["L2UI_ch3.summonwnd.summon_back31", "L2UI_ch3.summonwnd.summon_back32"];
    pub const TAB_NORMAL: &str = "L2UI_Ch3.SummonWnd.summon_tab2";
    pub const TAB_PRESSED: &str = "L2UI_Ch3.SummonWnd.summon_tab1";
    pub const TABS_POS: (i32, i32) = (super::summon_layout::RECT_100.x, super::summon_layout::RECT_100.y);
    pub const PAGE_RECT: Rect = super::summon_layout::RECT_F8;
    pub const SUMMON_ACTION_CATEGORY: u32 = 5;

    pub fn grip() -> Rect {
        Rect::new(0, 0, SIZE.0, TITLE_HEIGHT)
    }

    #[derive(Debug, Clone)]
    pub struct SummonedWndView {
        pub frame: FrameCtrl,
        pub tabs: TabStrip,
        pub level_label: String,
        pub detail_labels: [[String; 6]; 2],
        pub stats: PetStats,
        pub actions: Vec<crate::actions::ActionEntry>,
        pub action_pressed: Option<i32>,
    }

    impl SummonedWndView {
        pub fn new(lookup: impl Fn(u32) -> Option<String>) -> SummonedWndView {
            let text = |id: u32| lookup(id).unwrap_or_default();
            let mut tabs = TabStrip::new(0xfe001b, 0);
            for (page, label) in TAB_SYSSTRINGS.into_iter().take(2).enumerate() {
                tabs.add_fixed(TabSpec {
                    label: Some(text(label)), page: Some(page), size: TAB_SIZE,
                    textures: [Some(TAB_NORMAL.into()), Some(TAB_PRESSED.into())],
                    spacing: i32::from(page < 1), tooltip: -1, special_attachment: false,
                });
            }
            for button in &mut tabs.buttons { button.visible = true; }
            let mut frame = FrameCtrl::titled(&text(TITLE_SYSSTRING), true);
            frame.minimize_button = true;
            SummonedWndView {
                frame,
                tabs,
                level_label: text(LEVEL_SYSSTRING),
                detail_labels: DETAIL_LABELS.map(|column| column.map(text)),
                stats: PetStats::default(),
                actions: Vec::new(),
                action_pressed: None,
            }
        }

        pub fn page(&self) -> usize {
            self.tabs.control.current.unwrap_or(0)
        }

        pub fn press_tab(&mut self, index: usize) {
            let mut events = TabEvents::default();
            self.tabs.control.press(index, true, &mut events);
        }

        pub fn set_actions(&mut self, all: &[crate::actions::ActionEntry]) {
            self.actions = all.iter().filter(|a| a.category == SUMMON_ACTION_CATEGORY).take(ACTIONS_MAX).cloned().collect();
        }

        pub fn hit(&self, x: i32, y: i32) -> Option<PetHit> {
            if let Some(tab) = self.tabs.buttons.iter().position(|b| b.paint.rect.offset(TABS_POS.0, TABS_POS.1).contains(x, y)) {
                return Some(PetHit::Tab(tab));
            }
            if self.page() == PAGE_ACTIONS {
                return action_at(&self.actions, x - PAGE_RECT.x, y - PAGE_RECT.y).map(PetHit::Action);
            }
            None
        }

        pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
            self.frame.paint(list, x, y, SIZE.0, TITLE_HEIGHT, font, alpha);
            list.draw_texture(x, y + 0x14, 0x100, 0x43, 0, 0, 0x100, 0x43, BACK1, alpha);
            list.draw_texture(x, y + 0x57, 0x100, 0x17, 0, 0, 0x100, 0x17, BACK2, alpha);
            paint_hp_mp(list, x, y, &self.stats, font, alpha);
            paint_level(list, x, y, &self.level_label, &self.stats, font);
            self.tabs.paint(list, (x + TABS_POS.0, y + TABS_POS.1), font);
            let page = self.page().min(1);
            let (px, py) = (x + PAGE_RECT.x, y + PAGE_RECT.y);
            list.draw_texture(px, py, PAGE_RECT.w, PAGE_RECT.h, 0, 0, PAGE_RECT.w, PAGE_RECT.h, PAGE_BACKS[page], alpha);
            if page == PAGE_ACTIONS {
                paint_actions(list, px, py, &self.actions, self.action_pressed, alpha);
            } else {
                paint_detail(list, px, py, &self.detail_labels, &self.stats.detail, font);
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::font::Glyphs;

        #[test]
        fn the_summoned_window_has_info_and_summon_actions_only() {
            let font = BitmapFont::new("f".into(), Glyphs::parse(include_bytes!("../tests/fixtures/smallfont-e.gly")).unwrap());
            let mut view = SummonedWndView::new(|id| Some(format!("s{id}")));
            assert_eq!(view.tabs.buttons.len(), 2);
            view.stats = PetStats { hp: 50, max_hp: 100, mp: 10, max_mp: 100, level: 3, name: "Kat".into(), ..Default::default() };
            let all: Vec<_> = (0..8).map(|id| crate::actions::ActionEntry { id, category: if id < 3 { 5 } else { 4 }, icon: format!("i{id}"), name: String::new(), ..Default::default() }).collect();
            view.set_actions(&all);
            assert_eq!(view.actions.len(), 3);
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font, 1.0);
            assert!(list.quads.iter().any(|q| q.texture == BACK1 && q.y == 20.0 && q.h == 67.0));
            assert!(list.quads.iter().any(|q| q.texture == PAGE_BACKS[0] && q.y == 110.0));
            assert!(list.quads.iter().any(|q| q.texture == HP_BAR && q.w == 42.0));
            assert!(!list.quads.iter().any(|q| q.texture == EXP_BAR));
            view.press_tab(PAGE_ACTIONS);
            assert_eq!(view.hit(20 + 37, 110 + 12), Some(PetHit::Action(1)));
        }
    }
}
