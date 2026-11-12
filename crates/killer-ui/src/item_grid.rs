use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::inventory::{BLACK_TEXTURE, ITEM_CLICK, OUTLINE_SIZE, SLOT, grid_slot_at, grid_slot_pos_cols};
use crate::scroll::{ScrollHit, ScrollWnd, clamp_first_row, rows_for};
use crate::window::Rect;

pub const COUNT_COLOR: Argb = Argb(0xFFFF0000);
pub const ICON_BOX: &str = "L2ui_ch3.etc.iconbox";

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GridEntry {
    pub object_id: i32,
    pub item_id: i32,
    pub count: i32,
    pub enchant: i32,
    pub icon: String,
    pub name: String,
    pub add_name: String,
    pub description: String,
    pub weight: i32,
}

impl GridEntry {
    pub fn icon(&self) -> &str {
        if self.icon.is_empty() { BLACK_TEXTURE } else { &self.icon }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridHit {
    Slot(usize),
    Scroll(ScrollHit),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemGrid {
    pub rect: Rect,
    pub cols: usize,
    pub rows: usize,
    pub scroll_row: usize,
    pub scroll: ScrollWnd,
    pub selected: Option<usize>,
    pub show_count: bool,
    pub outline: Option<&'static str>,
    pub click_first: bool,
    pub empty_texture: Option<&'static str>,
}

impl ItemGrid {
    pub fn new(rect: Rect, cols: usize, rows: usize) -> ItemGrid {
        ItemGrid { rect, cols, rows, scroll: ScrollWnd::new(rect.w, rect.h), ..Default::default() }
    }

    pub fn visible(&self) -> usize {
        self.cols * self.rows
    }

    pub fn total_rows(&self, count: usize) -> i32 {
        rows_for(count, self.cols)
    }

    pub fn update_rows(&mut self, count: usize) {
        let total = self.total_rows(count);
        let visible = self.rows as i32;
        self.scroll_row = clamp_first_row(self.scroll_row as i32, visible, total).max(0) as usize;
        self.scroll.slider.set_range(total, visible);
        self.scroll.slider.set_pos(self.scroll_row as i32);
    }

    pub fn set_first_row(&mut self, first: i32) {
        self.scroll_row = first.max(0) as usize;
        self.scroll.slider.set_pos(first);
    }

    pub fn clear(&mut self) {
        self.scroll_row = 0;
        self.selected = None;
        self.scroll.slider.set_range(0, self.rows as i32);
        self.scroll.slider.set_pos(0);
    }

    pub fn scroll_hit(&self, x: i32, y: i32) -> Option<ScrollHit> {
        self.scroll.hit(self.rect.w, self.rect.h, x - self.rect.x, y - self.rect.y)
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<GridHit> {
        if !self.rect.contains(x, y) {
            return None;
        }
        if let Some(hit) = self.scroll_hit(x, y) {
            return Some(GridHit::Scroll(hit));
        }
        grid_slot_at(x - self.rect.x, y - self.rect.y, self.cols, self.rows, self.scroll_row).map(GridHit::Slot)
    }

    pub fn paint_cover(&self, list: &mut DrawList, x: i32, y: i32, entries: usize, covered: impl Fn(usize) -> bool, texture: &str) {
        let (gx, gy) = (x + self.rect.x, y + self.rect.y);
        let first = self.scroll_row * self.cols;
        for index in (first..entries).take(self.visible()).filter(|index| covered(*index)) {
            let (sx, sy) = grid_slot_pos_cols(index - first, self.cols);
            list.draw_texture(gx + sx, gy + sy, SLOT, SLOT, 0, 0, SLOT, SLOT, texture, 1.0);
        }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, entries: &[GridEntry], font: &BitmapFont) {
        let (gx, gy) = (x + self.rect.x, y + self.rect.y);
        let first = self.scroll_row * self.cols;
        for (index, entry) in entries.iter().enumerate().skip(first).take(self.visible()) {
            let (sx, sy) = grid_slot_pos_cols(index - first, self.cols);
            let (sx, sy) = (gx + sx, gy + sy);
            let selected = self.selected == Some(index);
            let click = |list: &mut DrawList| {
                list.draw_texture(sx - 1, sy - 1, OUTLINE_SIZE, OUTLINE_SIZE, 0, 0, OUTLINE_SIZE, OUTLINE_SIZE, ITEM_CLICK, 1.0);
            };
            let icon = if entry.icon.is_empty() { self.empty_texture.unwrap_or(BLACK_TEXTURE) } else { entry.icon() };
            match self.outline {
                Some(outline) => {
                    if selected && self.click_first {
                        click(list);
                    }
                    list.draw_texture(sx - 1, sy - 1, OUTLINE_SIZE, OUTLINE_SIZE, 0, 0, OUTLINE_SIZE, OUTLINE_SIZE, outline, 1.0);
                    list.draw_texture(sx, sy, SLOT, SLOT, 0, 0, SLOT, SLOT, icon, 1.0);
                    if selected && !self.click_first {
                        click(list);
                    }
                }
                None => {
                    if selected {
                        click(list);
                    }
                    list.draw_texture(sx, sy, SLOT, SLOT, 0, 0, SLOT, SLOT, icon, 1.0);
                }
            }
            if self.show_count && entry.count > 0 {
                list.draw_text(sx, sy, COUNT_COLOR, &entry.count.to_string(), font);
            }
        }
        self.scroll.paint(list, gx, gy, self.rect.w, self.rect.h);
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

    fn entries(n: usize) -> Vec<GridEntry> {
        (0..n).map(|i| GridEntry { object_id: 100 + i as i32, count: i as i32, icon: format!("icon{i}"), ..Default::default() }).collect()
    }

    #[test]
    fn the_grid_hit_tests_slots_by_the_ncitemwnd_steps_and_scrolls_by_rows() {
        let mut grid = ItemGrid::new(Rect::new(9, 48, 239, 139), 6, 4);
        assert_eq!(grid.hit(9 + 1, 48 + 1), Some(GridHit::Slot(0)));
        assert_eq!(grid.hit(9 + 1 + 37 * 2 + 3, 48 + 1 + 35 + 3), Some(GridHit::Slot(8)));
        assert_eq!(grid.hit(9 + 230, 48 + 5), Some(GridHit::Scroll(ScrollHit::Up)));
        assert_eq!(grid.hit(5, 5), None);
        grid.update_rows(40);
        assert!(grid.scroll.slider.shown);
        grid.set_first_row(2);
        assert_eq!(grid.hit(9 + 1, 48 + 1), Some(GridHit::Slot(12)));
        grid.update_rows(24);
        assert_eq!(grid.scroll_row, 0);
        assert!(!grid.scroll.slider.shown);
        grid.set_first_row(3);
        grid.clear();
        assert_eq!((grid.scroll_row, grid.selected), (0, None));
    }

    #[test]
    fn the_grid_paints_icons_outlines_the_selection_and_counts_only_when_asked() {
        let mut grid = ItemGrid::new(Rect::new(9, 215, 239, 104), 6, 3);
        grid.selected = Some(1);
        grid.show_count = true;
        let font = font();
        let mut list = DrawList::default();
        grid.paint(&mut list, 0, 20, &entries(3), &font);
        let icon = list.quads.iter().find(|q| q.texture == "icon1").unwrap();
        assert_eq!((icon.x, icon.y, icon.w), (9.0 + 1.0 + 37.0, 20.0 + 215.0 + 1.0, 32.0));
        let click = list.quads.iter().position(|q| q.texture == ITEM_CLICK).unwrap();
        assert_eq!((list.quads[click].x, list.quads[click].y, list.quads[click].w), (icon.x - 1.0, icon.y - 1.0, 34.0));
        assert_eq!(list.quads[click + 1].texture, "icon1");
        assert!(list.quads.iter().all(|q| q.texture != ICON_BOX));
        let counts: Vec<_> = list.quads.iter().filter(|q| q.texture == "f").collect();
        assert_eq!(counts.len(), 2);
        assert_eq!(counts[0].color, COUNT_COLOR.rgba());
        assert_eq!((counts[0].x, counts[0].y), (icon.x, icon.y));
        grid.show_count = false;
        grid.outline = Some(ICON_BOX);
        let mut list = DrawList::default();
        grid.paint(&mut list, 0, 0, &entries(3), &font);
        assert!(list.quads.iter().all(|q| q.texture != "f"));
        assert_eq!(list.quads.iter().filter(|q| q.texture == ICON_BOX).count(), 3);
        let icon = list.quads.iter().position(|q| q.texture == "icon1").unwrap();
        assert_eq!((list.quads[icon - 1].texture.as_str(), list.quads[icon + 1].texture.as_str()), (ICON_BOX, ITEM_CLICK));
    }

    #[test]
    fn the_grid_uses_the_c4_empty_slot_texture_when_configured() {
        let mut grid = ItemGrid::new(Rect::new(9, 48, 239, 139), 6, 4);
        grid.empty_texture = Some("NWindow.ChatBack");
        let mut list = DrawList::default();
        grid.paint(&mut list, 0, 0, &[GridEntry::default()], &font());
        assert_eq!(list.quads[0].texture, "NWindow.ChatBack");
    }
}
