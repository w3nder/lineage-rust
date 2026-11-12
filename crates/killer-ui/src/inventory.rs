use crate::color::{GOLD, TEXT};
use crate::draw::{DrawList, digits_width};
use crate::font::BitmapFont;
use crate::frame::{BUTTON_SIZE, BUTTON_Y, CLOSE_INSET, FrameCtrl, TITLE_HEIGHT};
use crate::scroll::{ScrollHit, ScrollWnd, clamp_first_row, rows_for};
use crate::window::Rect;

pub const INVENTORY_SIZE: (i32, i32) = (256, 401);
pub const INVENTORY_DEFAULT_Y_RISE: i32 = 252;
pub const SHORTCUT_BAR_WIDTH: i32 = 46;
pub const TITLE_SYSSTRING: u32 = 138;
pub const TAB_ITEM_SYSSTRING: u32 = 2;
pub const TAB_QUEST_SYSSTRING: u32 = 118;
pub const QUEST_TYPE2: i16 = 3;

pub const INVENTORY_BACK: &str = "L2UI_CH3.InventoryWnd.Inventory_Back";
pub const ADENA_ICON: &str = "L2UI.WindowIcon.WindowIconAdena";
pub const WEIGHT_ICON: &str = "L2UI.WindowIcon.Weight";
pub const TAB_NORMAL: &str = "L2UI_CH3.InventoryWnd.Inventory_tab2";
pub const TAB_ACTIVE: &str = "L2UI_CH3.InventoryWnd.Inventory_tab1";
pub const OUTLINE: &str = "l2ui_ch3.InventoryWnd.Inventory_OutLine";
pub const OUTLINE_DOWN: &str = "l2ui_ch3.InventoryWnd.Inventory_OutLine_down";
pub const ITEM_CLICK: &str = "l2ui.NWindow.item_click";
pub const DUAL_CAP: &str = "L2UI.InventoryWnd.Icon_dualcap";
pub const HENNA_MARK: &str = "L2UI_CH3.InventoryWnd.Inventory_Mark";
pub const HENNA_EMPTY: &str = "NWindow.ChatBack";
pub const TRASH: &str = "L2UI_CH3.InventoryWnd.Inventory_trash";
pub const TRASH_DRAG: &str = "L2UI_CH3.InventoryWnd.Inventory_trash_drag";
pub const RECIPE: &str = "L2UI_CH3.InventoryWnd.Inventory_recipe";
pub const RECIPE_DRAG: &str = "L2UI_CH3.InventoryWnd.Inventory_recipe_drag";
pub const BLACK_TEXTURE: &str = "NWindow.BlackTexture";
pub const WEIGHT_BARS: [&str; 5] = [
    "L2UI_CH3.PlayerStatusWnd.PS_weightbar1",
    "L2UI_CH3.PlayerStatusWnd.PS_weightbar2",
    "L2UI_CH3.PlayerStatusWnd.PS_weightbar3",
    "L2UI_CH3.PlayerStatusWnd.PS_weightbar4",
    "L2UI_CH3.PlayerStatusWnd.PS_weightbar5",
];

pub const BACK_RECT: Rect = Rect::new(0, 20, 256, 381);
pub const PAPERDOLL_RECT: Rect = Rect::new(8, 31, 240, 122);
pub const PAPERDOLL_X: [i32; 5] = [7, 46, 85, 130, 169];
pub const PAPERDOLL_Y: [i32; 3] = [7, 45, 83];
pub const PAPERDOLL_SLOTS: usize = 15;
pub const SLOT: i32 = 32;
pub const OUTLINE_SIZE: i32 = 34;
pub const OUTLINE_BACK_SIZE: i32 = 35;
pub const TAB_RECT: Rect = Rect::new(12, 159, 189, 23);
pub const TAB_SIZE: (i32, i32) = (94, 23);
pub const TAB_GAP: i32 = 1;
pub const COUNTER_RIGHT: i32 = 251;
pub const COUNTER_Y: i32 = 162;
pub const GRID_RECT: Rect = Rect::new(9, 189, 236, 139);
pub const GRID_COLS: usize = 6;
pub const GRID_ROWS: usize = 4;
pub const GRID_ORIGIN: (i32, i32) = (1, 1);
pub const GRID_GAP_X: i32 = 5;
pub const GRID_GAP_Y: i32 = 3;
pub const ADENA_ICON_POS: (i32, i32) = (98, 355);
pub const ADENA_ICON_SIZE: (i32, i32) = (16, 12);
pub const ADENA_RIGHT: i32 = 199;
pub const ADENA_Y: i32 = 356;
pub const WEIGHT_ICON_POS: (i32, i32) = (98, 372);
pub const WEIGHT_BAR_RECT: Rect = Rect::new(117, 372, 85, 12);
pub const WEIGHT_TEXT_Y: i32 = 2;
pub const TRASH_RECT: Rect = Rect::new(14, 351, 34, 34);
pub const RECIPE_RECT: Rect = Rect::new(208, 351, 34, 34);
pub const HENNA_MARK_X: i32 = 215;
pub const HENNA_MARK_Y: i32 = 8;
pub const HENNA_STEP: i32 = 27;
pub const HENNA_MARK_SIZE: i32 = 26;
pub const HENNA_ICON_X: i32 = 216;
pub const HENNA_ICON_Y: i32 = 9;
pub const HENNA_ICON_SIZE: i32 = 24;
pub const HENNA_EMPTY_X: i32 = 217;
pub const HENNA_SLOTS_MAX: usize = 3;

/// Quantos espaços de símbolo a classe tem, pelo `0x10074500`: a tabela de bytes de `0x10074550`
/// indexada pela classe (`jogador->+0x210`) escolhe entre os três retornos de `0x10074544`
/// (`1`, `2` e `3`); as classes de `0x58` a `0x76` têm três; fora disso a função devolve `-1` e o
/// laço do `NCEquipItemWnd::OnPaint` não desenha moldura nenhuma.
const HENNA_SLOTS_BY_CLASS: [u8; 58] = [
    0, 1, 2, 2, 1, 2, 2, 1, 2, 2, 0, 1, 2, 2, 2, 1, 2, 2, 0, 1, 2, 2, 1, 2, 2, 0, 1, 2, 2, 1, 2, 0,
    1, 2, 2, 1, 2, 2, 0, 1, 2, 2, 1, 2, 0, 1, 2, 1, 2, 0, 1, 2, 2, 0, 1, 2, 1, 2,
];

pub fn henna_slots(class_id: i32) -> Option<usize> {
    if let Ok(index) = usize::try_from(class_id)
        && let Some(step) = HENNA_SLOTS_BY_CLASS.get(index)
    {
        return Some(*step as usize + 1);
    }
    (0x58..=0x76).contains(&class_id).then_some(HENNA_SLOTS_MAX)
}

