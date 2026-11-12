#[path = "gm_inventory_layout_generated.rs"]
mod layout;

use crate::color::{GOLD, TEXT};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::{FrameContract, FrameHit, FrameTitle};
use crate::inventory::{
    ADENA_ICON, ADENA_ICON_POS, ADENA_ICON_SIZE, ADENA_RIGHT, ADENA_Y, BACK_RECT, BODY_PART_BACK, BODY_PART_CHEST, BODY_PART_FEET,
    BODY_PART_FULL_ARMOR, BODY_PART_GLOVES, BODY_PART_HAIR, BODY_PART_HEAD, BODY_PART_L_HAND, BODY_PART_LEGS, BODY_PART_LR_HAND,
    BODY_PART_NECK, BODY_PART_R_HAND, BODY_PART_UNDERWEAR, COUNTER_RIGHT, COUNTER_Y, DUAL_CAP, HENNA_MARK, HENNA_MARK_SIZE, HENNA_MARK_X,
    HENNA_MARK_Y, HENNA_SLOTS_MAX, HENNA_EMPTY, HENNA_EMPTY_X, HENNA_ICON_SIZE, HENNA_ICON_X, HENNA_ICON_Y, BLACK_TEXTURE, HENNA_STEP, INVENTORY_BACK, InventoryItemView, InventoryTab, PAPERDOLL_RECT,
    QUEST_TYPE2, SLOT, SLOT_FULL_ARMOR, SLOT_R_HAND, SLOT_TWO_HANDED, TAB_ACTIVE, TAB_NORMAL, WEIGHT_ICON, WEIGHT_ICON_POS, icon, item_wnd_wheel,
    paperdoll_slot_at, paperdoll_slot_pos, slot_frame, tab_rect, thousands,
};
use crate::scroll::{ScrollHit, ScrollWnd, rows_for};
use crate::window::{Rect, WindowId};

pub const ID: WindowId = WindowId::GM_INVENTORY;
pub const SIZE: (i32, i32) = layout::SIZE;
pub const FRAME_ID: u32 = 0xf0f00d;
pub const TAB_CONTROL_ID: u32 = 0xf0f00e;
pub const MAX_EQUIPPED: usize = 15;
pub const GRID_COLS: usize = 6;
pub const GRID_ROWS: usize = 4;
pub const GRID_ORIGIN: (i32, i32) = (1, 1);
pub const PAINT_STEP: (i32, i32) = (37, 35);
pub const HIT_STEP: (i32, i32) = (SLOT + 1, SLOT + 1);
pub const ADENA_ITEM_ID: i32 = 57;
pub const BOW_WEAPON_KIND: i32 = 6;
pub const AMMO_ETC_TYPES: [i32; 2] = [2, 0x10];
pub const INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 1, vertical_mode: 3, x: 201, y: 100 };

const FRAME: FrameContract = FrameContract {
    rect: layout::RECT_7C,
    id: FRAME_ID,
    mode: crate::frame::FrameMode::Title,
    title: FrameTitle::None,
    close_button: true,
    minimize_button: false,
};

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 - 200.0 - 1.0) as i32, 100)
}

pub fn grip() -> Rect {
    layout::RECT_7C
}

pub fn grid_rect() -> Rect {
    layout::RECT_F8
}

pub fn equipped_rect() -> Rect {
    layout::RECT_100
}

pub fn tabs_rect() -> Rect {
    layout::RECT_104
}

pub fn gm_slot(body_part: i32) -> i32 {
    if body_part & 2 != 0 {
        return 8;
    }
    if body_part & 4 != 0 {
        return 9;
    }
    if body_part & 0x10 != 0 {
        return 13;
    }
    if body_part & 0x20 != 0 {
        return 14;
    }
    match body_part {
        BODY_PART_UNDERWEAR => 0,
        BODY_PART_HEAD => 1,
        BODY_PART_BACK => 2,
        BODY_PART_HAIR => 3,
        BODY_PART_NECK => 4,
        BODY_PART_R_HAND => 5,
        BODY_PART_CHEST => 6,
        BODY_PART_L_HAND => 7,
        BODY_PART_GLOVES => 10,
        BODY_PART_LEGS => 11,
        BODY_PART_FEET => 12,
        BODY_PART_FULL_ARMOR => SLOT_FULL_ARMOR,
        BODY_PART_LR_HAND => SLOT_TWO_HANDED,
        _ => -1,
    }
}

