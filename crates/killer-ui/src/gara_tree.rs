use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::scroll::ScrollWnd;
use crate::tree::{apply_scroll_sync, ctrl_content, scroll_sync};
use crate::window::Rect;

pub const TEXT_COLOR: Argb = Argb(0xFF0000FF);
pub const TEXT_GAP: i32 = 5;
pub const ROW_GAP: i32 = 5;
pub const CHILD_INDENT: i32 = 5;
pub const BRANCH_EXTRA_WIDTH: i32 = 16;
pub const TEXT_EXTRA_WIDTH: i32 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GaraItem {
    pub icon: String,
    pub icon_w: i32,
    pub icon_h: i32,
    pub text: String,
    pub expanded: bool,
    pub quest_id: i32,
    pub children: Vec<GaraItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placed {
    pub path: Vec<usize>,
    pub rect: Rect,
    pub created: (i32, i32),
}

impl GaraItem {
    pub fn width(&self, font: &BitmapFont) -> i32 {
        self.icon_w + font.measure(&self.text) as i32 + TEXT_EXTRA_WIDTH
    }

    pub fn height(&self) -> i32 {
        self.icon_h
    }

    fn draw(&self, list: &mut DrawList, font: &BitmapFont, origin: (i32, i32), at: (i32, i32), path: &mut Vec<usize>, placed: &mut Vec<Placed>) -> i32 {
        let (x, y) = at;
        let width = self.width(font) + if self.children.is_empty() { 0 } else { BRANCH_EXTRA_WIDTH };
        placed.push(Placed { path: path.clone(), rect: Rect::new(x, y, width, self.height()), created: (self.width(font), self.height()) });
        list.draw_texture(origin.0 + x, origin.1 + y, self.icon_w, self.icon_h, 0, 0, self.icon_w, self.icon_h, &self.icon, 1.0);
        list.draw_text(origin.0 + self.icon_w + x + TEXT_GAP, origin.1 + y, TEXT_COLOR, &self.text, font);
        let mut used = self.icon_h + ROW_GAP;
        if self.expanded {
            for (index, child) in self.children.iter().enumerate() {
                path.push(index);
                used += child.draw(list, font, origin, (self.icon_w + x + CHILD_INDENT, used + y), path, placed);
                path.pop();
            }
        }
        used
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GaraTree {
    pub root: Option<GaraItem>,
    pub content_height: i32,
    pub scroll_offset: i32,
    pub placed: Vec<Placed>,
    pub window: Option<(i32, i32)>,
}

impl GaraTree {
    pub fn clear(&mut self) {
        self.root = None;
    }

    pub fn add_root(&mut self, item: GaraItem) -> bool {
        if self.root.is_some() {
            return false;
        }
        self.root = Some(item);
        true
    }

    pub fn add_child(&mut self, parent: &[usize], item: GaraItem) -> bool {
        let Some(mut node) = self.root.as_mut() else { return false };
        for &index in parent {
            let Some(child) = node.children.get_mut(index) else { return false };
            node = child;
        }
        node.children.push(item);
        true
    }

    pub fn paint(&mut self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) -> i32 {
        self.placed.clear();
        let mut placed = Vec::new();
        let start = list.quads.len();
        self.content_height = match &self.root {
            Some(root) => root.draw(list, font, origin, (0, 0), &mut Vec::new(), &mut placed),
            None => 0,
        };
        self.placed = placed;
        if let Some((width, height)) = self.window {
            list.clip_from(start, Rect::new(origin.0, origin.1 + self.scroll_offset, width, height));
            self.window = Some((width, self.content_height));
        }
        self.content_height
    }

    pub fn item_at(&self, x: i32, y: i32) -> Option<&[usize]> {
        self.placed.iter().rev().find(|p| p.rect.contains(x, y)).map(|p| p.path.as_slice())
    }
}

pub const LINE_STEP: i32 = 15;
pub const WHEEL_NOTCH: i32 = 120;
pub const WHEEL_LINES: i32 = 3;

pub fn scroll_line_up(scroll: i32) -> i32 {
    if scroll < 0 { (scroll + LINE_STEP).min(0) } else { scroll }
}

pub fn scroll_line_down(scroll: i32, minimum: i32) -> i32 {
    if scroll > minimum { (scroll - LINE_STEP).max(minimum) } else { scroll }
}

pub fn scroll_wheel(scroll: i32, minimum: i32, delta: i32) -> i32 {
    let notches = delta / WHEEL_NOTCH;
    if notches > 0 {
        if scroll < 0 { (scroll.saturating_add(LINE_STEP.saturating_mul(notches).saturating_mul(WHEEL_LINES))).min(0) } else { scroll }
    } else if scroll > minimum {
        (scroll.saturating_add(LINE_STEP.saturating_mul(notches).saturating_mul(WHEEL_LINES))).max(minimum)
    } else {
        scroll
    }
}

pub fn scroll_bar_rounded(value: i32) -> i32 {
    value.saturating_add(LINE_STEP - 1) / LINE_STEP * LINE_STEP
}

#[derive(Debug, Clone)]
pub struct GaraTreeCtrl {
    pub tree: GaraTree,
    pub scroll: ScrollWnd,
    pub size: (i32, i32),
    pub scroll_y: i32,
    pub min_scroll_y: i32,
}

impl GaraTreeCtrl {
    pub fn new(size: (i32, i32)) -> GaraTreeCtrl {
        GaraTreeCtrl { tree: GaraTree { window: Some((size.0 - 16, size.1)), ..GaraTree::default() }, scroll: ScrollWnd::new(size.0, size.1), size, scroll_y: 0, min_scroll_y: 0 }
    }

    pub fn tree_width(&self) -> i32 {
        self.size.0 - 16
    }

    pub fn paint(&mut self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let at = (origin.0, origin.1 + self.scroll_y);
        self.tree.scroll_offset = -self.scroll_y;
        self.tree.paint(list, at, font);
        self.sync_range();
    }

    pub fn sync_range(&mut self) {
        let slider = &mut self.scroll.slider;
        let sync = scroll_sync(self.size.1, slider.thumb, slider.thumb_y, ctrl_content(self.tree.content_height));
        let offset = apply_scroll_sync(slider, sync);
        self.min_scroll_y = offset;
        if matches!(sync, crate::tree::ScrollSync::Collapse) {
            self.scroll_y = 0;
        }
    }

    pub fn arranged(&self) -> Vec<&Placed> {
        self.tree
            .placed
            .iter()
            .filter(|p| {
                let right = p.created.0;
                let bottom = p.created.1 + self.scroll_y;
                right >= 0 && right <= self.tree_width() && bottom >= 0 && bottom <= self.size.1
            })
            .collect()
    }

    pub fn scroll_up(&mut self) {
        if self.scroll_y < 0 {
            self.scroll_y = scroll_line_up(self.scroll_y);
            self.scroll.slider.set_pos(-self.scroll_y);
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll_y > self.min_scroll_y {
            self.scroll_y = scroll_line_down(self.scroll_y, self.min_scroll_y);
            self.scroll.slider.set_pos(-self.scroll_y);
        }
    }

    pub fn on_scroll_bar(&mut self, value: i32) {
        self.scroll_y = -value;
    }

    pub fn wheel(&mut self, delta: i32) {
        let moved = scroll_wheel(self.scroll_y, self.min_scroll_y, delta);
        if moved != self.scroll_y {
            self.scroll_y = moved;
            self.scroll.slider.set_pos(-self.scroll_y);
        }
    }

    pub fn reset(&mut self) {
        self.tree.clear();
        self.tree.scroll_offset = 0;
        self.scroll_y = 0;
        self.min_scroll_y = 0;
        self.scroll.slider.set_pos(0);
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

    fn item(text: &str, expanded: bool) -> GaraItem {
        GaraItem { icon: "i".into(), icon_w: 16, icon_h: 16, text: text.into(), expanded, ..Default::default() }
    }

    #[test]
    fn draw_item_100152b0_indents_children_by_icon_plus_5_and_returns_icon_height_plus_5_per_row() {
        let mut tree = GaraTree::default();
        assert!(tree.add_root(item("root", true)));
        assert!(!tree.add_root(item("again", true)));
        assert!(tree.add_child(&[], item("a", false)));
        assert!(tree.add_child(&[], item("b", true)));
        assert!(tree.add_child(&[1], item("c", false)));
        assert!(!tree.add_child(&[7], item("x", false)));
        let font = font();
        let mut list = DrawList::default();
        let height = tree.paint(&mut list, (10, 20), &font);
        assert_eq!(height, 21 * 4);
        let rects: Vec<_> = tree.placed.iter().map(|p| (p.path.clone(), p.rect.x, p.rect.y)).collect();
        assert_eq!(rects, vec![(vec![], 0, 0), (vec![0], 21, 21), (vec![1], 21, 42), (vec![1, 0], 42, 63)]);
        assert_eq!(tree.placed[0].rect.w, 16 + font.measure("root") as i32 + 2 + 16);
        assert_eq!(tree.placed[1].rect.w, 16 + font.measure("a") as i32 + 2);
        let icons: Vec<_> = list.quads.iter().filter(|q| q.texture == "i").map(|q| (q.x, q.y, q.w, q.h)).collect();
        assert_eq!(icons[1], (31.0, 41.0, 16.0, 16.0));
        let text: Vec<_> = list.quads.iter().filter(|q| q.texture == "f").take(1).map(|q| (q.x, q.y, q.color)).collect();
        assert_eq!(text[0].2, TEXT_COLOR.rgba());
        assert_eq!(tree.item_at(22, 22), Some(&[0usize][..]));
        tree.root.as_mut().unwrap().children[1].expanded = false;
        let mut list = DrawList::default();
        assert_eq!(tree.paint(&mut list, (0, 0), &font), 21 * 3);
        assert_eq!(tree.item_at(43, 64), None);
        tree.clear();
        assert_eq!(tree.paint(&mut DrawList::default(), (0, 0), &font), 0);
    }

    #[test]
    fn nc_gara_tree_on_paint_clips_to_the_tree_window_and_then_takes_the_content_height() {
        let mut ctrl = GaraTreeCtrl::new((60, 40));
        ctrl.tree.add_root(item("root", true));
        ctrl.tree.add_child(&[], item("a-long-child-name", false));
        let mut list = DrawList::default();
        ctrl.paint(&mut list, (5, 7), &font());
        assert_eq!(ctrl.tree.window, Some((44, 42)));
        assert!(list.quads.iter().all(|q| q.x + q.w <= 5.0 + 44.0 + 0.01));
        let mut list = DrawList::default();
        ctrl.scroll_y = -10;
        ctrl.paint(&mut list, (5, 7), &font());
        assert!(list.quads.iter().all(|q| q.y >= 7.0 - 0.01 && q.y + q.h <= 7.0 + 42.0 + 0.01));
    }

    #[test]
    fn tree_ctrl_100155c0_shows_an_item_by_its_creation_size_and_the_scroll_without_its_own_position() {
        let mut ctrl = GaraTreeCtrl::new((100, 60));
        ctrl.tree.add_root(item("root", true));
        for n in 0..8 {
            ctrl.tree.add_child(&[], item(&format!("c{n}"), false));
        }
        ctrl.paint(&mut DrawList::default(), (0, 0), &font());
        assert_eq!(ctrl.arranged().len(), 9);
        ctrl.scroll_y = -40;
        assert_eq!(ctrl.arranged().len(), 0);
        ctrl.scroll_y = -16;
        assert_eq!(ctrl.arranged().len(), 9);
        let root = &ctrl.tree.placed[0];
        assert_eq!(root.created.0, root.rect.w - BRANCH_EXTRA_WIDTH);
    }

    #[test]
    fn the_scroll_steps_are_shared_by_quest_recipe_and_ssq_trees() {
        assert_eq!((scroll_line_up(-40), scroll_line_up(-10), scroll_line_up(0)), (-25, 0, 0));
        assert_eq!((scroll_line_down(0, -40), scroll_line_down(-35, -40), scroll_line_down(-40, -40)), (-15, -40, -40));
        assert_eq!((scroll_wheel(0, -200, -120), scroll_wheel(-100, -200, 240), scroll_wheel(0, -200, 120), scroll_wheel(0, -200, -60)), (-45, -10, 0, 0));
        assert_eq!((scroll_wheel(-190, -200, -120), scroll_bar_rounded(31), scroll_bar_rounded(30)), (-200, 45, 30));
    }

    #[test]
    fn tree_ctrl_100154c0_10015580_100156d0_scroll_by_15_and_three_lines_per_notch() {
        let mut ctrl = GaraTreeCtrl::new((100, 40));
        ctrl.tree.add_root(item("root", true));
        for n in 0..6 {
            ctrl.tree.add_child(&[], item(&format!("c{n}"), false));
        }
        let font = font();
        ctrl.paint(&mut DrawList::default(), (0, 0), &font);
        assert_eq!(ctrl.tree.content_height, 21 * 7);
        assert_eq!(ctrl.min_scroll_y, 40 - (21 * 7 + 21));
        ctrl.scroll_down();
        assert_eq!(ctrl.scroll_y, -15);
        ctrl.scroll_up();
        ctrl.scroll_up();
        assert_eq!(ctrl.scroll_y, 0);
        ctrl.wheel(-120);
        assert_eq!(ctrl.scroll_y, -45);
        ctrl.wheel(120 * 5);
        assert_eq!(ctrl.scroll_y, 0);
        ctrl.wheel(-120 * 100);
        assert_eq!(ctrl.scroll_y, ctrl.min_scroll_y);
        ctrl.on_scroll_bar(30);
        assert_eq!(ctrl.scroll_y, -30);
        ctrl.reset();
        assert_eq!((ctrl.scroll_y, ctrl.tree.root.is_none()), (0, true));
    }
}