/// Uma entrada da lista de símbolos do `NCEquipItemWnd` (`0x10098880`): o ícone vem do
/// `symbol_icon` do `hennagrp`, e o `dye_item_id` em zero faz o original tapar o ícone com o
/// `NWindow.ChatBack`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HennaMark {
    pub symbol_id: i32,
    pub dye_item_id: i32,
    pub icon: String,
    pub name: String,
    pub description: String,
}
pub const WEIGHT_THRESHOLDS: [f32; 4] = [50.0, 66.6, 80.0, 100.0];

pub const BODY_PART_UNDERWEAR: i32 = 0x0001;
pub const BODY_PART_R_EAR: i32 = 0x0002;
pub const BODY_PART_L_EAR: i32 = 0x0004;
pub const BODY_PART_NECK: i32 = 0x0008;
pub const BODY_PART_R_FINGER: i32 = 0x0010;
pub const BODY_PART_L_FINGER: i32 = 0x0020;
pub const BODY_PART_HEAD: i32 = 0x0040;
pub const BODY_PART_R_HAND: i32 = 0x0080;
pub const BODY_PART_L_HAND: i32 = 0x0100;
pub const BODY_PART_GLOVES: i32 = 0x0200;
pub const BODY_PART_CHEST: i32 = 0x0400;
pub const BODY_PART_LEGS: i32 = 0x0800;
pub const BODY_PART_FEET: i32 = 0x1000;
pub const BODY_PART_BACK: i32 = 0x2000;
pub const BODY_PART_LR_HAND: i32 = 0x4000;
pub const BODY_PART_FULL_ARMOR: i32 = 0x8000;
pub const BODY_PART_HAIR: i32 = 0x10000;
pub const BODY_PART_ALL_DRESS: i32 = 0x20000;

pub const SLOT_FULL_ARMOR: i32 = 15;
pub const SLOT_TWO_HANDED: i32 = 16;
pub const SLOT_ALL_DRESS: i32 = 17;
pub const SLOT_R_HAND: i32 = 5;
pub const BOW_WEAPON_KINDS: [i32; 2] = [6, 10];
pub const AMMO_ETC_TYPES: [i32; 2] = [2, 0x10];
pub const SLOT_CHEST: i32 = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PaperdollIds {
    pub r_ear: i32,
    pub l_ear: i32,
    pub r_finger: i32,
    pub l_finger: i32,
}

pub fn body_part_to_slot(body_part: i32, object_id: i32, ids: &PaperdollIds) -> i32 {
    if body_part & 6 != 0 {
        if ids.r_ear == object_id {
            return 8;
        }
        if ids.l_ear == object_id {
            return 9;
        }
    }
    if body_part & 0x30 != 0 {
        if ids.r_finger == object_id {
            return 13;
        }
        if ids.l_finger == object_id {
            return 14;
        }
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
        BODY_PART_ALL_DRESS => SLOT_ALL_DRESS,
        _ => -1,
    }
}

pub fn unequip_mask(body_part: i32, object_id: i32, ids: &PaperdollIds) -> Option<i32> {
    if body_part & 6 != 0 {
        if ids.r_ear == object_id {
            return Some(BODY_PART_R_EAR);
        }
        if ids.l_ear == object_id {
            return Some(BODY_PART_L_EAR);
        }
        return None;
    }
    if body_part & 0x30 != 0 {
        if ids.r_finger == object_id {
            return Some(BODY_PART_R_FINGER);
        }
        if ids.l_finger == object_id {
            return Some(BODY_PART_L_FINGER);
        }
        return None;
    }
    Some(body_part)
}

pub const TOOLTIP_COLUMN_STEP: i32 = 37;
pub const TOOLTIP_ROW_STEP: i32 = 35;
pub const PAPERDOLL_TIP_ROW: i32 = 38;
pub const PAPERDOLL_TIP_TOP: i32 = 11;
pub const PAPERDOLL_TIP_X: [i32; 5] = [7, 46, 85, 130, 169];

pub fn paperdoll_tooltip_anchor(slot: usize) -> (i32, i32) {
    (PAPERDOLL_RECT.x + PAPERDOLL_TIP_X[slot % 5], PAPERDOLL_RECT.y + PAPERDOLL_TIP_ROW * (slot / 5) as i32 + PAPERDOLL_TIP_TOP)
}

pub fn paperdoll_slot_pos(slot: usize) -> (i32, i32) {
    (PAPERDOLL_X[slot % 5], PAPERDOLL_Y[slot / 5])
}

pub fn paperdoll_slot_at(x: i32, y: i32) -> Option<usize> {
    let (x, y) = (x - PAPERDOLL_RECT.x, y - PAPERDOLL_RECT.y);
    (0..PAPERDOLL_SLOTS).find(|slot| {
        let (sx, sy) = paperdoll_slot_pos(*slot);
        (sx..=sx + SLOT).contains(&x) && (sy..=sy + SLOT).contains(&y)
    })
}

pub fn grid_slot_pos(index: usize) -> (i32, i32) {
    grid_slot_pos_cols(index, GRID_COLS)
}

pub fn grid_slot_pos_cols(index: usize, cols: usize) -> (i32, i32) {
    let col = (index % cols) as i32;
    let row = (index / cols) as i32;
    (GRID_ORIGIN.0 + col * (SLOT + GRID_GAP_X), GRID_ORIGIN.1 + row * (SLOT + GRID_GAP_Y))
}

pub fn grid_slot_at(x: i32, y: i32, cols: usize, rows: usize, scroll_row: usize) -> Option<usize> {
    let x = x - GRID_ORIGIN.0;
    let y = y - GRID_ORIGIN.1;
    let step_x = SLOT + GRID_GAP_X;
    let step_y = SLOT + GRID_GAP_Y;
    if x < 0 || y < 0 || x > step_x * cols as i32 || y > step_y * rows as i32 {
        return None;
    }
    let col = ((x / step_x) as usize).min(cols - 1);
    let row = ((y / step_y) as usize).min(rows - 1);
    Some((scroll_row + row) * cols + col)
}

pub const WHEEL_DELTA: i32 = 120;

pub fn item_wnd_wheel(first: i32, visible: i32, total: i32, delta: i32) -> i32 {
    let steps = delta / WHEEL_DELTA;
    if steps <= 0 {
        if total < visible {
            if (total - first) + steps <= visible {
                return 0;
            }
        } else if (total - first) + steps <= visible {
            return total - visible;
        }
        first - steps
    } else if steps < first {
        first - steps
    } else {
        0
    }
}

pub fn tab_rect(tab: InventoryTab) -> Rect {
    let (w, h) = TAB_SIZE;
    Rect::new(TAB_RECT.x + (w + TAB_GAP) * tab.index() as i32, TAB_RECT.y, w, h)
}

pub fn close_button() -> Rect {
    Rect::new(INVENTORY_SIZE.0 - CLOSE_INSET, BUTTON_Y, BUTTON_SIZE, BUTTON_SIZE)
}

