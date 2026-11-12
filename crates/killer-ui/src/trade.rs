#[path = "trade_layout_generated.rs"]
mod layout;

use crate::button::TextButton;
use crate::color::TEXT;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{BUTTON_SIZE, BUTTON_Y, CLOSE_INSET, FrameCtrl, TITLE_HEIGHT};
use crate::item_grid::{GridEntry, GridHit, ICON_BOX, ItemGrid};
use crate::shop::{is_stackable, trade_default_position};
use crate::window::Rect;

pub const TRADE_SIZE: (i32, i32) = layout::SIZE;
pub const TRADE_BACK: &str = "L2UI_ch3.TradeWnd.Trade_Back";
pub const ADD_BUTTON: &str = "l2ui_ch3.button.downbutton";
pub const BTN_NORMAL: &str = "L2UI_ch3.button.btn1_normal";
pub const BTN_DOWN: &str = "L2UI_ch3.button.btn1_normalon";
pub const BACK_RECT: Rect = Rect::new(0, 20, 256, 381);
pub const MINE_RECT: Rect = layout::RECT_E8;
pub const OFFERED_RECT: Rect = layout::RECT_EC;
pub const THEIRS_RECT: Rect = layout::RECT_F0;
pub const GRID_COLS: usize = 6;
pub const MINE_ROWS: usize = 4;
pub const OFFER_ROWS: usize = 2;
pub const ADD_RECT: Rect = layout::RECT_F8;
pub const OK_RECT: Rect = layout::RECT_FC;
pub const CANCEL_RECT: Rect = layout::RECT_100;
pub const MY_NAME_POS: (i32, i32) = (11, 184);
pub const PARTNER_NAME_POS: (i32, i32) = (11, 278);
pub const TITLE_SYSSTRING: u32 = 445;
pub const OK_SYSSTRING: u32 = 140;
pub const CANCEL_SYSSTRING: u32 = 141;
pub const MOVE_COUNT_SYSMSG: u32 = 72;
pub const TOO_MANY_INPUT_SYSMSG: u32 = 1036;
pub const TRADE_ID: i32 = 1;
pub const DIALOG_TYPE_TRADE_COUNT: i32 = 8;
pub const UPDATE_ADD: i16 = 1;
pub const UPDATE_REMOVE: i16 = 2;
pub const UPDATE_COUNT: i16 = 3;
pub const GRID_CAPACITY: usize = 160;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TradeEntry {
    pub grid: GridEntry,
    pub quantity: i64,
    pub consume_type: u32,
    pub tip: Option<crate::item_tip::ItemTipData>,
}

