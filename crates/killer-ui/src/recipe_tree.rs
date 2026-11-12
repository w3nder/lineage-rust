#[path = "recipe_tree_layout_generated.rs"]
mod layout;

use crate::button::TextButton;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::item_grid::GridEntry;
use crate::recipe::{self, CAPTION_COLOR, LABEL_COLOR, VALUE_COLOR};
use crate::scroll::{ScrollHit, ScrollWnd};
use crate::window::Rect;

pub const CONTROL: Rect = layout::RECT_EC;
pub const BACK: &str = "L2UI_CH3.RecipeWnd.Recipe5_Back";
pub const COLLAPSED: &str = "L2UI.RecipeWnd.RecipeTreeIconBack";
pub const EXPANDED: &str = "L2UI.RecipeWnd.RecipeTreeIconBack_click";
pub const LEAF: &str = "L2UI.RecipeWnd.RecipeTreeIconDisableBack";
pub const PLUS: &str = "L2UI.RecipeWnd.TreePlus";
pub const MINUS: &str = "L2UI.RecipeWnd.TreeMinus";
pub const CLOSE: Rect = layout::RECT_1D3C;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeNode {
    pub item: GridEntry,
    pub needed: i32,
    pub owned: i32,
    pub expanded: bool,
    pub children: Vec<RecipeNode>,
    pub name_width: i32,
}

impl RecipeNode {
    fn measure(&mut self, font: &BitmapFont) {
        self.name_width = font.measure(&self.item.name) as i32;
        for child in &mut self.children {
            child.measure(font);
        }
    }

    fn height(&self) -> i32 {
        37 + if self.expanded { self.children.iter().map(Self::height).sum() } else { 0 }
    }

    fn node_at(&self, x: i32, y: i32, origin: (i32, i32)) -> Option<Vec<usize>> {
        let width = 34 + self.name_width + if self.children.is_empty() { 0 } else { 16 };
        if Rect::new(origin.0, origin.1, width, 32).contains(x, y) {
            return Some(Vec::new());
        }
        if self.expanded {
            let mut cy = origin.1 + 37;
            for (i, child) in self.children.iter().enumerate() {
                if let Some(mut path) = child.node_at(x, y, (origin.0 + 16, cy)) {
                    path.insert(0, i);
                    return Some(path);
                }
                cy += child.height();
            }
        }
        None
    }

