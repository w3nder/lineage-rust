use crate::draw::DrawList;
use crate::scroll::SliderBar;
use crate::window::Rect;

pub const CONTENT_MARGIN: i32 = 0x15;
pub const SCROLL_SLACK: i32 = 0x1e;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tree {
    pub rect: Rect,
    pub has_children: bool,
    pub node_state: i32,
    pub scroll: i32,
    pub content_height: i32,
    pub holder_height: i32,
}

impl Tree {
    pub fn new(rect: Rect) -> Tree {
        Tree { rect, has_children: false, node_state: 0, scroll: 0, content_height: 0, holder_height: 0 }
    }

    pub fn paint(
        &mut self,
        list: &mut DrawList,
        origin: (i32, i32),
        paint_nodes: impl FnOnce(&mut DrawList, (i32, i32), i32) -> i32,
    ) -> Option<(i32, i32)> {
        if !self.has_children {
            return None;
        }
        let x = origin.0 + self.rect.x;
        let y = origin.1 + self.rect.y;
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        self.content_height = paint_nodes(&mut body, (x, y), self.node_state);
        list.append_clipped(body, Rect::new(x, y + self.scroll, self.rect.w, self.holder_height));
        self.rect.h = self.content_height;
        Some((self.rect.w, self.content_height))
    }
}