pub fn grip() -> Rect {
    Rect::new(0, 0, INVENTORY_SIZE.0, TITLE_HEIGHT)
}

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    (
        crate::window_restore::x87_to_int(f64::from(viewport.0 as f32) - f64::from(INVENTORY_SIZE.0) - f64::from(SHORTCUT_BAR_WIDTH)),
        crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) * 0.5 - f64::from(INVENTORY_DEFAULT_Y_RISE)),
    )
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, y) = default_position(viewport);
    crate::initial_position::InitialPosition {
        enabled: true,
        horizontal_mode: 1,
        vertical_mode: 5,
        x: viewport.0.wrapping_sub(x),
        y: (viewport.1 / 2).wrapping_sub(y),
    }
}

pub fn thousands(value: i64) -> String {
    let digits: Vec<char> = value.abs().to_string().chars().collect();
    let mut out = String::new();
    for (i, c) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*c);
    }
    if value < 0 { format!("-{out}") } else { out }
}

fn weight_bar_width(load: i32, max_load: i32) -> i32 {
    load.wrapping_mul(WEIGHT_BAR_RECT.w).checked_div(max_load).unwrap_or(0)
}

pub fn weight_bar(load: i32, max_load: i32) -> Option<(i32, &'static str)> {
    let width = weight_bar_width(load, max_load);
    if max_load == 0 || width <= 0 {
        return None;
    }
    let quotient = f64::from(load) / f64::from(max_load) * 100.0;
    let percent = f64::from(quotient as f32);
    let texture = if quotient <= 50.0 {
        WEIGHT_BARS[0]
    } else if percent <= 66.6 {
        WEIGHT_BARS[1]
    } else if percent <= 80.0 {
        WEIGHT_BARS[2]
    } else if percent <= 100.0 {
        WEIGHT_BARS[3]
    } else {
        WEIGHT_BARS[4]
    };
    Some((width, texture))
}

pub fn weight_percent_text(load: i32, max_load: i32) -> String {
    let percent = match weight_bar(load, max_load) {
        Some(_) => (f64::from(load) / f64::from(max_load) * 100.0) as f32,
        None => 0.0,
    };
    format!("{percent:.2}%")
}