impl TradeEntry {
    pub fn stackable(&self) -> bool {
        is_stackable(self.consume_type)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeGrid {
    Mine,
    Offered,
    Theirs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeHit {
    Close,
    Slot(TradeGrid, usize),
    Scroll(TradeGrid, crate::scroll::ScrollHit),
    Add,
    Ok,
    Cancel,
    Title,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TradeAction {
    None,
    AddItem { object_id: i32, count: i32 },
    AskCount { index: usize, item_name: String, max: i64, consume_type: u32 },
    TooMany,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TradeView {
    pub title: String,
    pub my_name: String,
    pub partner_name: String,
    pub mine: Vec<TradeEntry>,
    pub offered: Vec<TradeEntry>,
    pub theirs: Vec<TradeEntry>,
    pub mine_grid: ItemGrid,
    pub offered_grid: ItemGrid,
    pub theirs_grid: ItemGrid,
    pub add_pressed: bool,
    pub ok_pressed: bool,
    pub cancel_pressed: bool,
    pub ok_label: String,
    pub cancel_label: String,
}

pub fn close_button() -> Rect {
    Rect::new(TRADE_SIZE.0 - CLOSE_INSET, BUTTON_Y, BUTTON_SIZE, BUTTON_SIZE)
}

pub fn grip() -> Rect {
    Rect::new(0, 0, TRADE_SIZE.0, TITLE_HEIGHT)
}

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    trade_default_position(viewport)
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    crate::inventory::initial_position(viewport)
}

fn boxed_grid(rect: Rect, rows: usize) -> ItemGrid {
    let mut grid = ItemGrid::new(rect, GRID_COLS, rows);
    grid.outline = Some(ICON_BOX);
    grid
}

impl TradeView {
    pub fn grid_mut(&mut self, grid: TradeGrid) -> &mut ItemGrid {
        match grid {
            TradeGrid::Mine => &mut self.mine_grid,
            TradeGrid::Offered => &mut self.offered_grid,
            TradeGrid::Theirs => &mut self.theirs_grid,
        }
    }

    pub fn receive_add(&mut self, grid: TradeGrid, entry: TradeEntry) {
        let entries = match grid {
            TradeGrid::Mine => &mut self.mine,
            TradeGrid::Offered => &mut self.offered,
            TradeGrid::Theirs => &mut self.theirs,
        };
        if entries.len() >= GRID_CAPACITY {
            return;
        }
        if let Some(existing) = entries.iter_mut().find(|e| e.grid.object_id == entry.grid.object_id && e.stackable()) {
            existing.quantity = existing.quantity.wrapping_add(entry.quantity);
        } else {
            entries.push(entry);
        }
        let count = entries.len();
        self.grid_mut(grid).update_rows(count);
    }

    pub fn receive_update(&mut self, update_type: i16, entry: TradeEntry) {
        let index = self.mine.iter().position(|e| e.grid.object_id == entry.grid.object_id);
        match (update_type, index) {
            (UPDATE_ADD, _) => self.receive_add(TradeGrid::Mine, entry),
            (UPDATE_REMOVE, Some(index)) => { self.mine.remove(index); }
            (UPDATE_COUNT, Some(index)) => self.mine[index] = entry,
            _ => {}
        }
        self.mine_grid.update_rows(self.mine.len());
    }

    pub fn with_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> TradeView {
        let text = |id: u32| lookup(id).unwrap_or_else(|| format!("sysstring {id}"));
        TradeView {
            title: text(TITLE_SYSSTRING),
            ok_label: text(OK_SYSSTRING),
            cancel_label: text(CANCEL_SYSSTRING),
            mine_grid: boxed_grid(MINE_RECT, MINE_ROWS),
            offered_grid: boxed_grid(OFFERED_RECT, OFFER_ROWS),
            theirs_grid: boxed_grid(THEIRS_RECT, OFFER_ROWS),
            ..Default::default()
        }
    }

    pub fn clear(&mut self) {
        self.mine.clear();
        self.offered.clear();
        self.theirs.clear();
        self.mine_grid.clear();
        self.offered_grid.clear();
        self.theirs_grid.clear();
    }

    pub fn set_lists(&mut self, mine: Vec<TradeEntry>, offered: Vec<TradeEntry>, theirs: Vec<TradeEntry>) {
        self.mine = mine;
        self.offered = offered;
        self.theirs = theirs;
        self.mine_grid.update_rows(self.mine.len());
        self.offered_grid.update_rows(self.offered.len());
        self.theirs_grid.update_rows(self.theirs.len());
        if self.mine_grid.selected.is_some_and(|i| i >= self.mine.len()) {
            self.mine_grid.selected = None;
        }
    }

    pub fn entries(&self, grid: TradeGrid) -> &[TradeEntry] {
        match grid {
            TradeGrid::Mine => &self.mine,
            TradeGrid::Offered => &self.offered,
            TradeGrid::Theirs => &self.theirs,
        }
    }

    fn grid_entries(&self, grid: TradeGrid) -> Vec<GridEntry> {
        self.entries(grid).iter().map(|e| e.grid.clone()).collect()
    }

    pub fn select(&mut self, grid: TradeGrid, index: usize) {
        self.mine_grid.selected = (grid == TradeGrid::Mine).then_some(index);
        self.offered_grid.selected = None;
        self.theirs_grid.selected = None;
    }

    pub fn press_add(&mut self) -> TradeAction {
        let Some(index) = self.mine_grid.selected.filter(|i| *i < self.mine.len()) else { return TradeAction::None };
        let selected = self.mine_grid.selected;
        let action = self.add_entry(index);
        if matches!(action, TradeAction::AddItem { .. }) {
            self.mine_grid.selected = selected;
        }
        action
    }

    pub fn add_entry(&mut self, index: usize) -> TradeAction {
        let Some(entry) = self.mine.get(index) else { return TradeAction::None };
        if entry.stackable() && entry.quantity > 1 {
            return TradeAction::AskCount {
                index,
                item_name: entry.grid.name.clone(),
                max: i64::from(entry.grid.count),
                consume_type: entry.consume_type,
            };
        }
        self.mine_grid.selected = None;
        TradeAction::AddItem { object_id: entry.grid.object_id, count: 1 }
    }

    pub fn answer_count(&mut self, index: usize, count: i64) -> TradeAction {
        let Some(entry) = self.mine.get(index) else { return TradeAction::None };
        if count <= 0 {
            return TradeAction::None;
        }
        if count > entry.quantity {
            return TradeAction::TooMany;
        }
        self.mine_grid.selected = None;
        TradeAction::AddItem { object_id: entry.grid.object_id, count: count as i32 }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<TradeHit> {
        if !Rect::new(0, 0, TRADE_SIZE.0, TRADE_SIZE.1).contains(x, y) {
            return None;
        }
        if close_button().contains(x, y) {
            return Some(TradeHit::Close);
        }
        if y < TITLE_HEIGHT {
            return Some(TradeHit::Title);
        }
        if ADD_RECT.contains(x, y) {
            return Some(TradeHit::Add);
        }
        if OK_RECT.contains(x, y) {
            return Some(TradeHit::Ok);
        }
        if CANCEL_RECT.contains(x, y) {
            return Some(TradeHit::Cancel);
        }
        let grids = [(TradeGrid::Mine, &self.mine_grid), (TradeGrid::Offered, &self.offered_grid), (TradeGrid::Theirs, &self.theirs_grid)];
        for (which, grid) in grids {
            match grid.hit(x, y) {
                Some(GridHit::Slot(i)) => return Some(TradeHit::Slot(which, i)),
                Some(GridHit::Scroll(hit)) => return Some(TradeHit::Scroll(which, hit)),
                None => {}
            }
        }
        Some(TradeHit::Body)
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        let (w, h) = TRADE_SIZE;
        FrameCtrl::titled(&self.title, true).paint(list, x, y, w, h, font, alpha);
        let back = BACK_RECT;
        list.draw_texture(x + back.x, y + back.y, back.w, back.h, 0, 0, back.w, back.h, TRADE_BACK, alpha);
        if !self.my_name.is_empty() {
            list.draw_text(x + MY_NAME_POS.0, y + MY_NAME_POS.1, TEXT, &self.my_name, font);
        }
        if !self.partner_name.is_empty() {
            list.draw_text(x + PARTNER_NAME_POS.0, y + PARTNER_NAME_POS.1, TEXT, &self.partner_name, font);
        }
        self.mine_grid.paint(list, x, y, &self.grid_entries(TradeGrid::Mine), font);
        self.offered_grid.paint(list, x, y, &self.grid_entries(TradeGrid::Offered), font);
        self.theirs_grid.paint(list, x, y, &self.grid_entries(TradeGrid::Theirs), font);
        list.draw_texture(x + ADD_RECT.x, y + ADD_RECT.y, ADD_RECT.w, ADD_RECT.h, 0, 0, ADD_RECT.w, ADD_RECT.h, ADD_BUTTON, 1.0);
        TextButton::new(OK_RECT, BTN_NORMAL, BTN_DOWN, &self.ok_label).paint(list, x, y, self.ok_pressed, font);
        TextButton::new(CANCEL_RECT, BTN_NORMAL, BTN_DOWN, &self.cancel_label).paint(list, x, y, self.cancel_pressed, font);
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

    fn entry(object_id: i32, count: i32, consume_type: u32, icon: &str) -> TradeEntry {
        TradeEntry {
            grid: GridEntry { object_id, item_id: object_id, count, icon: icon.into(), name: format!("i{object_id}"), ..Default::default() },
            quantity: i64::from(count),
            consume_type,
            tip: None,
        }
    }

    fn view() -> TradeView {
        let mut view = TradeView::with_sysstrings(|id| Some(format!("s{id}")));
        view.my_name = "Luzal".into();
        view.partner_name = "Kerchief".into();
        view.set_lists(vec![entry(1, 1, 0, "a"), entry(2, 30, 1, "b")], vec![entry(3, 1, 0, "c")], vec![entry(4, 2, 1, "d")]);
        view
    }

    #[test]
    fn adding_sends_one_of_a_plain_item_and_asks_the_count_of_a_stack() {
        let mut view = view();
        assert_eq!(view.press_add(), TradeAction::None);
        view.select(TradeGrid::Mine, 0);
        assert_eq!(view.press_add(), TradeAction::AddItem { object_id: 1, count: 1 });
        assert_eq!(view.mine_grid.selected, Some(0));
        assert_eq!(view.add_entry(0), TradeAction::AddItem { object_id: 1, count: 1 });
        assert_eq!(view.mine_grid.selected, None);
        view.select(TradeGrid::Mine, 1);
        assert_eq!(view.press_add(), TradeAction::AskCount { index: 1, item_name: "i2".into(), max: 30, consume_type: 1 });
        assert_eq!(view.answer_count(1, 31), TradeAction::TooMany);
        assert_eq!(view.answer_count(1, 0), TradeAction::None);
        assert_eq!(view.answer_count(1, 12), TradeAction::AddItem { object_id: 2, count: 12 });
        assert_eq!(view.add_entry(9), TradeAction::None);
    }

    #[test]
    fn receive_1011ad30_merges_by_object_and_checks_capacity_before_merging() {
        let mut view = TradeView::with_sysstrings(|_| None);
        for grid in [TradeGrid::Mine, TradeGrid::Offered, TradeGrid::Theirs] {
            view.receive_add(grid, entry(7, 10, 1, "a"));
            view.receive_add(grid, entry(7, 5, 1, "a"));
            assert_eq!(view.entries(grid).len(), 1);
            assert_eq!(view.entries(grid)[0].quantity, 15);
            for id in 0..159 {
                view.receive_add(grid, entry(id + 100, 1, 0, "b"));
            }
            view.receive_add(grid, entry(7, 5, 1, "a"));
            assert_eq!(view.entries(grid).len(), 160);
            assert_eq!(view.entries(grid)[0].quantity, 15);
        }
    }

    #[test]
    fn update_1011af40_replaces_only_existing_and_removes_first_object() {
        let mut view = TradeView::with_sysstrings(|_| None);
        view.receive_update(3, entry(7, 3, 0, "a"));
        assert!(view.mine.is_empty());
        view.receive_update(1, entry(7, 1, 0, "a"));
        view.receive_update(1, entry(7, 1, 0, "a"));
        view.receive_update(3, entry(7, 10, 1, "b"));
        assert_eq!(view.mine[0].quantity, 10);
        assert_eq!(view.mine[1].quantity, 1);
        view.receive_update(2, entry(7, 0, 0, ""));
        assert_eq!(view.mine.len(), 1);
        assert_eq!(view.mine[0].quantity, 1);
        view.receive_update(4, entry(9, 2, 0, ""));
        assert_eq!(view.mine.len(), 1);
    }

    #[test]
    fn the_hit_test_follows_the_oncreate_rectangles() {
        let view = view();
        assert_eq!(view.hit(240, 10), Some(TradeHit::Close));
        assert_eq!(view.hit(10, 36), Some(TradeHit::Slot(TradeGrid::Mine, 0)));
        assert_eq!(view.hit(10, 201), Some(TradeHit::Slot(TradeGrid::Offered, 0)));
        assert_eq!(view.hit(10, 296), Some(TradeHit::Slot(TradeGrid::Theirs, 0)));
        assert_eq!(view.hit(145, 185), Some(TradeHit::Add));
        assert_eq!(view.hit(60, 380), Some(TradeHit::Ok));
        assert_eq!(view.hit(140, 380), Some(TradeHit::Cancel));
        assert_eq!(view.hit(128, 285), Some(TradeHit::Body));
        assert_eq!(default_position((1558, 933)), (1256, 214));
    }

    #[test]
    fn the_paint_draws_the_back_names_three_boxed_grids_and_the_buttons() {
        let font = font();
        let mut list = DrawList::default();
        view().paint(&mut list, 0, 0, &font, 1.0);
        let back = list.quads.iter().find(|q| q.texture == TRADE_BACK).unwrap();
        assert_eq!((back.x, back.y, back.w, back.h), (0.0, 20.0, 256.0, 381.0));
        let text: Vec<_> = list.quads.iter().filter(|q| q.texture == "f").collect();
        assert!(text.iter().any(|q| q.x == 11.0 && q.y == 184.0));
        assert!(text.iter().any(|q| q.x == 11.0 && q.y == 278.0));
        assert_eq!(list.quads.iter().filter(|q| q.texture == ICON_BOX).count(), 4);
        let d = list.quads.iter().find(|q| q.texture == "d").unwrap();
        assert_eq!((d.x, d.y), (10.0, 296.0));
        assert!(list.quads.iter().any(|q| q.texture == ADD_BUTTON && q.x == 140.0 && q.y == 181.0));
        assert!(list.quads.iter().any(|q| q.texture == BTN_NORMAL && q.x == 51.0 && q.y == 372.0));
    }
}