    fn paint(&self, list: &mut DrawList, x: i32, y: i32, root: bool, font: &BitmapFont) {
        list.draw_texture(x, y, 32, 32, 0, 0, 32, 32, &self.item.icon, 1.0);
        let overlay = if self.expanded {
            EXPANDED
        } else if self.children.is_empty() {
            LEAF
        } else {
            COLLAPSED
        };
        list.draw_texture(x, y, 32, 32, 0, 0, 32, 32, overlay, 1.0);
        if !self.children.is_empty() {
            list.draw_texture(x - 14, y + 10, 12, 12, 0, 0, 12, 12, if self.expanded { MINUS } else { PLUS }, 1.0);
        }
        list.draw_text(x + 37, y + 4, LABEL_COLOR, &self.item.name, font);
        if !root {
            list.draw_text(x + 37, y + 18, LABEL_COLOR, &format!("({}/{})", self.owned, self.needed), font);
            if self.owned < self.needed {
                list.draw_texture(x, y, 32, 32, 0, 0, 32, 32, recipe::SHORTFALL, 1.0);
            }
        }
        if self.expanded {
            let mut cy = y + 37;
            for child in &self.children {
                child.paint(list, x + 16, cy, false, font);
                cy += child.height();
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeHit {
    Close,
    CloseButton,
    Scroll(ScrollHit),
    Node(Vec<usize>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeTreeView {
    pub root: Option<RecipeNode>,
    pub product: GridEntry,
    pub grade_texture: Option<String>,
    pub mp: i32,
    pub success: i32,
    pub level: i32,
    pub mp_label: String,
    pub success_label: String,
    pub frame: FrameCtrl,
    pub close: TextButton,
    pub scroll: ScrollWnd,
    pub offset: i32,
    pub pressed: Option<TreeHit>,
}

impl RecipeTreeView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self {
            root: None,
            product: GridEntry::default(),
            grade_texture: None,
            mp: 0,
            success: 0,
            level: 0,
            mp_label: lookup(recipe::MP_COST_SYSSTRING).unwrap_or_default(),
            success_label: lookup(recipe::SUCCESS_RATE_SYSSTRING).unwrap_or_default(),
            frame: FrameCtrl::titled(&lookup(recipe::TREE_TOOLTIP_SYSSTRING).unwrap_or_default(), true),
            close: TextButton::new(
                CLOSE,
                recipe::BUTTON_TEXTURES.0,
                recipe::BUTTON_TEXTURES.1,
                &lookup(recipe::CLOSE_SYSSTRING).unwrap_or_default(),
            ),
            scroll: ScrollWnd::new(CONTROL.w, CONTROL.h),
            offset: 0,
            pressed: None,
        }
    }

    pub fn clear(&mut self) {
        self.root = None;
        self.product = GridEntry::default();
        self.grade_texture = None;
        (self.mp, self.success, self.level) = (0, 0, 0);
        self.offset = 0;
        self.pressed = None;
        self.scroll = ScrollWnd::new(CONTROL.w, CONTROL.h);
        self.update_scroll();
    }

    pub fn measure(&mut self, font: &BitmapFont) {
        if let Some(root) = &mut self.root {
            root.measure(font);
        }
    }

    pub fn update_scroll(&mut self) {
        let total = crate::tree::ctrl_content(self.root.as_ref().map_or(0, RecipeNode::height));
        let slider = &mut self.scroll.slider;
        let sync = crate::tree::scroll_sync(CONTROL.h, slider.thumb, slider.thumb_y, total);
        crate::tree::apply_scroll_sync(slider, sync);
        let offset = if sync == crate::tree::ScrollSync::Collapse { 0 } else { self.offset };
        self.set_offset(offset);
    }

    fn set_offset(&mut self, offset: i32) {
        self.offset = offset.clamp(0, (self.scroll.slider.total - CONTROL.h).max(0));
        self.scroll.slider.set_pos(self.offset);
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<TreeHit> {
        if Rect::new(
            layout::SIZE.0 - crate::frame::CLOSE_INSET,
            crate::frame::BUTTON_Y,
            crate::frame::BUTTON_SIZE,
            crate::frame::BUTTON_SIZE,
        )
        .contains(x, y)
        {
            return Some(TreeHit::Close);
        }
        if CLOSE.contains(x, y) {
            return Some(TreeHit::CloseButton);
        }
        if !CONTROL.contains(x, y) {
            return None;
        }
        if let Some(hit) = self.scroll.hit(CONTROL.w, CONTROL.h, x - CONTROL.x, y - CONTROL.y) {
            return Some(TreeHit::Scroll(hit));
        }
        self.root.as_ref()?.node_at(x - CONTROL.x, y - CONTROL.y + self.offset, (15, 5)).map(TreeHit::Node)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        self.pressed = self.hit(x, y);
        match &self.pressed {
            Some(TreeHit::Node(path)) => {
                let mut node = self.root.as_mut();
                for i in path {
                    node = node.and_then(|n| n.children.get_mut(*i));
                }
                if let Some(node) = node
                    && !node.children.is_empty()
                {
                    node.expanded = !node.expanded;
                }
                self.update_scroll();
            }
            Some(TreeHit::Scroll(hit)) => {
                self.scroll.up_pressed = *hit == ScrollHit::Up;
                self.scroll.down_pressed = *hit == ScrollHit::Down;
                if let ScrollHit::Slider(y) = hit {
                    self.scroll.slider.mouse_down(*y);
                }
            }
            _ => {}
        }
    }

    pub fn mouse_up(&mut self, over: Option<(i32, i32)>) -> bool {
        let released = over.and_then(|(x, y)| self.hit(x, y));
        let pressed = self.pressed.take();
        self.scroll.up_pressed = false;
        self.scroll.down_pressed = false;
        self.scroll.slider.mouse_up();
        if pressed != released {
            return false;
        }
        match pressed {
            Some(TreeHit::Close | TreeHit::CloseButton) => return true,
            Some(TreeHit::Scroll(ScrollHit::Up)) => self.set_offset(-crate::gara_tree::scroll_line_up(-self.offset)),
            Some(TreeHit::Scroll(ScrollHit::Down)) => {
                let minimum = -(self.scroll.slider.total - CONTROL.h).max(0);
                self.set_offset(-crate::gara_tree::scroll_line_down(-self.offset, minimum));
            }
            _ => {}
        }
        false
    }

    pub fn mouse_move(&mut self, y: i32) -> bool {
        if !self.scroll.slider.dragging {
            return false;
        }
        if let Some(pos) = self.scroll.slider.mouse_move(y - CONTROL.y - crate::scroll::BUTTON_SIZE) {
            self.offset = pos;
        }
        true
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        if !CONTROL.contains(x, y) {
            return false;
        }
        let minimum = -(self.scroll.slider.total - CONTROL.h).max(0);
        self.set_offset(-crate::gara_tree::scroll_wheel(-self.offset, minimum, delta));
        true
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        self.frame.paint(list, x, y, layout::SIZE.0, layout::SIZE.1, font, alpha);
        list.draw_texture(x, y + 20, 256, 381, 0, 0, 256, 381, BACK, alpha);
        if !self.product.icon.is_empty() {
            list.draw_texture(x + 16, y + 49, 32, 32, 0, 0, 32, 32, &self.product.icon, 1.0);
        }
        let name_end = list.draw_text(x + 58, y + 48, LABEL_COLOR, &self.product.name, font);
        if let Some(texture) = &self.grade_texture {
            list.draw_texture(name_end + 2, y + 48, 12, 12, 0, 0, 12, 12, texture, 1.0);
            if let Some(q) = list.quads.last_mut() {
                q.color = LABEL_COLOR.rgba();
            }
        }
        for (label, value, cy) in
            [(&self.mp_label, self.mp.to_string(), 64), (&self.success_label, format!("{}%", self.success), 80)]
        {
            let end = list.draw_text(x + 58, y + cy, CAPTION_COLOR, label, font);
            let end = list.draw_text(end, y + cy, LABEL_COLOR, recipe::SEPARATOR, font);
            list.draw_text(end, y + cy, VALUE_COLOR, &value, font);
        }
        list.draw_text(x + 214, y + 77, LABEL_COLOR, &format!("Lv.{}", self.level), font);
        let mut body = DrawList {capture_text:list.capture_text,..Default::default()};
        if let Some(root) = &self.root {
            root.paint(&mut body, x + CONTROL.x + 15, y + CONTROL.y + 5 - self.offset, true, font);
        }
        list.append_clipped(body,crate::window::Rect::new(x+CONTROL.x,y+CONTROL.y,CONTROL.w,CONTROL.h));
        self.scroll.paint(list, x + CONTROL.x, y + CONTROL.y, CONTROL.w, CONTROL.h);
        self.close.paint(list, x, y, self.pressed == Some(TreeHit::CloseButton), font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(children: Vec<RecipeNode>) -> RecipeNode {
        RecipeNode {
            item: GridEntry { name: "Material".into(), icon: "item.icon".into(), ..Default::default() },
            needed: 5,
            owned: 2,
            expanded: false,
            children,
            name_width: 48,
        }
    }

    #[test]
    fn recipe_tree_10015000_100f4ea0_expands_each_branch_and_retains_child_state() {
        let mut view = RecipeTreeView::new(|_| None);
        view.root = Some(node(vec![node(vec![node(vec![])]), node(vec![])]));
        view.update_scroll();
        assert_eq!(view.scroll.slider.total, 58);
        assert_eq!(view.hit(23, 114), Some(TreeHit::Node(vec![])));
        assert_eq!(view.hit(39, 151), None);
        view.mouse_down(23, 114);
        assert_eq!(view.scroll.slider.total, 132);
        view.mouse_down(39, 151);
        assert_eq!(view.scroll.slider.total, 169);
        assert_eq!(view.hit(55, 188), Some(TreeHit::Node(vec![0, 0])));
        view.mouse_down(23, 114);
        assert_eq!(view.hit(55, 188), None);
        view.mouse_down(23, 114);
        assert_eq!(view.hit(55, 188), Some(TreeHit::Node(vec![0, 0])));
        view.mouse_down(55, 188);
        assert!(!view.root.as_ref().unwrap().children[0].children[0].expanded);
    }

    #[test]
    fn recipe_tree_100ff520_100f53f0_scrolls_pixels_and_cancels_released_arrows() {
        let mut view = RecipeTreeView::new(|_| None);
        view.root = Some(node((0..20).map(|_| node(vec![])).collect()));
        view.mouse_down(23, 114);
        assert_eq!(view.scroll.slider.total, 798);
        assert!(!view.wheel(0, 0, -120));
        view.wheel(23, 114, -119);
        assert_eq!(view.offset, 0);
        view.wheel(23, 114, -120);
        assert_eq!(view.offset, 45);
        view.mouse_down(245, 355);
        view.mouse_up(None);
        assert_eq!(view.offset, 45);
        view.mouse_down(245, 355);
        view.mouse_up(Some((245, 355)));
        assert_eq!(view.offset, 60);
        view.wheel(23, 114, 12000);
        let thumb = CONTROL.y + crate::scroll::BUTTON_SIZE + 2;
        view.mouse_down(245, thumb);
        assert!(view.mouse_move(10000));
        assert_eq!(view.offset, 544);
        view.mouse_up(None);
        assert!(!view.mouse_move(thumb));
        view.clear();
        assert_eq!(view.offset, 0);
        assert!(view.root.is_none());
    }
}
