#[path = "multisell_controls_generated.rs"]
mod controls;
#[path = "multisell_layout_generated.rs"]
mod layout;
#[path = "multisell_background_generated.rs"]
mod background;

use crate::button::TextButton;
use crate::color::TEXT;
use crate::draw::DrawList;
use crate::edit_box::{EditBox, FontMetrics};
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::item_grid::{GridEntry, GridHit, ItemGrid};
use crate::multisell_detail::ItemDetail;
use crate::pixel_scroll::PixelScroll;
use crate::scroll::{ScrollHit, line_down, line_up};
use crate::window::Rect;

pub const SIZE: (i32, i32) = layout::SIZE;

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    crate::mainwnd::default_position(viewport)
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    crate::mainwnd::initial_position(viewport)
}

pub const BACK: &str = "L2UI_CH3.MultiSellWnd.multicell_back";
pub const PLUS: &str = "L2UI_CH3.MultiSellWnd.Multisell_plusicon";
pub const LIST_RECT: Rect = layout::RECT_E8;
pub const PRODUCT_RECT: Rect = layout::RECT_EC;
pub const NEEDED_RECT: Rect = layout::RECT_F0;
pub const OK_RECT: Rect = controls::BUTTON_F8.rect_at((0, 0));
pub const CANCEL_RECT: Rect = controls::BUTTON_FC.rect_at((0, 0));
pub const QUANTITY_EDIT: crate::child_contract::ChildContract = controls::CHILD_F4;
pub const QUANTITY_RECT: Rect = layout::RECT_F4;
const ICON_BOX: &str = "L2ui_ch3.etc.iconbox";
const ROW_STEP: i32 = 36;
const NAME_X: i32 = 46;
const NAME_Y: i32 = 10;
const PRODUCT_NAME_WIDTH: i32 = 180;
const NEEDED_NAME_WIDTH: i32 = 170;
pub const GRID_COLS: usize = 6;
pub const GRID_ROWS: usize = 9;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultiSellHit {
    Close,
    Offer(usize),
    Scroll(ScrollHit),
    Ok,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OfferView {
    pub entry_id: i32,
    pub item: GridEntry,
    pub products: Vec<ItemDetail>,
    pub ingredients: Vec<GridEntry>,
    pub ingredient_labels: Vec<crate::item_name::ItemNameLabel>,
    pub stackable: bool,
}

#[derive(Debug, Clone)]
pub struct MultiSellView {
    pub title: String,
    pub list_label: String,
    pub information_label: String,
    pub needed_label: String,
    pub quantity_label: String,
    pub ok_label: String,
    pub cancel_label: String,
    pub offers: Vec<OfferView>,
    pub selected: Option<usize>,
    pub quantity: EditBox,
    pub symbols: l2_dat::symbolname::Symbols,
    pub product_scroll: PixelScroll,
    pub needed_scroll: PixelScroll,
    pub list_id: i32,
    pub grid: ItemGrid,
    pub ok: TextButton,
    pub cancel: TextButton,
    pub pressed: Option<MultiSellHit>,
    pub frame: FrameCtrl,
}

impl MultiSellView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id| lookup(id).unwrap_or_else(|| format!("sysstring {id}"));
        Self {
            title: text(136), list_label: text(562), information_label: text(564), needed_label: text(565),
            quantity_label: text(808), ok_label: text(140), cancel_label: text(141), offers: Vec::new(),
            selected: None, quantity: EditBox::from_contract(&QUANTITY_EDIT), symbols: Default::default(),
            product_scroll: PixelScroll::new((PRODUCT_RECT.w, PRODUCT_RECT.h)), needed_scroll: PixelScroll::new((NEEDED_RECT.w, NEEDED_RECT.h)), list_id: 0,
            grid: ItemGrid::new(LIST_RECT, GRID_COLS, GRID_ROWS),
            ok: controls::BUTTON_F8.build((0, 0), text),
            cancel: controls::BUTTON_FC.build((0, 0), text),
            pressed: None, frame: controls::FRAME_7C.build(text),
        }
    }

    pub fn clear(&mut self) {
        self.offers.clear();
        self.selected = None;
        self.list_id = 0;
        self.grid.clear();
        self.pressed = None;
        self.sync_scrolls();
    }

    pub fn set_offers(&mut self, list_id: i32, offers: Vec<OfferView>) {
        self.clear();
        self.list_id = list_id;
        self.offers = offers;
        self.grid.update_rows(self.offers.len());
        self.sync_scrolls();
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<MultiSellHit> {
        if controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) { return Some(MultiSellHit::Close); }
        match self.grid.hit(x, y) {
            Some(GridHit::Slot(index)) => Some(MultiSellHit::Offer(index)),
            Some(GridHit::Scroll(hit)) => Some(MultiSellHit::Scroll(hit)),
            None if controls::BUTTON_F8.hit(&self.ok, x, y).is_some() => Some(MultiSellHit::Ok),
            None if controls::BUTTON_FC.hit(&self.cancel, x, y).is_some() => Some(MultiSellHit::Cancel),
            _ => None,
        }
    }

    pub fn select(&mut self, index: usize) {
        if index < self.offers.len() {
            self.selected = Some(index);
            self.grid.selected = Some(index);
            self.quantity.set_text("1");
            self.quantity.enabled = self.offers[index].stackable;
            self.sync_scrolls();
        }
    }

    fn sync_scrolls(&mut self) {
        let (products, needed) = self.selected_offer().map_or((0, 0), |offer| (offer.products.len(), offer.ingredients.len()));
        self.product_scroll.set_content_height(if products > 1 { products as i32 * ROW_STEP + 5 } else { 0 });
        self.needed_scroll.set_content_height(if needed > 0 { needed as i32 * ROW_STEP + 5 } else { 0 });
    }

    pub fn bar_down(&mut self, x: i32, y: i32) -> bool {
        let product = self.product_scroll.mouse_down(x - PRODUCT_RECT.x, y - PRODUCT_RECT.y);
        product || self.needed_scroll.mouse_down(x - NEEDED_RECT.x, y - NEEDED_RECT.y)
    }

    pub fn bar_move(&mut self, y: i32) -> bool {
        let product = self.product_scroll.mouse_move(y - PRODUCT_RECT.y);
        let needed = self.needed_scroll.mouse_move(y - NEEDED_RECT.y);
        product || needed
    }

    pub fn bar_up(&mut self) {
        self.product_scroll.mouse_up();
        self.needed_scroll.mouse_up();
        self.grid.scroll.slider.mouse_up();
    }

    pub fn grid_slider_down(&mut self, x: i32, y: i32) -> bool {
        match self.grid.scroll_hit(x, y) {
            Some(ScrollHit::Slider(position)) => self.grid.scroll.slider.mouse_down(position),
            _ => false,
        }
    }

    pub fn grid_slider_move(&mut self, y: i32) -> bool {
        let Some(first) = self.grid.scroll.slider.mouse_move(y - self.grid.rect.y - crate::scroll::BUTTON_SIZE) else { return false };
        self.grid.set_first_row(first);
        true
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        if LIST_RECT.contains(x, y) {
            self.scroll(if delta > 0 { ScrollHit::Up } else { ScrollHit::Down });
        } else if PRODUCT_RECT.contains(x, y) {
            self.product_scroll.wheel(delta);
        } else if NEEDED_RECT.contains(x, y) {
            self.needed_scroll.wheel(delta);
        } else {
            return false;
        }
        true
    }

    pub fn quantity_mouse_down(&mut self, x: i32, y: i32, font: &BitmapFont) {
        if !QUANTITY_RECT.contains(x, y) {
            self.quantity_blur();
            return;
        }
        if !self.quantity.focused {
            self.quantity.focus();
        }
        let metrics = FontMetrics(font);
        self.quantity.mouse_down(x, &metrics);
        self.quantity.layout(&metrics);
    }

    pub fn quantity_mouse_move(&mut self, x: i32, font: &BitmapFont) {
        let metrics = FontMetrics(font);
        self.quantity.mouse_move(x, &metrics);
        self.quantity.layout(&metrics);
    }

    pub fn quantity_blur(&mut self) {
        if self.quantity.focused {
            self.quantity.kill_focus();
        }
    }

    pub fn quantity_shift(&mut self, pressed: bool) -> bool {
        if !self.quantity.focused {
            return false;
        }
        if pressed {
            self.quantity.key_down_shift();
        } else {
            self.quantity.key_up(crate::edit_box::KEY_SHIFT);
        }
        true
    }

    pub fn quantity_key(&mut self, key: crate::chat::InputKey, font: &BitmapFont) -> bool {
        if !self.quantity.focused {
            return false;
        }
        let metrics = FontMetrics(font);
        if self.quantity.input_key(key, &metrics) == crate::edit_box::EditInput::Escape {
            self.quantity_blur();
            self.quantity.layout(&metrics);
        }
        true
    }

    pub fn selected_offer(&self) -> Option<&OfferView> {
        self.selected.and_then(|index| self.offers.get(index))
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32, lookup: &impl Fn(u32) -> Option<String>) {
        let _ = background::BACKGROUND.paint(list, (x, y), alpha);
        controls::FRAME_7C.paint(&self.frame, list, (x, y), font, alpha);
        list.draw_text(x + 11, y + 32, TEXT, &self.list_label, font);
        list.draw_text(x + 267, y + 32, TEXT, &self.information_label, font);
        list.draw_text(x + 267, y + 203, TEXT, &self.needed_label, font);
        let entries: Vec<_> = self.offers.iter().map(|offer| offer.item.clone()).collect();
        self.grid.paint(list, x, y, &entries, font);
        if let Some(offer) = self.selected_offer() {
            let (px, py) = (x + PRODUCT_RECT.x, y + PRODUCT_RECT.y);
            let shift = self.product_scroll.offset;
            let start = list.quads.len();
            if let [detail] = offer.products.as_slice() {
                detail.paint(list, (px, py), font, lookup);
                detail.name_label().paint(list, (px + NAME_X, py + NAME_Y), PRODUCT_NAME_WIDTH, font, &self.symbols);
            } else {
                for (index, detail) in offer.products.iter().enumerate() {
                    let top = py + shift + index as i32 * ROW_STEP;
                    list.draw_texture(px + 10, top + 5, 34, 34, 0, 0, 34, 34, ICON_BOX, alpha);
                    list.draw_texture(px + 11, top + 6, 32, 32, 0, 0, 32, 32, &detail.icon, alpha);
                    detail.name_label().paint(list, (px + NAME_X, top + NAME_Y), PRODUCT_NAME_WIDTH, font, &self.symbols);
                }
            }
            list.clip_from(start, Rect::new(px, py, PRODUCT_RECT.w, PRODUCT_RECT.h));
            self.product_scroll.paint_bar(list, px, py);
            let (nx, ny) = (x + NEEDED_RECT.x, y + NEEDED_RECT.y);
            let shift = self.needed_scroll.offset;
            let start = list.quads.len();
            for (index, item) in offer.ingredients.iter().enumerate() {
                let top = ny + shift + index as i32 * ROW_STEP;
                list.draw_texture(nx + 10, top + 5, 34, 34, 0, 0, 34, 34, ICON_BOX, alpha);
                list.draw_texture(nx + 11, top + 6, 32, 32, 0, 0, 32, 32, item.icon(), alpha);
                if let Some(label) = offer.ingredient_labels.get(index) {
                    label.paint(list, (nx + NAME_X, top + NAME_Y), NEEDED_NAME_WIDTH, font, &self.symbols);
                }
                list.draw_text(nx + NAME_X, top + 25, TEXT, &format!("x {}", item.count), font);
            }
            list.clip_from(start, Rect::new(nx, ny, NEEDED_RECT.w, NEEDED_RECT.h));
            self.needed_scroll.paint_bar(list, nx, ny);
        }
        list.draw_text(x + 164 - font.measure(&self.quantity_label) as i32, y + 378, TEXT, &self.quantity_label, font);
        self.quantity.paint(list, (x, y), &FontMetrics(font), font);
        controls::BUTTON_F8.paint(&self.ok, list, (x, y), self.pressed == Some(MultiSellHit::Ok), font);
        controls::BUTTON_FC.paint(&self.cancel, list, (x, y), self.pressed == Some(MultiSellHit::Cancel), font);
    }

    pub fn scroll(&mut self, hit: ScrollHit) {
        let total = self.grid.total_rows(self.offers.len());
        let first = self.grid.scroll_row as i32;
        let next = match hit { ScrollHit::Up => line_up(first, GRID_ROWS as i32, total), ScrollHit::Down => line_down(first, GRID_ROWS as i32, total), _ => first };
        self.grid.set_first_row(next);
    }
}

