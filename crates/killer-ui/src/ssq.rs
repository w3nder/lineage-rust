#[path = "ssq_layout_generated.rs"]
mod layout;

use std::collections::{BTreeMap, BTreeSet};

use crate::button::TextButton;
use crate::color::{Argb, GOLD, TARGET_VALUE, TEXT};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::scroll::{ScrollHit, ScrollWnd};
use crate::tab_control::TabEvents;
use crate::tab_strip::{TabSpec, TabStrip};
use crate::window::{Rect, WindowId};

pub const WINDOW_ID: WindowId = WindowId(2);
pub const SIZE: (i32, i32) = layout::SIZE;
pub const PAGE: Rect = layout::RECT_F0;
pub const PREDICTIONS: Rect = Rect::new(8, 151, 240, 212);
pub const FESTIVALS: Rect = Rect::new(8, 59, 240, 304);
pub const SEALS: Rect = Rect::new(9, 59, 238, 304);
pub const BACK: &str = "L2UI_CH3.SSQWnd.SSQ_back";
pub const STATUS_BACK: &str = "L2UI_CH3.SSQWnd.SSQ_back1";
pub const WINNER_BACK: &str = "L2UI_CH3.SSQWnd.SSQ_back1_1";
pub const TREE_BACK: &str = "L2UI_CH3.SSQWnd.ssq_back2";
pub const STRIPE: &str = "L2UI_CH3.Etc.TextBackline";
pub const TAB_ID: u32 = 0x0fffa0ff;
pub const TOGGLE_ID: u32 = 0xff0f0f11;
pub const REFRESH_STATUS_ID: u32 = 0xff0f0f12;
pub const REFRESH_FESTIVALS_ID: u32 = 0xff00f0a1;
pub const REFRESH_SEALS_ID: u32 = 0xff00f0a0;
pub const SYSSTRINGS: &[u32] = &[
    27, 814, 815, 816, 817, 818, 819, 820, 821, 822, 823, 829, 830, 831, 832, 844, 845, 846, 923, 924, 824, 825, 826,
    828, 833, 834, 835, 836, 837, 838, 839, 840, 882, 931, 932, 933, 934, 936, 937, 938, 939, 940, 941, 942, 943, 944,
    945, 946, 947, 948, 949, 950, 951, 952, 953, 954, 955, 956, 957, 958, 959, 960, 961, 962, 963, 964, 965, 966, 967,
    968,
];

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    (0, crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) * 0.5 - 252.0))
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, y) = default_position(viewport);
    crate::initial_position::InitialPosition {
        enabled: true,
        horizontal_mode: 0,
        vertical_mode: 5,
        x,
        y: (viewport.1 / 2).wrapping_sub(y),
    }
}

pub fn grip() -> Rect {
    Rect::new(0, 0, SIZE.0, crate::frame::TITLE_HEIGHT)
}