pub const SELECT_TEX: &str = "L2UI_CH3.ListCtrl.TextSelect";
pub const SELECT_TEX2: &str = "L2UI_CH3.ListCtrl.TextSelect2";
pub const TV_TEXT_COLOR: crate::color::Argb = crate::color::Argb(0xFFDCDCDC);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TvBox {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl TvBox {
    fn hit(&self, x: i32, y: i32) -> bool {
        self.x <= x && self.w + self.x > x && self.y <= y && self.h + self.y > y
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TvData {
    pub text: String,
    pub icon_box: TvBox,
    pub icon2_box: TvBox,
    pub text_box: TvBox,
    pub icon2_tex: Option<String>,
    pub expanded_tex: Option<String>,
    pub collapsed_tex: Option<String>,
    pub text_offset_x: i32,
    pub collapsed_offset_y: i32,
    pub center_on_icon2: bool,
}

impl Default for TvData {
    fn default() -> TvData {
        TvData {
            text: String::new(),
            icon_box: TvBox::default(),
            icon2_box: TvBox::default(),
            text_box: TvBox::default(),
            icon2_tex: None,
            expanded_tex: None,
            collapsed_tex: None,
            text_offset_x: 0,
            collapsed_offset_y: 0,
            center_on_icon2: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TvItem {
    pub data: TvData,
    pub children: Vec<TvItem>,
    pub expanded: bool,
    pub selected: bool,
    pub visible: bool,
}

impl TvItem {
    pub fn new(data: TvData) -> TvItem {
        TvItem { data, children: Vec::new(), expanded: false, selected: false, visible: true }
    }

    pub fn expand(&mut self) {
        self.expanded = true;
        for child in &mut self.children {
            child.visible = true;
        }
    }

    pub fn collapse(&mut self) {
        self.expanded = false;
        for child in &mut self.children {
            child.visible = false;
        }
    }

    fn find_at(&self, x: i32, y: i32, path: &mut Vec<usize>) -> bool {
        if !self.visible {
            return false;
        }
        if self.data.icon_box.hit(x, y) || self.data.text_box.hit(x, y) {
            return true;
        }
        for (index, child) in self.children.iter().enumerate() {
            path.push(index);
            if child.find_at(x, y, path) {
                return true;
            }
            path.pop();
        }
        false
    }

    fn at(&self, path: &[usize]) -> Option<&TvItem> {
        path.iter().try_fold(self, |item, &index| item.children.get(index))
    }

    fn at_mut(&mut self, path: &[usize]) -> Option<&mut TvItem> {
        path.iter().try_fold(self, |item, &index| item.children.get_mut(index))
    }

    fn draw(&mut self, list: &mut DrawList, font: &crate::font::BitmapFont, origin: (i32, i32), x: i32, y: i32) -> i32 {
        let data = &mut self.data;
        let icon = data.icon_box;
        if self.expanded {
            if let Some(texture) = &data.expanded_tex {
                list.draw_texture(origin.0 + x, origin.1 + y, icon.w, icon.h, 0, 0, icon.w, icon.h, texture, 1.0);
                data.icon_box.x = x;
                data.icon_box.y = y;
            }
        } else if let Some(texture) = &data.collapsed_tex {
            list.draw_texture(origin.0 + x, origin.1 + data.collapsed_offset_y + y, icon.w, icon.h, 0, 0, icon.w, icon.h, texture, 1.0);
            data.icon_box.x = x;
            data.icon_box.y = y;
        }
        let text_box = data.text_box;
        if self.selected {
            list.draw_texture(origin.0 + icon.w + data.text_offset_x + x, origin.1 + y, text_box.w - 0x31, text_box.h, 0, 0, 0x10, 0xD, SELECT_TEX, 1.0);
            list.draw_texture(origin.0 + x + icon.w + text_box.w + data.text_offset_x - 0x31, origin.1 + y, 0x20, text_box.h, 0, 0, 0x20, 0xD, SELECT_TEX2, 1.0);
        }
        if let Some(texture) = &data.icon2_tex {
            let icon2 = data.icon2_box;
            list.draw_texture(origin.0 + icon.w + data.text_offset_x + x, origin.1 + y, icon2.w, icon2.h, 0, 0, icon2.w, icon2.h, texture, 1.0);
            data.icon2_box.x = icon.w + data.text_offset_x + x;
            data.icon2_box.y = y;
        }
        data.text_box.x = icon.w + data.text_offset_x + x;
        data.text_box.y = y;
        let char_height = font.line_height as i32;
        let reference = if data.center_on_icon2 { data.icon2_box.h } else { icon.h };
        let text_y = (reference - char_height) / 2 + y + 1;
        list.draw_text(origin.0 + data.icon2_box.w + icon.w + data.text_offset_x + x + 3, origin.1 + text_y, TV_TEXT_COLOR, &data.text, font);
        let mut used = data.icon2_box.h + 4;
        if self.expanded {
            for child in &mut self.children {
                used += child.draw(list, font, origin, icon.w + x + 3, y + used);
            }
        } else {
            for child in &mut self.children {
                child.visible = false;
            }
        }
        used
    }
}

#[derive(Debug, Clone, Default)]
pub struct TvTree {
    pub root: Option<TvItem>,
    pub selected: Option<Vec<usize>>,
    pub hover: Option<Vec<usize>>,
    pub content_height: i32,
    pub scroll_offset: i32,
}

impl TvTree {
    pub fn add_item(&mut self, data: TvData, parent: Option<&[usize]>) -> Option<Vec<usize>> {
        let Some(parent) = parent else {
            if self.root.is_some() {
                return None;
            }
            self.root = Some(TvItem::new(data));
            return Some(Vec::new());
        };
        let node = self.root.as_mut()?.at_mut(parent)?;
        node.children.push(TvItem::new(data));
        let mut path = parent.to_vec();
        path.push(node.children.len() - 1);
        Some(path)
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.selected = None;
        self.hover = None;
    }

    pub fn item(&self, path: &[usize]) -> Option<&TvItem> {
        self.root.as_ref()?.at(path)
    }

    pub fn find_item_at(&self, x: i32, y: i32) -> Option<Vec<usize>> {
        let mut path = Vec::new();
        self.root.as_ref()?.find_at(x as u16 as i32, y as u16 as i32, &mut path).then_some(path)
    }

    pub fn mouse_down(&mut self, rx: i32, ry: i32) {
        let Some(hit) = self.find_item_at(rx, ry) else { return };
        if self.selected.as_deref() != Some(hit.as_slice()) {
            if let Some(old) = self.selected.take()
                && let Some(item) = self.root.as_mut().and_then(|root| root.at_mut(&old))
            {
                item.selected = false;
            }
            if let Some(item) = self.root.as_mut().and_then(|root| root.at_mut(&hit)) {
                item.selected = true;
            }
            self.selected = Some(hit.clone());
        }
        if let Some(item) = self.root.as_mut().and_then(|root| root.at_mut(&hit)) {
            if item.expanded {
                item.collapse();
            } else {
                item.expand();
            }
        }
    }

    pub fn mouse_move(&mut self, rx: i32, ry: i32, holder: (i32, i32)) {
        self.hover = if rx > 0 && rx <= holder.0 && ry > self.scroll_offset && ry < self.scroll_offset + holder.1 {
            self.find_item_at(rx, ry)
        } else {
            None
        };
    }

    pub fn paint(&mut self, list: &mut DrawList, origin: (i32, i32), font: &crate::font::BitmapFont, width: i32, holder_height: i32) -> i32 {
        let start = list.quads.len();
        let height = self.root.as_mut().map_or(0, |root| root.draw(list, font, origin, 0, 0));
        list.clip_from(start, Rect::new(origin.0, origin.1 + self.scroll_offset, width, holder_height));
        self.content_height = height;
        height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollSync {
    Collapse,
    Range { offset: i32, range: i32, visible: i32 },
}

pub fn scroll_sync(viewport_height: i32, thumb: i32, thumb_y: i32, content: i32) -> ScrollSync {
    if thumb + thumb_y > viewport_height - SCROLL_SLACK {
        ScrollSync::Collapse
    } else if content > viewport_height {
        ScrollSync::Range { offset: viewport_height - content, range: content, visible: viewport_height }
    } else {
        ScrollSync::Range { offset: 0, range: content, visible: viewport_height }
    }
}

pub fn apply_scroll_sync(bar: &mut SliderBar, sync: ScrollSync) -> i32 {
    match sync {
        ScrollSync::Collapse => {
            bar.set_pos(0);
            0
        }
        ScrollSync::Range { offset, range, visible } => {
            bar.set_range(range, visible);
            offset
        }
    }
}

pub fn ctrl_content(parent_content_height: i32) -> i32 {
    parent_content_height + CONTENT_MARGIN
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_10032f80_paints_nodes_clipped_at_the_scrolled_origin_and_resizes_to_the_content() {
        let mut tree = Tree::new(Rect::new(10, 20, 100, 50));
        tree.has_children = true;
        tree.holder_height = 50;
        tree.node_state = 3;
        tree.scroll = 4;
        let mut list = DrawList::default();
        let mut seen = None;
        let resized = tree.paint(&mut list, (1, 2), |body, at, state| {
            seen = Some((at, state));
            body.draw_texture(11, 22, 100, 80, 0, 0, 100, 80, "n", 1.0);
            80
        });
        assert_eq!(seen, Some(((11, 22), 3)));
        assert_eq!(resized, Some((100, 80)));
        assert_eq!(tree.content_height, 80);
        assert_eq!(tree.rect.h, 80);
        assert_eq!((list.quads[0].y, list.quads[0].h), (26.0, 50.0));
        tree.holder_height = 30;
        let mut list = DrawList::default();
        tree.paint(&mut list, (1, 2), |body, _, _| {
            body.draw_texture(11, 22, 100, 80, 0, 0, 100, 80, "n", 1.0);
            80
        });
        assert_eq!(list.quads[0].h, 30.0);
    }

    #[test]
    fn tree_10032f80_does_nothing_without_children() {
        let mut tree = Tree::new(Rect::new(0, 0, 10, 10));
        let mut list = DrawList::default();
        assert_eq!(tree.paint(&mut list, (0, 0), |_, _, _| 99), None);
        assert!(list.quads.is_empty());
        assert_eq!(tree.rect.h, 10);
    }

    #[test]
    fn tree_ctrl_1002b7f0_collapses_when_the_items_overrun_the_viewport() {
        assert_eq!(scroll_sync(100, 60, 11, 40), ScrollSync::Collapse);
        assert_eq!(scroll_sync(100, 60, 10, 140), ScrollSync::Range { offset: -40, range: 140, visible: 100 });
        assert_eq!(scroll_sync(100, 10, 10, 40), ScrollSync::Range { offset: 0, range: 40, visible: 100 });
        assert_eq!(scroll_sync(100, 0, 0, 0), ScrollSync::Range { offset: 0, range: 0, visible: 100 });
        assert_eq!(ctrl_content(40), 61);
    }

    #[test]
    fn tree_ctrl_1002b7f0_drives_the_slider_through_1002c500_and_1002c510() {
        let mut bar = SliderBar::new(100);
        bar.pos = 5;
        assert_eq!(apply_scroll_sync(&mut bar, scroll_sync(100, 0, 0, 140)), -40);
        assert_eq!((bar.total, bar.visible), (140, 100));
        bar.total = 50;
        bar.pos = 7;
        assert_eq!(apply_scroll_sync(&mut bar, ScrollSync::Collapse), 0);
        assert_eq!(bar.pos, 0);
    }

    fn font() -> crate::font::BitmapFont {
        use crate::font::{Glyphs, gly};
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        crate::font::BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn data(text: &str) -> TvData {
        TvData {
            text: text.into(),
            icon_box: TvBox { x: 0, y: 0, w: 12, h: 14 },
            icon2_box: TvBox { x: 0, y: 0, w: 0, h: 14 },
            text_box: TvBox { x: 0, y: 0, w: 60, h: 14 },
            expanded_tex: Some("open".into()),
            collapsed_tex: Some("closed".into()),
            collapsed_offset_y: 2,
            ..TvData::default()
        }
    }

    #[test]
    fn nctree_add_item_keeps_one_root_and_appends_children() {
        let mut tree = TvTree::default();
        assert_eq!(tree.add_item(data("r"), None), Some(vec![]));
        assert_eq!(tree.add_item(data("again"), None), None);
        assert_eq!(tree.add_item(data("a"), Some(&[])), Some(vec![0]));
        assert_eq!(tree.add_item(data("b"), Some(&[])), Some(vec![1]));
        assert_eq!(tree.add_item(data("c"), Some(&[1])), Some(vec![1, 0]));
        assert_eq!(tree.add_item(data("x"), Some(&[5])), None);
        tree.clear();
        assert!(tree.root.is_none() && tree.selected.is_none());
    }

    #[test]
    fn nctree_draw_item_10033880_places_icons_text_and_hides_children_of_closed_items() {
        let mut tree = TvTree::default();
        tree.add_item(data("root"), None);
        tree.add_item(data("a"), Some(&[]));
        let font = font();
        let mut list = DrawList::default();
        assert_eq!(tree.paint(&mut list, (10, 20), &font, 80, 40), 18);
        assert!(!tree.item(&[0]).unwrap().visible);
        let closed: Vec<_> = list.quads.iter().filter(|q| q.texture == "closed").map(|q| (q.x, q.y)).collect();
        assert_eq!(closed, vec![(10.0, 22.0)]);
        let root = &tree.item(&[]).unwrap().data;
        assert_eq!((root.icon_box.x, root.icon_box.y, root.text_box.x, root.text_box.y), (0, 0, 12, 0));
        tree.mouse_down(13, 3);
        assert_eq!(tree.selected, Some(vec![]));
        assert!(tree.item(&[]).unwrap().expanded && tree.item(&[0]).unwrap().visible);
        let mut list = DrawList::default();
        assert_eq!(tree.paint(&mut list, (10, 20), &font, 80, 40), 36);
        assert!(list.quads.iter().any(|q| q.texture == "open" && q.x == 10.0 && q.y == 20.0));
        let sel: Vec<_> = list.quads.iter().filter(|q| q.texture == SELECT_TEX).map(|q| (q.x, q.w, q.uw, q.vh)).collect();
        assert_eq!(sel, vec![(22.0, 60.0 - 49.0, 16.0, 13.0)]);
        let second: Vec<_> = list.quads.iter().filter(|q| q.texture == SELECT_TEX2).map(|q| (q.x, q.w)).collect();
        assert_eq!(second, vec![(10.0 + 12.0 + 60.0 - 49.0, 32.0)]);
        let child = &tree.item(&[0]).unwrap().data;
        assert_eq!((child.text_box.x, child.text_box.y), (12 + 3 + 12, 18));
    }

    #[test]
    fn nctree_click_selects_moves_the_mark_and_toggles_the_hit_item_only() {
        let mut tree = TvTree::default();
        tree.add_item(data("root"), None);
        tree.add_item(data("a"), Some(&[]));
        let font = font();
        tree.mouse_down(1, 1);
        tree.paint(&mut DrawList::default(), (0, 0), &font, 80, 40);
        tree.mouse_down(1, 1);
        assert!(!tree.item(&[]).unwrap().expanded);
        tree.mouse_down(1, 1);
        tree.paint(&mut DrawList::default(), (0, 0), &font, 80, 40);
        tree.mouse_down(20, 20);
        assert_eq!(tree.selected, Some(vec![0]));
        assert!(!tree.item(&[]).unwrap().selected && tree.item(&[0]).unwrap().selected);
        tree.mouse_down(500, 500);
        assert_eq!(tree.selected, Some(vec![0]));
        assert_eq!(tree.find_item_at(-1, 0), None);
    }

    #[test]
    fn nctree_mouse_move_hovers_only_inside_the_holder_area() {
        let mut tree = TvTree::default();
        tree.add_item(data("root"), None);
        tree.paint(&mut DrawList::default(), (0, 0), &font(), 80, 40);
        tree.mouse_move(5, 5, (80, 40));
        assert_eq!(tree.hover, Some(vec![]));
        tree.mouse_move(0, 5, (80, 40));
        assert_eq!(tree.hover, None);
        tree.mouse_move(5, 40, (80, 40));
        assert_eq!(tree.hover, None);
    }
}