pub fn grip() -> Rect { controls::FRAME_7C.rect_at((0, 0)) }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn show_item_100a2e90_prefills_one_and_locks_the_amount_unless_the_list_allows_it() {
        let mut view = MultiSellView::new(|_| None);
        let offer = |stackable| OfferView { stackable, ..Default::default() };
        view.set_offers(1, vec![offer(false), offer(true)]);
        view.quantity.set_text("9");
        view.select(0);
        assert_eq!((view.quantity.get_text().as_deref(), view.quantity.enabled), (Some("1"), false));
        view.select(1);
        assert_eq!((view.quantity.get_text().as_deref(), view.quantity.enabled), (Some("1"), true));
    }

    #[test]
    fn products_and_ingredients_list_their_names_in_36_pixel_rows_with_the_x_count() {
        let font = BitmapFont::new("font".into(), crate::font::Glyphs {
            tex_width: 256, tex_height: 16,
            glyphs: (32u8..=126).map(|c| (char::from(c), crate::font::GlyphRect { x: i32::from(c), width: 6, y: 0, height: 12 })).collect(),
        });
        let label = |name: &str| crate::item_name::ItemNameLabel { name: name.into(), type2: 2, ..Default::default() };
        let ingredient = |count| GridEntry { count, icon: "icon.a".into(), ..Default::default() };
        let detail = |name: &str| ItemDetail { icon: "icon.p".into(), name: name.into(), type2: 5, packet_type2: 5, ..Default::default() };
        let mut view = MultiSellView::new(|id| Some(id.to_string()));
        view.set_offers(1, vec![OfferView {
            products: vec![detail("First"), detail("Second")],
            ingredients: vec![ingredient(500), ingredient(7)],
            ingredient_labels: vec![label("Coin"), label("Ticket")],
            ..Default::default()
        }]);
        view.select(0);
        let mut list = DrawList { capture_text: true, ..Default::default() };
        view.paint(&mut list, 0, 0, &font, 1.0, &|_| None);
        let at = |text: &str| list.text_runs.iter().find(|run| run.text == text).map(|run| (run.rect[0] as i32, run.rect[1] as i32));
        assert_eq!(at("First"), Some((PRODUCT_RECT.x + 46, PRODUCT_RECT.y + 10)));
        assert_eq!(at("Second"), Some((PRODUCT_RECT.x + 46, PRODUCT_RECT.y + 46)));
        assert_eq!(at("Coin"), Some((NEEDED_RECT.x + 46, NEEDED_RECT.y + 10)));
        assert_eq!(at("x 500"), Some((NEEDED_RECT.x + 46, NEEDED_RECT.y + 25)));
        assert_eq!(at("Ticket"), Some((NEEDED_RECT.x + 46, NEEDED_RECT.y + 46)));
        assert_eq!(at("x 7"), Some((NEEDED_RECT.x + 46, NEEDED_RECT.y + 61)));
    }

    #[test]
    fn the_needed_and_product_lists_scroll_by_wheel_buttons_and_slider_inside_their_rects() {
        let font = BitmapFont::new("font".into(), crate::font::Glyphs {
            tex_width: 256, tex_height: 16,
            glyphs: (32u8..=126).map(|c| (char::from(c), crate::font::GlyphRect { x: i32::from(c), width: 6, y: 0, height: 12 })).collect(),
        });
        let ingredient = |count| GridEntry { count, icon: "icon.a".into(), ..Default::default() };
        let rows = (NEEDED_RECT.h / ROW_STEP + 4) as usize;
        let mut view = MultiSellView::new(|id| Some(id.to_string()));
        view.set_offers(1, vec![OfferView {
            ingredients: (0..rows).map(|i| ingredient(i as i32 + 1)).collect(),
            ingredient_labels: (0..rows).map(|i| crate::item_name::ItemNameLabel { name: format!("Item{i}"), ..Default::default() }).collect(),
            products: vec![ItemDetail::default()],
            ..Default::default()
        }]);
        view.select(0);
        assert_eq!(view.needed_scroll.minimum, NEEDED_RECT.h - (rows as i32 * ROW_STEP + 5));
        assert!(!view.wheel(5, 5, -120));
        assert!(view.wheel(NEEDED_RECT.x + 5, NEEDED_RECT.y + 5, -120));
        assert_eq!(view.needed_scroll.offset, -45);
        assert_eq!(view.product_scroll.offset, 0);
        let bar = crate::scroll::scroll_rects(NEEDED_RECT.w, NEEDED_RECT.h);
        assert!(view.bar_down(NEEDED_RECT.x + bar.down.x + 1, NEEDED_RECT.y + bar.down.y + 1));
        assert_eq!(view.needed_scroll.offset, -60);
        view.bar_up();
        let mut list = DrawList { capture_text: true, ..Default::default() };
        view.paint(&mut list, 0, 0, &font, 1.0, &|_| None);
        let at = |text: &str| list.text_runs.iter().find(|run| run.text == text).map(|run| run.rect[1] as i32);
        assert_eq!(at("x 5"), Some(NEEDED_RECT.y - 60 + 4 * ROW_STEP + 25).filter(|y| *y >= NEEDED_RECT.y));
        assert_eq!(at("x 1"), None);
    }

    #[test]
    fn quantity_edit_contract_matches_the_layout_and_constructor_state() {
        assert_eq!(QUANTITY_EDIT.rect, layout::RECT_F4);
        assert_eq!(QUANTITY_EDIT.kind, crate::child_contract::ChildKind::EditBox);
        assert_eq!(QUANTITY_EDIT.edit_kind(), Some(crate::child_contract::EDIT_KIND_TINTED));
        assert!(!QUANTITY_EDIT.masks_input());
        assert_eq!(QUANTITY_EDIT.vtable, 0x101a2730);
    }

    #[test]
    fn ncvipshopwnd_100a3bc0_uses_the_generated_children_for_paint_and_hits() {
        let mut view = MultiSellView::new(|id| Some(id.to_string()));
        for y in -2..24 {
            for x in 480..520 {
                assert_eq!(view.hit(x, y) == Some(MultiSellHit::Close), Rect::new(489, 3, 15, 15).contains(x, y));
            }
        }
        let font = BitmapFont::new("font".into(), crate::font::Glyphs {
            tex_width: 256, tex_height: 16,
            glyphs: (32u8..=126).map(|c| (char::from(c), crate::font::GlyphRect { x: i32::from(c), width: 6, y: 0, height: 12 })).collect(),
        });
        for (hit, button) in [(MultiSellHit::Ok, OK_RECT), (MultiSellHit::Cancel, CANCEL_RECT)] {
            assert_eq!(view.hit(button.x + 1, button.y + 1), Some(hit));
            view.pressed = Some(hit);
            for origin in [(0, 0), (31, 53)] {
                let mut list = DrawList::default();
                view.paint(&mut list, origin.0, origin.1, &font, 1.0, &|_| None);
                let back = list.quads.iter().find(|q| q.texture == BACK).unwrap();
                assert_eq!((back.x, back.y, back.w, back.h, back.uw, back.vh),
                    (origin.0 as f32, (origin.1 + 20) as f32, 512.0, 381.0, 512.0, 381.0));
                let pressed = list.quads.iter().find(|q| q.texture == controls::BUTTON_F8.down).unwrap();
                assert_eq!((pressed.x, pressed.y), ((origin.0 + button.x) as f32, (origin.1 + button.y) as f32));
            }
        }
    }
}