pub fn close_button() -> Rect {
    Rect::new(
        SIZE.0 - crate::frame::CLOSE_INSET,
        crate::frame::BUTTON_Y,
        crate::frame::BUTTON_SIZE,
        crate::frame::BUTTON_SIZE,
    )
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Labels {
    pub strings: BTreeMap<u32, String>,
    pub messages: BTreeMap<i32, String>,
}

impl Labels {
    pub fn from_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self {
            strings: SYSSTRINGS.iter().filter_map(|&id| lookup(id).map(|s| (id, s))).collect(),
            messages: BTreeMap::new(),
        }
    }

    pub fn text(&self, id: u32) -> &str {
        self.strings.get(&id).map(String::as_str).unwrap_or_default()
    }

    pub fn message(&self, id: i32) -> &str {
        self.messages.get(&id).map(String::as_str).unwrap_or_default()
    }

    pub fn cabal(&self, id: u8) -> &str {
        self.text(match id {
            1 => 815,
            2 => 814,
            _ => 27,
        })
    }

    pub fn seal(&self, id: u8) -> &str {
        self.text(match id {
            1 => 816,
            2 => 817,
            3 => 818,
            _ => 27,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Summary {
    pub cycle: i32,
    pub period_message: i32,
    pub until_message: i32,
    pub player_cabal: u8,
    pub player_seal: u8,
    pub stones: i32,
    pub adena: i32,
    pub dusk_stone_score: i32,
    pub dusk_festival_score: i32,
    pub dusk_total_score: i32,
    pub dusk_percent: u8,
    pub dawn_stone_score: i32,
    pub dawn_festival_score: i32,
    pub dawn_total_score: i32,
    pub dawn_percent: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Prediction {
    pub id: u8,
    pub owner: u8,
    pub message: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Festival {
    pub id: u8,
    pub level_score: i32,
    pub dusk_score: i32,
    pub dusk_members: Vec<String>,
    pub dawn_score: i32,
    pub dawn_members: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FestivalGroup {
    pub flag: u8,
    pub festivals: Vec<Festival>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seal {
    pub id: u8,
    pub owner: u8,
    pub dawn_percent: u8,
    pub dusk_percent: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Close,
    RequestStatus(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    Close,
    Tab(usize),
    Refresh,
    Toggle,
    Scroll(ScrollHit),
    Group(usize),
    Festival(usize, usize),
    Seal(usize),
}

#[derive(Debug, Clone)]
pub struct SsqView {
    pub id: WindowId,
    pub labels: Labels,
    pub tabs: TabStrip,
    pub prediction_mode: bool,
    pub requested: [bool; 2],
    pub period: u8,
    pub summary: Option<Summary>,
    pub winning_cabal: u8,
    pub predictions: Vec<Prediction>,
    pub scroll: ScrollWnd,
    pub pressed: Option<Hit>,
    pub content_height: i32,
    pub festivals: Vec<FestivalGroup>,
    pub seals: Vec<Seal>,
    pub retain_limit: u8,
    pub claim_limit: u8,
    expanded_groups: BTreeSet<usize>,
    expanded_festivals: BTreeSet<(usize, usize)>,
    expanded_seals: BTreeSet<usize>,
    tree_hits: Vec<(Rect, Hit)>,
    saved_scroll: [Option<(ScrollWnd, i32)>; 3],
    toggle_label: u32,
    layout_font: Option<BitmapFont>,
}

impl SsqView {
    pub fn new(labels: Labels) -> Self {
        let mut tabs = TabStrip::new(TAB_ID, 0);
        for (page, id) in [824, 825, 826].into_iter().enumerate() {
            tabs.add_fixed(TabSpec {
                label: Some(labels.text(id).to_owned()),
                page: Some(page),
                size: (74, 23),
                textures: [
                    Some("L2UI_CH3.TabButton.normal_tab_on".into()),
                    Some("L2UI_CH3.TabButton.normal_tab".into()),
                ],
                spacing: 1,
                tooltip: -1,
                special_attachment: false,
            });
        }
        for button in &mut tabs.buttons {
            button.visible = true;
        }
        Self {
            id: WINDOW_ID,
            labels,
            tabs,
            prediction_mode: false,
            requested: [false; 2],
            period: 0,
            summary: None,
            winning_cabal: 0,
            predictions: Vec::new(),
            scroll: ScrollWnd::new(PREDICTIONS.w, PREDICTIONS.h),
            pressed: None,
            content_height: 0,
            festivals: Vec::new(),
            seals: Vec::new(),
            retain_limit: 0,
            claim_limit: 0,
            expanded_groups: BTreeSet::new(),
            expanded_festivals: BTreeSet::new(),
            expanded_seals: BTreeSet::new(),
            tree_hits: Vec::new(),
            saved_scroll: [None, None, None],
            toggle_label: 937,
            layout_font: None,
        }
    }

    pub fn set_font(&mut self, font: &BitmapFont) {
        self.layout_font = Some(font.clone());
        self.update_layout();
    }

    pub fn update_layout(&mut self) {
        let Some(font) = self.layout_font.as_ref() else {
            return;
        };
        self.tree_hits.clear();
        let Some(rect) = self.viewport() else {
            return;
        };
        let h = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
        let mut total = 0;
        match self.tab() {
            0 => {
                let mut previous_message_height = 0;
                for prediction in &self.predictions {
                    let label = format!("{} : ", self.labels.seal(prediction.id));
                    total += 17 + previous_message_height + 2 * text_height(&label, font);
                    previous_message_height =
                        wrapped_height(self.labels.message(prediction.message), rect.w - 26, font);
                }
            }
            1 => {
                for (group_index, group) in self.festivals.iter().enumerate() {
                    self.tree_hits.push((Rect::new(rect.x + 3, rect.y + 5 + total, 32, 14), Hit::Group(group_index)));
                    total += h + 10;
                    if !self.expanded_groups.contains(&group_index) {
                        continue;
                    }
                    for (index, festival) in group.festivals.iter().enumerate() {
                        self.tree_hits.push((
                            Rect::new(rect.x + 10, rect.y + 5 + total, 32, 14),
                            Hit::Festival(group_index, index),
                        ));
                        total += 2 * text_height(self.labels.text(festival_name_id(festival.id)), font) + 14;
                        if self.expanded_festivals.contains(&(group_index, index)) {
                            total += festival_detail_height(festival, h);
                        }
                    }
                }
            }
            2 => {
                let label_height = text_height(self.labels.text(823), font);
                for (index, seal) in self.seals.iter().enumerate() {
                    self.tree_hits.push((Rect::new(rect.x + 2, rect.y + 5 + total, 32, 14), Hit::Seal(index)));
                    total += 3 * (label_height + 7) + 25;
                    if self.expanded_seals.contains(&index) {
                        total += seal_detail_height(rect.w - 16, seal.id, &self.labels, font);
                    }
                }
            }
            _ => {}
        }
        self.content_height = total;
        self.scroll.slider.set_range(total, rect.h);
        self.scroll_to(self.scroll.slider.pos);
    }

    pub fn tab(&self) -> usize {
        self.tabs.control.current.unwrap_or(0)
    }

    pub fn select_tab(&mut self, index: usize) -> Option<Action> {
        if index > 2 {
            return None;
        }
        let previous = self.tab();
        if previous != index {
            self.saved_scroll[previous] = Some((self.scroll.clone(), self.content_height));
            let _ = self.tabs.control.select(index, true, &mut TabEvents::default());
            if let Some((scroll, height)) = self.saved_scroll[index].take() {
                self.scroll = scroll;
                self.content_height = height;
                self.tree_hits.clear();
            } else {
                self.reset_scroll();
            }
        }
        self.update_layout();
        if index > 0 && !self.requested[index - 1] {
            self.requested[index - 1] = true;
            Some(Action::RequestStatus(index as u8 + 1))
        } else {
            None
        }
    }

    pub fn command(&mut self, id: u32, argument: usize) -> Option<Action> {
        match id {
            TAB_ID => self.select_tab(argument),
            TOGGLE_ID => {
                self.prediction_mode = !self.prediction_mode;
                self.toggle_label = 939;
                self.update_layout();
                Some(Action::RequestStatus(if self.prediction_mode { 4 } else { 1 }))
            }
            REFRESH_STATUS_ID => Some(Action::RequestStatus(if self.prediction_mode { 4 } else { 1 })),
            REFRESH_FESTIVALS_ID => Some(Action::RequestStatus(2)),
            REFRESH_SEALS_ID => Some(Action::RequestStatus(3)),
            _ => None,
        }
    }

    pub fn receive_summary(&mut self, period: u8, summary: Summary) {
        self.period = period;
        self.summary = Some(summary);
        self.prediction_mode = false;
        self.toggle_label = 937;
        self.update_layout();
    }

    pub fn receive_predictions(&mut self, period: u8, winning_cabal: u8, predictions: Vec<Prediction>) {
        self.period = period;
        self.winning_cabal = winning_cabal;
        self.predictions = predictions;
        self.update_layout();
    }

    pub fn receive_festivals(&mut self, period: u8, groups: Vec<FestivalGroup>) {
        self.period = period;
        self.festivals.clear();
        for mut group in groups {
            for festival in &mut group.festivals {
                festival.dusk_members.truncate(9);
                festival.dawn_members.truncate(9);
            }
            if let Some(existing) = self.festivals.iter_mut().find(|existing| existing.flag == group.flag) {
                existing.festivals.extend(group.festivals);
            } else {
                self.festivals.push(group);
            }
        }
        self.expanded_groups.clear();
        self.expanded_festivals.clear();
        self.saved_scroll[1] = None;
        if self.tab() == 1 {
            self.reset_scroll();
        }
        self.update_layout();
    }

    pub fn receive_seals(&mut self, period: u8, retain_limit: u8, claim_limit: u8, seals: Vec<Seal>) {
        self.period = period;
        self.retain_limit = retain_limit;
        self.claim_limit = claim_limit;
        self.seals = seals;
        self.expanded_seals.clear();
        self.saved_scroll[2] = None;
        if self.tab() == 2 {
            self.reset_scroll();
        }
        self.update_layout();
    }

    fn viewport(&self) -> Option<Rect> {
        match self.tab() {
            0 if self.prediction_mode => Some(PREDICTIONS),
            1 => Some(FESTIVALS),
            2 => Some(SEALS),
            _ => None,
        }
    }

    fn reset_scroll(&mut self) {
        let rect = self.viewport().unwrap_or(PREDICTIONS);
        self.scroll = ScrollWnd::new(rect.w, rect.h);
        self.content_height = 0;
        self.tree_hits.clear();
    }

    fn refresh_rect(&self) -> Rect {
        if self.tab() == 0 { Rect::new(51, 372, 76, 23) } else { Rect::new(91, 372, 76, 23) }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<Hit> {
        if close_button().contains(x, y) {
            return Some(Hit::Close);
        }
        for (index, button) in self.tabs.buttons.iter().enumerate() {
            if button.paint.rect.contains(x - 12, y - 25) {
                return Some(Hit::Tab(index));
            }
        }
        if self.refresh_rect().contains(x, y) {
            return Some(Hit::Refresh);
        }
        if self.tab() == 0 && Rect::new(131, 372, 76, 23).contains(x, y) {
            return Some(Hit::Toggle);
        }
        let rect = self.viewport()?;
        if Rect::new(rect.x, rect.y, rect.w - 15, rect.h).contains(x, y)
            && let Some((_, hit)) = self.tree_hits.iter().find(|(rect, _)| rect.contains(x, y + self.scroll_offset())) {
            return Some(*hit);
        }
        self.scroll.hit(rect.w, rect.h, x - rect.x, y - rect.y).map(Hit::Scroll)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> bool {
        self.pressed = self.hit(x, y);
        match self.pressed {
            Some(Hit::Group(index)) if !self.expanded_groups.remove(&index) => {
                self.expanded_groups.insert(index);
            }
            Some(Hit::Festival(group, index)) if !self.expanded_festivals.remove(&(group, index)) => {
                self.expanded_festivals.insert((group, index));
            }
            Some(Hit::Seal(index)) if !self.expanded_seals.remove(&index) => {
                self.expanded_seals.insert(index);
            }
            Some(Hit::Scroll(ScrollHit::Slider(y))) => {
                self.scroll.slider.mouse_down(y);
            }
            Some(Hit::Scroll(ScrollHit::Up)) => self.scroll.up_pressed = true,
            Some(Hit::Scroll(ScrollHit::Down)) => self.scroll.down_pressed = true,
            _ => {}
        }
        self.update_layout();
        self.pressed.is_some()
    }

    pub fn mouse_move(&mut self, _x: i32, y: i32) -> bool {
        if !self.scroll.slider.dragging {
            return false;
        }
        if let Some(rect) = self.viewport() {
            self.scroll.slider.mouse_move(y - rect.y - 15);
        }
        true
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<Action> {
        let hit = self.hit(x, y);
        let pressed = self.pressed.take();
        self.scroll.slider.mouse_up();
        self.scroll.up_pressed = false;
        self.scroll.down_pressed = false;
        if pressed != hit {
            return None;
        }
        match hit {
            Some(Hit::Close) => Some(Action::Close),
            Some(Hit::Tab(index)) => self.select_tab(index),
            Some(Hit::Toggle) => self.command(TOGGLE_ID, 0),
            Some(Hit::Refresh) => self.command(
                match self.tab() {
                    1 => REFRESH_FESTIVALS_ID,
                    2 => REFRESH_SEALS_ID,
                    _ => REFRESH_STATUS_ID,
                },
                0,
            ),
            Some(Hit::Scroll(ScrollHit::Up)) => {
                self.scroll_to(-crate::gara_tree::scroll_line_up(-self.scroll.slider.pos));
                None
            }
            Some(Hit::Scroll(ScrollHit::Down)) => {
                let minimum = self.viewport().map_or(0, |rect| -(self.content_height - rect.h).max(0));
                self.scroll_to(-crate::gara_tree::scroll_line_down(-self.scroll.slider.pos, minimum));
                None
            }
            _ => None,
        }
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        let Some(rect) = self.viewport().filter(|rect| rect.contains(x, y)) else {
            return false;
        };
        let minimum = -(self.content_height - rect.h).max(0);
        self.scroll_to(-crate::gara_tree::scroll_wheel(-self.scroll.slider.pos, minimum, delta));
        true
    }

    pub fn scroll_to(&mut self, position: i32) {
        let Some(rect) = self.viewport() else {
            return;
        };
        self.scroll.slider.set_pos(position.clamp(0, (self.content_height - rect.h).max(0)));
    }

    fn scroll_offset(&self) -> i32 {
        let position = self.scroll.slider.pos;
        if self.tab() == 0 { position } else { crate::gara_tree::scroll_bar_rounded(position) }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        FrameCtrl::titled(self.labels.text(882), true).paint(list, x, y, SIZE.0, SIZE.1, font, 1.0);
        if self.pressed == Some(Hit::Close) {
            let rect = close_button();
            list.draw_texture(
                x + rect.x,
                y + rect.y,
                rect.w,
                rect.h,
                0,
                0,
                rect.w,
                rect.h,
                "L2UI_CH3.FrameCtrl.FrameCloseOnBtn",
                1.0,
            );
        }
        list.draw_texture(x, y + 20, 256, 28, 0, 0, 256, 28, BACK, 1.0);
        self.tabs.paint(list, (x + 12, y + 25), font);
        let back = match self.tab() {
            0 if self.prediction_mode => WINNER_BACK,
            0 => STATUS_BACK,
            _ => TREE_BACK,
        };
        list.draw_texture(x, y + PAGE.y, PAGE.w, PAGE.h, 0, 0, PAGE.w, PAGE.h, back, 1.0);
        if self.tab() == 0 {
            let start = list.quads.len();
            if self.prediction_mode {
                self.paint_winner(list, x, y + PAGE.y, font);
            } else if let Some(summary) = &self.summary {
                paint_summary(list, (x, y + PAGE.y), font, &self.labels, self.period, summary);
            }
            list.clip_from(start, Rect::new(x + PAGE.x, y + PAGE.y, PAGE.w, PAGE.h));
            let button = TextButton::new(
                Rect::new(131, 372, 76, 23),
                "L2ui_ch3.button.Btn1_normal",
                "L2ui_ch3.button.Btn1_normalOn",
                self.labels.text(self.toggle_label),
            );
            button.paint(list, x, y, self.pressed == Some(Hit::Toggle), font);
        }
        if self.tab() == 1 {
            self.paint_festivals(list, x, y, font);
        }
        if self.tab() == 2 {
            self.paint_seals(list, x, y, font);
        }
        let refresh = TextButton::new(
            self.refresh_rect(),
            "L2ui_ch3.button.Btn1_normal",
            "L2ui_ch3.button.Btn1_normalOn",
            self.labels.text(938),
        );
        refresh.paint(list, x, y, self.pressed == Some(Hit::Refresh), font);
    }

    fn paint_winner(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let label = self.labels.text(937);
        list.draw_text(x + 11, y + 13, TEXT, label, font);
        if matches!(self.winning_cabal, 1 | 2) {
            list.draw_text(
                x + 16 + font.measure(label) as i32,
                y + 13,
                GOLD,
                &format!("{} {}", self.labels.cabal(self.winning_cabal), self.labels.text(828)),
                font,
            );
        }
        let message = if self.winning_cabal == 0 {
            self.labels.message(1293).to_owned()
        } else {
            self.labels.message(1288).replace("$s1", self.labels.cabal(self.winning_cabal))
        };
        paint_wrapped(list, (x + 14, y + 32), 226, &message, TARGET_VALUE, font);
        list.draw_text(x + 11, y + 88, TEXT, self.labels.text(940), font);
        let rect = PREDICTIONS;
        let origin = (x + rect.x, y - PAGE.y + rect.y);
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        let mut previous_message_height = 0;
        let mut total = 0;
        for (index, prediction) in self.predictions.iter().enumerate() {
            let label = format!("{} : ", self.labels.seal(prediction.id));
            let height = text_height(&label, font);
            let top = (previous_message_height + 15 + height * 2) * index as i32;
            let dy = origin.1 + top - self.scroll_offset();
            body.draw_text(origin.0 + 6, dy + 3, TARGET_VALUE, &label, font);
            let owner = if matches!(prediction.owner, 1 | 2) {
                self.labels.cabal(prediction.owner)
            } else {
                self.labels.text(936)
            };
            body.draw_text(origin.0 + 6 + font.measure(&label) as i32, dy + 3, GOLD, owner, font);
            let message = self.labels.message(prediction.message);
            paint_wrapped(&mut body, (origin.0 + 10, dy + 9 + height), rect.w - 26, message, TARGET_VALUE, font);
            total += 17 + previous_message_height + height * 2;
            previous_message_height = wrapped_height(message, rect.w - 26, font);
        }
        for stripe in 0..total / 16 {
            if stripe % 2 == 1 {
                body.draw_texture(
                    origin.0,
                    origin.1 + stripe * 17 + 2 - self.scroll_offset(),
                    rect.w,
                    16,
                    0,
                    0,
                    8,
                    15,
                    STRIPE,
                    1.0,
                );
            }
        }
        list.append_clipped(body, Rect::new(origin.0, origin.1, rect.w, rect.h));
        self.scroll.paint(list, origin.0, origin.1, rect.w, rect.h);
    }
}

pub fn paint_summary(
    list: &mut DrawList,
    origin: (i32, i32),
    font: &BitmapFont,
    labels: &Labels,
    period: u8,
    summary: &Summary,
) {
    let (x, y) = origin;
    let h = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
    let label = format!("{} - ", labels.text(833));
    list.draw_text(x + 11, y + 13, TEXT, &label, font);
    list.draw_text(
        x + 11 + font.measure(&label) as i32,
        y + 13,
        GOLD,
        &format!("{} {}", summary.cycle, labels.text(934)),
        font,
    );
    if summary.period_message > 0 {
        list.draw_text(x + 14, y + 34, TARGET_VALUE, labels.message(summary.period_message), font);
    }
    if summary.until_message > 0 {
        list.draw_text(x + 14, y + 38 + h, GOLD, labels.message(summary.until_message), font);
    }
    list.draw_text(x + 11, y + 73, TEXT, labels.text(834), font);
    for (id, top, value) in [
        (835, 94, labels.cabal(summary.player_cabal).to_owned()),
        (836, 98 + h, labels.seal(summary.player_seal).to_owned()),
        (931, 102 + h * 2, format!("{}{}", summary.stones, labels.text(932))),
    ] {
        list.draw_text(x + 14, y + top, TARGET_VALUE, labels.text(id), font);
        list.draw_text(x + 100, y + top, GOLD, &value, font);
    }
    list.draw_text(x + 100, y + 106 + h * 3, GOLD, &format!("({}{})", summary.adena, labels.text(933)), font);
    list.draw_text(x + 11, y + 168, TEXT, labels.text(824), font);
    list.draw_text(
        x + 11 + font.measure(labels.text(824)) as i32,
        y + 168,
        GOLD,
        &format!(" - {}", labels.text(if period == 3 { 838 } else { 837 })),
        font,
    );
    for (top, cabal, percent, back, bar) in [
        (192, 2, summary.dawn_percent, "L2UI_CH3.SSQWnd.ssq_bar2back", "L2UI_CH3.SSQWnd.ssq_bar22"),
        (210, 1, summary.dusk_percent, "L2UI_CH3.SSQWnd.ssq_bar1back", "L2UI_CH3.SSQWnd.ssq_bar12"),
    ] {
        list.draw_text(x + 14, y + top, GOLD, labels.cabal(cabal), font);
        list.draw_texture(x + 92, y + top, 150, 11, 0, 0, 8, 11, back, 1.0);
        list.draw_texture(x + 92, y + top, (f32::from(percent) * 1.5) as i32, 11, 0, 0, 8, 11, bar, 1.0);
    }
    list.draw_text(x + 82, y + 237, TARGET_VALUE, labels.cabal(2), font);
    list.draw_text(x + 166, y + 237, TARGET_VALUE, labels.cabal(1), font);
    for (index, id) in [839, 825, 840].into_iter().enumerate() {
        list.draw_text(x + 14, y + 252 + index as i32 * (h + 8), TARGET_VALUE, labels.text(id), font);
    }
    for (left, values) in [
        (78, [summary.dawn_stone_score, summary.dawn_festival_score, summary.dawn_total_score]),
        (166, [summary.dusk_stone_score, summary.dusk_festival_score, summary.dusk_total_score]),
    ] {
        for (index, value) in values.into_iter().enumerate() {
            let top = 250 + index as i32 * 19;
            list.slices3_h(
                x + left,
                y + top,
                79,
                17,
                ("L2UI_CH3.Etc.TextBox1", 8),
                "L2UI_CH3.Etc.TextBox2",
                ("L2UI_CH3.Etc.TextBox3", 8),
                17,
                1.0,
            );
            let text = value.to_string();
            list.draw_text(x + left + 77 - font.measure(&text) as i32, y + top + 9 - h / 2, TARGET_VALUE, &text, font);
        }
    }
}

fn wrapped_height(text: &str, width: i32, font: &BitmapFont) -> i32 {
    crate::text_align::measure_height(text, width, font, 1, false, true)
}

fn paint_wrapped(list: &mut DrawList, origin: (i32, i32), width: i32, text: &str, color: Argb, font: &BitmapFont) {
    crate::text_align::paint(
        list,
        origin,
        &crate::text_align::bitmap_lines(text, width, font, 1, false),
        color,
        font,
        true,
    );
}

impl SsqView {
    fn paint_festivals(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let rect = FESTIVALS;
        let ox = x + rect.x;
        let oy = y + rect.y - self.scroll_offset();
        let h = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        let mut top = 5;
        for (group_index, group) in self.festivals.iter().enumerate() {
            let expanded = self.expanded_groups.contains(&group_index);
            paint_tree_icon(&mut body, (ox + 3, oy + top), expanded, false);
            if expanded {
                paint_tree_selection(&mut body, (ox + 17, oy + top + 1), rect.w - 16);
            }
            if group.flag == 0 {
                body.draw_text(ox + 22, oy + top + 2, TEXT, self.labels.text(845), font);
            }
            top += h + 10;
            if !expanded {
                continue;
            }
            for (index, festival) in group.festivals.iter().enumerate() {
                let expanded = self.expanded_festivals.contains(&(group_index, index));
                paint_tree_icon(&mut body, (ox + 10, oy + top), expanded, true);
                let name = self.labels.text(festival_name_id(festival.id));
                let name_height = text_height(name, font);
                body.draw_text(ox + 29, oy + top + 2, TARGET_VALUE, name, font);
                let result = if self.period == 1 {
                    self.labels.text(829).to_owned()
                } else if festival.dawn_score > festival.dusk_score {
                    format!("({} {})", self.labels.text(923), self.labels.text(828))
                } else if festival.dawn_score < festival.dusk_score {
                    format!("({} {})", self.labels.text(924), self.labels.text(828))
                } else {
                    format!("({})", self.labels.text(846))
                };
                body.draw_text(ox + 34 + font.measure(name) as i32, oy + top + 2, GOLD, &result, font);
                let label = format!("{} : ", self.labels.text(831));
                body.draw_text(ox + 29, oy + top + name_height + 6, TARGET_VALUE, &label, font);
                body.draw_text(
                    ox + 29 + font.measure(&label) as i32,
                    oy + top + name_height + 6,
                    GOLD,
                    &festival.level_score.to_string(),
                    font,
                );
                top += name_height * 2 + 14;
                if expanded {
                    top += paint_festival_detail(&mut body, (ox + 17, oy + top), font, &self.labels, festival);
                }
            }
        }
        self.finish_tree(list, body, (x, y), rect);
    }

    fn finish_tree(&self, list: &mut DrawList, body: DrawList, origin: (i32, i32), rect: Rect) {
        list.append_clipped(body, Rect::new(origin.0 + rect.x, origin.1 + rect.y, rect.w - 16, rect.h));
        self.scroll.paint(list, origin.0 + rect.x, origin.1 + rect.y, rect.w, rect.h);
    }

    fn paint_seals(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let rect = SEALS;
        let ox = x + rect.x;
        let oy = y + rect.y - self.scroll_offset();
        let h = text_height(self.labels.text(823), font);
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        let mut top = 5;
        for (index, seal) in self.seals.iter().enumerate() {
            let expanded = self.expanded_seals.contains(&index);
            paint_tree_icon(&mut body, (ox + 2, oy + top), expanded, false);
            if expanded {
                paint_tree_selection(&mut body, (ox + 16, oy + top + 1), rect.w - 16);
            }
            body.draw_text(ox + 20, oy + top + 2, TEXT, self.labels.seal(seal.id), font);
            let label = self.labels.text(823);
            body.draw_text(ox + 2, oy + top + h + 7, TARGET_VALUE, label, font);
            body.draw_text(
                ox + 6 + font.measure(label) as i32,
                oy + top + h + 7,
                GOLD,
                self.labels.cabal(seal.owner),
                font,
            );
            for (row, cabal, percent) in [(2, 2, seal.dusk_percent), (3, 1, seal.dawn_percent)] {
                let by = oy + top + row * (h + 7);
                body.draw_text(ox + 2, by, TARGET_VALUE, self.labels.cabal(cabal), font);
                let limit = if seal.owner == cabal { self.retain_limit } else { self.claim_limit };
                paint_seal_bar(&mut body, (ox + 80, by), cabal, percent, limit);
                let text = format!("{percent}%");
                body.draw_text(
                    ox + 80 + 70 - font.measure(&text) as i32 / 2,
                    by + if row == 2 { 1 } else { 0 },
                    TEXT,
                    &text,
                    font,
                );
            }
            top += 3 * (h + 7) + 25;
            if expanded {
                top += paint_seal_detail(&mut body, (ox + 2, oy + top), rect.w - 16, seal.id, &self.labels, font);
            }
        }
        self.finish_tree(list, body, (x, y), rect);
    }
}

fn festival_name_id(festival: u8) -> u32 {
    match festival {
        1 => 819,
        2 => 820,
        3 => 821,
        4 => 844,
        5 => 822,
        _ => 27,
    }
}

fn text_height(text: &str, font: &BitmapFont) -> i32 {
    text.chars().filter_map(|c| font.glyph(c)).map(|glyph| glyph.height as i32).max().unwrap_or(0)
}

fn paint_tree_icon(list: &mut DrawList, origin: (i32, i32), expanded: bool, child: bool) {
    let texture = match (child, expanded) {
        (false, false) => "L2UI_CH3.QUESTWND.QuestWndPlusBtn",
        (false, true) => "L2UI_CH3.QUESTWND.QuestWndMinusBtn",
        (true, false) => "L2UI_CH3.QUESTWND.QuestWndDownBtn",
        (true, true) => "L2UI_CH3.QUESTWND.QuestWndUpBtn",
    };
    list.draw_texture(origin.0, origin.1, 14, 14, 0, 0, 14, 14, texture, 1.0);
}

fn paint_tree_selection(list: &mut DrawList, origin: (i32, i32), width: i32) {
    list.draw_texture(origin.0, origin.1, width - 49, 13, 0, 0, 16, 13, "L2UI_CH3.ListCtrl.TextSelect", 1.0);
    list.draw_texture(origin.0 + width - 49, origin.1, 32, 13, 0, 0, 32, 13, "L2UI_CH3.ListCtrl.TextSelect2", 1.0);
}

fn paint_tree_panel(
    list: &mut DrawList,
    origin: (i32, i32),
    width: i32,
    height: i32,
    stripe_step: i32,
    stripe_divisor: i32,
) {
    list.slices3_v(
        origin.0 - 5,
        origin.1 - 3,
        width,
        height,
        ("L2UI_CH3.QUESTWND.QUESTWNDPANEL1", 8),
        "L2UI_CH3.QUESTWND.QUESTWNDPANEL2",
        ("L2UI_CH3.QUESTWND.QUESTWNDPANEL3", 8),
        width,
        1.0,
    );
    for row in 0..height / stripe_divisor {
        if row % 2 == 1 {
            list.draw_texture(origin.0 - 5, origin.1 - 2 + row * stripe_step, width, 15, 0, 0, 8, 15, STRIPE, 1.0);
        }
    }
}

fn festival_detail_height(festival: &Festival, glyph_height: i32) -> i32 {
    (festival.dawn_members.iter().filter(|name| !name.is_empty()).count()
        + festival.dusk_members.iter().filter(|name| !name.is_empty()).count()
        + 7) as i32
        * (glyph_height + 4)
}

fn seal_description(seal: u8, labels: &Labels) -> &str {
    labels.message(match seal {
        1 => 1178,
        2 => 1179,
        3 => 1180,
        _ => 27,
    })
}

fn seal_pairs(seal: u8) -> i32 {
    match seal {
        1 => 8,
        2 => 6,
        _ => 0,
    }
}

fn seal_detail_height(width: i32, seal: u8, labels: &Labels, font: &BitmapFont) -> i32 {
    let description_height = wrapped_height(seal_description(seal, labels), width, font);
    let h = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
    description_height + 24 + seal_pairs(seal) * 2 * (h + 6)
}

fn paint_festival_detail(
    list: &mut DrawList,
    origin: (i32, i32),
    font: &BitmapFont,
    labels: &Labels,
    festival: &Festival,
) -> i32 {
    let h = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
    let dawn_count = festival.dawn_members.iter().filter(|name| !name.is_empty()).count();
    let height = festival_detail_height(festival, h);
    paint_tree_panel(list, origin, 211, height, 16, 15);
    for (cabal, score, members, row) in [
        (2, festival.dawn_score, &festival.dawn_members, 0),
        (1, festival.dusk_score, &festival.dusk_members, dawn_count + 4),
    ] {
        let (x, y) = (origin.0, origin.1 + row as i32 * (h + 4));
        list.draw_text(x, y, TARGET_VALUE, labels.cabal(cabal), font);
        list.draw_text(x, y + text_height(labels.text(830), font) + 4, TARGET_VALUE, labels.text(830), font);
        list.draw_text(x + font.measure(labels.text(830)) as i32 + 4, y + h + 4, GOLD, &score.to_string(), font);
        list.draw_text(x, y + 2 * (h + 4), TARGET_VALUE, labels.text(832), font);
        let count = members.iter().filter(|name| !name.is_empty()).count();
        for (index, name) in members.iter().take(count).enumerate() {
            list.draw_text(x, y + (index as i32 + 3) * (h + 4), GOLD, name, font);
        }
    }
    height
}

fn paint_seal_bar(list: &mut DrawList, origin: (i32, i32), cabal: u8, percent: u8, limit: u8) {
    let (back, base, excess) = if cabal == 2 {
        ("L2UI_CH3.SSQWnd.ssq_bar2back", "L2UI_CH3.SSQWnd.ssq_bar21", "L2UI_CH3.SSQWnd.ssq_bar22")
    } else {
        ("L2UI_CH3.SSQWnd.ssq_bar1back", "L2UI_CH3.SSQWnd.ssq_bar11", "L2UI_CH3.SSQWnd.ssq_bar12")
    };
    let length = |value: u8| value as f64 * f64::from(0.01_f32) * 140.0;
    let width = length(percent) as i32;
    let threshold = length(limit) as f32;
    let base_width = if percent > limit { threshold as i32 } else { width };
    list.draw_texture(origin.0, origin.1, 140, 11, 0, 0, 8, 11, back, 1.0);
    list.draw_texture(origin.0, origin.1, base_width, 11, 0, 0, 8, 11, base, 1.0);
    if percent > limit {
        list.draw_texture(
            origin.0 + threshold as i32,
            origin.1,
            length(percent - limit) as i32,
            11,
            0,
            0,
            8,
            11,
            excess,
            1.0,
        );
    }
    list.draw_texture(
        origin.0 + (length(limit) + 80.0) as i32 - 80,
        origin.1,
        1,
        11,
        0,
        0,
        1,
        11,
        "L2UI_CH3.SSQWnd.ssq_barline",
        1.0,
    );
}

fn paint_seal_detail(
    list: &mut DrawList,
    origin: (i32, i32),
    width: i32,
    seal: u8,
    labels: &Labels,
    font: &BitmapFont,
) -> i32 {
    let description = seal_description(seal, labels);
    let description_height = wrapped_height(description, width, font);
    let h = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
    let pairs = seal_pairs(seal);
    let height = seal_detail_height(width, seal, labels, font);
    for (top, part_height, texture) in [
        (origin.1 - 3, 8, "L2UI_CH3.QUESTWND.QUESTWNDPANEL1"),
        (origin.1 + 5, height - 16, "L2UI_CH3.QUESTWND.QUESTWNDPANEL2"),
        (origin.1 + height - 11, 8, "L2UI_CH3.QUESTWND.QUESTWNDPANEL3"),
    ] {
        list.draw_texture(origin.0 - 5, top, width + 14, part_height, 0, 0, width, 8, texture, 1.0);
    }
    for row in 0..height / 17 {
        if row % 2 == 1 {
            list.draw_texture(origin.0 - 5, origin.1 - 2 + row * 18, width, 15, 0, 0, 8, 15, STRIPE, 1.0);
        }
    }
    paint_wrapped(list, origin, width, description, TARGET_VALUE, font);
    let first = if seal == 1 { 941 } else { 957 };
    for pair in 0..pairs {
        let top = origin.1 + description_height + 24 + pair * 2 * (h + 6);
        list.draw_text(origin.0, top, TARGET_VALUE, labels.text((first + pair * 2) as u32), font);
        list.draw_text(origin.0, top + h + 6, TARGET_VALUE, ":", font);
        list.draw_text(origin.0 + 4, top + h + 6, GOLD, labels.text((first + pair * 2 + 1) as u32), font);
    }
    height
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn labels() -> Labels {
        let mut labels = Labels::from_sysstrings(|id| Some(format!("sys{id}")));
        labels.messages.insert(1288, "Winner $s1".into());
        labels.messages.insert(1293, "No winner".into());
        labels.messages.insert(100, "Period".into());
        labels.messages.insert(101, "Until".into());
        labels
    }

    #[test]
    fn status_100fbe40_clips_long_messages_to_its_page() {
        let long = "Wide message ".repeat(60);
        let mut labels = labels();
        labels.messages.insert(100, long.clone());
        labels.messages.insert(1293, long.clone());
        let mut view = SsqView::new(labels);
        view.receive_summary(0, Summary { period_message: 100, ..Summary::default() });
        for prediction in [false, true] {
            view.prediction_mode = prediction;
            let mut list = DrawList { capture_text: true, ..DrawList::default() };
            view.paint(&mut list, 16, 20, &font());
            let page = Rect::new(16 + PAGE.x, 20 + PAGE.y, PAGE.w, PAGE.h);
            let runs: Vec<_> = list.text_runs.iter().filter(|run| run.clip == Some(page)).collect();
            assert!(!runs.is_empty());
            for run in runs {
                for quad in &list.quads[run.quads.clone()] {
                    assert!(quad.x >= page.x as f32);
                    assert!(quad.x + quad.w <= (page.x + page.w) as f32);
                    assert!(quad.y >= page.y as f32);
                    assert!(quad.y + quad.h <= (page.y + page.h) as f32);
                }
            }
        }
    }

    #[test]
    fn main_board_100fb930_requests_each_secondary_tab_once() {
        let mut view = SsqView::new(labels());
        assert_eq!(view.select_tab(0), None);
        assert_eq!(view.select_tab(1), Some(Action::RequestStatus(2)));
        assert_eq!(view.select_tab(2), Some(Action::RequestStatus(3)));
        for tab in [0, 1, 2, 1, 2] {
            assert_eq!(view.select_tab(tab), None);
        }
        assert_eq!(view.select_tab(3), None);
        assert_eq!(view.tab(), 2);
    }

    #[test]
    fn status_100fb750_toggle_and_100fb7d0_refresh_preserve_mode() {
        let mut view = SsqView::new(labels());
        assert_eq!(view.command(REFRESH_STATUS_ID, 0), Some(Action::RequestStatus(1)));
        assert_eq!(view.command(TOGGLE_ID, 0), Some(Action::RequestStatus(4)));
        assert_eq!(view.command(REFRESH_STATUS_ID, 0), Some(Action::RequestStatus(4)));
        assert!(view.prediction_mode);
        assert_eq!(view.command(TOGGLE_ID, 0), Some(Action::RequestStatus(1)));
        assert!(!view.prediction_mode);
        assert_eq!(view.toggle_label, 939);
        view.receive_summary(0, Summary::default());
        assert_eq!(view.toggle_label, 937);
        assert_eq!(view.command(REFRESH_FESTIVALS_ID, 0), Some(Action::RequestStatus(2)));
        assert_eq!(view.command(REFRESH_SEALS_ID, 0), Some(Action::RequestStatus(3)));
        assert_eq!(view.command(0, 0), None);
    }

    #[test]
    fn summary_100fa680_keeps_all_six_values_and_bar_uvs() {
        let mut view = SsqView::new(labels());
        view.receive_summary(
            3,
            Summary {
                cycle: 7,
                period_message: 100,
                until_message: 101,
                player_cabal: 2,
                player_seal: 3,
                stones: 500,
                adena: 250,
                dusk_stone_score: 11,
                dusk_festival_score: 12,
                dusk_total_score: 23,
                dusk_percent: 40,
                dawn_stone_score: 21,
                dawn_festival_score: 22,
                dawn_total_score: 43,
                dawn_percent: 60,
            },
        );
        let mut list = DrawList { capture_text: true, ..Default::default() };
        view.paint(&mut list, 100, 50, &font());
        for text in ["11", "12", "23", "21", "22", "43", "Period", "Until", "500sys932", "(250sys933)", " - sys838"] {
            assert!(list.text_runs.iter().any(|run| run.text == text), "{text}");
        }
        let bars: Vec<_> = list
            .quads
            .iter()
            .filter(|q| q.texture.ends_with("ssq_bar22") || q.texture.ends_with("ssq_bar12"))
            .collect();
        assert_eq!(bars.len(), 2);
        assert_eq!(
            (bars[0].x, bars[0].y, bars[0].w, bars[0].h, bars[0].uw, bars[0].vh),
            (192.0, 290.0, 90.0, 11.0, 8.0, 11.0)
        );
        assert_eq!((bars[1].x, bars[1].y, bars[1].w), (192.0, 308.0, 60.0));
        assert_eq!(
            list.quads.iter().filter(|q| q.texture.ends_with("TextBox2") && q.uw == 8.0 && q.vh == 17.0).count(),
            6
        );
    }

    #[test]
    fn winner_100fb3d0_and_predictions_100fb990_use_received_messages() {
        let mut view = SsqView::new(labels());
        view.command(TOGGLE_ID, 0);
        view.receive_predictions(
            3,
            1,
            vec![Prediction { id: 3, owner: 2, message: 100 }, Prediction { id: 1, owner: 0, message: 101 }],
        );
        let mut list = DrawList { capture_text: true, ..Default::default() };
        view.paint(&mut list, 0, 0, &font());
        for text in ["Winner sys815", "sys818 : ", "sys814", "sys816 : ", "sys936", "Period", "Until"] {
            assert!(list.text_runs.iter().any(|run| run.text == text), "{text}");
        }
        view.receive_predictions(3, 0, Vec::new());
        let mut list = DrawList { capture_text: true, ..Default::default() };
        view.paint(&mut list, 0, 0, &font());
        assert!(list.text_runs.iter().any(|run| run.text == "No winner"));
    }

    #[test]
    fn mouse_dispatch_uses_c4_rectangles_and_release_cancellation() {
        let mut view = SsqView::new(labels());
        assert_eq!(view.hit(90, 26), Some(Hit::Tab(1)));
        assert!(view.mouse_down(132, 374));
        assert_eq!(view.mouse_up(10, 350), None);
        assert!(!view.prediction_mode);
        assert!(view.mouse_down(132, 374));
        assert_eq!(view.mouse_up(132, 374), Some(Action::RequestStatus(4)));
        assert_eq!(view.hit(52, 374), Some(Hit::Refresh));
    }

    #[test]
    fn all_wire_cabal_and_seal_values_follow_native_name_fallback() {
        let labels = labels();
        for id in 0..=u8::MAX {
            assert_eq!(
                labels.cabal(id),
                match id {
                    1 => "sys815",
                    2 => "sys814",
                    _ => "sys27",
                }
            );
            assert_eq!(
                labels.seal(id),
                match id {
                    1 => "sys816",
                    2 => "sys817",
                    3 => "sys818",
                    _ => "sys27",
                }
            );
        }
    }
    #[test]
    fn tree_scroll_keeps_each_page_and_native_fifteen_pixel_offset() {
        let mut view = SsqView::new(labels());
        view.select_tab(1);
        view.content_height = 1000;
        view.scroll.slider.set_range(1000, 304);
        view.scroll_to(16);
        assert_eq!(view.scroll_offset(), 30);
        view.select_tab(2);
        assert_eq!(view.scroll_offset(), 0);
        view.content_height = 800;
        view.scroll.slider.set_range(800, 304);
        view.scroll_to(47);
        assert_eq!(view.scroll_offset(), 60);
        view.select_tab(1);
        assert_eq!(view.scroll.slider.pos, 16);
        assert_eq!(view.content_height, 1000);
        view.select_tab(2);
        assert_eq!(view.scroll.slider.pos, 47);
    }

    #[test]
    fn festival_nodes_expand_on_press_and_packet_rebuild_collapses_them() {
        let mut view = SsqView::new(labels());
        let groups = vec![FestivalGroup {
            flag: 0,
            festivals: vec![Festival {
                id: 4,
                level_score: 100,
                dusk_score: 10,
                dawn_score: 20,
                dusk_members: vec!["Dusk".into()],
                dawn_members: (0..12).map(|i| format!("Member{i}")).collect(),
            }],
        }];
        view.receive_festivals(3, groups.clone());
        view.select_tab(1);
        let font = font();
        view.set_font(&font);
        let mut list = DrawList { capture_text: true, ..Default::default() };
        view.paint(&mut list, 0, 0, &font);
        assert_eq!(view.content_height, 23);
        assert_eq!(view.hit(12, 65), Some(Hit::Group(0)));
        view.mouse_down(12, 65);
        view.paint(&mut list, 0, 0, &font);
        assert_eq!(view.content_height, 63);
        assert!(list.text_runs.iter().any(|run| run.text == "sys844"));
        assert!(list.text_runs.iter().any(|run| run.text == "(sys923 sys828)"));
        assert_eq!(view.hit(19, 88), Some(Hit::Festival(0, 0)));
        view.mouse_down(19, 88);
        view.paint(&mut list, 0, 0, &font);
        assert_eq!(view.content_height, 63 + 17 * 17);
        assert_eq!(view.festivals[0].festivals[0].dawn_members.len(), 9);
        view.receive_festivals(3, groups);
        assert!(view.expanded_groups.is_empty());
        assert!(view.expanded_festivals.is_empty());
    }

    #[test]
    fn seals_100fe700_match_native_x87_threshold_and_excess() {
        let mut list = DrawList::default();
        paint_seal_bar(&mut list, (80, 45), 2, 60, 20);
        let quads: Vec<_> = list.quads.iter().map(|q| (q.x, q.y, q.w, q.h)).collect();
        assert_eq!(
            quads,
            [(80.0, 45.0, 140.0, 11.0), (80.0, 45.0, 28.0, 11.0), (108.0, 45.0, 55.0, 11.0), (107.0, 45.0, 1.0, 11.0)]
        );
        let mut list = DrawList::default();
        paint_seal_bar(&mut list, (80, 65), 1, 40, 35);
        assert_eq!(list.quads[1].w, 49.0);
        assert_eq!((list.quads[2].x, list.quads[2].w), (129.0, 6.0));
        assert_eq!(list.quads[3].x, 128.0);
    }

    #[test]
    fn seal_details_use_all_benefits_and_native_panel_dimensions() {
        let mut labels = labels();
        for id in [1178, 1179, 1180] {
            labels.messages.insert(id, "Description".into());
        }
        for (seal, first, count) in [(1, 941, 16), (2, 957, 12), (3, 0, 0)] {
            let mut list = DrawList { capture_text: true, ..Default::default() };
            let height = paint_seal_detail(&mut list, (11, 150), 222, seal, &labels, &font());
            assert_eq!(height, 37 + count * 19);
            for id in first..first + count {
                assert!(list.text_runs.iter().any(|run| run.text == format!("sys{id}")));
            }
            let top = &list.quads[0];
            assert_eq!((top.x, top.y, top.w, top.h, top.uw, top.vh), (6.0, 147.0, 236.0, 8.0, 222.0, 8.0));
        }
    }
    #[test]
    fn close_requires_release_inside_native_frame_button() {
        let mut view = SsqView::new(labels());
        assert_eq!(grip(), Rect::new(0, 0, 256, 20));
        assert_eq!(close_button(), Rect::new(233, 3, 15, 15));
        assert_eq!(view.hit(234, 4), Some(Hit::Close));
        view.mouse_down(234, 4);
        assert_eq!(view.mouse_up(232, 4), None);
        view.mouse_down(234, 4);
        assert_eq!(view.mouse_up(234, 4), Some(Action::Close));
    }

    #[test]
    fn initial_position_preserves_native_x87_conversion_and_anchor() {
        for (height, y) in [(768, 132), (933, 214), (503, 0), (501, -1), (-1, -252), (16_777_217, 8_388_356)] {
            assert_eq!(default_position((1558, height)), (0, y));
            let position = initial_position((1558, height));
            assert_eq!((position.horizontal_mode, position.vertical_mode), (0, 5));
            assert_eq!((height / 2).wrapping_sub(position.y), y);
        }
    }

    #[test]
    fn immutable_paint_uses_layout_updated_before_drawing() {
        let font = font();
        let mut view = SsqView::new(labels());
        view.set_font(&font);
        view.receive_seals(3, 20, 35, vec![Seal { id: 1, owner: 2, dawn_percent: 40, dusk_percent: 60 }]);
        view.select_tab(2);
        assert_eq!(view.hit(12, 65), Some(Hit::Seal(0)));
        assert_eq!(view.content_height, 85);
        view.mouse_down(12, 65);
        assert_eq!(view.content_height, 426);
        let height = view.content_height;
        let hits = view.tree_hits.clone();
        let view = &view;
        for _ in 0..2 {
            view.paint(&mut DrawList::default(), 0, 0, &font);
            assert_eq!(view.content_height, height);
            assert_eq!(view.tree_hits, hits);
        }
    }

    #[test]
    fn multiline_uses_native_six_pixel_leading_and_a_glyph_height() {
        let font = font();
        assert_eq!(wrapped_height("A\nB\nC", 222, &font), 51);
        let mut list = DrawList { capture_text: true, ..Default::default() };
        paint_wrapped(&mut list, (10, 20), 222, "A\nB\nC", TEXT, &font);
        assert_eq!(list.text_runs.iter().map(|run| run.rect[1]).collect::<Vec<_>>(), [20.0, 39.0, 58.0]);
    }

    #[test]
    #[ignore]
    fn client_smallfont_matches_every_fixture_glyph_and_ssq_layout() {
        let client = std::env::var("KILLER_CLIENT_DIR").unwrap();
        let bytes = std::fs::read(std::path::Path::new(&client).join("system/smallfont-e.gly")).unwrap();
        let glyphs = Glyphs::parse(&bytes).unwrap();
        let fixture = Glyphs::parse(include_bytes!("../tests/fixtures/smallfont-e.gly")).unwrap();
        assert_eq!(glyphs, fixture);
        let font = BitmapFont::new("L2Font-e.SmallFont-e".into(), glyphs);
        assert_eq!(font.glyphs.glyphs.len(), 95);
        let mut view = SsqView::new(labels());
        view.set_font(&font);
        view.receive_seals(3, 20, 35, vec![Seal { id: 1, owner: 2, dawn_percent: 40, dusk_percent: 60 }]);
        view.select_tab(2);
        view.mouse_down(12, 65);
        assert_eq!(view.content_height, 426);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font);
        assert!(!list.quads.is_empty());
    }

    #[test]
    #[ignore]
    fn every_glyph_file_of_the_client_lays_out_and_paints_all_ssq_tabs() {
        let client = std::env::var("KILLER_CLIENT_DIR").unwrap();
        let mut checked = 0;
        for name in ["smallfont-e.gly", "smallfont.gly", "largefont-e.gly", "largefont.gly"] {
            let bytes = std::fs::read(std::path::Path::new(&client).join("system").join(name)).unwrap();
            let glyphs = Glyphs::parse(&bytes).unwrap();
            let font = BitmapFont::new(name.into(), glyphs);
            for tab in 0..4 {
                let mut view = SsqView::new(labels());
                view.set_font(&font);
                view.receive_seals(3, 20, 35, vec![Seal { id: 1, owner: 2, dawn_percent: 40, dusk_percent: 60 }]);
                view.select_tab(tab);
                let mut list = DrawList::default();
                view.paint(&mut list, 0, 0, &font);
                assert!(!list.quads.is_empty(), "{name} aba {tab}");
                checked += 1;
            }
        }
        assert_eq!(checked, 16);
    }

    #[test]
    fn tabs_use_normal_tab_on_for_the_unselected_state_as_ncssqmainboard_oncreate_100fc760() {
        let view = SsqView::new(labels());
        for button in &view.tabs.buttons {
            assert_eq!(
                button.paint.textures,
                [Some("L2UI_CH3.TabButton.normal_tab_on".into()), Some("L2UI_CH3.TabButton.normal_tab".into()), None]
            );
        }
    }
}
