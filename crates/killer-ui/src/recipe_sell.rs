#[path = "recipe_sell_layout_generated.rs"]
mod layout;

use crate::button::TextButton;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::item_grid::GridEntry;
use crate::recipe::{CAPTION_COLOR, LABEL_COLOR, VALUE_COLOR};

pub const SIZE: (i32, i32) = layout::SIZE;
use crate::scroll::{ScrollHit, ScrollWnd};
use crate::window::Rect;

pub const LIST: Rect = layout::RECT_E8;
pub const CLOSE: Rect = layout::RECT_EC;

#[derive(Clone, Debug)]
pub struct SellEntry {
    pub recipe_id: i32,
    pub product: GridEntry,
    pub cost: i32,
    pub success: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SellHit {
    Row(usize),
    Scroll(ScrollHit),
    Close,
}

#[derive(Clone, Debug)]
pub struct RecipeSellView {
    pub frame: FrameCtrl,
    pub close: TextButton,
    pub labels: [String; 5],
    pub entries: Vec<SellEntry>,
    pub selected: Option<usize>,
    pub first: usize,
    pub scroll: ScrollWnd,
    pub pressed: Option<SellHit>,
    pub manufacturer: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub adena: i32,
}

pub fn grouped(value: i32) -> String {
    let s = value.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            result.push(',');
        }
        result.push(c);
    }
    result
}

pub fn price_color(text: &str) -> Argb {
    let n = text.chars().filter(|c| *c != ',').count();
    if n < 5 { LABEL_COLOR } else { Argb([0xffff80ff, 0xffffff00, 0xff00ff00, 0xff00ffff][(n - 2) % 4]) }
}