pub fn grid_hit_slot(x: i32, y: i32, scroll_row: usize) -> Option<usize> {
    let (x, y) = (x - GRID_ORIGIN.0, y - GRID_ORIGIN.1);
    if x < 0 || y < 0 || x > HIT_STEP.0 * GRID_COLS as i32 || y > HIT_STEP.1 * GRID_ROWS as i32 {
        return None;
    }
    let col = ((x / HIT_STEP.0) as usize).min(GRID_COLS - 1);
    let row = ((y / HIT_STEP.1) as usize).min(GRID_ROWS - 1);
    Some((scroll_row + row) * GRID_COLS + col)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GmInventoryHit {
    Close,
    Equipped(usize),
    Tab(InventoryTab),
    Scroll(ScrollHit),
    Grid(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GmHenna {
    pub icon: String,
    pub usable: bool,
}

#[derive(Clone, Debug)]
pub struct GmInventoryView {
    pub frame: FrameCtrl,
    pub limit: i32,
    pub tab: InventoryTab,
    pub equipped: Vec<(i32, InventoryItemView)>,
    pub pages: [Vec<InventoryItemView>; 2],
    pub hennas: Vec<GmHenna>,
    pub selected: Option<i32>,
    pub scroll_row: usize,
    pub tab_names: [String; 2],
    scroll: ScrollWnd,
    earring_toggle: bool,
    ring_toggle: bool,
    pressed: Option<GmInventoryHit>,
}

impl GmInventoryView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Self {
            frame: FRAME.build(text),
            limit: 0,
            tab: InventoryTab::Item,
            equipped: Vec::new(),
            pages: [Vec::new(), Vec::new()],
            hennas: Vec::new(),
            selected: None,
            scroll_row: 0,
            tab_names: [text(crate::inventory::TAB_ITEM_SYSSTRING), text(crate::inventory::TAB_QUEST_SYSSTRING)],
            scroll: ScrollWnd::new(layout::RECT_F8.w, layout::RECT_F8.h),
            earring_toggle: false,
            ring_toggle: false,
            pressed: None,
        }
    }

    pub fn clear(&mut self) {
        self.equipped.clear();
        self.pages = [Vec::new(), Vec::new()];
        self.selected = None;
        self.scroll_row = 0;
        self.earring_toggle = false;
        self.ring_toggle = false;
        self.update_rows();
    }

    pub fn clear_hennas(&mut self) {
        self.hennas.clear();
    }

    pub fn add_henna(&mut self, icon: &str, usable: bool) {
        let icon = if icon.is_empty() { BLACK_TEXTURE } else { icon };
        self.hennas.push(GmHenna { icon: icon.to_string(), usable });
    }

    pub fn add_item(&mut self, item: InventoryItemView) {
        if item.equipped {
            self.add_equipped(item);
        } else {
            let page = usize::from(item.quest);
            self.pages[page].push(item);
            self.update_rows();
        }
    }

    fn add_equipped(&mut self, item: InventoryItemView) {
        if self.equipped.len() >= MAX_EQUIPPED {
            return;
        }
        let mut body_part = item.body_part;
        if body_part & 6 != 0 {
            if self.earring_toggle {
                body_part &= !4;
            } else {
                body_part &= !2;
            }
            self.earring_toggle = !self.earring_toggle;
        } else if body_part & 0x30 != 0 {
            if self.ring_toggle {
                body_part &= !0x20;
            } else {
                body_part &= !0x10;
            }
            self.ring_toggle = !self.ring_toggle;
        }
        self.equipped.push((body_part, item));
    }

    pub fn ammo_equipped(&self) -> bool {
        self.equipped.iter().any(|(_, item)| item.tip.as_ref().is_some_and(|tip| AMMO_ETC_TYPES.contains(&tip.etc_family)))
    }

    pub fn counted(&self) -> usize {
        self.equipped.len() + self.pages[0].len() + self.pages[1].len()
    }

    pub fn adena(&self) -> i64 {
        self.pages[0].iter().filter(|item| item.item_id == ADENA_ITEM_ID).map(|item| i64::from(item.count)).sum()
    }

    fn page(&self) -> &[InventoryItemView] {
        &self.pages[self.tab.index()]
    }

    pub fn total_rows(&self) -> i32 {
        rows_for(self.page().len(), GRID_COLS)
    }

    pub fn update_rows(&mut self) {
        let total = self.total_rows();
        let visible = GRID_ROWS as i32;
        self.scroll_row = crate::scroll::clamp_first_row(self.scroll_row as i32, visible, total).max(0) as usize;
        self.scroll.slider.set_range(total, visible);
        self.scroll.slider.set_pos(self.scroll_row as i32);
    }

    pub fn select_tab(&mut self, tab: InventoryTab) {
        self.tab = tab;
        self.scroll_row = 0;
        self.update_rows();
    }

    pub fn wheel(&mut self, x: i32, y: i32, delta: i32) -> bool {
        if !layout::RECT_F8.contains(x, y) {
            return false;
        }
        let first = item_wnd_wheel(self.scroll_row as i32, GRID_ROWS as i32, self.total_rows(), delta);
        self.scroll_row = first.max(0) as usize;
        self.scroll.slider.set_pos(first);
        true
    }

    pub fn equipped_item(&self, slot: usize) -> Option<&InventoryItemView> {
        let wanted = slot as i32;
        self.equipped.iter().map(|(body_part, item)| (gm_slot(*body_part), item)).find_map(|(mapped, item)| {
            (mapped == wanted
                || (mapped == SLOT_TWO_HANDED && wanted == crate::inventory::SLOT_R_HAND)
                || (mapped == SLOT_FULL_ARMOR && wanted == crate::inventory::SLOT_CHEST))
                .then_some(item)
        })
    }

    pub fn grid_item(&self, index: usize) -> Option<&InventoryItemView> {
        self.page().get(index)
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<GmInventoryHit> {
        if !Rect::new(0, 0, SIZE.0, SIZE.1).contains(x, y) {
            return None;
        }
        if FRAME.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(GmInventoryHit::Close);
        }
        if layout::RECT_100.contains(x, y)
            && let Some(slot) = paperdoll_slot_at(x, y)
        {
            return Some(GmInventoryHit::Equipped(slot));
        }
        for tab in [InventoryTab::Item, InventoryTab::Quest] {
            if tab_rect(tab).contains(x, y) {
                return Some(GmInventoryHit::Tab(tab));
            }
        }
        let grid = layout::RECT_F8;
        if let Some(hit) = self.scroll.hit(grid.w, grid.h, x - grid.x, y - grid.y) {
            return Some(GmInventoryHit::Scroll(hit));
        }
        if grid.contains(x, y)
            && let Some(index) = grid_hit_slot(x - grid.x, y - grid.y, self.scroll_row)
        {
            return Some(GmInventoryHit::Grid(index));
        }
        None
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<GmInventoryHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        match hit {
            Some(GmInventoryHit::Equipped(slot)) => self.selected = self.equipped_item(slot).map(|item| item.object_id),
            Some(GmInventoryHit::Grid(index)) => self.selected = self.grid_item(index).map(|item| item.object_id),
            Some(GmInventoryHit::Scroll(ScrollHit::Up)) => {
                self.scroll_row = self.scroll_row.saturating_sub(1);
                self.update_rows();
            }
            Some(GmInventoryHit::Scroll(ScrollHit::Down)) => {
                self.scroll_row += 1;
                self.update_rows();
            }
            _ => {}
        }
        hit
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<bool> {
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            GmInventoryHit::Close => Some(true),
            GmInventoryHit::Tab(tab) => {
                self.select_tab(tab);
                Some(false)
            }
            _ => Some(false),
        }
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32) {
        let (x, y) = origin;
        FRAME.paint(&self.frame, list, origin, font, alpha);
        let back = BACK_RECT;
        list.draw_texture(x + back.x, y + back.y, back.w, back.h, 0, 0, back.w, back.h, INVENTORY_BACK, alpha);
        let adena = thousands(self.adena());
        let adena_width = font.measure(&adena) as i32;
        list.draw_text(x + ADENA_RIGHT - adena_width, y + ADENA_Y, TEXT, &adena, font);
        let (width, height) = ADENA_ICON_SIZE;
        list.draw_texture(x + ADENA_ICON_POS.0, y + ADENA_ICON_POS.1, width, height, 0, 0, width, height, ADENA_ICON, 1.0);
        list.draw_texture(x + WEIGHT_ICON_POS.0, y + WEIGHT_ICON_POS.1, width, height, 0, 0, width, height, WEIGHT_ICON, 1.0);
        let counter = format!("({}/{})", self.counted(), self.limit);
        let counter_width = font.measure(&counter) as i32;
        list.draw_text(x + COUNTER_RIGHT - counter_width, y + COUNTER_Y, GOLD, &counter, font);
        self.paint_equipped(list, x + PAPERDOLL_RECT.x, y + PAPERDOLL_RECT.y);
        self.paint_grid(list, x + layout::RECT_F8.x, y + layout::RECT_F8.y);
        for (tab, name) in [InventoryTab::Item, InventoryTab::Quest].into_iter().zip(&self.tab_names) {
            let rect = tab_rect(tab);
            let texture = if tab == self.tab { TAB_ACTIVE } else { TAB_NORMAL };
            list.draw_texture(x + rect.x, y + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, texture, 1.0);
            let text_width = font.measure(name) as i32;
            list.draw_text(
                x + rect.x + rect.w / 2 - text_width / 2,
                y + rect.y + rect.h / 2 - font.line_height as i32 / 2,
                crate::color::BUTTON_TEXT,
                name,
                font,
            );
        }
    }

    fn paint_equipped(&self, list: &mut DrawList, x: i32, y: i32) {
        for slot in 0..HENNA_SLOTS_MAX {
            let top = y + HENNA_MARK_Y + slot as i32 * HENNA_STEP;
            list.draw_texture(x + HENNA_MARK_X, top, HENNA_MARK_SIZE, HENNA_MARK_SIZE, 0, 0, HENNA_MARK_SIZE, HENNA_MARK_SIZE, HENNA_MARK, 1.0);
            let Some(henna) = self.hennas.get(slot) else { continue };
            let (iy, size) = (y + HENNA_ICON_Y + slot as i32 * HENNA_STEP, HENNA_ICON_SIZE);
            list.draw_texture(x + HENNA_ICON_X, iy, size, size, 0, 0, SLOT, SLOT, &henna.icon, 1.0);
            if !henna.usable {
                list.draw_texture(x + HENNA_EMPTY_X, iy, size, size, 0, 0, size, size, HENNA_EMPTY, 1.0);
            }
        }
        let ammo = self.ammo_equipped();
        for (body_part, item) in &self.equipped {
            let mut slot = gm_slot(*body_part);
            let selected = self.selected == Some(item.object_id);
            if ammo && slot >= 0 && item.tip.as_ref().is_some_and(|tip| tip.weapon_kind == BOW_WEAPON_KIND) {
                slot = SLOT_R_HAND;
            }
            match slot {
                0..=14 => {
                    let (sx, sy) = paperdoll_slot_pos(slot as usize);
                    icon(list, x + sx, y + sy, item.icon(0));
                    slot_frame(list, x + sx, y + sy, selected);
                }
                SLOT_TWO_HANDED => {
                    let icons = item.special_icons();
                    icon(list, x + 7, y + 45, &icons[0]);
                    icon(list, x + 85, y + 45, &icons[1]);
                    icon(list, x + 85, y + 45, DUAL_CAP);
                    slot_frame(list, x + 7, y + 45, selected);
                }
                SLOT_FULL_ARMOR => {
                    let icons = item.special_icons();
                    icon(list, x + 46, y + 45, &icons[0]);
                    icon(list, x + 46, y + 83, &icons[1]);
                    icon(list, x + 46, y + 83, DUAL_CAP);
                    slot_frame(list, x + 46, y + 45, selected);
                }
                _ => {}
            }
        }
    }

    fn paint_grid(&self, list: &mut DrawList, x: i32, y: i32) {
        let first = self.scroll_row * GRID_COLS;
        for (index, item) in self.page().iter().enumerate().skip(first).take(GRID_COLS * GRID_ROWS) {
            let cell = index - first;
            let (col, row) = ((cell % GRID_COLS) as i32, (cell / GRID_COLS) as i32);
            let (sx, sy) = (GRID_ORIGIN.0 + col * PAINT_STEP.0, GRID_ORIGIN.1 + row * PAINT_STEP.1);
            icon(list, x + sx, y + sy, item.icon(0));
            slot_frame(list, x + sx, y + sy, self.selected == Some(item.object_id));
        }
        self.scroll.paint(list, x, y, layout::RECT_F8.w, layout::RECT_F8.h);
    }
}

pub fn quest_item(type2: i16) -> bool {
    type2 == QUEST_TYPE2
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(object_id: i32, item_id: i32, body_part: i32, equipped: bool, quest: bool, count: i32) -> InventoryItemView {
        InventoryItemView { object_id, item_id, count, body_part, equipped, quest, icons: vec![format!("icon{object_id}")], ..Default::default() }
    }

    fn view() -> GmInventoryView {
        GmInventoryView::new(|id| Some(format!("s{id}")))
    }

    #[test]
    fn the_layout_matches_the_on_create_snapshot() {
        assert_eq!(SIZE, (256, 401));
        assert_eq!(grid_rect(), Rect::new(9, 189, 236, 139));
        assert_eq!(equipped_rect(), Rect::new(8, 31, 240, 122));
        assert_eq!(tabs_rect(), Rect::new(12, 159, 189, 23));
        let view = view();
        assert_eq!(view.tab_names, ["s2".to_string(), "s118".to_string()]);
    }

    #[test]
    fn the_slot_follows_the_gm_rule_without_the_object_id() {
        assert_eq!(gm_slot(2), 8);
        assert_eq!(gm_slot(4), 9);
        assert_eq!(gm_slot(0x10), 13);
        assert_eq!(gm_slot(0x20), 14);
        assert_eq!(gm_slot(0x400), 6);
        assert_eq!(gm_slot(crate::inventory::BODY_PART_ALL_DRESS), -1);
    }

    #[test]
    fn earrings_and_rings_alternate_between_their_two_slots() {
        let mut view = view();
        for id in 1..=2 {
            view.add_item(item(id, 100, 6, true, false, 1));
        }
        for id in 3..=4 {
            view.add_item(item(id, 200, 0x30, true, false, 1));
        }
        let slots: Vec<i32> = view.equipped.iter().map(|(body_part, _)| gm_slot(*body_part)).collect();
        assert_eq!(slots, vec![9, 8, 14, 13]);
        view.clear();
        view.add_item(item(5, 100, 6, true, false, 1));
        assert_eq!(gm_slot(view.equipped[0].0), 9);
    }

    #[test]
    fn items_go_to_the_equipped_grid_or_the_item_and_quest_pages_and_the_equipped_list_is_capped() {
        let mut view = view();
        view.add_item(item(1, 57, 0, false, false, 1000));
        view.add_item(item(2, 5, 0, false, true, 1));
        view.add_item(item(3, 57, 0, false, false, 234));
        for id in 10..30 {
            view.add_item(item(id, 7, 0x400, true, false, 1));
        }
        assert_eq!((view.pages[0].len(), view.pages[1].len(), view.equipped.len()), (2, 1, 15));
        assert_eq!(view.adena(), 1234);
        assert_eq!(view.counted(), 18);
    }

    #[test]
    fn the_hit_test_uses_the_33_pixel_step_while_painting_uses_37_by_35() {
        assert_eq!(grid_hit_slot(1, 1, 0), Some(0));
        assert_eq!(grid_hit_slot(1 + 33, 1, 0), Some(1));
        assert_eq!(grid_hit_slot(1, 1 + 33, 2), Some(18));
        assert_eq!(grid_hit_slot(0, 5, 0), None);
        assert_eq!(grid_hit_slot(1 + 33 * 6 + 1, 1, 0), None);
        let mut view = view();
        for id in 1..=3 {
            view.add_item(item(id, 7, 0, false, false, 1));
        }
        let font = {
            let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
            BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
        };
        let mut list = DrawList::default();
        view.paint(&mut list, (0, 0), &font, 1.0);
        let second = list.quads.iter().find(|quad| quad.texture == "icon2").unwrap();
        assert_eq!((second.x, second.y), (9.0 + 1.0 + 37.0, 189.0 + 1.0));
    }

    #[test]
    fn tabs_switch_pages_and_select_by_object_id() {
        let mut view = view();
        view.add_item(item(1, 5, 0, false, false, 1));
        view.add_item(item(2, 6, 0, false, true, 1));
        let quest = tab_rect(InventoryTab::Quest);
        view.mouse_down(quest.x + 2, quest.y + 2);
        assert_eq!(view.mouse_up(quest.x + 2, quest.y + 2), Some(false));
        assert_eq!(view.tab, InventoryTab::Quest);
        view.mouse_down(grid_rect().x + 2, grid_rect().y + 2);
        assert_eq!(view.selected, Some(2));
    }

    #[test]
    fn a_bow_goes_to_the_right_hand_slot_alone_while_ammunition_is_equipped() {
        let font = {
            let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
            BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
        };
        let bow_icons = || vec!["bow0".to_string(), "bow1".to_string(), "bow2".to_string()];
        let mut bow = item(1, 100, BODY_PART_LR_HAND, true, false, 1);
        bow.icons = bow_icons();
        bow.tip = Some(crate::item_tip::ItemTipData { weapon_kind: 6, ..Default::default() });
        let mut sword = item(3, 101, BODY_PART_LR_HAND, true, false, 1);
        sword.icons = bow_icons();
        sword.tip = Some(crate::item_tip::ItemTipData { weapon_kind: 1, ..Default::default() });
        let mut arrow = item(2, 17, 0x80, true, false, 100);
        arrow.tip = Some(crate::item_tip::ItemTipData { etc_family: 2, ..Default::default() });
        let paint = |items: Vec<InventoryItemView>| {
            let mut view = view();
            for item in items {
                view.add_item(item);
            }
            let mut list = DrawList::default();
            view.paint(&mut list, (0, 0), &font, 1.0);
            list
        };
        let origin = (PAPERDOLL_RECT.x as f32, PAPERDOLL_RECT.y as f32);
        let with_ammo = paint(vec![bow.clone(), arrow.clone()]);
        let quads: Vec<_> = with_ammo.quads.iter().filter(|q| q.texture.starts_with("bow")).collect();
        assert_eq!(quads.len(), 1);
        assert_eq!((quads[0].texture.as_str(), quads[0].x, quads[0].y), ("bow0", origin.0 + 7.0, origin.1 + 45.0));
        let without = paint(vec![bow.clone()]);
        assert_eq!(without.quads.iter().filter(|q| q.texture.starts_with("bow")).count(), 2);
        let sword_with_ammo = paint(vec![sword, arrow]);
        assert_eq!(sword_with_ammo.quads.iter().filter(|q| q.texture.starts_with("bow")).count(), 2);
    }

    #[test]
    fn the_paint_layers_match_the_reference_on_paint_quads() {
        let font = {
            let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
            BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
        };
        let view = view();
        let mut list = DrawList::default();
        view.paint(&mut list, (0, 0), &font, 1.0);
        let find = |name: &str| list.quads.iter().find(|q| q.texture == name).map(|q| (q.x, q.y, q.w, q.h, q.color[3]));
        assert_eq!(find(INVENTORY_BACK), Some((0.0, 20.0, 256.0, 381.0, 1.0)));
        assert_eq!(find(ADENA_ICON), Some((98.0, 355.0, 16.0, 12.0, 1.0)));
        assert_eq!(find(WEIGHT_ICON), Some((98.0, 372.0, 16.0, 12.0, 1.0)));
        let boxes = list.quads.iter().filter(|q| q.texture == HENNA_MARK).map(|q| (q.x, q.y, q.w, q.h)).collect::<Vec<_>>();
        assert_eq!(boxes, vec![(223.0, 39.0, 26.0, 26.0), (223.0, 66.0, 26.0, 26.0), (223.0, 93.0, 26.0, 26.0)]);
        assert_eq!((TEXT, GOLD), (crate::color::Argb(0xFFDCDCDC), crate::color::Argb(0xFFB09B79)));
    }

    #[test]
    fn the_gm_tattoos_paint_their_icon_and_a_veil_over_the_unusable_ones() {
        let font = {
            let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
            BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
        };
        let mut view = view();
        view.add_henna("icon.a", true);
        view.add_henna("", false);
        let mut list = DrawList::default();
        view.paint(&mut list, (0, 0), &font, 1.0);
        let at = |name: &str| list.quads.iter().filter(|q| q.texture == name).map(|q| (q.x, q.y, q.w, q.h)).collect::<Vec<_>>();
        assert_eq!(at("icon.a"), vec![(224.0, 40.0, 24.0, 24.0)]);
        assert_eq!(at(BLACK_TEXTURE), vec![(224.0, 67.0, 24.0, 24.0)]);
        assert_eq!(at(HENNA_EMPTY), vec![(225.0, 67.0, 24.0, 24.0)]);
        view.clear();
        assert_eq!(view.hennas.len(), 2);
        view.clear_hennas();
        assert!(view.hennas.is_empty());
    }
}
