use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::frame_contract::{FrameContract, FrameHit, FrameTitle};
use crate::scroll::ScrollHit;
use crate::skills::{SkillHit, SkillTab, SkillView, SkillsView};
use crate::window::{Rect, WindowId};

pub const ID: WindowId = WindowId::GM_SKILLS;
pub const SIZE: (i32, i32) = (256, 355);
pub const FRAME_ID: u32 = 0xf0f117;
pub const TAB_CTRL_ID: u32 = 0xf0f012;
pub const GRIP: Rect = Rect::new(0, 0, 256, 20);
pub const PAGE_ORIGIN: (i32, i32) = (0, 20);
pub const INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 1, vertical_mode: 5, x: 209, y: 100 };

const FRAME: FrameContract = FrameContract {
    rect: GRIP,
    id: FRAME_ID,
    mode: FrameMode::Title,
    title: FrameTitle::None,
    close_button: true,
    minimize_button: false,
};

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 - 208.0 - 1.0) as i32, 100)
}

#[derive(Clone, Debug)]
pub struct GmSkillsView {
    pub frame: FrameCtrl,
    pub view: SkillsView,
    pressed: Option<GmSkillsPress>,
    dragging: bool,
    cursor_drag: bool,
    pickup: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GmSkillsPress {
    Close,
    Tab(SkillTab),
    Scroll(ScrollHit),
    Page,
}

impl GmSkillsView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Self { frame: FRAME.build(text), view: SkillsView::gm_with_sysstrings(&lookup), pressed: None, dragging: false, cursor_drag: false, pickup: false }
    }

    pub fn clear(&mut self) {
        self.view.skills.clear();
        self.view.selected = None;
        self.view.hovered = None;
        self.view.scroll_row = 0;
        self.view.update_rows();
    }

    pub fn add(&mut self, skill: SkillView) {
        if skill.level <= 0 {
            return;
        }
        self.view.skills.push(skill);
        self.view.update_rows();
    }

    fn local(x: i32, y: i32) -> (i32, i32) {
        (x - PAGE_ORIGIN.0, y - PAGE_ORIGIN.1)
    }

    pub fn slot_at(&self, x: i32, y: i32) -> Option<usize> {
        let (px, py) = Self::local(x, y);
        match self.view.hit(px, py) {
            Some(SkillHit::Slot(index)) if self.view.slot_skill(index).is_some() => Some(index),
            _ => None,
        }
    }

    pub fn hover(&mut self, x: i32, y: i32) -> Option<usize> {
        self.view.hovered = self.slot_at(x, y);
        self.view.hovered
    }

    pub fn wheel(&mut self, delta: i32) {
        let (first, visible, total) = (self.view.scroll_row as i32, self.view.rows() as i32, self.view.total_rows());
        self.view.set_first_row(crate::inventory::item_wnd_wheel(first, visible, total, delta));
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> bool {
        if FRAME.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            self.pressed = Some(GmSkillsPress::Close);
            return true;
        }
        let (px, py) = Self::local(x, y);
        self.pressed = self.view.hit(px, py).map(|hit| match hit {
            SkillHit::Tab(tab) => GmSkillsPress::Tab(tab),
            SkillHit::Scroll(scroll) => GmSkillsPress::Scroll(scroll),
            SkillHit::Slot(index) => {
                self.view.selected = self.view.slot_skill(index).map(|skill| skill.id);
                self.view.pressed = true;
                if self.view.selected.is_some() {
                    self.dragging = true;
                    self.pickup = true;
                }
                GmSkillsPress::Page
            }
            SkillHit::Page => GmSkillsPress::Page,
        });
        match self.pressed {
            Some(GmSkillsPress::Scroll(ScrollHit::Up)) => self.view.scroll.up_pressed = true,
            Some(GmSkillsPress::Scroll(ScrollHit::Down)) => self.view.scroll.down_pressed = true,
            Some(GmSkillsPress::Scroll(_)) => {
                self.view.scroll.slider.mouse_down(py - crate::scroll::BUTTON_SIZE - crate::skills::LIST_RECT.y);
            }
            _ => {}
        }
        self.pressed.is_some()
    }

    pub fn take_pickup(&mut self) -> bool {
        std::mem::take(&mut self.pickup)
    }

    pub fn cursor_hover(&mut self) {
        self.cursor_drag = self.dragging;
    }

    pub fn drag_icon(&self) -> Option<String> {
        if !self.cursor_drag {
            return None;
        }
        let selected = self.view.selected?;
        self.view.skills.iter().find(|skill| skill.id == selected).map(|skill| skill.icon.clone()).filter(|icon| !icon.is_empty())
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> bool {
        self.dragging = false;
        self.cursor_drag = false;
        let pressed = self.pressed.take();
        let (px, py) = Self::local(x, y);
        let released = FRAME.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close);
        let (first, visible, total) = (self.view.scroll_row as i32, self.view.rows() as i32, self.view.total_rows());
        match pressed {
            Some(GmSkillsPress::Close) => released,
            Some(GmSkillsPress::Tab(tab)) => {
                if self.view.hit(px, py) == Some(SkillHit::Tab(tab)) {
                    self.view.tab = tab;
                    self.view.scroll_row = 0;
                    self.view.update_rows();
                }
                false
            }
            Some(GmSkillsPress::Scroll(hit)) => {
                self.view.scroll.up_pressed = false;
                self.view.scroll.down_pressed = false;
                self.view.scroll.slider.mouse_up();
                let next = match hit {
                    ScrollHit::Up => crate::scroll::line_up(first, visible, total),
                    ScrollHit::Down => crate::scroll::line_down(first, visible, total),
                    _ => first,
                };
                self.view.set_first_row(next);
                false
            }
            _ => false,
        }
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        FRAME.paint(&self.frame, list, origin, font, alpha);
        self.view.paint(list, origin.0 + PAGE_ORIGIN.0, origin.1 + PAGE_ORIGIN.1, font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(id: i32, level: i32, passive: bool) -> SkillView {
        SkillView { id, level, passive, ..Default::default() }
    }

    #[test]
    fn skills_without_level_are_dropped_and_the_tabs_split_active_from_passive() {
        let mut view = GmSkillsView::new(|id| Some(format!("s{id}")));
        view.add(skill(1, 0, false));
        view.add(skill(2, 3, false));
        view.add(skill(3, 1, true));
        assert_eq!(view.view.listed().len(), 1);
        view.view.tab = SkillTab::Passive;
        assert_eq!(view.view.listed()[0].id, 3);
        view.clear();
        assert!(view.view.skills.is_empty());
    }

    #[test]
    fn the_list_shows_seven_rows_and_scrolls_by_one() {
        let mut view = GmSkillsView::new(|id| Some(format!("s{id}")));
        for id in 0..60 {
            view.add(skill(id + 1, 1, false));
        }
        assert_eq!(view.view.rows(), 7);
        assert_eq!(view.view.total_rows(), 10);
        view.wheel(-crate::inventory::WHEEL_DELTA);
        let first = view.view.scroll_row;
        assert!(first <= 3);
        for _ in 0..20 {
            view.wheel(-crate::inventory::WHEEL_DELTA);
        }
        assert_eq!(view.view.scroll_row, 3);
    }

    #[test]
    fn a_slot_click_only_selects_and_never_acts() {
        let mut view = GmSkillsView::new(|id| Some(format!("s{id}")));
        view.add(skill(9, 1, false));
        let (x, y) = (crate::skills::LIST_RECT.x + 2, crate::skills::LIST_RECT.y + PAGE_ORIGIN.1 + 2);
        assert!(view.mouse_down(x, y));
        assert!(!view.mouse_up(x, y));
        assert_eq!(view.view.selected, Some(9));
    }

    #[test]
    fn the_carried_skill_is_the_cursor_until_the_button_is_released_and_is_never_dropped() {
        let mut view = GmSkillsView::new(|id| Some(format!("s{id}")));
        view.add(SkillView { id: 9, level: 1, icon: "icon.skill9".into(), ..Default::default() });
        let (x, y) = (crate::skills::LIST_RECT.x + 2, crate::skills::LIST_RECT.y + PAGE_ORIGIN.1 + 2);
        assert!(view.drag_icon().is_none());
        assert!(view.mouse_down(x, y));
        assert!(view.take_pickup());
        assert!(!view.take_pickup());
        assert!(view.drag_icon().is_none());
        view.cursor_hover();
        assert_eq!(view.drag_icon().as_deref(), Some("icon.skill9"));
        assert!(!view.mouse_up(x, y));
        assert!(view.drag_icon().is_none());
        view.cursor_hover();
        assert!(view.drag_icon().is_none());
    }

    #[test]
    fn the_gm_paint_layers_match_the_reference_on_paint_and_on_create() {
        let font = {
            let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
            BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
        };
        let mut view = GmSkillsView::new(|id| Some(format!("s{id}")));
        view.add(SkillView { id: 9, level: 1, icon: "icon.skill9".into(), ..Default::default() });
        let mut list = DrawList::default();
        view.paint(&mut list, (0, 0), &font, 1.0);
        let find = |name: &str| list.quads.iter().find(|q| q.texture == name).map(|q| (q.x, q.y, q.w, q.h, q.color[3]));
        assert_eq!(find(crate::skills::SKILL_BACK), Some((0.0, 20.0, 256.0, 335.0, 1.0)));
        assert_eq!(find(crate::skills::TAB_ACTIVE), Some((12.0, 28.0, 94.0, 23.0, 1.0)));
        assert_eq!(find(crate::skills::TAB_NORMAL), Some((107.0, 28.0, 94.0, 23.0, 1.0)));
        assert_eq!(find("icon.skill9"), Some((10.0, 61.0, 32.0, 32.0, 1.0)));
    }
}