impl RecipeSellView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self {
            frame: FrameCtrl::titled(&lookup(663).unwrap_or_default(), true),
            close: TextButton::new(
                CLOSE,
                crate::recipe::BUTTON_TEXTURES.0,
                crate::recipe::BUTTON_TEXTURES.1,
                &lookup(646).unwrap_or_default(),
            ),
            labels: [643, 644, 641, 642, 469].map(|id| lookup(id).unwrap_or_default()),
            entries: Vec::new(),
            selected: None,
            first: 0,
            scroll: ScrollWnd::new(LIST.w, LIST.h),
            pressed: None,
            manufacturer: -1,
            mp: 0,
            max_mp: 0,
            adena: 0,
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.selected = None;
        self.first = 0;
        self.pressed = None;
        self.scroll.up_pressed = false;
        self.scroll.down_pressed = false;
        self.scroll.slider.mouse_up();
        self.update_scroll();
    }

    pub fn count(&self) -> usize {
        self.entries.len().min(100)
    }

    pub fn update_scroll(&mut self) {
        self.scroll.slider.set_range(self.count() as i32, 5);
        self.scroll.slider.set_pos(self.first as i32);
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<SellHit> {
        if CLOSE.contains(x, y)
            || Rect::new(
                SIZE.0 - crate::frame::CLOSE_INSET,
                crate::frame::BUTTON_Y,
                crate::frame::BUTTON_SIZE,
                crate::frame::BUTTON_SIZE,
            )
            .contains(x, y)
        {
            return Some(SellHit::Close);
        }
        if let Some(hit) = self.scroll.hit(LIST.w, LIST.h, x - LIST.x, y - LIST.y) {
            return Some(SellHit::Scroll(hit));
        }
        if LIST.contains(x, y) {
            let row = (y - LIST.y - 4) / 47;
            if y >= LIST.y + 4 && (0..5).contains(&row) && self.first + (row as usize) < self.count() {
                return Some(SellHit::Row(self.first + row as usize));
            }
        }
        None
    }

    pub fn down(&mut self, x: i32, y: i32) -> Option<i32> {
        self.pressed = self.hit(x, y);
        match self.pressed {
            Some(SellHit::Row(i)) => {
                self.selected = Some(i);
                Some(self.entries[i].recipe_id)
            }
            Some(SellHit::Scroll(hit)) => {
                self.scroll.up_pressed = hit == ScrollHit::Up;
                self.scroll.down_pressed = hit == ScrollHit::Down;
                if let ScrollHit::Slider(y) = hit {
                    self.scroll.slider.mouse_down(y);
                }
                None
            }
            None if LIST.contains(x, y) => self.selected.and_then(|i| self.entries.get(i)).map(|entry| entry.recipe_id),
            _ => None,
        }
    }

    pub fn up(&mut self, over: Option<(i32, i32)>) -> bool {
        let hit = self.pressed.take();
        self.scroll.up_pressed = false;
        self.scroll.down_pressed = false;
        self.scroll.slider.mouse_up();
        if hit != over.and_then(|(x, y)| self.hit(x, y)) {
            return false;
        }
        match hit {
            Some(SellHit::Close) => return true,
            Some(SellHit::Scroll(ScrollHit::Up)) => {
                self.first = crate::scroll::line_up(self.first as i32, 5, self.count() as i32).max(0) as usize
            }
            Some(SellHit::Scroll(ScrollHit::Down)) => {
                self.first = crate::scroll::line_down(self.first as i32, 5, self.count() as i32).max(0) as usize
            }
            _ => {}
        }
        self.update_scroll();
        false
    }

    pub fn mouse_move(&mut self, y: i32) -> bool {
        if !self.scroll.slider.dragging {
            return false;
        }
        if let Some(first) = self.scroll.slider.mouse_move(y - LIST.y - 15) {
            self.first = first.max(0) as usize;
        }
        true
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        if !LIST.contains(x, y) {
            return false;
        }
        self.first = crate::inventory::item_wnd_wheel(self.first as i32, 5, self.count() as i32, delta).max(0) as usize;
        self.update_scroll();
        true
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        self.frame.paint(list, x, y, SIZE.0, SIZE.1, font, alpha);
        list.draw_texture(x, y + 20, 256, 381, 0, 0, 256, 381, "L2UI_CH3.RecipeWnd.Recipe3_Back", alpha);
        list.draw_text(x + 11, y + 32, LABEL_COLOR, &self.labels[0], font);
        list.draw_text(x + 18, y + 323, LABEL_COLOR, &self.labels[1], font);
        if self.max_mp != 0 {
            let width =
                crate::window_restore::x87_to_int(f64::from(self.mp.wrapping_mul(165)) / f64::from(self.max_mp))
                    .min(165);
            if width >= 1 {
                list.draw_texture(x + 75, y + 323, width, 12, 0, 0, 8, 12, crate::recipe::MP_BAR, alpha);
            }
        }
        let adena = grouped(self.adena);
        list.draw_text(x + 157 - font.measure(&adena) as i32, y + 341, LABEL_COLOR, &adena, font);
        let start = list.quads.len();
        for (i, entry) in self.entries.iter().enumerate().take(self.count()).skip(self.first).take(5) {
            let top = y + LIST.y + (i - self.first) as i32 * 50 + 4;
            if self.selected == Some(i) {
                list.draw_texture(x + LIST.x, top, LIST.w - 17, 46, 0, 0, 32, 40, "L2UI_CH3.etc.IconSelect2", 1.0);
            }
            list.draw_texture(x + LIST.x + 7, top + 7, 32, 32, 0, 0, 32, 32, entry.product.icon(), 1.0);
            let tx = x + LIST.x + 49;
            list.draw_text(tx, top + 3, LABEL_COLOR, &entry.product.name, font);
            let px = list.draw_text(tx, top + 17, Argb(0xffffff00), &self.labels[2], font);
            let px = list.draw_text(px, top + 17, LABEL_COLOR, crate::recipe::SEPARATOR, font);
            let price = grouped(entry.cost);
            let px = list.draw_text(px, top + 17, price_color(&price), &price, font);
            list.draw_text(px + 5, top + 17, Argb(0xffffff00), &self.labels[4], font);
            let px = list.draw_text(tx, top + 31, CAPTION_COLOR, &self.labels[3], font);
            let px = list.draw_text(px, top + 31, LABEL_COLOR, crate::recipe::SEPARATOR, font);
            list.draw_text(px, top + 31, VALUE_COLOR, &format!("{}%", entry.success), font);
        }
        list.clip_from(start,crate::window::Rect::new(x+LIST.x,y+LIST.y,LIST.w,LIST.h));
        self.scroll.paint(list, x + LIST.x, y + LIST.y, LIST.w, LIST.h);
        self.close.paint(list, x, y, self.pressed == Some(SellHit::Close), font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recipe_sell_100efb60_hit_pitch_differs_from_100f0530_paint_pitch() {
        let mut view = RecipeSellView::new(|_| None);
        for i in 0..105 {
            view.entries.push(SellEntry { recipe_id: i, product: GridEntry::default(), cost: 0, success: 100 });
        }
        view.update_scroll();
        assert_eq!(view.hit(20, 52), None);
        assert_eq!(view.hit(20, 53), Some(SellHit::Row(0)));
        assert_eq!(view.hit(20, 99), Some(SellHit::Row(0)));
        assert_eq!(view.hit(20, 100), Some(SellHit::Row(1)));
        view.wheel(20, 60, -120000);
        assert_eq!(view.first, 95);
        assert_eq!(view.hit(20, 287), Some(SellHit::Row(99)));
        assert_eq!(view.hit(20, 288), None);
        assert_eq!(view.down(20, 288), None);
        assert_eq!(view.down(20, 287), Some(99));
        assert_eq!(view.down(20, 288), Some(99));
        assert_eq!(view.down(20, 52), Some(99));
        assert_eq!(view.down(20, 304), None);
        assert_eq!(view.selected, Some(99));
        view.wheel(20, 60, 120000);
        assert_eq!(view.first, 0);
    }

    #[test]
    fn recipe_sell_1006b640_10034dd0_formats_and_colors_all_decimal_lengths() {
        for (n, text, color) in [
            (999, "999", 0xffdcdcdc),
            (1000, "1,000", 0xffdcdcdc),
            (10000, "10,000", 0xff00ffff),
            (100000, "100,000", 0xffff80ff),
            (1000000, "1,000,000", 0xffffff00),
            (10000000, "10,000,000", 0xff00ff00),
            (100000000, "100,000,000", 0xff00ffff),
            (2000000000, "2,000,000,000", 0xffff80ff),
        ] {
            assert_eq!(grouped(n), text);
            assert_eq!(price_color(text), Argb(color));
        }
    }
}