pub fn weight_tip_text(load: i32, max_load: i32) -> String {
    format!("[{load}]/[{max_load}]")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InventoryTab {
    #[default]
    Item,
    Quest,
}

impl InventoryTab {
    pub fn index(self) -> usize {
        match self {
            InventoryTab::Item => 0,
            InventoryTab::Quest => 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InventoryHit {
    Paperdoll(usize),
    Henna(usize),
    Grid(usize),
    Scroll(ScrollHit),
    Tab(InventoryTab),
    Trash,
    Recipe,
    Weight,
    Close,
    Title,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InventoryItemView {
    pub object_id: i32,
    pub item_id: i32,
    pub count: i32,
    pub enchant: i32,
    pub body_part: i32,
    pub equipped: bool,
    pub quest: bool,
    pub icons: Vec<String>,
    pub name: String,
    pub popup: i32,
    pub recipe: bool,
    pub crystallizable: bool,
    pub add_name: String,
    pub description: String,
    pub weight: i32,
    pub tip: Option<crate::item_tip::ItemTipData>,
}

impl InventoryItemView {
    pub fn icon(&self, index: usize) -> &str {
        self.icons.get(index).map(String::as_str).filter(|s| !s.is_empty()).unwrap_or(BLACK_TEXTURE)
    }

    pub(crate) fn special_icons(&self) -> [String; 4] {
        let present = |i: usize| self.icons.get(i).filter(|s| !s.is_empty()).cloned();
        let Some(first) = present(1) else {
            let icon = self.icon(0).to_string();
            return [icon.clone(), icon.clone(), icon.clone(), icon];
        };
        let Some(second) = present(2) else {
            return [first.clone(), first.clone(), first.clone(), first];
        };
        let Some(third) = present(3) else {
            return [first, second.clone(), second.clone(), second];
        };
        let fourth = present(4).unwrap_or_else(|| third.clone());
        [first, second, third, fourth]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InventoryView {
    pub title: String,
    pub tab_names: [String; 2],
    pub tab: InventoryTab,
    pub items: Vec<InventoryItemView>,
    pub adena: i64,
    pub load: i32,
    pub max_load: i32,
    pub limit: i32,
    pub paperdoll_ids: PaperdollIds,
    pub selected: Option<i32>,
    pub scroll_row: usize,
    pub scroll: ScrollWnd,
    pub dwarven_craft: bool,
    pub over_trash: bool,
    pub over_recipe: bool,
    pub henna: Vec<HennaMark>,
    pub henna_slots: usize,
}

impl InventoryView {
    pub fn with_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> InventoryView {
        let text = |id: u32| lookup(id).unwrap_or_else(|| format!("sysstring {id}"));
        InventoryView {
            title: text(TITLE_SYSSTRING),
            tab_names: [text(TAB_ITEM_SYSSTRING), text(TAB_QUEST_SYSSTRING)],
            scroll: ScrollWnd::new(GRID_RECT.w, GRID_RECT.h),
            ..Default::default()
        }
    }

    pub fn total_rows(&self) -> i32 {
        rows_for(self.listed().len(), GRID_COLS)
    }

    pub fn update_rows(&mut self) {
        let total = self.total_rows();
        let visible = GRID_ROWS as i32;
        self.scroll_row = clamp_first_row(self.scroll_row as i32, visible, total).max(0) as usize;
        self.scroll.slider.set_range(total, visible);
        self.scroll.slider.set_pos(self.scroll_row as i32);
    }

    pub fn set_first_row(&mut self, first: i32) {
        self.scroll_row = first.max(0) as usize;
        self.scroll.slider.set_pos(first);
    }

    pub fn scroll_hit(&self, x: i32, y: i32) -> Option<ScrollHit> {
        self.scroll.hit(GRID_RECT.w, GRID_RECT.h, x - GRID_RECT.x, y - GRID_RECT.y)
    }

    pub fn equipped(&self) -> impl Iterator<Item = &InventoryItemView> {
        self.items.iter().filter(|i| i.equipped)
    }

    pub fn listed(&self) -> Vec<&InventoryItemView> {
        let quest = self.tab == InventoryTab::Quest;
        self.items.iter().filter(|i| !i.equipped && i.quest == quest).collect()
    }

    pub fn move_listed(&mut self, object_id: i32, target: usize) -> bool {
        let quest = self.tab == InventoryTab::Quest;
        let page = |items: &[InventoryItemView]| -> Vec<usize> {
            items.iter().enumerate().filter(|(_, i)| !i.equipped && i.quest == quest).map(|(index, _)| index).collect()
        };
        let before = page(&self.items);
        let Some(current) = before.iter().position(|&index| self.items[index].object_id == object_id) else { return false };
        if target == current {
            return false;
        }
        let item = self.items.remove(before[current]);
        let after = page(&self.items);
        let at = match after.get(target) {
            Some(&index) => index,
            None => after.last().map_or(self.items.len(), |&index| index + 1),
        };
        self.items.insert(at, item);
        true
    }

    pub fn counted(&self) -> usize {
        self.items.len()
    }

    pub fn grid_slot_origin(&self, index: usize) -> (i32, i32) {
        let cell = index.saturating_sub(self.scroll_row * GRID_COLS);
        (GRID_RECT.x + TOOLTIP_COLUMN_STEP * (cell % GRID_COLS) as i32, GRID_RECT.y + TOOLTIP_ROW_STEP * (cell / GRID_COLS) as i32)
    }

    pub fn grid_item(&self, index: usize) -> Option<&InventoryItemView> {
        self.listed().get(index).copied()
    }

    pub fn paperdoll_item(&self, slot: usize) -> Option<&InventoryItemView> {
        let wanted = slot as i32;
        self.equipped().find(|item| {
            let mapped = body_part_to_slot(item.body_part, item.object_id, &self.paperdoll_ids);
            mapped == wanted
                || (mapped == SLOT_TWO_HANDED && wanted == SLOT_R_HAND)
                || ((mapped == SLOT_FULL_ARMOR || mapped == SLOT_ALL_DRESS) && wanted == SLOT_CHEST)
        })
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<InventoryHit> {
        if !Rect::new(0, 0, INVENTORY_SIZE.0, INVENTORY_SIZE.1).contains(x, y) {
            return None;
        }
        if close_button().contains(x, y) {
            return Some(InventoryHit::Close);
        }
        if y < TITLE_HEIGHT {
            return Some(InventoryHit::Title);
        }
        if let Some(slot) = self.henna_at(x, y) {
            return Some(InventoryHit::Henna(slot));
        }
        if PAPERDOLL_RECT.contains(x, y)
            && let Some(slot) = paperdoll_slot_at(x, y)
        {
            return Some(InventoryHit::Paperdoll(slot));
        }
        for tab in [InventoryTab::Item, InventoryTab::Quest] {
            if tab_rect(tab).contains(x, y) {
                return Some(InventoryHit::Tab(tab));
            }
        }
        if let Some(hit) = self.scroll_hit(x, y) {
            return Some(InventoryHit::Scroll(hit));
        }
        if GRID_RECT.contains(x, y)
            && let Some(index) = grid_slot_at(x - GRID_RECT.x, y - GRID_RECT.y, GRID_COLS, GRID_ROWS, self.scroll_row)
        {
            return Some(InventoryHit::Grid(index));
        }
        if self.dwarven_craft && TRASH_RECT.contains(x, y) {
            return Some(InventoryHit::Trash);
        }
        if self.dwarven_craft && RECIPE_RECT.contains(x, y) {
            return Some(InventoryHit::Recipe);
        }
        if WEIGHT_BAR_RECT.contains(x, y) {
            return Some(InventoryHit::Weight);
        }
        Some(InventoryHit::Body)
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        let (w, h) = INVENTORY_SIZE;
        FrameCtrl::titled(&self.title, true).paint(list, x, y, w, h, font, alpha);
        let back = BACK_RECT;
        list.draw_texture(x + back.x, y + back.y, back.w, back.h, 0, 0, back.w, back.h, INVENTORY_BACK, alpha);
        let adena = thousands(self.adena);
        let adena_w = font.measure(&adena) as i32;
        list.draw_text(x + ADENA_RIGHT - adena_w, y + ADENA_Y, TEXT, &adena, font);
        let (aw, ah) = ADENA_ICON_SIZE;
        list.draw_texture(x + ADENA_ICON_POS.0, y + ADENA_ICON_POS.1, aw, ah, 0, 0, aw, ah, ADENA_ICON, 1.0);
        list.draw_texture(x + WEIGHT_ICON_POS.0, y + WEIGHT_ICON_POS.1, aw, ah, 0, 0, aw, ah, WEIGHT_ICON, 1.0);
        let counter = format!("({}/{})", self.counted(), self.limit);
        let counter_w = font.measure(&counter) as i32;
        list.draw_text(x + COUNTER_RIGHT - counter_w, y + COUNTER_Y, GOLD, &counter, font);
        self.paint_paperdoll(list, x + PAPERDOLL_RECT.x, y + PAPERDOLL_RECT.y);
        self.paint_grid(list, x + GRID_RECT.x, y + GRID_RECT.y);
        self.paint_tabs(list, x, y, font);
        self.paint_weight(list, x + WEIGHT_BAR_RECT.x, y + WEIGHT_BAR_RECT.y);
        if self.dwarven_craft {
            let recipe = if self.over_recipe { RECIPE_DRAG } else { RECIPE };
            list.draw_texture(
                x + RECIPE_RECT.x,
                y + RECIPE_RECT.y,
                RECIPE_RECT.w,
                RECIPE_RECT.h,
                0,
                0,
                RECIPE_RECT.w,
                RECIPE_RECT.h,
                recipe,
                1.0,
            );
            let trash = if self.over_trash { TRASH_DRAG } else { TRASH };
            list.draw_texture(
                x + TRASH_RECT.x,
                y + TRASH_RECT.y,
                TRASH_RECT.w,
                TRASH_RECT.h,
                0,
                0,
                TRASH_RECT.w,
                TRASH_RECT.h,
                trash,
                1.0,
            );
        }
    }

    fn paint_tabs(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        for (tab, name) in [InventoryTab::Item, InventoryTab::Quest].into_iter().zip(&self.tab_names) {
            let rect = tab_rect(tab);
            let texture = if tab == self.tab { TAB_ACTIVE } else { TAB_NORMAL };
            list.draw_texture(x + rect.x, y + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, texture, 1.0);
            let text_w = font.measure(name) as i32;
            let text_x = x + rect.x + rect.w / 2 - text_w / 2;
            let text_y = y + rect.y + rect.h / 2 - font.line_height as i32 / 2;
            list.draw_text(text_x, text_y, crate::color::BUTTON_TEXT, name, font);
        }
    }

    fn paint_slot_frame(&self, list: &mut DrawList, x: i32, y: i32, selected: bool) {
        slot_frame(list, x, y, selected);
    }

    /// A coluna de símbolos à direita do painel de equipamento, pelo fim do
    /// `NCEquipItemWnd::OnPaint @ 0x10094100`: uma moldura por espaço de símbolo da classe, o ícone
    /// do símbolo quando existe, e o `NWindow.ChatBack` por cima quando o `dye_item_id` é zero.
    /// Qual moldura de símbolo está sob o ponto, na janela. As molduras ficam em
    /// `(215, 8 + 27i, 26, 26)` dentro do painel de equipamento.
    pub fn henna_at(&self, x: i32, y: i32) -> Option<usize> {
        let (x, y) = (x - PAPERDOLL_RECT.x, y - PAPERDOLL_RECT.y);
        if !(HENNA_MARK_X..HENNA_MARK_X + HENNA_MARK_SIZE).contains(&x) {
            return None;
        }
        let row = y - HENNA_MARK_Y;
        let slot = usize::try_from(row.div_euclid(HENNA_STEP)).ok()?;
        (row >= 0 && row.rem_euclid(HENNA_STEP) < HENNA_MARK_SIZE && slot < self.henna_slots).then_some(slot)
    }

    pub fn henna_mark(&self, slot: usize) -> Option<&HennaMark> {
        self.henna.get(slot)
    }

    fn paint_henna(&self, list: &mut DrawList, x: i32, y: i32) {
        for slot in 0..self.henna_slots {
            let top = y + HENNA_MARK_Y + slot as i32 * HENNA_STEP;
            let size = HENNA_MARK_SIZE;
            list.draw_texture(x + HENNA_MARK_X, top, size, size, 0, 0, size, size, HENNA_MARK, 1.0);
            let Some(mark) = self.henna.get(slot) else { continue };
            let icon = match mark.icon.is_empty() {
                true => BLACK_TEXTURE,
                false => mark.icon.as_str(),
            };
            let (iy, size) = (y + HENNA_ICON_Y + slot as i32 * HENNA_STEP, HENNA_ICON_SIZE);
            list.draw_texture(x + HENNA_ICON_X, iy, size, size, 0, 0, SLOT, SLOT, icon, 1.0);
            if mark.dye_item_id == 0 {
                list.draw_texture(x + HENNA_EMPTY_X, iy, size, size, 0, 0, size, size, HENNA_EMPTY, 1.0);
            }
        }
    }

    pub fn ammo_equipped(&self) -> bool {
        self.equipped().any(|item| item.tip.as_ref().is_some_and(|tip| AMMO_ETC_TYPES.contains(&tip.etc_family)))
    }

    fn paint_paperdoll(&self, list: &mut DrawList, x: i32, y: i32) {
        self.paint_henna(list, x, y);
        let ammo = self.ammo_equipped();
        for item in self.equipped() {
            let mut slot = body_part_to_slot(item.body_part, item.object_id, &self.paperdoll_ids);
            if (0..=SLOT_ALL_DRESS).contains(&slot)
                && ammo
                && item.tip.as_ref().is_some_and(|tip| BOW_WEAPON_KINDS.contains(&tip.weapon_kind))
            {
                slot = SLOT_R_HAND;
            }
            let selected = self.selected == Some(item.object_id);
            match slot {
                0..=14 => {
                    let (sx, sy) = paperdoll_slot_pos(slot as usize);
                    icon(list, x + sx, y + sy, item.icon(0));
                    self.paint_slot_frame(list, x + sx, y + sy, selected);
                }
                SLOT_TWO_HANDED => {
                    let icons = item.special_icons();
                    icon(list, x + 7, y + 45, &icons[0]);
                    icon(list, x + 85, y + 45, &icons[1]);
                    icon(list, x + 85, y + 45, DUAL_CAP);
                    self.paint_slot_frame(list, x + 7, y + 45, selected);
                }
                SLOT_FULL_ARMOR => {
                    let icons = item.special_icons();
                    icon(list, x + 46, y + 45, &icons[0]);
                    icon(list, x + 46, y + 83, &icons[1]);
                    icon(list, x + 46, y + 83, DUAL_CAP);
                    self.paint_slot_frame(list, x + 46, y + 45, selected);
                }
                SLOT_ALL_DRESS => {
                    let icons = item.special_icons();
                    icon(list, x + 46, y + 45, &icons[0]);
                    icon(list, x + 46, y + 83, &icons[1]);
                    icon(list, x + 46, y + 83, DUAL_CAP);
                    icon(list, x + 7, y + 83, &icons[2]);
                    icon(list, x + 7, y + 83, DUAL_CAP);
                    icon(list, x + 85, y + 83, &icons[3]);
                    icon(list, x + 85, y + 83, DUAL_CAP);
                    self.paint_slot_frame(list, x + 46, y + 45, selected);
                }
                _ => {}
            }
        }
    }

    fn paint_grid(&self, list: &mut DrawList, x: i32, y: i32) {
        let first = self.scroll_row * GRID_COLS;
        for (index, item) in self.listed().into_iter().enumerate().skip(first).take(GRID_COLS * GRID_ROWS) {
            let (sx, sy) = grid_slot_pos(index - first);
            icon(list, x + sx, y + sy, item.icon(0));
            self.paint_slot_frame(list, x + sx, y + sy, self.selected == Some(item.object_id));
        }
        self.scroll.paint(list, x, y, GRID_RECT.w, GRID_RECT.h);
    }

    fn paint_weight(&self, list: &mut DrawList, x: i32, y: i32) {
        let start = list.quads.len();
        if let Some((width, texture)) = weight_bar(self.load, self.max_load) {
            list.draw_texture(x, y, width, WEIGHT_BAR_RECT.h, 0, 0, width, WEIGHT_BAR_RECT.h, texture, 1.0);
        }
        let text = weight_percent_text(self.load, self.max_load);
        list.draw_digits(x + WEIGHT_BAR_RECT.w / 2 - digits_width(&text) / 2, y + WEIGHT_TEXT_Y, &text, TEXT);
        list.clip_from(start, Rect::new(x, y, WEIGHT_BAR_RECT.w, WEIGHT_BAR_RECT.h));
    }
}

pub(crate) fn slot_frame(list: &mut DrawList, x: i32, y: i32, selected: bool) {
    if selected {
        list.draw_texture(
            x - 1,
            y - 1,
            OUTLINE_SIZE,
            OUTLINE_SIZE,
            0,
            0,
            OUTLINE_SIZE,
            OUTLINE_SIZE,
            ITEM_CLICK,
            1.0,
        );
    } else {
        list.draw_texture(
            x - 1,
            y - 1,
            OUTLINE_SIZE,
            OUTLINE_SIZE,
            0,
            0,
            OUTLINE_SIZE,
            OUTLINE_SIZE,
            OUTLINE_DOWN,
            1.0,
        );
        list.draw_texture(
            x - 1,
            y - 1,
            OUTLINE_BACK_SIZE,
            OUTLINE_BACK_SIZE,
            0,
            0,
            OUTLINE_BACK_SIZE,
            OUTLINE_BACK_SIZE,
            OUTLINE,
            1.0,
        );
    }
}

pub(crate) fn icon(list: &mut DrawList, x: i32, y: i32, texture: &str) {
    list.draw_texture(x, y, SLOT, SLOT, 0, 0, SLOT, SLOT, texture, 1.0);
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

    fn item(object_id: i32, item_id: i32, count: i32, equipped: bool, body_part: i32, icon: &str) -> InventoryItemView {
        InventoryItemView {
            object_id,
            item_id,
            count,
            equipped,
            body_part,
            icons: vec![icon.to_string()],
            ..Default::default()
        }
    }

    #[test]
    fn the_multi_slot_icons_follow_the_chain_of_the_paint_0x10089dd0() {
        let with = |names: [&str; 5]| InventoryItemView { icons: names.iter().map(|name| name.to_string()).collect(), ..Default::default() };
        assert_eq!(with(["a", "", "", "", ""]).special_icons(), ["a", "a", "a", "a"].map(String::from));
        assert_eq!(with(["a", "b", "", "c", "d"]).special_icons(), ["b", "b", "b", "b"].map(String::from));
        assert_eq!(with(["a", "b", "c", "", "d"]).special_icons(), ["b", "c", "c", "c"].map(String::from));
        assert_eq!(with(["a", "b", "c", "d", ""]).special_icons(), ["b", "c", "d", "d"].map(String::from));
        assert_eq!(with(["a", "b", "c", "d", "e"]).special_icons(), ["b", "c", "d", "e"].map(String::from));
    }

    fn view() -> InventoryView {
        let mut view = InventoryView::with_sysstrings(|id| Some(format!("s{id}")));
        view.items = vec![
            item(1, 57, 2500, false, 0, "icon.etc_adena_i00"),
            item(2, 2369, 1, true, BODY_PART_R_HAND, "icon.weapon_squires_sword_i00"),
            item(3, 1146, 1, true, BODY_PART_CHEST, "icon.armor_t01_u_i00"),
            item(4, 1060, 5, false, 0, "icon.etc_potion_red_i00"),
            item(5, 112, 1, true, BODY_PART_R_EAR, "icon.accessary_earing_i00"),
            item(6, 1, 1, false, 0, ""),
        ];
        view.items[5].quest = true;
        view.paperdoll_ids = PaperdollIds { r_ear: 5, ..Default::default() };
        view.adena = 2500;
        view.load = 300;
        view.max_load = 1000;
        view.limit = 80;
        view
    }

    #[test]
    fn the_wheel_moves_the_first_row_by_the_delta_in_wheel_units_and_clamps_like_on_mouse_wheel() {
        assert_eq!(item_wnd_wheel(0, 4, 10, -WHEEL_DELTA), 1);
        assert_eq!(item_wnd_wheel(0, 4, 10, -2 * WHEEL_DELTA), 2);
        assert_eq!(item_wnd_wheel(5, 4, 10, -WHEEL_DELTA), 6);
        assert_eq!(item_wnd_wheel(6, 4, 10, -WHEEL_DELTA), 6);
        assert_eq!(item_wnd_wheel(0, 4, 2, -WHEEL_DELTA), 0);
        assert_eq!(item_wnd_wheel(2, 4, 10, WHEEL_DELTA), 1);
        assert_eq!(item_wnd_wheel(1, 4, 10, WHEEL_DELTA), 0);
        assert_eq!(item_wnd_wheel(3, 4, 10, 5 * WHEEL_DELTA), 0);
        assert_eq!(item_wnd_wheel(3, 4, 10, 60), 3);
        assert_eq!(item_wnd_wheel(3, 4, 10, -60), 3);
    }

    #[test]
    fn body_parts_map_to_the_fifteen_paperdoll_slots_and_the_three_specials() {
        let ids = PaperdollIds { r_ear: 10, l_ear: 11, r_finger: 12, l_finger: 13 };
        assert_eq!(body_part_to_slot(BODY_PART_UNDERWEAR, 0, &ids), 0);
        assert_eq!(body_part_to_slot(BODY_PART_HEAD, 0, &ids), 1);
        assert_eq!(body_part_to_slot(BODY_PART_BACK, 0, &ids), 2);
        assert_eq!(body_part_to_slot(BODY_PART_HAIR, 0, &ids), 3);
        assert_eq!(body_part_to_slot(BODY_PART_NECK, 0, &ids), 4);
        assert_eq!(body_part_to_slot(BODY_PART_R_HAND, 0, &ids), 5);
        assert_eq!(body_part_to_slot(BODY_PART_CHEST, 0, &ids), 6);
        assert_eq!(body_part_to_slot(BODY_PART_L_HAND, 0, &ids), 7);
        assert_eq!(body_part_to_slot(BODY_PART_R_EAR, 10, &ids), 8);
        assert_eq!(body_part_to_slot(BODY_PART_L_EAR, 11, &ids), 9);
        assert_eq!(body_part_to_slot(BODY_PART_R_EAR | BODY_PART_L_EAR, 11, &ids), 9);
        assert_eq!(body_part_to_slot(BODY_PART_R_EAR, 99, &ids), -1);
        assert_eq!(body_part_to_slot(BODY_PART_GLOVES, 0, &ids), 10);
        assert_eq!(body_part_to_slot(BODY_PART_LEGS, 0, &ids), 11);
        assert_eq!(body_part_to_slot(BODY_PART_FEET, 0, &ids), 12);
        assert_eq!(body_part_to_slot(BODY_PART_R_FINGER, 12, &ids), 13);
        assert_eq!(body_part_to_slot(BODY_PART_L_FINGER, 13, &ids), 14);
        assert_eq!(body_part_to_slot(BODY_PART_FULL_ARMOR, 0, &ids), SLOT_FULL_ARMOR);
        assert_eq!(body_part_to_slot(BODY_PART_LR_HAND, 0, &ids), SLOT_TWO_HANDED);
        assert_eq!(body_part_to_slot(BODY_PART_ALL_DRESS, 0, &ids), SLOT_ALL_DRESS);
        assert_eq!(body_part_to_slot(0x40000, 0, &ids), -1);
        assert_eq!(unequip_mask(BODY_PART_R_EAR | BODY_PART_L_EAR, 11, &ids), Some(BODY_PART_L_EAR));
        assert_eq!(unequip_mask(BODY_PART_R_FINGER, 12, &ids), Some(BODY_PART_R_FINGER));
        assert_eq!(unequip_mask(BODY_PART_L_FINGER, 7, &ids), None);
        assert_eq!(unequip_mask(BODY_PART_CHEST, 0, &ids), Some(BODY_PART_CHEST));
    }

    #[test]
    fn paperdoll_slots_are_a_five_by_three_grid_from_7_7() {
        assert_eq!(paperdoll_slot_pos(0), (7, 7));
        assert_eq!(paperdoll_slot_pos(4), (169, 7));
        assert_eq!(paperdoll_slot_pos(7), (85, 45));
        assert_eq!(paperdoll_slot_pos(14), (169, 83));
        assert_eq!(paperdoll_slot_at(8 + 7, 31 + 7), Some(0));
        assert_eq!(paperdoll_slot_at(8 + 46 + 31, 31 + 45 + 31), Some(6));
        assert_eq!(paperdoll_slot_at(8 + 46 + 32, 31 + 45), Some(6), "PtInRect inclui a borda");
        assert_eq!(paperdoll_slot_at(8 + 46 + 33, 31 + 45), None);
        assert_eq!(paperdoll_slot_at(8 + 169, 31 + 83), Some(14));
        assert_eq!(paperdoll_slot_at(0, 0), None);
    }

    #[test]
    fn grid_slots_step_37_by_35_from_1_1_and_the_hit_test_clamps_the_last_column() {
        assert_eq!(grid_slot_pos(0), (1, 1));
        assert_eq!(grid_slot_pos(5), (186, 1));
        assert_eq!(grid_slot_pos(6), (1, 36));
        assert_eq!(grid_slot_pos(23), (186, 106));
        assert_eq!(grid_slot_at(1, 1, 6, 4, 0), Some(0));
        assert_eq!(grid_slot_at(37, 1, 6, 4, 0), Some(0));
        assert_eq!(grid_slot_at(38, 1, 6, 4, 0), Some(1));
        assert_eq!(grid_slot_at(222, 36, 6, 4, 0), Some(11));
        assert_eq!(grid_slot_at(223, 1, 6, 4, 0), Some(5));
        assert_eq!(grid_slot_at(224, 1, 6, 4, 0), None);
        assert_eq!(grid_slot_at(1, 141, 6, 4, 0), Some(18));
        assert_eq!(grid_slot_at(1, 142, 6, 4, 0), None);
        assert_eq!(grid_slot_at(0, 1, 6, 4, 0), None);
        assert_eq!(grid_slot_at(1, 1, 6, 4, 2), Some(12));
    }

    #[test]
    fn tabs_counter_buttons_and_close_are_where_on_create_puts_them() {
        assert_eq!(tab_rect(InventoryTab::Item), Rect::new(12, 159, 94, 23));
        assert_eq!(tab_rect(InventoryTab::Quest), Rect::new(107, 159, 94, 23));
        assert_eq!(TRASH_RECT, Rect::new(14, 351, 34, 34));
        assert_eq!(RECIPE_RECT, Rect::new(208, 351, 34, 34));
        assert_eq!(close_button(), Rect::new(233, 3, 15, 15));
        assert_eq!(grip(), Rect::new(0, 0, 256, 20));
        assert_eq!(default_position((1558, 933)), (1256, 214));
        let view = view();
        assert_eq!(view.hit(20, 40), Some(InventoryHit::Paperdoll(0)));
        assert_eq!(view.hit(150, 170), Some(InventoryHit::Tab(InventoryTab::Quest)));
        assert_eq!(view.hit(10, 190), Some(InventoryHit::Grid(0)));
        assert_eq!(view.hit(10 + 37, 190), Some(InventoryHit::Grid(1)));
        assert_eq!(view.hit(20, 360), Some(InventoryHit::Body));
        assert_eq!(view.hit(215, 360), Some(InventoryHit::Body));
        assert_eq!(view.hit(240, 10), Some(InventoryHit::Close));
        assert_eq!(view.hit(100, 10), Some(InventoryHit::Title));
        assert_eq!(view.hit(300, 10), None);
        let dwarf = InventoryView { dwarven_craft: true, ..view };
        assert_eq!(dwarf.hit(20, 360), Some(InventoryHit::Trash));
        assert_eq!(dwarf.hit(215, 360), Some(InventoryHit::Recipe));
    }

    /// A coluna de símbolos do painel de equipamento, pelo fim do
    /// `NCEquipItemWnd::OnPaint @ 0x10094100`: uma moldura `Inventory_Mark` de 26 × 26 por espaço
    /// de símbolo da classe em `(215, 8 + 27i)`, o ícone de 24 × 24 em `(216, 9 + 27i)` e o
    /// `NWindow.ChatBack` por cima só quando o `dye_item_id` é zero. As coordenadas são relativas
    /// ao painel, que fica em `(8, 31)` na janela.
    #[test]
    fn the_paperdoll_draws_one_henna_mark_per_class_slot() {
        let mut view = view();
        view.henna_slots = 3;
        view.henna = vec![
            HennaMark { symbol_id: 145, dye_item_id: 4589, icon: "icon.sym_a".into(), ..Default::default() },
            HennaMark { symbol_id: 146, dye_item_id: 0, icon: "icon.sym_b".into(), ..Default::default() },
        ];
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font(), 1.0);
        let at = |texture: &str| -> Vec<(f32, f32, f32)> {
            list.quads.iter().filter(|q| q.texture == texture).map(|q| (q.x, q.y, q.w)).collect()
        };
        let (px, py) = (PAPERDOLL_RECT.x as f32, PAPERDOLL_RECT.y as f32);
        assert_eq!(
            at(HENNA_MARK),
            (0..3).map(|i| (px + 215.0, py + 8.0 + 27.0 * i as f32, 26.0)).collect::<Vec<_>>()
        );
        assert_eq!(at("icon.sym_a"), [(px + 216.0, py + 9.0, 24.0)]);
        let icon = list.quads.iter().find(|q| q.texture == "icon.sym_a").expect("ícone");
        assert_eq!((icon.uw, icon.vh), (SLOT as f32, SLOT as f32), "o ícone de 32 é esticado pra 24, não cortado");
        assert_eq!(at("icon.sym_b"), [(px + 216.0, py + 36.0, 24.0)]);
        assert_eq!(at(HENNA_EMPTY), [(px + 217.0, py + 36.0, 24.0)], "só o de dye_item_id zero é tapado");

        assert_eq!(view.hit(PAPERDOLL_RECT.x + 215, PAPERDOLL_RECT.y + 8), Some(InventoryHit::Henna(0)));
        assert_eq!(view.hit(PAPERDOLL_RECT.x + 240, PAPERDOLL_RECT.y + 33), Some(InventoryHit::Henna(0)));
        assert!(!matches!(view.hit(PAPERDOLL_RECT.x + 215, PAPERDOLL_RECT.y + 34), Some(InventoryHit::Henna(_))), "o vão entre as molduras");
        assert_eq!(view.hit(PAPERDOLL_RECT.x + 215, PAPERDOLL_RECT.y + 35), Some(InventoryHit::Henna(1)));
        assert!(!matches!(view.hit(PAPERDOLL_RECT.x + 214, PAPERDOLL_RECT.y + 8), Some(InventoryHit::Henna(_))));
        assert_eq!(view.henna_mark(1).map(|m| m.symbol_id), Some(146));
        assert_eq!(view.henna_mark(2), None);

        view.henna_slots = 0;
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font(), 1.0);
        assert!(list.quads.iter().all(|q| q.texture != HENNA_MARK), "sem espaços não há moldura");
        assert!(!matches!(view.hit(PAPERDOLL_RECT.x + 215, PAPERDOLL_RECT.y + 8), Some(InventoryHit::Henna(_))), "sem espaços não há acerto");
    }

    /// Os espaços por classe saem da tabela de bytes de `0x10074550` (1, 2 ou 3) e do intervalo
    /// `0x58..=0x76`; fora disso o `0x10074500` devolve `-1`.
    #[test]
    fn henna_slots_follow_the_class_table_of_10074500() {
        assert_eq!(henna_slots(0), Some(1), "Human Fighter sem profissão");
        assert_eq!(henna_slots(1), Some(2), "primeira profissão");
        assert_eq!(henna_slots(2), Some(3), "segunda profissão");
        assert_eq!(henna_slots(10), Some(1), "Human Mage sem profissão");
        assert_eq!(henna_slots(57), Some(3), "última linha da tabela (0x39)");
        assert_eq!(henna_slots(58), None, "0x3a fica fora da tabela e abaixo de 0x58");
        assert_eq!(henna_slots(0x58), Some(3));
        assert_eq!(henna_slots(0x76), Some(3));
        assert_eq!(henna_slots(0x77), None);
        assert_eq!(henna_slots(-1), None);
    }

    #[test]
    fn listed_items_follow_the_tab_and_the_paperdoll_finds_the_equipped_ones() {
        let view = view();
        assert_eq!(view.listed().iter().map(|i| i.object_id).collect::<Vec<_>>(), vec![1, 4]);
        let quest = InventoryView { tab: InventoryTab::Quest, ..view.clone() };
        assert_eq!(quest.listed().iter().map(|i| i.object_id).collect::<Vec<_>>(), vec![6]);
        assert_eq!(view.counted(), 6);
        assert_eq!(view.paperdoll_item(5).map(|i| i.object_id), Some(2));
        assert_eq!(view.paperdoll_item(6).map(|i| i.object_id), Some(3));
        assert_eq!(view.paperdoll_item(8).map(|i| i.object_id), Some(5));
        assert_eq!(view.paperdoll_item(9), None);
        assert_eq!(view.grid_item(1).map(|i| i.object_id), Some(4));
        let mut bow = view.clone();
        bow.items[1].body_part = BODY_PART_LR_HAND;
        assert_eq!(bow.paperdoll_item(5).map(|i| i.object_id), Some(2));
        assert_eq!(bow.paperdoll_item(7), None);
        bow.items[2].body_part = BODY_PART_FULL_ARMOR;
        assert_eq!(bow.paperdoll_item(6).map(|i| i.object_id), Some(3));
        assert_eq!(bow.paperdoll_item(11), None);
    }

    #[test]
    fn thousands_and_weight_follow_the_client_formats() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(2500), "2,500");
        assert_eq!(thousands(1234567), "1,234,567");
        assert_eq!(thousands(-1234), "-1,234");
        assert_eq!(weight_bar(300, 1000), Some((25, WEIGHT_BARS[0])));
        assert_eq!(weight_bar(600, 1000), Some((51, WEIGHT_BARS[1])));
        assert_eq!(weight_bar(700, 1000), Some((59, WEIGHT_BARS[2])));
        assert_eq!(weight_bar(900, 1000), Some((76, WEIGHT_BARS[3])));
        assert_eq!(weight_bar(1100, 1000), Some((93, WEIGHT_BARS[4])));
        assert_eq!(weight_bar(1, 0), None);
        assert_eq!(weight_bar(1, 1000), None);
        assert_eq!(weight_percent_text(1, 1000), "0.00%");
        assert_eq!(weight_tip_text(1, 2), "[1]/[2]");
        assert_eq!(weight_percent_text(300, 1000), "30.00%");
        assert_eq!(weight_percent_text(1, 3), "33.33%");
        assert_eq!(weight_percent_text(1, 0), "0.00%");
    }

    #[test]
    fn two_handed_and_full_armor_draw_the_extra_icons_with_the_dual_cap() {
        let mut view = view();
        view.items[1].body_part = BODY_PART_LR_HAND;
        view.items[1].icons =
            vec!["icon.weapon_bow_i00".into(), "icon.weapon_bow_i01".into(), "icon.weapon_bow_i02".into()];
        view.items[2].body_part = BODY_PART_FULL_ARMOR;
        view.dwarven_craft = true;
        view.over_trash = true;
        view.over_recipe = true;
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font(), 1.0);
        let bow1: Vec<_> = list.quads.iter().filter(|q| q.texture == "icon.weapon_bow_i01").collect();
        assert_eq!((bow1[0].x, bow1[0].y), (15.0, 76.0));
        let bow2: Vec<_> = list.quads.iter().filter(|q| q.texture == "icon.weapon_bow_i02").collect();
        assert_eq!((bow2[0].x, bow2[0].y), (93.0, 76.0));
        let caps: Vec<_> = list.quads.iter().filter(|q| q.texture == DUAL_CAP).collect();
        assert_eq!(caps.len(), 2);
        assert_eq!((caps[0].x, caps[0].y), (93.0, 76.0));
        assert_eq!((caps[1].x, caps[1].y), (54.0, 114.0));
        let armor: Vec<_> = list.quads.iter().filter(|q| q.texture == "icon.armor_t01_u_i00").collect();
        assert_eq!(armor.len(), 2);
        assert_eq!((armor[0].x, armor[0].y, armor[1].x, armor[1].y), (54.0, 76.0, 54.0, 114.0));
        assert!(list.quads.iter().any(|q| q.texture == TRASH_DRAG));
        assert!(list.quads.iter().any(|q| q.texture == RECIPE_DRAG));
        assert!(!list.quads.iter().any(|q| q.texture == BLACK_TEXTURE));
        let quest = InventoryView { tab: InventoryTab::Quest, ..view };
        let mut list = DrawList::default();
        quest.paint(&mut list, 0, 0, &font(), 1.0);
        assert!(list.quads.iter().any(|q| q.texture == BLACK_TEXTURE));
    }

    #[test]
    fn a_bow_or_dual_weapon_draws_in_the_right_hand_slot_alone_while_ammunition_is_worn() {
        let tip = |weapon_kind: i32, etc_family: i32| Some(crate::item_tip::ItemTipData { weapon_kind, etc_family, ..Default::default() });
        for kind in BOW_WEAPON_KINDS {
            let mut view = InventoryView::with_sysstrings(|id| Some(format!("s{id}")));
            let mut bow = item(1, 7575, 1, true, BODY_PART_LR_HAND, "icon.bow0");
            bow.icons.push("icon.bow1".into());
            bow.icons.push("icon.bow2".into());
            bow.tip = tip(kind, 0);
            let mut arrow = item(2, 17, 100, true, BODY_PART_L_HAND, "icon.arrow0");
            arrow.tip = tip(0, 2);
            view.items = vec![bow.clone(), arrow];
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font(), 1.0);
            let bow0: Vec<_> = list.quads.iter().filter(|q| q.texture == "icon.bow0").collect();
            assert_eq!((bow0.len(), bow0[0].x, bow0[0].y), (1, 15.0, 76.0));
            assert!(!list.quads.iter().any(|q| q.texture == "icon.bow1" || q.texture == DUAL_CAP));
            view.items.truncate(1);
            let mut list = DrawList::default();
            view.paint(&mut list, 0, 0, &font(), 1.0);
            assert!(list.quads.iter().any(|q| q.texture == DUAL_CAP));
        }
    }

    #[test]
    fn dropping_an_item_on_another_cell_of_the_page_moves_it_there_and_past_the_end_moves_it_last() {
        let mut view = view();
        view.items.push(item(7, 1061, 1, false, 0, "icon.etc_potion_red_i00"));
        let ids = |view: &InventoryView| view.listed().iter().map(|i| i.object_id).collect::<Vec<_>>();
        assert_eq!(ids(&view), [1, 4, 7]);
        assert!(view.move_listed(7, 0));
        assert_eq!(ids(&view), [7, 1, 4]);
        assert!(view.move_listed(7, 1));
        assert_eq!(ids(&view), [1, 7, 4]);
        assert!(!view.move_listed(7, 1));
        assert!(view.move_listed(1, 40));
        assert_eq!(ids(&view), [7, 4, 1]);
        assert_eq!(view.items.iter().filter(|i| i.equipped).map(|i| i.object_id).collect::<Vec<_>>(), [2, 3, 5]);
        assert!(!view.move_listed(2, 0));
    }
}
