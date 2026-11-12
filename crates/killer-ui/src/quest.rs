use crate::button::TextButton;
use crate::chat::wrap_line;
use crate::color::{Argb, GOLD, TEXT};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::scroll::{ScrollHit, ScrollWnd};
use crate::window::Rect;

pub const PAGE_SIZE: (i32, i32) = (256, 335);
pub const QUEST_BACK: &str = "L2UI_CH3.QUESTWND.QuestWndBack";
pub const TOOLTIP_BTN: &str = "L2UI_CH3.QUESTWND.QuestWndToolTipBtn";
pub const PLUS_BTN: &str = "L2UI_CH3.QUESTWND.QuestWndPlusBtn";
pub const MINUS_BTN: &str = "L2UI_CH3.QUESTWND.QuestWndMinusBtn";
pub const UP_BTN: &str = "L2UI_CH3.QUESTWND.QuestWndUpBtn";
pub const DOWN_BTN: &str = "L2UI_CH3.QUESTWND.QuestWndDownBtn";
pub const PANELS: [&str; 3] = ["L2UI_CH3.QUESTWND.QUESTWNDPANEL1", "L2UI_CH3.QUESTWND.QUESTWNDPANEL2", "L2UI_CH3.QUESTWND.QUESTWNDPANEL3"];
pub const MENU_OUTLINE: &str = "L2UI_CH3.Etc.menu_outline";
pub const TEXT_SELECT: &str = "L2UI_CH3.ListCtrl.TextSelect";
pub const TEXT_SELECT_END: &str = "L2UI_CH3.ListCtrl.TextSelect2";
pub const INFO_ICONS: [&str; 5] = [
    "L2UI_CH3.QUESTWND.QuestWndInfoIcon_1",
    "L2UI_CH3.QUESTWND.QuestWndInfoIcon_2",
    "L2UI_CH3.QUESTWND.QuestWndInfoIcon_3",
    "L2UI_CH3.QUESTWND.QuestWndInfoIcon_4",
    "L2UI_CH3.QUESTWND.QuestWndInfoIcon_5",
];
pub const BTN_NORMAL: &str = "L2UI_CH3.BUTTON.Btn1_normal";
pub const BTN_DOWN: &str = "L2UI_CH3.BUTTON.Btn1_normalOn";
pub const ACCEPTED_SYSSTRING: u32 = 324;
pub const SELECT_QUEST_SYSMSG: u32 = 1201;
pub const ABORT_SYSSTRING: u32 = 385;
pub const LEVEL_SYSSTRING: u32 = 922;
pub const UNLIMITED_SYSSTRING: u32 = 858;
pub const AND_ABOVE_SYSSTRING: u32 = 859;
pub const COMPLETE_SYSSTRING: u32 = 898;
pub const NO_LIMIT_SYSSTRING: u32 = 866;
pub const TOOLTIP_SYSSTRINGS: [u32; 5] = [861, 862, 863, 864, 865];
pub const TITLE_POS: (i32, i32) = (11, 12);
pub const COUNTER_POS: (i32, i32) = (190, 12);
pub const TOOLTIP_BTN_RECT: Rect = Rect::new(232, 10, 15, 15);
pub const TREE_RECT: Rect = Rect::new(7, 27, 242, 274);
pub const TREE_WIDTH: i32 = 226;
pub const ABORT_RECT: Rect = Rect::new(91, 306, 77, 23);
pub const NODE_ICON: i32 = 14;
pub const GLYPH: i32 = 11;
pub const MAX_QUESTS: usize = 15;
pub const STATE_BITS: i32 = 30;
pub const MASK_FLAG: i32 = 1 << 30;
pub const ROOT_X: i32 = 3;
pub const ROOT_Y: i32 = 5;
pub const ROOT_HEIGHT: i32 = 34;
pub const STEP_X: i32 = 10;
pub const STEP_HEIGHT: i32 = 19;
pub const BODY_X: i32 = 17;
pub const BODY_WRAP_MARGIN: i32 = 5;
pub const DESCRIPTION_STEP: i32 = 19;
pub const ITEM_STEP: i32 = 37;
pub const BODY_PADDING: i32 = 10;
pub const ITEM_ICON: i32 = 32;
pub const PANEL_WIDTH: i32 = 211;
pub const PANEL_EDGE: i32 = 8;
pub const HIT_CHILD_EXTRA: i32 = 16;
pub const LINE_STEP: i32 = 15;
pub const WHEEL_NOTCH: i32 = 45;
pub const LOCATION_RECT: Rect = Rect::new(10, 310, 100, 12);
pub const LOCATION_SYSSTRING: u32 = 860;
pub const TIP_FRAME: &str = "L2UI_CH3.Tooltip.Tooltip";
pub const TIP_ICONS: [&str; 5] = [
    "L2UI_CH3.QuestWnd.QuestWndInfoIcon_1",
    "L2UI_CH3.QuestWnd.QuestWndInfoIcon_2",
    "L2UI_CH3.QuestWnd.QuestWndInfoIcon_3",
    "L2UI_CH3.QuestWnd.QuestWndInfoIcon_4",
    "L2UI_CH3.QuestWnd.QuestWndInfoIcon_5",
];
pub const TIP_SIZE: (i32, i32) = (184, 105);
pub const TIP_ANCHOR: (i32, i32) = (232, 10);
pub const TIP_PAD: i32 = 5;
pub const TIP_STEP: i32 = 19;
pub const TIP_ICON: i32 = 16;
pub const TIP_TEXT_X: i32 = 21;
pub const GRAY: Argb = Argb(0xFF8C8C8C);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QuestItem {
    pub item_id: i32,
    pub icon: String,
    pub name: String,
    pub required: i32,
    pub current: i32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct QuestStep {
    pub state: i32,
    pub sub_name: String,
    pub description: String,
    pub target_name: String,
    pub journal_disp: bool,
    pub counts_disp: bool,
    pub target_loc: [f32; 3],
    pub minimap_only: bool,
    pub items: Vec<QuestItem>,
    pub expanded: bool,
    pub selected: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct QuestNode {
    pub quest_id: i32,
    pub title: String,
    pub quest_type: i32,
    pub level: i32,
    pub max_level: i32,
    pub steps: Vec<QuestStep>,
    pub expanded: bool,
    pub selected: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestHit {
    Quest(usize),
    Step(usize, usize),
    Abort,
    Tooltip,
    Location,
    Scroll,
    Page,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct QuestTarget {
    pub name: String,
    pub loc: [f32; 3],
    pub id: i32,
    pub title: String,
    pub minimap_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct QuestLabels {
    pub level: String,
    pub unlimited: String,
    pub and_above: String,
    pub complete: String,
    pub no_limit: String,
    pub tooltip: [String; 5],
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct QuestView {
    pub quests: Vec<QuestNode>,
    pub selected: Option<usize>,
    pub marked: Option<i32>,
    pub abort_pressed: bool,
    pub accepted_label: String,
    pub abort_label: String,
    pub labels: QuestLabels,
    pub scroll: i32,
    pub tree_offset: i32,
    pub tooltip_hover: bool,
    pub gm: bool,
    pub count: Option<i32>,
    pub location: bool,
    pub location_label: String,
    pub screen: (i32, i32),
    pub minimap_only_flags: bool,
    pub shown: Option<QuestTarget>,
    font: Option<BitmapFont>,
    bar: ScrollWnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    Root(usize),
    Step(usize, usize),
    Body(usize, usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeBox {
    pub kind: NodeKind,
    pub x: i32,
    pub y: i32,
    pub rect: Rect,
}

pub fn paint_icon_tip(list: &mut DrawList, window: (i32, i32), anchor: (i32, i32), screen: (i32, i32), rows: &[(&str, &str)], font: &BitmapFont) {
    let (wx, wy) = window;
    let (w, h) = (TIP_SIZE.0, TIP_STEP * rows.len() as i32 + 2 * TIP_PAD);
    let mut tx = anchor.0;
    if screen.0 > 0 {
        let overflow = wx + anchor.0 + w - screen.0;
        if overflow > 0 {
            tx -= overflow;
        } else if wx + anchor.0 < 0 {
            tx = -wx;
        }
    }
    let mut ty = anchor.1 - h;
    if wy + ty < 0 {
        ty = anchor.1;
    }
    let start = list.quads.len();
    crate::tooltip::paint_frame(list, wx + tx, wy + ty, w, h);
    for (index, (icon, text)) in rows.iter().enumerate() {
        let line = wy + ty + TIP_PAD + TIP_STEP * index as i32;
        list.draw_texture(wx + tx + TIP_PAD, line, TIP_ICON, TIP_ICON, 0, 0, TIP_ICON, TIP_ICON, icon, 1.0);
        list.draw_text(wx + tx + TIP_TEXT_X, line, TEXT, text, font);
    }
    list.clip_from(start, Rect::new(wx + tx, wy + ty, w, h));
}

pub fn states_of(cond: i32) -> Vec<i32> {
    if cond <= 0 {
        return Vec::new();
    }
    let mask = if cond & MASK_FLAG != 0 { cond } else { (1i64 << cond.min(STATE_BITS)) as i32 - 1 };
    (0..STATE_BITS).filter(|i| mask & (1 << i) != 0).map(|i| i + 1).collect()
}

pub fn description_rows(description: &str, width: i32, font: &BitmapFont) -> Vec<String> {
    description
        .replace("\\n", "\n")
        .split('\n')
        .map(|paragraph| paragraph.trim_end())
        .flat_map(|paragraph| wrap_line(paragraph, width, 0, font).into_iter().map(|(_, row)| row))
        .collect()
}

pub fn description_height(rows: usize, line: i32) -> i32 {
    line + DESCRIPTION_STEP * (rows.max(1) as i32 - 1)
}

pub fn body_height(description_rows: usize, items: usize, line: i32) -> i32 {
    description_height(description_rows, line) + ITEM_STEP * items as i32 + BODY_PADDING
}

pub fn level_text(label: &str, level: i32, max_level: i32, no_limit: &str, and_above: &str) -> String {
    if level > 0 && max_level > 0 {
        format!("({label}:{level}~{max_level})")
    } else if level > 0 {
        format!("({label}:{level} {and_above})")
    } else {
        format!("({label}:{no_limit})")
    }
}

pub fn count_text(required: i32, current: i32, not_last_and_listed: bool, labels: &QuestLabels) -> String {
    match (required, not_last_and_listed) {
        (r, true) if r > 0 => format!("({}/{})", labels.complete, r),
        (r, false) if r > 0 => format!("({current}/{r})"),
        (0, true) => format!("({}/{})", labels.complete, labels.unlimited),
        (0, false) => format!("({current}/{})", labels.unlimited),
        (r, true) => format!("({}/{}{})", labels.complete, -r, labels.and_above),
        (r, false) => format!("({current}/{}{})", -r, labels.and_above),
    }
}

pub fn glyph_textures(quest_type: i32) -> Option<(&'static str, &'static str)> {
    if !(0..=3).contains(&quest_type) {
        return None;
    }
    let first = INFO_ICONS[usize::from(quest_type & 1 != 0)];
    let second = INFO_ICONS[if quest_type & 2 != 0 { 2 } else { 3 }];
    Some((first, second))
}

impl QuestView {
    pub fn with_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> QuestView {
        let text = |id: u32| lookup(id).unwrap_or_else(|| format!("sysstring {id}"));
        QuestView {
            location: true,
            location_label: text(LOCATION_SYSSTRING),
            accepted_label: text(ACCEPTED_SYSSTRING),
            abort_label: text(ABORT_SYSSTRING),
            labels: QuestLabels {
                level: text(LEVEL_SYSSTRING),
                unlimited: text(UNLIMITED_SYSSTRING),
                and_above: text(AND_ABOVE_SYSSTRING),
                complete: text(COMPLETE_SYSSTRING),
                no_limit: text(NO_LIMIT_SYSSTRING),
                tooltip: TOOLTIP_SYSSTRINGS.map(text),
            },
            bar: ScrollWnd::new(TREE_RECT.w, TREE_RECT.h),
            ..Default::default()
        }
    }

    pub fn set_font(&mut self, font: &BitmapFont) {
        self.font = Some(font.clone());
    }

    pub fn reset(&mut self) {
        self.quests.clear();
        self.selected = None;
        self.scroll = 0;
        self.tree_offset = 0;
        self.bar = ScrollWnd::new(TREE_RECT.w, TREE_RECT.h);
    }

    pub fn nodes(&self) -> (Vec<NodeBox>, i32) {
        self.layout(self.font.as_ref())
    }

    fn layout(&self, font: Option<&BitmapFont>) -> (Vec<NodeBox>, i32) {
        let line = font.map_or(13, |font| font.line_height as i32);
        let width_of = |text: &str| font.map_or(0, |font| font.measure(text) as i32);
        let mut nodes = Vec::new();
        let mut cursor = ROOT_Y;
        let mut total = 0;
        for (qi, quest) in self.quests.iter().enumerate() {
            let root_y = cursor;
            let extra = if quest.steps.is_empty() { 0 } else { HIT_CHILD_EXTRA };
            nodes.push(NodeBox {
                kind: NodeKind::Root(qi),
                x: ROOT_X,
                y: root_y,
                rect: Rect::new(ROOT_X, root_y, NODE_ICON + width_of(&quest.title) + 2 + extra, NODE_ICON),
            });
            let mut height = ROOT_HEIGHT;
            if quest.expanded {
                for (si, step) in quest.steps.iter().enumerate() {
                    let step_y = root_y + height;
                    nodes.push(NodeBox {
                        kind: NodeKind::Step(qi, si),
                        x: STEP_X,
                        y: step_y,
                        rect: Rect::new(STEP_X, step_y, NODE_ICON + width_of(&step.sub_name) + 2 + HIT_CHILD_EXTRA, NODE_ICON),
                    });
                    height += STEP_HEIGHT;
                    if step.expanded {
                        let wrap = TREE_WIDTH - BODY_X - BODY_WRAP_MARGIN;
                        let rows = font.map_or_else(
                            || step.description.replace("\\n", "\n").split('\n').count(),
                            |font| description_rows(&step.description, wrap, font).len(),
                        );
                        let body = body_height(rows, step.items.len(), line);
                        nodes.push(NodeBox {
                            kind: NodeKind::Body(qi, si),
                            x: BODY_X,
                            y: root_y + height,
                            rect: Rect::new(BODY_X, root_y + height, ITEM_ICON + 2, ITEM_ICON),
                        });
                        height += body;
                    }
                }
            }
            cursor += height;
            total += height;
        }
        (nodes, total)
    }

    pub fn content_height(&self) -> i32 {
        self.nodes().1
    }

    pub fn minimum_scroll(&self) -> i32 {
        let content = self.content_height();
        if content > TREE_RECT.h { TREE_RECT.h - content } else { 0 }
    }

    fn sync_bar(&mut self) {
        let content = self.content_height();
        let slider = &mut self.bar.slider;
        let sync = crate::tree::scroll_sync(TREE_RECT.h, slider.thumb, slider.thumb_y, content);
        crate::tree::apply_scroll_sync(slider, sync);
        slider.set_pos(-self.scroll);
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<QuestHit> {
        if !Rect::new(0, 0, PAGE_SIZE.0, PAGE_SIZE.1).contains(x, y) {
            return None;
        }
        if !self.gm && ABORT_RECT.contains(x, y) {
            return Some(QuestHit::Abort);
        }
        if TOOLTIP_BTN_RECT.contains(x, y) {
            return Some(QuestHit::Tooltip);
        }
        if !self.gm && LOCATION_RECT.contains(x, y) {
            return Some(QuestHit::Location);
        }
        if TREE_RECT.contains(x, y) {
            if self.bar_hit(x, y).is_some() {
                return Some(QuestHit::Scroll);
            }
            let (lx, ly) = (x - TREE_RECT.x, y - TREE_RECT.y + self.tree_offset);
            for node in self.nodes().0 {
                let on_screen = node.x >= 0 && node.x <= TREE_RECT.w && (0..=TREE_RECT.h).contains(&(node.y + self.scroll));
                if !on_screen || !node.rect.contains(lx, ly) {
                    continue;
                }
                match node.kind {
                    NodeKind::Root(q) => return Some(QuestHit::Quest(q)),
                    NodeKind::Step(q, s) => return Some(QuestHit::Step(q, s)),
                    NodeKind::Body(..) => {}
                }
            }
        }
        Some(QuestHit::Page)
    }

    fn bar_hit(&self, x: i32, y: i32) -> Option<ScrollHit> {
        self.bar.hit(TREE_RECT.w, TREE_RECT.h, x - TREE_RECT.x, y - TREE_RECT.y)
    }

    pub fn toggle_location(&mut self) {
        self.location = !self.location;
        if !self.location {
            self.minimap_only_flags = false;
            self.shown = None;
        }
    }

    pub fn current_step_index(&self) -> Option<(usize, usize)> {
        let (nodes, _) = self.nodes();
        nodes.iter().rev().find_map(|node| match node.kind {
            NodeKind::Step(q, s) => Some((q, s)),
            _ => None,
        })
    }

    pub fn current_step(&self) -> Option<(&QuestNode, &QuestStep, bool)> {
        let (q, s) = self.current_step_index()?;
        let quest = self.quests.get(q)?;
        let step = quest.steps.get(s)?;
        Some((quest, step, s + 1 == quest.steps.len()))
    }

    pub fn target(&self) -> Option<QuestTarget> {
        if !self.location || self.gm {
            return None;
        }
        let (quest, step, last) = self.current_step()?;
        if !last || step.target_name.is_empty() {
            return None;
        }
        Some(QuestTarget {
            name: step.target_name.clone(),
            loc: step.target_loc,
            id: quest.quest_id,
            title: quest.title.clone(),
            minimap_only: step.minimap_only,
        })
    }

    pub fn refresh_target(&mut self) -> Option<QuestTarget> {
        self.refresh_target_painted(true)
    }

    pub fn refresh_target_painted(&mut self, painted: bool) -> Option<QuestTarget> {
        if painted && let Some(target) = self.target() {
            self.minimap_only_flags = !target.minimap_only;
            self.shown = Some(target);
        }
        self.shown.clone()
    }

    pub fn show_info(&mut self, target: QuestTarget) {
        self.shown = Some(target);
        self.minimap_only_flags = true;
    }

    pub fn abort_target(&self) -> Option<i32> {
        self.selected.and_then(|index| self.quests.get(index)).map(|quest| quest.quest_id)
    }

    pub fn apply_marked(&mut self) {
        let Some(id) = self.marked else { return };
        if let Some(index) = self.quests.iter().position(|quest| quest.quest_id == id) {
            let quest = &mut self.quests[index];
            quest.expanded = true;
            quest.selected = true;
            if let Some(step) = quest.steps.last_mut() {
                step.expanded = true;
                step.selected = true;
            }
            self.selected = Some(index);
            self.accordion();
        }
    }

    fn accordion(&mut self) {
        for quest in &mut self.quests {
            if !quest.selected && quest.expanded {
                quest.expanded = false;
            }
        }
        self.selected = self.quests.iter().position(|quest| quest.selected);
    }

    pub fn toggle(&mut self, hit: QuestHit) {
        match hit {
            QuestHit::Quest(q) => {
                for quest in &mut self.quests {
                    quest.selected = false;
                }
                if let Some(quest) = self.quests.get_mut(q)
                    && !quest.steps.is_empty()
                {
                    quest.expanded = !quest.expanded;
                    quest.selected = quest.expanded;
                }
                self.accordion();
            }
            QuestHit::Step(q, s) => {
                if let Some(quest) = self.quests.get_mut(q) {
                    for step in &mut quest.steps {
                        step.selected = false;
                    }
                    if let Some(step) = quest.steps.get_mut(s) {
                        step.expanded = !step.expanded;
                        step.selected = step.expanded;
                    }
                }
            }
            _ => {}
        }
        self.clamp_scroll();
    }

    fn clamp_scroll(&mut self) {
        let minimum = self.minimum_scroll();
        self.scroll = self.scroll.clamp(minimum, 0);
        self.sync_bar();
    }

    pub fn line_down(&mut self) {
        let minimum = self.minimum_scroll();
        if self.scroll > minimum {
            self.scroll = crate::gara_tree::scroll_line_down(self.scroll, minimum);
            self.tree_offset += LINE_STEP;
            self.sync_bar();
        }
    }

    pub fn line_up(&mut self) {
        if self.scroll < 0 {
            self.scroll = crate::gara_tree::scroll_line_up(self.scroll);
            self.tree_offset -= LINE_STEP;
            self.sync_bar();
        }
    }

    pub fn wheel(&mut self, delta: i32) {
        let old = self.scroll;
        let minimum = self.minimum_scroll();
        self.scroll = crate::gara_tree::scroll_wheel(self.scroll, minimum, delta);
        self.tree_offset += old - self.scroll;
        self.sync_bar();
    }

    pub fn scroll_mouse_down(&mut self, x: i32, y: i32) -> bool {
        match self.bar_hit(x, y) {
            Some(ScrollHit::Up) => {
                self.bar.up_pressed = true;
                self.line_up();
                true
            }
            Some(ScrollHit::Down) => {
                self.bar.down_pressed = true;
                self.line_down();
                true
            }
            Some(ScrollHit::Slider(local)) => {
                self.bar.slider.mouse_down(local);
                true
            }
            None => false,
        }
    }

    pub fn on_scroll_bar(&mut self, value: i32) {
        let old = self.scroll;
        self.scroll = crate::gara_tree::scroll_bar_rounded(value);
        self.tree_offset += old - self.scroll;
    }

    pub fn scroll_mouse_move(&mut self, y: i32) {
        if let Some(position) = self.bar.slider.mouse_move(y - TREE_RECT.y - crate::scroll::BUTTON_SIZE) {
            self.on_scroll_bar(position);
        }
    }

    pub fn scroll_mouse_up(&mut self) {
        self.bar.up_pressed = false;
        self.bar.down_pressed = false;
        self.bar.slider.mouse_up();
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        self.tooltip_hover = Rect::new(TOOLTIP_BTN_RECT.x, TOOLTIP_BTN_RECT.y, TOOLTIP_BTN_RECT.w + 1, TOOLTIP_BTN_RECT.h + 1).contains(x, y);
        self.scroll_mouse_move(y);
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let (w, h) = PAGE_SIZE;
        list.draw_texture(x, y, w, h, 0, 0, w, h, QUEST_BACK, 1.0);
        list.draw_text(x + TITLE_POS.0, y + TITLE_POS.1, TEXT, &self.accepted_label, font);
        let counter = format!("({}/{})", self.count.unwrap_or(self.quests.len() as i32), MAX_QUESTS);
        list.draw_text(x + COUNTER_POS.0, y + COUNTER_POS.1, GOLD, &counter, font);
        let t = TOOLTIP_BTN_RECT;
        list.draw_texture(x + t.x, y + t.y, t.w, t.h, 0, 0, t.w, t.h, TOOLTIP_BTN, 1.0);
        let start = list.quads.len();
        let origin = (x + TREE_RECT.x, y + TREE_RECT.y - self.tree_offset);
        let line = font.line_height as i32;
        let (nodes, total) = self.layout(Some(font));
        for node in &nodes {
            self.paint_node(list, origin, node, font, line);
        }
        list.clip_from(start, Rect::new(x + TREE_RECT.x, y + TREE_RECT.y, TREE_WIDTH, TREE_RECT.h));
        if total > TREE_RECT.h {
            let mut bar = ScrollWnd::new(TREE_RECT.w, TREE_RECT.h);
            bar.slider.set_range(total, TREE_RECT.h);
            bar.slider.set_pos(-self.scroll);
            bar.up_pressed = self.bar.up_pressed;
            bar.down_pressed = self.bar.down_pressed;
            bar.paint(list, x + TREE_RECT.x, y + TREE_RECT.y, TREE_RECT.w, TREE_RECT.h);
        }
        if !self.gm {
            let mut check = crate::button::CheckBox::new(LOCATION_RECT, &self.location_label);
            check.checked = self.location;
            check.paint(list, x, y, font);
        }
        if self.tooltip_hover {
            self.paint_tooltip(list, x, if self.gm { y - 20 } else { y }, font);
        }
        if !self.gm {
            TextButton::new(ABORT_RECT, BTN_NORMAL, BTN_DOWN, &self.abort_label).paint(list, x, y, self.abort_pressed, font);
        }
    }

    fn paint_tooltip(&self, list: &mut DrawList, wx: i32, wy: i32, font: &BitmapFont) {
        let rows: Vec<(&str, &str)> = TIP_ICONS.iter().copied().zip(self.labels.tooltip.iter().map(String::as_str)).collect();
        paint_icon_tip(list, (wx, wy), TIP_ANCHOR, self.screen, &rows, font);
    }

    fn paint_node(&self, list: &mut DrawList, origin: (i32, i32), node: &NodeBox, font: &BitmapFont, line: i32) {
        let (px, py) = (origin.0 + node.x, origin.1 + node.y);
        match node.kind {
            NodeKind::Root(q) => self.paint_root(list, (px, py), &self.quests[q], font, line),
            NodeKind::Step(q, s) => self.paint_step(list, (px, py), &self.quests[q], s, font),
            NodeKind::Body(q, s) => self.paint_body(list, (px, py), &self.quests[q], s, font, line),
        }
    }

    fn paint_root(&self, list: &mut DrawList, (x, y): (i32, i32), quest: &QuestNode, font: &BitmapFont, line: i32) {
        let icon = if quest.expanded { MINUS_BTN } else { PLUS_BTN };
        list.draw_texture(x, y, NODE_ICON, NODE_ICON, 0, 0, NODE_ICON, NODE_ICON, icon, 1.0);
        if quest.expanded {
            let bar = TREE_WIDTH - 47 - 2;
            list.draw_texture(x + NODE_ICON, y + 1, bar, 13, 0, 0, 16, 13, TEXT_SELECT, 1.0);
            list.draw_texture(x + NODE_ICON + TREE_WIDTH - 47 - 2, y + 1, 32, 13, 0, 0, 32, 13, TEXT_SELECT_END, 1.0);
        }
        list.draw_text(x + 19, y + 2, TEXT, &quest.title, font);
        let width = font.measure(&quest.title) as i32;
        if let Some((first, second)) = glyph_textures(quest.quest_type) {
            list.draw_texture(x + width + 24, y, GLYPH, GLYPH, 0, 0, GLYPH, GLYPH, second, 1.0);
            list.draw_texture(x + width + 35, y, GLYPH, GLYPH, 0, 0, GLYPH, GLYPH, first, 1.0);
        }
        let level = level_text(&self.labels.level, quest.level, quest.max_level, &self.labels.no_limit, &self.labels.and_above);
        list.draw_text(x + 19, y + line + 2, GOLD, &level, font);
    }

    fn paint_step(&self, list: &mut DrawList, (x, y): (i32, i32), quest: &QuestNode, index: usize, font: &BitmapFont) {
        let step = &quest.steps[index];
        let icon = if step.expanded { UP_BTN } else { DOWN_BTN };
        list.draw_texture(x, y, NODE_ICON, NODE_ICON, 0, 0, NODE_ICON, NODE_ICON, icon, 1.0);
        list.draw_text(x + 19, y, TEXT, &step.sub_name, font);
        let width = font.measure(&step.sub_name) as i32;
        let info = !step.target_name.is_empty();
        if info {
            list.draw_texture(x + NODE_ICON + width + 10, y, GLYPH, GLYPH, 0, 0, GLYPH, GLYPH, INFO_ICONS[4], 1.0);
        }
        let not_last = index + 1 != quest.steps.len();
        if not_last && step.journal_disp {
            let dx = if info { x + width + NODE_ICON + 26 } else { x + width + NODE_ICON + 10 };
            list.draw_text(dx, y, GOLD, &self.labels.complete, font);
        }
    }

    fn paint_body(&self, list: &mut DrawList, (x, y): (i32, i32), quest: &QuestNode, index: usize, font: &BitmapFont, line: i32) {
        let step = &quest.steps[index];
        let wrap = TREE_WIDTH - BODY_X - BODY_WRAP_MARGIN;
        let rows = description_rows(&step.description, wrap, font);
        let description = description_height(rows.len(), line);
        let total = body_height(rows.len(), step.items.len(), line);
        list.draw_texture(x - 5, y - 2, PANEL_WIDTH, PANEL_EDGE, 0, 0, PANEL_WIDTH, PANEL_EDGE, PANELS[0], 1.0);
        list.draw_texture(x - 5, y + 6, PANEL_WIDTH, total - 16, 0, 0, PANEL_WIDTH, PANEL_EDGE, PANELS[1], 1.0);
        list.draw_texture(x - 5, y + total - 10, PANEL_WIDTH, PANEL_EDGE, 0, 0, PANEL_WIDTH, PANEL_EDGE, PANELS[2], 1.0);
        for (row, text) in rows.iter().enumerate() {
            list.draw_text(x, y + row as i32 * DESCRIPTION_STEP, GRAY, text, font);
        }
        let not_last = index + 1 != quest.steps.len();
        for (item_index, item) in step.items.iter().enumerate() {
            let top = y + description + ITEM_STEP * item_index as i32;
            list.draw_texture(x, top + 5, ITEM_ICON, ITEM_ICON, 0, 0, ITEM_ICON, ITEM_ICON, &item.icon, 1.0);
            list.draw_texture(x - 1, top + 4, 34, 34, 0, 0, 34, 34, MENU_OUTLINE, 1.0);
            list.draw_text(x + 37, top + 5, GOLD, &item.name, font);
            let count = count_text(item.required, item.current, not_last && step.counts_disp, &self.labels);
            list.draw_text(x + 37, top + line + 10, TEXT, &count, font);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let bytes = gly(&[(0x20, &glyphs)], (1024, 128));
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&bytes).unwrap())
    }

    fn view() -> QuestView {
        let mut view = QuestView::with_sysstrings(|id| Some(format!("s{id}")));
        view.set_font(&font());
        view
    }

    #[test]
    fn nc_quest_list_info_wnd_100ebfe0_location_sets_a_sticky_target_until_the_checkbox_is_cleared() {
        let mut view = view();
        let target = QuestTarget { name: "Gatekeeper".into(), loc: [1.0, 2.0, 3.0], id: 5, title: "A Quest".into(), minimap_only: false };
        view.show_info(target.clone());
        assert!(view.minimap_only_flags);
        assert_eq!(view.refresh_target_painted(false), Some(target.clone()));
        assert_eq!(view.refresh_target_painted(true), Some(target.clone()));
        view.toggle_location();
        assert!(!view.minimap_only_flags);
        assert_eq!(view.refresh_target_painted(true), None);
    }

    fn quest(id: i32, steps: usize) -> QuestNode {
        QuestNode {
            quest_id: id,
            title: format!("q{id}"),
            quest_type: 1,
            level: 10,
            max_level: 20,
            steps: (0..steps)
                .map(|index| QuestStep {
                    state: index as i32 + 1,
                    sub_name: format!("step{index}"),
                    description: format!("desc{index}"),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn the_cond_is_a_bit_mask_of_states_or_a_count_of_states() {
        assert_eq!(states_of(0), Vec::<i32>::new());
        assert_eq!(states_of(1), vec![1]);
        assert_eq!(states_of(3), vec![1, 2, 3]);
        assert_eq!(states_of(MASK_FLAG | 4), vec![3]);
        assert_eq!(states_of(MASK_FLAG | 6), vec![2, 3]);
    }

    #[test]
    fn the_description_breaks_on_the_escaped_newlines_of_the_dat() {
        let font = font();
        assert_eq!(description_rows("Bring it.\\nNow.", 600, &font), vec!["Bring it.".to_string(), "Now.".to_string()]);
        assert_eq!(description_rows("Go.\\n", 600, &font), vec!["Go.".to_string(), String::new()]);
    }

    #[test]
    fn end_list_opens_and_selects_the_marked_quest_and_its_last_step() {
        let mut view = view();
        view.quests = vec![quest(1, 2), quest(2, 2)];
        view.apply_marked();
        assert_eq!((view.selected, view.quests[0].expanded), (None, false));
        view.marked = Some(2);
        view.apply_marked();
        assert_eq!((view.selected, view.quests[1].expanded, view.quests[1].selected), (Some(1), true, true));
        assert!(view.quests[1].steps[1].expanded && view.quests[1].steps[1].selected);
        assert!(!view.quests[1].steps[0].expanded);
        assert!(!view.quests[0].expanded);
        assert_eq!(view.abort_target(), Some(2));
        assert_eq!(MAX_QUESTS, 15);
    }

    #[test]
    fn the_layout_stacks_roots_of_34_steps_of_19_and_the_body_after_its_step() {
        let mut view = view();
        view.quests = vec![quest(1, 2), quest(2, 1)];
        assert_eq!(view.content_height(), 68);
        view.quests[0].expanded = true;
        assert_eq!(view.content_height(), 34 + 19 * 2 + 34);
        view.quests[0].steps[0].expanded = true;
        let (nodes, total) = view.nodes();
        let body = nodes.iter().find(|node| matches!(node.kind, NodeKind::Body(0, 0))).unwrap();
        assert_eq!((body.x, body.y), (17, 5 + 34 + 19));
        assert_eq!(total, 34 + 19 + (13 + 10) + 19 + 34);
        let second_root = nodes.iter().find(|node| matches!(node.kind, NodeKind::Root(1))).unwrap();
        assert_eq!((second_root.x, second_root.y), (3, 5 + total - 34));
    }

    #[test]
    fn the_body_height_counts_description_rows_and_items() {
        assert_eq!(description_height(1, 13), 13);
        assert_eq!(description_height(3, 13), 13 + 19 * 2);
        assert_eq!(body_height(2, 3, 13), 13 + 19 + 37 * 3 + 10);
    }

    #[test]
    fn a_click_inverts_the_expansion_selects_it_and_collapses_the_other_roots() {
        let mut view = view();
        view.quests = vec![quest(1, 2), quest(2, 2)];
        view.toggle(QuestHit::Quest(0));
        assert_eq!((view.quests[0].expanded, view.selected), (true, Some(0)));
        view.toggle(QuestHit::Quest(1));
        assert_eq!((view.quests[0].expanded, view.quests[1].expanded, view.selected), (false, true, Some(1)));
        view.toggle(QuestHit::Quest(1));
        assert_eq!((view.quests[1].expanded, view.selected), (false, None));
    }

    #[test]
    fn clicking_a_step_toggles_its_body_and_clears_the_sibling_selection() {
        let mut view = view();
        view.quests = vec![quest(1, 2)];
        view.toggle(QuestHit::Quest(0));
        view.toggle(QuestHit::Step(0, 0));
        view.toggle(QuestHit::Step(0, 1));
        let steps = &view.quests[0].steps;
        assert!(steps[0].expanded && steps[1].expanded);
        assert!(!steps[0].selected && steps[1].selected);
        view.toggle(QuestHit::Step(0, 1));
        assert!(!view.quests[0].steps[1].expanded);
        assert_eq!(view.selected, Some(0));
    }

    #[test]
    fn hit_uses_the_node_window_which_is_icon_plus_title_width() {
        let mut view = view();
        view.quests = vec![quest(1, 2)];
        let (px, py) = (TREE_RECT.x + 4, TREE_RECT.y + 5 + 2);
        assert_eq!(view.hit(px, py), Some(QuestHit::Quest(0)));
        assert_eq!(view.hit(TREE_RECT.x + 150, py), Some(QuestHit::Page));
        view.toggle(QuestHit::Quest(0));
        assert_eq!(view.hit(TREE_RECT.x + 11, TREE_RECT.y + 5 + 34 + 2), Some(QuestHit::Step(0, 0)));
        assert_eq!(view.hit(100, 310), Some(QuestHit::Abort));
        assert_eq!(view.hit(240, 15), Some(QuestHit::Tooltip));
    }

    #[test]
    fn the_wheel_moves_three_steps_of_fifteen_and_stops_at_the_ends() {
        let mut view = view();
        view.quests = (0..12).map(|id| quest(id, 1)).collect();
        assert_eq!(view.content_height(), 12 * 34);
        assert_eq!(view.minimum_scroll(), 274 - 408);
        view.wheel(-120);
        assert_eq!((view.scroll, view.tree_offset), (-45, 45));
        view.wheel(-120 * 10);
        assert_eq!((view.scroll, view.tree_offset), (-134, 134));
        view.wheel(120);
        assert_eq!((view.scroll, view.tree_offset), (-89, 89));
        view.wheel(120 * 10);
        assert_eq!((view.scroll, view.tree_offset), (0, 0));
    }

    #[test]
    fn nc_quest_gara_tree_ctrl_onscrollbar_100ff490_stores_the_rounded_value_without_the_sign_change() {
        let mut view = view();
        view.on_scroll_bar(31);
        assert_eq!((view.scroll, view.tree_offset), (45, -45));
        view.on_scroll_bar(30);
        assert_eq!((view.scroll, view.tree_offset), (30, -30));
    }

    #[test]
    fn the_line_buttons_move_the_tree_by_the_whole_step_even_when_the_scroll_is_clamped() {
        let mut view = view();
        view.quests = (0..9).map(|id| quest(id, 1)).collect();
        assert_eq!(view.minimum_scroll(), 274 - 306);
        view.line_down();
        view.line_down();
        view.line_down();
        assert_eq!(view.scroll, -32);
        assert_eq!(view.tree_offset, 45);
        view.line_up();
        assert_eq!((view.scroll, view.tree_offset), (-17, 30));
    }

    #[test]
    fn paint_draws_panels_icons_counts_and_the_scrollbar() {
        let font = font();
        let mut view = view();
        let mut node = quest(1, 2);
        node.steps[0].items = vec![QuestItem { item_id: 5, icon: "icon.q".into(), name: "Letter".into(), required: 3, current: 1 }];
        node.steps[0].target_name = "Someone".into();
        node.steps[0].journal_disp = true;
        node.steps[0].counts_disp = true;
        node.expanded = true;
        node.steps[0].expanded = true;
        view.quests = vec![node];
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font);
        for texture in [QUEST_BACK, MINUS_BTN, UP_BTN, DOWN_BTN, PANELS[0], PANELS[1], PANELS[2], MENU_OUTLINE, "icon.q", INFO_ICONS[4], INFO_ICONS[1], INFO_ICONS[3], TEXT_SELECT, TEXT_SELECT_END] {
            assert!(list.quads.iter().any(|quad| quad.texture == texture), "{texture}");
        }
        assert!(list.quads.iter().all(|quad| quad.texture != crate::scroll::UP_BTN));
        let root = list.quads.iter().find(|quad| quad.texture == MINUS_BTN).unwrap();
        assert_eq!((root.x, root.y, root.w), (10.0, 32.0, 14.0));
    }

    #[test]
    fn counts_follow_the_six_formats_of_the_binary() {
        let labels = QuestLabels { complete: "C".into(), unlimited: "U".into(), and_above: "+".into(), ..Default::default() };
        assert_eq!(count_text(5, 2, true, &labels), "(C/5)");
        assert_eq!(count_text(5, 2, false, &labels), "(2/5)");
        assert_eq!(count_text(0, 2, true, &labels), "(C/U)");
        assert_eq!(count_text(0, 2, false, &labels), "(2/U)");
        assert_eq!(count_text(-3, 2, true, &labels), "(C/3+)");
        assert_eq!(count_text(-3, 2, false, &labels), "(2/3+)");
    }

    #[test]
    fn the_level_line_has_three_forms_and_the_glyphs_follow_the_quest_type_bits() {
        assert_eq!(level_text("L", 5, 9, "none", "up"), "(L:5~9)");
        assert_eq!(level_text("L", 5, 0, "none", "up"), "(L:5 up)");
        assert_eq!(level_text("L", 0, 0, "none", "up"), "(L:none)");
        assert_eq!(glyph_textures(0), Some((INFO_ICONS[0], INFO_ICONS[3])));
        assert_eq!(glyph_textures(1), Some((INFO_ICONS[1], INFO_ICONS[3])));
        assert_eq!(glyph_textures(2), Some((INFO_ICONS[0], INFO_ICONS[2])));
        assert_eq!(glyph_textures(3), Some((INFO_ICONS[1], INFO_ICONS[2])));
        assert_eq!(glyph_textures(4), None);
    }
}
