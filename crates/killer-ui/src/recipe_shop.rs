#[path = "recipe_shop_layout_generated.rs"]
mod layout;

use crate::button::TextButton;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, TITLE_HEIGHT};
use crate::item_grid::{GridEntry, GridHit, ItemGrid};
use crate::window::Rect;

pub const SIZE: (i32, i32) = layout::SIZE;
pub const STYLE: u32 = 0x1002;
pub const BACK: &str = "L2UI_CH3.RecipeWnd.Recipe1_Back";
pub const TOP_RECT: Rect = layout::RECT_E8;
pub const BOTTOM_RECT: Rect = layout::RECT_EC;
pub const GRID_COLS: usize = 6;
pub const GRID_ROWS: usize = 4;
pub const UP_RECT: Rect = layout::RECT_FC;
pub const DOWN_RECT: Rect = layout::RECT_100;
pub const UP_NORMAL: &str = "L2UI_CH3.ScrollBar.ScrollBarUpBtn";
pub const UP_DOWN: &str = "L2UI_CH3.ScrollBar.ScrollBarUpOnBtn";
pub const DOWN_NORMAL: &str = "L2UI_CH3.ScrollBar.ScrollBarDownBtn";
pub const DOWN_DOWN: &str = "L2UI_CH3.ScrollBar.ScrollBarDownOnBtn";
pub const BTN_NORMAL: &str = "L2UI_CH3.Button.btn1_normal";
pub const BTN_DOWN: &str = "L2UI_CH3.Button.btn1_normalon";
pub const ABORT_RECT: Rect = layout::RECT_10C;
pub const MESSAGE_RECT: Rect = layout::RECT_108;
pub const START_RECT: Rect = layout::RECT_104;
pub const ABORT_SYSSTRING: u32 = 0x181;
pub const MESSAGE_SYSSTRING: u32 = 0x180;
pub const START_SYSSTRING: u32 = 0x1ac;
pub const RECIPE_LIST_SYSSTRING: u32 = 643;
pub const PRODUCTION_LIST_SYSSTRING: u32 = 661;
pub const COMMON_TITLE_SYSSTRING: u32 = 0x4bc;
pub const DWARF_TITLE_SYSSTRING: u32 = 0x4bd;
pub const LABEL_POS: [(i32, i32); 2] = [(11, 32), (11, 198)];
pub const LABEL_COLOR: Argb = Argb(0xffdcdcdc);
pub const COUNTER_COLOR: Argb = Argb(0xffb09b79);
pub const COUNTER_Y: i32 = 0xc6;
pub const COUNTER_RIGHT_INSET: i32 = 5;
pub const PRODUCTION_MAX: usize = 0x14;
pub const PRICE_DIALOG_TYPE: i32 = 0x23;
pub const PRICE_SYSMSG: u32 = 0x3c3;
pub const MESSAGE_DIALOG_TYPE: i32 = 0x24;
pub const MESSAGE_SYSMSG: u32 = 0x14e;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShopEntry {
    pub recipe_id: i32,
    pub cost: i32,
    pub grid: GridEntry,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeShopHit {
    Top(usize),
    Bottom(usize),
    Up,
    Down,
    Abort,
    Message,
    Start,
    Close,
    Scroll(bool, crate::scroll::ScrollHit),
}

#[derive(Debug, Clone)]
pub struct RecipeShopView {
    pub frame: FrameCtrl,
    pub titles: (String, String),
    pub labels: [String; 2],
    pub buttons: [TextButton; 3],
    pub top: Vec<ShopEntry>,
    pub bottom: Vec<ShopEntry>,
    pub top_grid: ItemGrid,
    pub bottom_grid: ItemGrid,
    pub pressed: Option<RecipeShopHit>,
}

pub fn grip() -> Rect {
    Rect::new(0, 0, SIZE.0, TITLE_HEIGHT)
}

impl RecipeShopView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> RecipeShopView {
        let text = |id: u32| lookup(id).unwrap_or_default();
        RecipeShopView {
            frame: FrameCtrl::titled(&text(DWARF_TITLE_SYSSTRING), true),
            titles: (text(COMMON_TITLE_SYSSTRING), text(DWARF_TITLE_SYSSTRING)),
            labels: [text(RECIPE_LIST_SYSSTRING), text(PRODUCTION_LIST_SYSSTRING)],
            buttons: [
                TextButton::new(ABORT_RECT, BTN_NORMAL, BTN_DOWN, &text(ABORT_SYSSTRING)),
                TextButton::new(MESSAGE_RECT, BTN_NORMAL, BTN_DOWN, &text(MESSAGE_SYSSTRING)),
                TextButton::new(START_RECT, BTN_NORMAL, BTN_DOWN, &text(START_SYSSTRING)),
            ],
            top: Vec::new(),
            bottom: Vec::new(),
            top_grid: ItemGrid::new(TOP_RECT, GRID_COLS, GRID_ROWS),
            bottom_grid: ItemGrid::new(BOTTOM_RECT, GRID_COLS, GRID_ROWS),
            pressed: None,
        }
    }

    pub fn open(&mut self, common: bool, top: Vec<ShopEntry>, bottom: Vec<ShopEntry>) {
        self.frame.title = if common { self.titles.0.clone() } else { self.titles.1.clone() };
        self.pressed = None;
        self.top = top.into_iter().take(160).collect();
        self.bottom = bottom.into_iter().take(160).collect();
        self.top_grid.clear();
        self.bottom_grid.clear();
        self.top_grid.update_rows(self.top.len());
        self.bottom_grid.update_rows(self.bottom.len());
    }

    pub fn add_production(&mut self, recipe_id: i32, cost: i32) {
        if let Some(entry) = self.top.iter_mut().find(|entry| entry.recipe_id == recipe_id) {
            entry.cost = cost;
            if self.bottom.len() < 160 && !self.bottom.iter().any(|other| other.grid.item_id == entry.grid.item_id) {
                self.bottom.push(entry.clone());
            }
            self.top_grid.selected = None;
            self.bottom_grid.update_rows(self.bottom.len());
        }
    }

    pub fn remove_selected_production(&mut self) {
        if let Some(index) = self.bottom_grid.selected.filter(|index| *index < self.bottom.len()) {
            self.bottom.remove(index);
            self.bottom_grid.selected = None;
            self.bottom_grid.update_rows(self.bottom.len());
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<RecipeShopHit> {
        let close = Rect::new(
            SIZE.0 - crate::frame::CLOSE_INSET,
            crate::frame::BUTTON_Y,
            crate::frame::BUTTON_SIZE,
            crate::frame::BUTTON_SIZE,
        );
        if close.contains(x, y) {
            return Some(RecipeShopHit::Close);
        }
        if UP_RECT.contains(x, y) {
            return Some(RecipeShopHit::Up);
        }
        if DOWN_RECT.contains(x, y) {
            return Some(RecipeShopHit::Down);
        }
        for (index, hit) in [RecipeShopHit::Abort, RecipeShopHit::Message, RecipeShopHit::Start].into_iter().enumerate()
        {
            if self.buttons[index].contains(x, y) {
                return Some(hit);
            }
        }
        for (bottom, grid, count) in
            [(false, &self.top_grid, self.top.len()), (true, &self.bottom_grid, self.bottom.len())]
        {
            match grid.hit(x, y) {
                Some(GridHit::Slot(i)) if i < count => {
                    return Some(if bottom { RecipeShopHit::Bottom(i) } else { RecipeShopHit::Top(i) });
                }
                Some(GridHit::Scroll(hit)) => return Some(RecipeShopHit::Scroll(bottom, hit)),
                _ => {}
            }
        }
        None
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        let (w, h) = SIZE;
        self.frame.paint(list, x, y, w, TITLE_HEIGHT, font, alpha);
        list.draw_texture(x, y + TITLE_HEIGHT, w, h - TITLE_HEIGHT, 0, 0, w, h - TITLE_HEIGHT, BACK, alpha);
        for (label, (lx, ly)) in self.labels.iter().zip(LABEL_POS) {
            list.draw_text(x + lx, y + ly, LABEL_COLOR, label, font);
        }
        let counter = format!("({}/{})", self.bottom.len(), PRODUCTION_MAX);
        list.draw_text(
            x + w - font.measure(&counter) as i32 - COUNTER_RIGHT_INSET,
            y + COUNTER_Y,
            COUNTER_COLOR,
            &counter,
            font,
        );
        let top: Vec<GridEntry> = self.top.iter().map(|entry| entry.grid.clone()).collect();
        let bottom: Vec<GridEntry> = self.bottom.iter().map(|entry| entry.grid.clone()).collect();
        self.top_grid.paint(list, x, y, &top, font);
        self.bottom_grid.paint(list, x, y, &bottom, font);
        let arrow = |list: &mut DrawList, rect: Rect, normal: &str, down: &str, pressed: bool| {
            list.draw_texture(
                x + rect.x,
                y + rect.y,
                rect.w,
                rect.h,
                0,
                0,
                rect.w,
                rect.h,
                if pressed { down } else { normal },
                alpha,
            );
        };
        arrow(list, UP_RECT, UP_NORMAL, UP_DOWN, self.pressed == Some(RecipeShopHit::Up));
        arrow(list, DOWN_RECT, DOWN_NORMAL, DOWN_DOWN, self.pressed == Some(RecipeShopHit::Down));
        for (button, hit) in
            self.buttons.iter().zip([RecipeShopHit::Abort, RecipeShopHit::Message, RecipeShopHit::Start])
        {
            button.paint(list, x, y, self.pressed == Some(hit), font);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::Glyphs;

    fn entry(recipe_id: i32) -> ShopEntry {
        ShopEntry {
            recipe_id,
            cost: 0,
            grid: GridEntry { item_id: recipe_id, icon: format!("icon{recipe_id}"), count: 1, ..Default::default() },
        }
    }

    #[test]
    fn the_shop_lists_recipes_moves_them_to_production_and_counts_up_to_20() {
        let font =
            BitmapFont::new("f".into(), Glyphs::parse(include_bytes!("../tests/fixtures/smallfont-e.gly")).unwrap());
        let mut view = RecipeShopView::new(|id| Some(format!("s{id}")));
        view.open(true, vec![entry(1), entry(2)], vec![]);
        assert_eq!(view.frame.title, format!("s{COMMON_TITLE_SYSSTRING}"));
        view.add_production(2, 500);
        assert_eq!((view.bottom.len(), view.bottom[0].recipe_id, view.bottom[0].cost), (1, 2, 500));
        assert_eq!(view.top.len(), 2);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font, 1.0);
        assert!(list.quads.iter().any(|q| q.texture == BACK && q.y == 20.0 && q.h == 381.0));
        assert!(list.quads.iter().any(|q| q.texture == "icon2" && q.y == 216.0));
        assert_eq!(view.hit(12, 49), Some(RecipeShopHit::Top(0)));
        assert_eq!(view.hit(12, 216), Some(RecipeShopHit::Bottom(0)));
        assert_eq!(view.hit(131, 195), Some(RecipeShopHit::Down));
        assert_eq!(view.hit(175, 375), Some(RecipeShopHit::Start));
        view.bottom_grid.selected = Some(0);
        view.remove_selected_production();
        assert!(view.bottom.is_empty());
    }
}
