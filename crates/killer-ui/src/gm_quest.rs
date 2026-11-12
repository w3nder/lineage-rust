use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::frame_contract::{FrameContract, FrameHit, FrameTitle};
use crate::quest::{QuestHit, QuestNode, QuestView};
use crate::window::{Rect, WindowId};

pub const ID: WindowId = WindowId::GM_QUEST;
pub const SIZE: (i32, i32) = (256, 355);
pub const FRAME_ID: u32 = 0xf0f119;
pub const GRIP: Rect = Rect::new(0, 0, 256, 20);
pub const PAGE_ORIGIN: (i32, i32) = (0, 20);
pub const TOOLTIP_HOVER: Rect = Rect::new(232, 10, 15, 15);
pub const INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 1, vertical_mode: 3, x: 209, y: 100 };

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
pub struct GmQuestView {
    pub frame: FrameCtrl,
    pub view: QuestView,
    pub items: Vec<(i32, i32)>,
    pub added: i32,
    pressed_close: bool,
}

impl GmQuestView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let mut view = QuestView::with_sysstrings(&lookup);
        view.gm = true;
        view.count = Some(0);
        Self { frame: FRAME.build(|id| lookup(id).unwrap_or_default()), view, items: Vec::new(), added: 0, pressed_close: false }
    }

    pub fn set_font(&mut self, font: &BitmapFont) {
        self.view.set_font(font);
    }

    pub fn reset(&mut self) {
        self.view.reset();
        self.items.clear();
        self.added = 0;
        self.view.count = Some(0);
    }

    pub fn add_quest(&mut self, node: Option<QuestNode>) {
        if let Some(node) = node {
            self.view.quests.push(node);
        }
        self.added += 1;
        self.view.count = Some(self.added);
        self.refresh_counts();
    }

    pub fn add_item(&mut self, item_id: i32, count: i32) {
        self.items.push((item_id, count));
        self.refresh_counts();
    }

    fn refresh_counts(&mut self) {
        let items = &self.items;
        let Some(last) = self.view.quests.last_mut() else { return };
        for step in &mut last.steps {
            for item in &mut step.items {
                item.current = items.iter().filter(|(id, _)| *id == item.item_id).map(|(_, count)| *count).sum();
            }
        }
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        self.view.tooltip_hover = TOOLTIP_HOVER.contains(x, y);
        self.view.scroll_mouse_move(y - PAGE_ORIGIN.1);
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> bool {
        if FRAME.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            self.pressed_close = true;
            return true;
        }
        let (px, py) = (x - PAGE_ORIGIN.0, y - PAGE_ORIGIN.1);
        match self.view.hit(px, py) {
            Some(hit @ (QuestHit::Quest(_) | QuestHit::Step(_, _))) => self.view.toggle(hit),
            Some(QuestHit::Scroll) => {
                self.view.scroll_mouse_down(px, py);
            }
            _ => {}
        }
        false
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> bool {
        self.view.scroll_mouse_up();
        std::mem::take(&mut self.pressed_close) && FRAME.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close)
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        let tree = crate::quest::TREE_RECT;
        if !tree.contains(x - PAGE_ORIGIN.0, y - PAGE_ORIGIN.1) {
            return false;
        }
        self.view.wheel(delta);
        true
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        FRAME.paint(&self.frame, list, origin, font, alpha);
        self.view.paint(list, origin.0 + PAGE_ORIGIN.0, origin.1 + PAGE_ORIGIN.1, font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quest::{QuestItem, QuestStep};

    fn node(id: i32) -> QuestNode {
        QuestNode {
            quest_id: id,
            title: format!("q{id}"),
            steps: vec![QuestStep {
                state: 1,
                items: vec![QuestItem { item_id: 7, ..Default::default() }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    #[test]
    fn the_counter_counts_every_add_even_when_the_quest_is_unknown_and_items_sum_on_the_last_root() {
        let mut view = GmQuestView::new(|id| Some(format!("s{id}")));
        view.add_quest(Some(node(1)));
        view.add_quest(None);
        view.add_quest(Some(node(2)));
        view.add_item(7, 3);
        view.add_item(7, 4);
        view.add_item(9, 1);
        assert_eq!(view.view.count, Some(3));
        assert_eq!(view.view.quests.len(), 2);
        assert_eq!(view.view.quests[1].steps[0].items[0].current, 7);
        assert_eq!(view.view.quests[0].steps[0].items[0].current, 0);
        view.reset();
        assert_eq!((view.view.count, view.view.quests.len(), view.items.len()), (Some(0), 0, 0));
    }

    #[test]
    fn there_is_no_abort_button_and_the_tooltip_hover_stays_on_the_frame_bar() {
        let mut view = GmQuestView::new(|id| Some(format!("s{id}")));
        let abort = crate::quest::ABORT_RECT;
        assert_ne!(view.view.hit(abort.x + 1, abort.y + 1), Some(QuestHit::Abort));
        view.mouse_move(235, 12);
        assert!(view.view.tooltip_hover);
        view.mouse_move(235, 32);
        assert!(!view.view.tooltip_hover);
    }

    #[test]
    fn the_gm_paint_layers_and_the_tooltip_match_the_reference() {
        let font = {
            let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
            crate::font::BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
        };
        let mut view = GmQuestView::new(|id| Some(format!("s{id}")));
        view.mouse_move(235, 12);
        let mut list = DrawList::default();
        view.paint(&mut list, (0, 0), &font, 1.0);
        let find = |name: &str| list.quads.iter().find(|q| q.texture == name).map(|q| (q.x, q.y, q.w, q.h, q.color[3]));
        assert_eq!(find(crate::quest::QUEST_BACK), Some((0.0, 20.0, 256.0, 335.0, 1.0)));
        assert_eq!(find(crate::quest::TOOLTIP_BTN), Some((232.0, 30.0, 15.0, 15.0, 1.0)));
        let icons: Vec<_> = list.quads.iter().filter(|q| q.texture.contains("QuestWndInfoIcon")).collect();
        assert_eq!(icons.len(), 5);
        assert!(icons.iter().all(|q| q.w == 16.0 && q.h == 16.0));
    }
}
