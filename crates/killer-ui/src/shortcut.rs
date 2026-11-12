use crate::color::TEXT;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::inventory::BLACK_TEXTURE;
use crate::keys::{ShortcutKeys, VK_0, VK_9, VK_F1, VK_F12, VK_OEM_MINUS, VK_OEM_PLUS};
use crate::cooltime::{self, CoolFrame};
use crate::skills::TOGGLE_EFFECT;
use crate::window::Rect;

pub const SHORTCUT_LONG: i32 = 504;
pub const SHORTCUT_SHORT: i32 = 46;

pub fn creation_position(viewport: (i32, i32)) -> (i32, i32) {
    (
        crate::window_restore::x87_to_int(f64::from(viewport.0 as f32) - f64::from(SHORTCUT_SHORT)),
        crate::window_restore::x87_to_int(f64::from(viewport.1 as f32) * 0.5 - 252.0),
    )
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    let (x, _) = creation_position(viewport);
    crate::initial_position::InitialPosition {
        enabled: true, horizontal_mode: 1, vertical_mode: 2,
        x: viewport.0.wrapping_sub(x), y: SHORTCUT_LONG,
    }
}

pub const GRIP: i32 = 12;
pub const BACK_LENGTH: i32 = 492;
pub const SLOTS: usize = 12;
pub const PAGES: i32 = 10;
pub const ICON_SIZE: i32 = 32;
pub const SLOT_START: i32 = 33;
pub const SLOT_STEP: i32 = 37;
pub const GROUP_GAP: i32 = 5;
pub const LABEL_SIZE: i32 = 16;
pub const PAGE_BUTTON_SIZE: i32 = 14;
pub const SMALL_BUTTON_SIZE: i32 = 15;
pub const NEXT_BUTTON_V: (i32, i32) = (31, 13);
pub const PREV_BUTTON_V: (i32, i32) = (1, 13);
pub const NEXT_BUTTON_H: (i32, i32) = (13, 1);
pub const PREV_BUTTON_H: (i32, i32) = (13, 31);
pub const ROTATE_BUTTON_V: (i32, i32) = (16, 489);
pub const ROTATE_BUTTON_H: (i32, i32) = (489, 16);
pub const JOYPAD_BUTTON_V: (i32, i32) = (1, 489);
pub const JOYPAD_BUTTON_H: (i32, i32) = (489, 1);
pub const SLOT_CROSS: i32 = 7;
pub const PAD_SIZE: (i32, i32) = (172, 82);
pub const PAD_BACK: &str = "L2UI_CH3.Joypad.joypad_back";
pub const PAD_BACK_RECT: Rect = Rect::new(12, 0, 160, 82);
pub const PAD_L: &str = "L2UI_ch3.Joypad.joypad_L";
pub const PAD_R: &str = "L2UI_ch3.Joypad.joypad_R";
pub const PAD_L_HOLD: &str = "L2UI_ch3.Joypad.joypad_L_HOLD";
pub const PAD_R_HOLD: &str = "L2UI_ch3.Joypad.joypad_R_HOLD";
pub const PAD_L_POS: (i32, i32) = (0x26, 6);
pub const PAD_R_POS: (i32, i32) = (0x82, 6);
pub const PAD_MARK_SIZE: (i32, i32) = (0x10, 0xE);
pub const PAD_NEXT_BUTTON: (i32, i32) = (0xD, 0x13);
pub const PAD_PREV_BUTTON: (i32, i32) = (0xD, 0x31);
pub const PAD_JOYPAD_BUTTON: (i32, i32) = (0x9D, 0x22);
pub const PAD_TOOLTIP_ANCHORS: [(i32, i32); 4] = [(0x23, 0x1A), (0x49, 0x2C), (0x6F, 0x1A), (0x49, 7)];
pub const PAD_SLOTS: [(i32, i32); 4] = [(0x27, 0x19), (0x4C, 0x2B), (0x71, 0x19), (0x4C, 7)];
pub const OUTLINE_SIZE: i32 = 34;
pub const MACRO_TEXT_OFFSET: (i32, i32) = (4, 4);
pub const MACRO_TEXT_STEP: i32 = 13;
pub const DRAG_OUT_MARGIN: i32 = 30;
pub const SOUND_EMPTY_SLOT: i32 = 2;
pub const SOUND_COOLTIME_END: i32 = 9;
pub const TOTAL_SLOTS: usize = SLOTS * PAGES as usize;
pub const SLOT_OUTLINE: &str = "L2UI_CH3.Etc.menu_outline";
pub const SLOT_OUTLINE_DOWN: &str = "L2UI_CH3.Etc.menu_outline_down";
pub const RECIPE_ICON: &str = "L2UI_ch3.ShortcutWnd.shortcut_recipeicon";
pub const SHORTCUT_TYPE_ITEM: i32 = 1;
pub const SHORTCUT_TYPE_SKILL: i32 = 2;
pub const SHORTCUT_TYPE_ACTION: i32 = 3;
pub const SHORTCUT_TYPE_MACRO: i32 = 4;
pub const SHORTCUT_TYPE_RECIPE: i32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotKind {
    Item { object_id: i32 },
    Skill { id: i32, level: i32 },
    Action { id: i32 },
    Macro { id: i32 },
    Recipe { object_id: i32 },
}

impl SlotKind {
    pub fn type_id(self) -> i32 {
        match self {
            SlotKind::Item { .. } => SHORTCUT_TYPE_ITEM,
            SlotKind::Skill { .. } => SHORTCUT_TYPE_SKILL,
            SlotKind::Action { .. } => SHORTCUT_TYPE_ACTION,
            SlotKind::Macro { .. } => SHORTCUT_TYPE_MACRO,
            SlotKind::Recipe { .. } => SHORTCUT_TYPE_RECIPE,
        }
    }

    pub fn register_id(self) -> i32 {
        match self {
            SlotKind::Item { object_id } | SlotKind::Recipe { object_id } => object_id,
            SlotKind::Skill { id, .. } | SlotKind::Action { id } | SlotKind::Macro { id } => id,
        }
    }

    pub fn skill_id(self) -> Option<i32> {
        match self {
            SlotKind::Skill { id, .. } => Some(id),
            _ => None,
        }
    }
}

pub fn is_soulshot_item(item_id: i32) -> bool {
    matches!(item_id, 1463..=1467 | 1835 | 2509..=2514 | 3947..=3952 | 5789..=5790 | 6645..=6647)
}

pub const QUICK_USE_INTERVAL_MS: u32 = 200;
pub const TOGGLE_SKILL_INTERVAL_MS: u32 = 1000;
pub const ACTION_ATTACK: i32 = 2;
pub const ACTION_PICK_UP: i32 = 5;
pub const ACTION_ATTACK_INTERVAL_MS: u32 = 200;
pub const ACTION_PICK_UP_INTERVAL_MS: u32 = 500;

pub fn is_quick_use_item(item_id: i32) -> bool {
    matches!(
        item_id,
        0x2DE..=0x2DF
            | 0x424..=0x425
            | 0x55E..=0x55F
            | 0x5B7..=0x5BB
            | 0x603..=0x604
            | 0x727..=0x72B
            | 0x9CD..=0x9D2
            | 0xF6B..=0xF70
            | 0x1245..=0x1248
            | 0x1472
            | 0x15D7..=0x15D8
            | 0x169D..=0x169E
            | 0x1793..=0x1794
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyMode {
    OnKeyUp,
    OnCoolTime,
    Interval(u32),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ShortcutSlot {
    pub kind: Option<SlotKind>,
    pub item_id: i32,
    pub icon: String,
    pub macro_acronym: String,
    pub name: String,
    pub toggled: bool,
    pub toggle_skill: bool,
    pub disabled: bool,
    pub soulshot: bool,
    pub cooldown: Option<f32>,
    pub cool: Option<CoolFrame>,
}

impl ShortcutSlot {
    pub fn new(kind: SlotKind, icon: &str) -> ShortcutSlot {
        ShortcutSlot { kind: Some(kind), icon: icon.to_string(), ..Default::default() }
    }

    pub fn icon(&self) -> &str {
        if self.icon.is_empty() { BLACK_TEXTURE } else { &self.icon }
    }

    pub fn key_mode(&self) -> KeyMode {
        match self.kind {
            Some(SlotKind::Item { .. }) if is_quick_use_item(self.item_id) => KeyMode::Interval(QUICK_USE_INTERVAL_MS),
            Some(SlotKind::Skill { .. }) if self.toggle_skill => KeyMode::Interval(TOGGLE_SKILL_INTERVAL_MS),
            Some(SlotKind::Skill { .. }) => KeyMode::OnCoolTime,
            Some(SlotKind::Action { id }) if id == ACTION_ATTACK => KeyMode::Interval(ACTION_ATTACK_INTERVAL_MS),
            Some(SlotKind::Action { id }) if id == ACTION_PICK_UP => KeyMode::Interval(ACTION_PICK_UP_INTERVAL_MS),
            _ => KeyMode::OnKeyUp,
        }
    }

    fn key_id(&self) -> i32 {
        match self.kind {
            Some(SlotKind::Item { .. }) => self.item_id,
            Some(SlotKind::Skill { id, .. } | SlotKind::Action { id } | SlotKind::Macro { id }) => id,
            Some(SlotKind::Recipe { object_id }) => object_id,
            None => 0,
        }
    }

    fn cool_idle(&self) -> bool {
        !matches!(self.cool, Some(CoolFrame::Running(_)))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PageButton {
    Modern(crate::modern_shortcut::ModernBarButton),
    Next,
    Prev,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AutoPotion {
    pub item_id: i32,
    pub name: String,
    pub icon: String,
    pub count: i32,
    pub on: bool,
    pub percent: i32,
    pub flash: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AutoShot {
    pub item_id: i32,
    pub name: String,
    pub icon: String,
    pub count: i32,
    pub on: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoBar {
    pub potions: [AutoPotion; 3],
    pub shots: [AutoShot; 2],
    pub carried: i32,
}

impl Default for AutoBar {
    fn default() -> Self {
        let potion = |percent| AutoPotion { percent, on: true, ..Default::default() };
        AutoBar { potions: [potion(70), potion(70), potion(50)], shots: Default::default(), carried: 0 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BarEvent {
    Use(usize),
    Sound(i32),
    Register { kind: i32, slot: usize, id: i32 },
    Delete(usize),
    AutoSoulShot { object_id: i32, enable: bool },
    AutoShot { item_id: i32, enable: bool },
    PageChanged(i32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropTarget {
    Bar(i32, i32),
    Margin,
    Outside(i32, i32),
}

pub fn outside_drop_target(bar: crate::window::Rect, x: i32, y: i32) -> DropTarget {
    let shift = |value: i32, start: i32, length: i32| {
        if value < start {
            value + DRAG_OUT_MARGIN
        } else if start + length < value {
            value - DRAG_OUT_MARGIN
        } else {
            value
        }
    };
    if bar.contains(shift(x, bar.x, bar.w), shift(y, bar.y, bar.h)) { DropTarget::Margin } else { DropTarget::Outside(x, y) }
}

pub const AUTO_FLASH_FRAMES: u32 = 30;

pub const SHORTCUT_BACK: &str = "L2UI_CH3.ShortcutWnd.shortcut_back";
pub const SHORTCUT_BACKV: &str = "L2UI_CH3.ShortcutWnd.shortcut_backv";
pub const SHORTCUT_NEXT: &str = "L2UI_CH3.ShortcutWnd.shortcut_next";
pub const SHORTCUT_PREV: &str = "L2UI_CH3.ShortcutWnd.shortcut_prev";
pub const SHORTCUT_NEXTV: &str = "L2UI_CH3.ShortcutWnd.shortcut_nextv";
pub const SHORTCUT_PREVV: &str = "L2UI_CH3.ShortcutWnd.shortcut_prevv";
pub const SHORTCUT_ROTATE: &str = "L2UI_CH3.ShortcutWnd.shortcut_rotate";
pub const SHORTCUT_JOYPAD: &str = "L2UI_CH3.ShortcutWnd.shortcut_joypad";
pub const SHORTCUT_F: [&str; SLOTS] = [
    "L2UI_CH3.ShortcutWnd.shortcut_f01",
    "L2UI_CH3.ShortcutWnd.shortcut_f02",
    "L2UI_CH3.ShortcutWnd.shortcut_f03",
    "L2UI_CH3.ShortcutWnd.shortcut_f04",
    "L2UI_CH3.ShortcutWnd.shortcut_f05",
    "L2UI_CH3.ShortcutWnd.shortcut_f06",
    "L2UI_CH3.ShortcutWnd.shortcut_f07",
    "L2UI_CH3.ShortcutWnd.shortcut_f08",
    "L2UI_CH3.ShortcutWnd.shortcut_f09",
    "L2UI_CH3.ShortcutWnd.shortcut_f10",
    "L2UI_CH3.ShortcutWnd.shortcut_f11",
    "L2UI_CH3.ShortcutWnd.shortcut_f12",
];

#[derive(Debug, Clone, PartialEq)]
pub struct ShortcutView {
    pub modern: Option<crate::modern_shortcut::ModernShortcuts>,
    pub vertical: bool,
    pub layout_updated: bool,
    pub pad_mode: bool,
    pub pad_hold: i32,
    pub joypad_visible: bool,
    pub page: i32,
    pub slots: Vec<ShortcutSlot>,
    pub armed: Vec<bool>,
    pub elapsed: Vec<u32>,
    key_kinds: Vec<Option<SlotKind>>,
    pub pressed: Option<usize>,
    pub auto: AutoBar,
    pub auto_pressed: Option<crate::modern_shortcut::AutoHit>,
    pub dragging: bool,
    pub enter_chatting: bool,
    pub dialog_visible: bool,
    pub events: Vec<BarEvent>,
}

impl ShortcutView {
    pub fn new(vertical: bool) -> ShortcutView {
        ShortcutView {
            modern: None,
            vertical,
            layout_updated: !vertical,
            pad_mode: false,
            pad_hold: 0,
            joypad_visible: true,
            page: 0,
            slots: vec![ShortcutSlot::default(); TOTAL_SLOTS],
            armed: vec![true; TOTAL_SLOTS],
            elapsed: vec![0; TOTAL_SLOTS],
            key_kinds: vec![None; TOTAL_SLOTS],
            pressed: None,
            auto: AutoBar::default(),
            auto_pressed: None,
            dragging: false,
            enter_chatting: false,
            dialog_visible: false,
            events: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        for slot in &mut self.slots {
            *slot = ShortcutSlot::default();
        }
    }

    pub fn set(&mut self, index: usize, slot: ShortcutSlot) {
        if let Some(target) = self.slots.get_mut(index) {
            if self.key_kinds[index] != slot.kind {
                self.key_kinds[index] = slot.kind;
                self.armed[index] = true;
                self.elapsed[index] = 0;
            }
            *target = slot;
        }
    }

    pub fn tick(&mut self, delta_ms: u32) {
        for index in 0..TOTAL_SLOTS {
            let slot = &self.slots[index];
            if slot.kind.is_none() || slot.key_id() <= 0 || self.armed[index] {
                continue;
            }
            match slot.key_mode() {
                KeyMode::OnCoolTime => {
                    if slot.cool_idle() {
                        self.armed[index] = true;
                    }
                }
                KeyMode::Interval(interval) => {
                    self.elapsed[index] = self.elapsed[index].saturating_add(delta_ms);
                    if self.elapsed[index] > interval {
                        self.elapsed[index] = 0;
                        self.armed[index] = true;
                    }
                }
                KeyMode::OnKeyUp => {}
            }
        }
    }

    fn key_released(&mut self, index: usize) {
        if !self.armed[index] && matches!(self.slots[index].key_mode(), KeyMode::OnKeyUp | KeyMode::OnCoolTime) {
            self.armed[index] = true;
        }
    }

    pub fn remove(&mut self, index: usize) {
        if let Some(target) = self.slots.get_mut(index) {
            *target = ShortcutSlot::default();
        }
    }

    pub fn slot(&self, index: usize) -> Option<&ShortcutSlot> {
        self.slots.get(index).filter(|s| s.kind.is_some())
    }

    pub fn take_events(&mut self) -> Vec<BarEvent> {
        std::mem::take(&mut self.events)
    }

    pub fn set_page(&mut self, page: i32) {
        if !(0..PAGES).contains(&page) {
            return;
        }
        if page != self.page {
            let first = (self.page * SLOTS as i32) as usize;
            for index in first..first + SLOTS {
                self.armed[index] = true;
                self.elapsed[index] = 0;
            }
        }
        self.page = page;
        self.events.push(BarEvent::PageChanged(page));
    }

    pub fn next_page(&mut self) {
        let page = if self.page < PAGES - 1 { self.page + 1 } else { 0 };
        self.set_page(page);
    }

    pub fn prev_page(&mut self) {
        let page = if self.page > 0 { self.page - 1 } else { PAGES - 1 };
        self.set_page(page);
    }

    pub fn next_button(&self) -> Rect {
        if self.pad_mode {
            return Rect::new(PAD_NEXT_BUTTON.0, PAD_NEXT_BUTTON.1, SMALL_BUTTON_SIZE, SMALL_BUTTON_SIZE);
        }
        let (x, y) = if self.vertical { NEXT_BUTTON_V } else { NEXT_BUTTON_H };
        Rect::new(x, y, PAGE_BUTTON_SIZE, PAGE_BUTTON_SIZE)
    }

    pub fn prev_button(&self) -> Rect {
        if self.pad_mode {
            return Rect::new(PAD_PREV_BUTTON.0, PAD_PREV_BUTTON.1, SMALL_BUTTON_SIZE, SMALL_BUTTON_SIZE);
        }
        let (x, y) = if self.vertical { PREV_BUTTON_V } else { PREV_BUTTON_H };
        Rect::new(x, y, PAGE_BUTTON_SIZE, PAGE_BUTTON_SIZE)
    }

    pub fn page_button_at(&self, x: i32, y: i32) -> Option<PageButton> {
        if let Some(modern) = self.modern { return modern.button_at(x, y).map(PageButton::Modern); }
        if self.next_button().contains(x, y) {
            Some(PageButton::Next)
        } else if self.prev_button().contains(x, y) {
            Some(PageButton::Prev)
        } else {
            None
        }
    }

    pub fn press_page_button(&mut self, button: PageButton) {
        match button {
            PageButton::Modern(button) => {
                use crate::modern_shortcut::ModernBarButton::*;
                let Some(modern) = self.modern.as_mut() else { return };
                match button {
                    Expand => modern.count = (modern.count + 1).min(5),
                    Collapse => modern.count = modern.count.saturating_sub(1).max(1),
                    Lock => modern.locked = !modern.locked,
                    Next(0) => self.next_page(),
                    Prev(0) => self.prev_page(),
                    Next(row) | Prev(row) => {
                        if let Some(page) = row.checked_sub(1).and_then(|row| modern.pages.get_mut(row)) {
                            *page = (*page + if matches!(button, Next(_)) { 1 } else { PAGES - 1 }) % PAGES;
                        }
                    }
                }
            }
            PageButton::Next => self.next_page(),
            PageButton::Prev => self.prev_page(),
        }
    }

    fn key_slot(&self, vk: u32) -> Option<usize> {
        let base = self.page as usize * SLOTS;
        if (VK_F1..=VK_F12).contains(&vk) {
            return Some(base + (vk - VK_F1) as usize);
        }
        if !self.enter_chatting || self.dialog_visible {
            return None;
        }
        match vk {
            VK_0 => Some(base + 9),
            0x31..=VK_9 => Some(base + (vk - 0x31) as usize),
            VK_OEM_PLUS => Some(base + 11),
            VK_OEM_MINUS => Some(base + 10),
            _ => None,
        }
    }

    pub fn slot_rect(&self, index_in_page: usize) -> Rect {
        if let Some(modern) = self.modern { return modern.slot_rect(0, index_in_page); }
        let offset = slot_offsets()[index_in_page];
        if self.vertical {
            Rect::new(SLOT_CROSS, offset, ICON_SIZE, ICON_SIZE)
        } else {
            Rect::new(offset, SLOT_CROSS, ICON_SIZE, ICON_SIZE)
        }
    }

    pub fn slot_at(&self, x: i32, y: i32) -> Option<usize> {
        if let Some(modern) = self.modern { return modern.slot_at(x, y, self.page); }
        if self.pad_mode {
            return self.pad_slot_at(x, y);
        }
        let (across, along) = if self.vertical { (x, y) } else { (y, x) };
        if !(across > 5 && across <= ICON_SIZE + 6) {
            return None;
        }
        let mut found = None;
        for (i, offset) in slot_offsets().into_iter().enumerate() {
            if along >= offset && along <= ICON_SIZE + offset {
                found = Some(self.page as usize * SLOTS + i);
            }
        }
        found
    }

    pub fn tooltip_anchor(&self, index: usize) -> (i32, i32) {
        let rel = index.saturating_sub(self.page as usize * SLOTS).min(SLOTS - 1);
        if self.pad_mode {
            return PAD_TOOLTIP_ANCHORS[rel % 4];
        }
        let offset = slot_offsets()[rel];
        if self.vertical { (SLOT_CROSS, offset) } else { (offset, SLOT_CROSS) }
    }

    pub fn pad_slot_at(&self, x: i32, y: i32) -> Option<usize> {
        PAD_SLOTS.iter().position(|&(bx, by)| x >= bx && x < ICON_SIZE + bx && y >= by && y <= ICON_SIZE + by).map(|i| {
            i + self.page as usize * SLOTS + self.pad_hold as usize * 4
        })
    }

    pub fn set_pad_hold_left(&mut self, hold: bool) {
        self.pad_hold = i32::from(hold);
    }

    pub fn set_pad_hold_right(&mut self, hold: bool) {
        self.pad_hold = if hold { 2 } else { 0 };
    }

    pub fn pad_clicked(&mut self, rect: Rect) -> Option<Rect> {
        if !self.pad_mode {
            if !self.joypad_visible {
                return None;
            }
            self.pad_mode = true;
            self.pad_hold = 0;
            let (w, h) = PAD_SIZE;
            return Some(Rect::new(rect.x, rect.y, w, h));
        }
        self.pad_mode = false;
        Some(self.relayout(rect))
    }

    pub fn use_slot(&mut self, index: usize) {
        if self.slot(index).is_none() {
            self.events.push(BarEvent::Sound(SOUND_EMPTY_SLOT));
            return;
        }
        self.events.push(BarEvent::Use(index));
    }

    pub fn auto_at(&self, x: i32, y: i32) -> Option<crate::modern_shortcut::AutoHit> {
        self.modern.and_then(|modern| modern.auto_at(x, y))
    }

    pub fn assign_auto(&mut self, hit: crate::modern_shortcut::AutoHit, item_id: i32) -> bool {
        use crate::modern_shortcut::AutoHit;
        if self.edit_locked() || item_id <= 0 {
            return false;
        }
        match hit {
            AutoHit::Potion(vital) => {
                let potion = &mut self.auto.potions[vital.index()];
                potion.item_id = item_id;
                true
            }
            AutoHit::Shot(kind) if kind.accepts(item_id) => {
                let shot = &mut self.auto.shots[kind.index()];
                if shot.on && shot.item_id != item_id {
                    self.events.push(BarEvent::AutoShot { item_id: shot.item_id, enable: false });
                }
                shot.item_id = item_id;
                true
            }
            _ => false,
        }
    }

    pub fn auto_drag(&mut self, x: i32) {
        if let Some(crate::modern_shortcut::AutoHit::Level(vital)) = self.auto_pressed
            && let Some(modern) = self.modern
        {
            self.auto.potions[vital.index()].percent = modern.auto_percent(vital, x);
        }
    }

    pub fn auto_dragging(&self) -> Option<crate::modern_shortcut::Vital> {
        match self.auto_pressed {
            Some(crate::modern_shortcut::AutoHit::Level(vital)) => Some(vital),
            _ => None,
        }
    }

    pub fn auto_wheel(&mut self, x: i32, y: i32, ticks: i32) -> bool {
        use crate::modern_shortcut::{AUTO_MAX, AUTO_MIN, AUTO_STEP, AutoHit};
        let Some(AutoHit::Potion(vital) | AutoHit::Level(vital)) = self.auto_at(x, y) else { return false };
        let potion = &mut self.auto.potions[vital.index()];
        potion.percent = (potion.percent + ticks.signum() * AUTO_STEP * ticks.abs().max(1)).clamp(AUTO_MIN, AUTO_MAX);
        true
    }

    fn auto_release(&mut self, hit: crate::modern_shortcut::AutoHit) {
        use crate::modern_shortcut::AutoHit;
        match hit {
            AutoHit::Potion(vital) => {
                let potion = &mut self.auto.potions[vital.index()];
                potion.on = !potion.on;
            }
            AutoHit::Level(_) => {}
            AutoHit::Shot(kind) if self.auto.shots[kind.index()].item_id != 0 => {
                let shot = &self.auto.shots[kind.index()];
                self.events.push(BarEvent::AutoShot { item_id: shot.item_id, enable: !shot.on });
            }
            _ => {}
        }
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        self.auto_pressed = self.auto_at(x, y);
        self.auto_drag(x);
        let hit = self.slot_at(x, y);
        self.pressed = hit;
        self.dragging = false;
        if let Some(index) = hit
            && self.slot(index).is_some()
        {
            self.dragging = !self.edit_locked();
        }
    }

    /// Descarta o arrasto sem gerar evento: usado quando o slot foi solto numa janela que o
    /// consome, como a linha de comando do editor de macro.
    pub fn cancel_drag(&mut self) {
        self.pressed = None;
        self.dragging = false;
    }

    pub fn mouse_up(&mut self, target: DropTarget) {
        if let Some(pressed) = self.auto_pressed.take()
            && let DropTarget::Bar(x, y) = target
            && self.auto_at(x, y) == Some(pressed)
        {
            self.auto_release(pressed);
        }
        let Some(pressed) = self.pressed.take() else {
            self.dragging = false;
            return;
        };
        let dragging = std::mem::take(&mut self.dragging);
        if !self.armed.get(pressed).copied().unwrap_or(true) {
            return;
        }
        match target {
            DropTarget::Bar(x, y) => {
                let released = self.slot_at(x, y);
                if released == Some(pressed) {
                    if self.slot(pressed).is_some() {
                        self.use_slot(pressed);
                        self.armed[pressed] = false;
                        self.key_released(pressed);
                    }
                } else if dragging
                    && let Some(dest) = released
                    && let Some(source) = self.slot(pressed).and_then(|s| s.kind)
                {
                    self.events.push(BarEvent::Register {
                        kind: source.type_id(),
                        slot: dest,
                        id: source.register_id(),
                    });
                    self.events.push(BarEvent::Delete(pressed));
                    if let Some(swapped) = self.slot(dest).and_then(|s| s.kind) {
                        self.events.push(BarEvent::Register {
                            kind: swapped.type_id(),
                            slot: pressed,
                            id: swapped.register_id(),
                        });
                    }
                }
            }
            DropTarget::Margin => {}
            DropTarget::Outside(_, _) => {
                if dragging && let Some(slot) = self.slot(pressed) {
                    if slot.soulshot
                        && let Some(SlotKind::Item { object_id }) = slot.kind
                        && !self
                            .slots
                            .iter()
                            .enumerate()
                            .any(|(i, s)| i != pressed && s.kind == Some(SlotKind::Item { object_id }) && s.disabled)
                    {
                        self.events.push(BarEvent::AutoSoulShot { object_id, enable: false });
                    }
                    self.events.push(BarEvent::Delete(pressed));
                }
            }
        }
    }

    pub fn right_click(&mut self, x: i32, y: i32) {
        if let Some(crate::modern_shortcut::AutoHit::Potion(vital)) = self.auto_at(x, y) {
            if !self.edit_locked() {
                self.auto.potions[vital.index()].item_id = 0;
            }
            return;
        }
        if let Some(index) = self.slot_at(x, y)
            && let Some(slot) = self.slot(index)
            && slot.soulshot
            && let Some(SlotKind::Item { object_id }) = slot.kind
        {
            self.events.push(BarEvent::AutoSoulShot { object_id, enable: !slot.disabled });
        }
    }

    pub fn edit_locked(&self) -> bool { self.modern.is_some_and(|modern| modern.locked) }

    pub fn size(&self) -> (i32, i32) {
        if let Some(modern) = self.modern { return (crate::modern_shortcut::WIDTH, modern.height()); }
        if self.pad_mode { return PAD_SIZE; }
        if self.vertical { (SHORTCUT_SHORT, SHORTCUT_LONG) } else { (SHORTCUT_LONG, SHORTCUT_SHORT) }
    }

    pub fn frame(&self) -> FrameCtrl {
        FrameCtrl::new(if self.vertical && !self.pad_mode { FrameMode::ThinBar } else { FrameMode::VerticalGrip })
    }

    pub fn grip(&self) -> Rect {
        if self.modern.is_some() { return Rect::new(0, crate::modern_shortcut::AUTO, crate::modern_shortcut::WIDTH - 34, crate::modern_shortcut::TOP); }
        let (w, h) = self.size();
        self.frame().grip(w, h)
    }

    pub fn rotate_button(&self) -> Rect {
        if self.modern.is_some() { return Rect::new(0,0,0,0); }
        let (x, y) = if self.vertical { ROTATE_BUTTON_V } else { ROTATE_BUTTON_H };
        Rect::new(x, y, SMALL_BUTTON_SIZE, SMALL_BUTTON_SIZE)
    }

    pub fn joypad_button(&self) -> Rect {
        let (x, y) = if self.pad_mode {
            PAD_JOYPAD_BUTTON
        } else if self.vertical {
            JOYPAD_BUTTON_V
        } else {
            JOYPAD_BUTTON_H
        };
        Rect::new(x, y, SMALL_BUTTON_SIZE, SMALL_BUTTON_SIZE)
    }

    pub fn rotate(&mut self, rect: Rect) -> Rect {
        self.vertical = !self.vertical;
        self.relayout(rect)
    }

    pub fn relayout(&mut self, rect: Rect) -> Rect {
        self.layout_updated = true;
        let (w, h) = self.size();
        let x = rect.x + rect.w - rect.h;
        let y = rect.y + rect.h - rect.w;
        if self.vertical { Rect::new(x, y.max(0), w, h) } else { Rect::new(x.max(0), y, w, h) }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        let (w, h) = self.size();
        self.frame().paint(list, x, y, w, h, font, alpha);
        if self.pad_mode {
            self.paint_pad(list, x, y, font, alpha);
        } else if self.vertical {
            list.draw_texture(
                x,
                y + GRIP,
                SHORTCUT_SHORT,
                BACK_LENGTH,
                0,
                0,
                SHORTCUT_SHORT,
                BACK_LENGTH,
                SHORTCUT_BACKV,
                alpha,
            );
            let (next, prev) = (self.next_button(), self.prev_button());
            let (next_texture, prev_texture) =
                if self.layout_updated { (SHORTCUT_NEXT, SHORTCUT_PREV) } else { (SHORTCUT_NEXTV, SHORTCUT_PREVV) };
            button(list, x + next.x, y + next.y, PAGE_BUTTON_SIZE, next_texture);
            button(list, x + prev.x, y + prev.y, PAGE_BUTTON_SIZE, prev_texture);
            small_button(list, x, y, self.rotate_button(), SHORTCUT_ROTATE);
            if self.joypad_visible { small_button(list, x, y, self.joypad_button(), SHORTCUT_JOYPAD); }
            self.paint_page(list, x, y, 22, 16, 17, 21, font);
            for (i, slot) in slot_offsets().into_iter().enumerate() {
                self.paint_slot(list, x + SLOT_CROSS, y + slot, i, font, true);
                list.draw_texture(
                    x + 4,
                    y + slot - 1,
                    LABEL_SIZE,
                    LABEL_SIZE,
                    0,
                    0,
                    LABEL_SIZE,
                    LABEL_SIZE,
                    SHORTCUT_F[i],
                    1.0,
                );
            }
        } else {
            list.draw_texture(
                x + GRIP,
                y,
                BACK_LENGTH,
                SHORTCUT_SHORT,
                0,
                0,
                BACK_LENGTH,
                SHORTCUT_SHORT,
                SHORTCUT_BACK,
                alpha,
            );
            let (next, prev) = (self.next_button(), self.prev_button());
            button(list, x + next.x, y + next.y, PAGE_BUTTON_SIZE, SHORTCUT_NEXTV);
            button(list, x + prev.x, y + prev.y, PAGE_BUTTON_SIZE, SHORTCUT_PREVV);
            small_button(list, x, y, self.rotate_button(), SHORTCUT_ROTATE);
            if self.joypad_visible { small_button(list, x, y, self.joypad_button(), SHORTCUT_JOYPAD); }
            self.paint_page(list, x, y, 19, 18, 14, 18, font);
            for (i, slot) in slot_offsets().into_iter().enumerate() {
                self.paint_slot(list, x + slot, y + SLOT_CROSS, i, font, true);
                list.draw_texture(
                    x + slot - 1,
                    y + 4,
                    LABEL_SIZE,
                    LABEL_SIZE,
                    0,
                    0,
                    LABEL_SIZE,
                    LABEL_SIZE,
                    SHORTCUT_F[i],
                    1.0,
                );
            }
        }
    }

    fn paint_pad(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        let back = PAD_BACK_RECT;
        list.draw_texture(x + back.x, y + back.y, back.w, back.h, 0, 0, back.w, back.h, PAD_BACK, alpha);
        let (next, prev) = (self.next_button(), self.prev_button());
        button(list, x + next.x, y + next.y, SMALL_BUTTON_SIZE, SHORTCUT_NEXTV);
        button(list, x + prev.x, y + prev.y, SMALL_BUTTON_SIZE, SHORTCUT_PREVV);
        small_button(list, x, y, self.joypad_button(), SHORTCUT_JOYPAD);
        let (left, right) = match self.pad_hold {
            1 => (PAD_L_HOLD, PAD_R),
            2 => (PAD_L, PAD_R_HOLD),
            _ => (PAD_L, PAD_R),
        };
        let (mw, mh) = PAD_MARK_SIZE;
        list.draw_texture(x + PAD_L_POS.0, y + PAD_L_POS.1, mw, mh, 0, 0, mw, mh, left, 1.0);
        list.draw_texture(x + PAD_R_POS.0, y + PAD_R_POS.1, mw, mh, 0, 0, mw, mh, right, 1.0);
        self.paint_page(list, x, y, 19, 37, 13, 17, font);
        for k in (self.pad_hold as usize * 4)..(self.pad_hold as usize * 4 + 4) {
            if k >= SLOTS {
                return;
            }
            let (sx, sy) = PAD_SLOTS[k % 4];
            self.paint_slot(list, x + sx, y + sy, k, font, false);
            list.draw_texture(x + sx - 1, y + sy - 1, OUTLINE_SIZE, OUTLINE_SIZE, 0, 0, OUTLINE_SIZE, OUTLINE_SIZE, SLOT_OUTLINE, 1.0);
            list.draw_texture(x + sx - 1, y + sy - 1, LABEL_SIZE, LABEL_SIZE, 0, 0, LABEL_SIZE, LABEL_SIZE, SHORTCUT_F[k], 1.0);
        }
    }

    fn paint_slot(&self, list: &mut DrawList, x: i32, y: i32, index_in_page: usize, font: &BitmapFont, frame: bool) {
        let index = self.page as usize * SLOTS + index_in_page;
        let Some(slot) = self.slot(index) else { return };
        list.draw_texture(x, y, ICON_SIZE, ICON_SIZE, 0, 0, ICON_SIZE, ICON_SIZE, slot.icon(), 1.0);
        if frame {
            list.draw_texture(
                x - 1,
                y - 1,
                OUTLINE_SIZE,
                OUTLINE_SIZE,
                0,
                0,
                OUTLINE_SIZE,
                OUTLINE_SIZE,
                SLOT_OUTLINE,
                1.0,
            );
        }
        if self.pressed == Some(index) {
            list.draw_texture(
                x - 1,
                y - 1,
                OUTLINE_SIZE,
                OUTLINE_SIZE,
                0,
                0,
                OUTLINE_SIZE,
                OUTLINE_SIZE,
                SLOT_OUTLINE_DOWN,
                1.0,
            );
        }
        if matches!(slot.kind, Some(SlotKind::Macro { .. })) {
            let (mut cx, mut cy) = (x + MACRO_TEXT_OFFSET.0, y + MACRO_TEXT_OFFSET.1);
            for (i, letter) in slot.macro_acronym.chars().enumerate() {
                list.draw_text(cx, cy, TEXT, &letter.to_string(), font);
                if i % 2 == 0 {
                    cx += MACRO_TEXT_STEP;
                } else {
                    cy += MACRO_TEXT_STEP;
                    cx -= MACRO_TEXT_STEP;
                }
            }
        }
        if slot.toggled || slot.disabled {
            list.draw_texture(x, y, ICON_SIZE, ICON_SIZE, 0, 0, ICON_SIZE, ICON_SIZE, TOGGLE_EFFECT, 1.0);
        }
        if self.vertical && !self.pad_mode && matches!(slot.kind, Some(SlotKind::Recipe { .. })) {
            list.draw_texture(x, y, ICON_SIZE, ICON_SIZE, 0, 0, ICON_SIZE, ICON_SIZE, RECIPE_ICON, 1.0);
        }
        if let Some(frame) = slot.cool {
            cooltime::paint(list, x, y, ICON_SIZE, frame);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn paint_page(
        &self,
        list: &mut DrawList,
        x: i32,
        y: i32,
        center_x: i32,
        digit_y: i32,
        tens_x: i32,
        ones_x: i32,
        font: &BitmapFont,
    ) {
        let number = self.page + 1;
        if number < 10 {
            let text = number.to_string();
            list.draw_digits(x + center_x - font.measure(&text) as i32 / 2, y + digit_y, &text, TEXT);
        } else if number == 10 {
            list.draw_digits(x + tens_x, y + digit_y, "1", TEXT);
            list.draw_digits(x + ones_x, y + digit_y, "0", TEXT);
        }
    }
}

impl ShortcutKeys for ShortcutView {
    fn key_down(&mut self, vk: u32) -> bool {
        let Some(index) = self.key_slot(vk) else { return false };
        if !self.armed[index] {
            return true;
        }
        if index >= TOTAL_SLOTS {
            return false;
        }
        if self.slot(index).is_none() {
            self.events.push(BarEvent::Sound(SOUND_EMPTY_SLOT));
            return false;
        }
        self.events.push(BarEvent::Use(index));
        self.armed[index] = false;
        false
    }

    fn key_up(&mut self, vk: u32) -> bool {
        if let Some(index) = self.key_slot(vk) {
            self.key_released(index);
        }
        false
    }
}

pub fn slot_offsets() -> [i32; SLOTS] {
    let mut offsets = [0; SLOTS];
    let mut position = SLOT_START;
    for (i, offset) in offsets.iter_mut().enumerate() {
        *offset = position;
        position += GROUP_GAP + ICON_SIZE;
        if i == 3 || i == 7 {
            position += GROUP_GAP;
        }
    }
    offsets
}

fn button(list: &mut DrawList, x: i32, y: i32, size: i32, texture: &str) {
    list.draw_texture(x, y, size, size, 0, 0, size, size, texture, 1.0);
}

fn small_button(list: &mut DrawList, x: i32, y: i32, rect: Rect, texture: &str) {
    button(list, x + rect.x, y + rect.y, rect.w, texture);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_vertical_bar_swaps_its_arrows_to_next_and_prev_after_it_is_laid_out_again() {
        let fresh = ShortcutView::new(true);
        assert_eq!(with_texture(&painted(&fresh, 1.0), SHORTCUT_NEXTV).len(), 1);
        let mut turned = ShortcutView::new(false);
        turned.rotate(crate::window::Rect::new(100, 100, 504, 46));
        assert!(turned.vertical && turned.layout_updated);
        let quads = painted(&turned, 1.0);
        assert_eq!((with_texture(&quads, SHORTCUT_NEXT)[0].x, with_texture(&quads, SHORTCUT_NEXT)[0].y), (31.0, 13.0));
        assert_eq!((with_texture(&quads, SHORTCUT_PREV)[0].x, with_texture(&quads, SHORTCUT_PREV)[0].y), (1.0, 13.0));
    }

    #[test]
    fn slots_are_hit_over_the_inclusive_rectangle_of_slot_at() {
        let mut view = ShortcutView::new(true);
        assert_eq!(view.slot_at(6, 33), Some(0));
        assert_eq!(view.slot_at(5, 33), None);
        assert_eq!(view.slot_at(38, 65), Some(0));
        assert_eq!(view.slot_at(39, 33), None);
        assert_eq!(view.slot_at(10, 66), None);
        view.vertical = false;
        assert_eq!(view.slot_at(65, 6), Some(0));
        assert_eq!(view.slot_at(33, 5), None);
    }

    #[test]
    fn the_pad_layout_is_172_by_82_with_a_diamond_of_four_slots_for_the_hold() {
        let mut view = ShortcutView::new(true);
        let rect = view.pad_clicked(crate::window::Rect::new(10, 10, 46, 504)).unwrap();
        assert_eq!(rect, crate::window::Rect::new(10, 10, 172, 82));
        assert_eq!(view.size(), (172, 82));
        assert_eq!(view.pad_slot_at(0x27, 0x19), Some(0));
        assert_eq!(view.pad_slot_at(0x4C, 0x2B + 32), Some(1));
        assert_eq!(view.pad_slot_at(0x71 + 32, 0x19), None);
        view.set_pad_hold_right(true);
        assert_eq!(view.pad_slot_at(0x4C, 7), Some(11));
        view.page = 2;
        assert_eq!(view.slot_at(0x27, 0x19), Some(2 * SLOTS + 8));
        let quads = painted(&view, 1.0);
        assert!(!with_texture(&quads, PAD_BACK).is_empty());
        assert!(!with_texture(&quads, PAD_R_HOLD).is_empty() && !with_texture(&quads, PAD_L).is_empty());
        assert_eq!(with_texture(&quads, SLOT_OUTLINE).len(), 4);
        assert_eq!(view.rotate_button().w, SMALL_BUTTON_SIZE);
        let back = view.pad_clicked(rect).unwrap();
        assert_eq!(back, crate::window::Rect::new(100, 0, 46, 504));
        assert!(!view.pad_mode);
    }

    #[test]
    fn a_drop_within_thirty_pixels_of_the_bar_does_not_delete_the_slot() {
        let bar = crate::window::Rect::new(100, 200, 504, 46);
        assert_eq!(outside_drop_target(bar, 90, 190), DropTarget::Margin);
        assert_eq!(outside_drop_target(bar, 70, 200), DropTarget::Margin);
        assert_eq!(outside_drop_target(bar, 69, 200), DropTarget::Outside(69, 200));
        assert_eq!(outside_drop_target(bar, 300, 246 + 29), DropTarget::Margin);
        assert_eq!(outside_drop_target(bar, 300, 246 + 30), DropTarget::Outside(300, 276));
        assert_eq!(outside_drop_target(bar, 604 + 29, 220), DropTarget::Margin);
    }

    use crate::draw::{DIGIT_TEXTURE, Quad};
    use crate::font::{Glyphs, gly};
    use crate::keys::VK_F3;

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        let bytes = gly(&[(0x20, &glyphs)], (1024, 128));
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(&bytes).unwrap())
    }

    fn painted(view: &ShortcutView, alpha: f32) -> Vec<Quad> {
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font(), alpha);
        list.quads
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    #[test]
    fn is_soulshot_item_1006e5f0_covers_every_shot_range() {
        let shots: Vec<i32> = (0..10000).filter(|id| is_soulshot_item(*id)).collect();
        assert_eq!(shots, [1463, 1464, 1465, 1466, 1467, 1835, 2509, 2510, 2511, 2512, 2513, 2514, 3947, 3948, 3949, 3950, 3951, 3952, 5789, 5790, 6645, 6646, 6647]);
    }

    #[test]
    fn slots_step_thirty_seven_with_five_more_after_the_fourth_and_eighth() {
        assert_eq!(slot_offsets(), [33, 70, 107, 144, 186, 223, 260, 297, 339, 376, 413, 450]);
    }

    #[test]
    fn option_read_100c827a_controls_the_joypad_child_in_both_orientations() {
        for vertical in [false, true] {
            let mut view = ShortcutView::new(vertical);
            for visible in [false, true, false] {
                view.joypad_visible = visible;
                let quads = painted(&view, 1.0);
                assert_eq!(with_texture(&quads, SHORTCUT_JOYPAD).len(), usize::from(visible));
                assert_eq!(with_texture(&quads, SHORTCUT_ROTATE).len(), 1);
            }
        }
    }

    #[test]
    fn vertical_bar_is_46_by_504_with_the_thin_bar_on_top() {
        let view = ShortcutView::new(true);
        assert_eq!(view.size(), (46, 504));
        assert_eq!(view.frame().grip(46, 504), crate::window::Rect::new(0, 0, 46, 12));
        let quads = painted(&view, 0.5);
        let back = with_texture(&quads, SHORTCUT_BACKV);
        assert_eq!(
            (back[0].x, back[0].y, back[0].w, back[0].h, back[0].uw, back[0].vh, back[0].color[3]),
            (0.0, 12.0, 46.0, 492.0, 46.0, 492.0, 0.5)
        );
        let bar = with_texture(&quads, crate::frame::SMALLBAR_H2);
        assert_eq!((bar[0].y, bar[0].h, bar[0].color[3]), (0.0, 12.0, 0.5));
        let next = with_texture(&quads, SHORTCUT_NEXTV);
        assert_eq!((next[0].x, next[0].y, next[0].w, next[0].color[3]), (31.0, 13.0, 14.0, 1.0));
        assert_eq!((with_texture(&quads, SHORTCUT_PREVV)[0].x, with_texture(&quads, SHORTCUT_PREVV)[0].y), (1.0, 13.0));
        assert_eq!(
            (
                with_texture(&quads, SHORTCUT_ROTATE)[0].x,
                with_texture(&quads, SHORTCUT_ROTATE)[0].y,
                with_texture(&quads, SHORTCUT_ROTATE)[0].w
            ),
            (16.0, 489.0, 15.0)
        );
        assert_eq!(
            (with_texture(&quads, SHORTCUT_JOYPAD)[0].x, with_texture(&quads, SHORTCUT_JOYPAD)[0].y),
            (1.0, 489.0)
        );
        let labels: Vec<(f32, f32)> =
            SHORTCUT_F.iter().map(|t| (with_texture(&quads, t)[0].x, with_texture(&quads, t)[0].y)).collect();
        assert_eq!(labels[0], (4.0, 32.0));
        assert_eq!(labels[11], (4.0, 449.0));
        assert!(SHORTCUT_F.iter().all(|t| with_texture(&quads, t)[0].w == 16.0));
        let page = with_texture(&quads, DIGIT_TEXTURE);
        assert_eq!((page[0].x, page[0].y, page[0].u), (19.0, 16.0, 8.0));
        assert!(with_texture(&quads, SHORTCUT_BACK).is_empty());
    }

    #[test]
    fn horizontal_bar_is_504_by_46_with_the_grip_on_the_left() {
        let view = ShortcutView::new(false);
        assert_eq!(view.size(), (504, 46));
        assert_eq!(view.frame().grip(504, 46), crate::window::Rect::new(0, 0, 12, 46));
        let quads = painted(&view, 1.0);
        let back = with_texture(&quads, SHORTCUT_BACK);
        assert_eq!((back[0].x, back[0].y, back[0].w, back[0].h), (12.0, 0.0, 492.0, 46.0));
        assert_eq!((with_texture(&quads, SHORTCUT_NEXTV)[0].x, with_texture(&quads, SHORTCUT_NEXTV)[0].y), (13.0, 1.0));
        assert_eq!((with_texture(&quads, SHORTCUT_PREVV)[0].x, with_texture(&quads, SHORTCUT_PREVV)[0].y), (13.0, 31.0));
        assert_eq!(
            (with_texture(&quads, SHORTCUT_ROTATE)[0].x, with_texture(&quads, SHORTCUT_ROTATE)[0].y),
            (489.0, 16.0)
        );
        assert_eq!(
            (with_texture(&quads, SHORTCUT_JOYPAD)[0].x, with_texture(&quads, SHORTCUT_JOYPAD)[0].y),
            (489.0, 1.0)
        );
        assert_eq!((with_texture(&quads, SHORTCUT_F[0])[0].x, with_texture(&quads, SHORTCUT_F[0])[0].y), (32.0, 4.0));
        assert_eq!(with_texture(&quads, SHORTCUT_F[4])[0].x, 185.0);
        let page = with_texture(&quads, DIGIT_TEXTURE);
        assert_eq!((page[0].x, page[0].y), (16.0, 18.0));
        let mut tenth = view;
        tenth.page = 9;
        let quads = painted(&tenth, 1.0);
        let page = with_texture(&quads, DIGIT_TEXTURE);
        assert_eq!((page[0].x, page[0].u, page[1].x, page[1].u), (14.0, 8.0, 18.0, 0.0));
    }

    #[test]
    fn rotating_keeps_the_bottom_right_corner_and_clamps_the_shrinking_side_at_zero() {
        let mut view = ShortcutView::new(true);
        assert_eq!(view.rotate_button(), Rect::new(16, 489, 15, 15));
        assert_eq!(view.joypad_button(), Rect::new(1, 489, 15, 15));
        assert_eq!(view.grip(), Rect::new(0, 0, 46, 12));
        let rect = view.rotate(Rect::new(1234, 108, 46, 504));
        assert!(!view.vertical);
        assert_eq!(rect, Rect::new(776, 566, 504, 46));
        assert_eq!(view.rotate_button(), Rect::new(489, 16, 15, 15));
        assert_eq!(view.joypad_button(), Rect::new(489, 1, 15, 15));
        assert_eq!(view.grip(), Rect::new(0, 0, 12, 46));
        assert_eq!(view.rotate(rect), Rect::new(1234, 108, 46, 504));
        assert!(view.vertical);
        assert_eq!(view.rotate(Rect::new(100, 300, 46, 504)), Rect::new(0, 758, 504, 46));
        assert_eq!(view.rotate(Rect::new(0, 758, 504, 46)), Rect::new(458, 300, 46, 504));
        assert_eq!(view.rotate(Rect::new(458, 300, 46, 504)), Rect::new(0, 758, 504, 46));
        assert_eq!(view.rotate(Rect::new(500, 20, 504, 46)), Rect::new(958, 0, 46, 504));
    }

    fn stocked() -> ShortcutView {
        let mut view = ShortcutView::new(false);
        view.set(0, ShortcutSlot::new(SlotKind::Item { object_id: 10 }, "icon.etc_potion_red_i00"));
        view.set(1, ShortcutSlot::new(SlotKind::Skill { id: 3, level: 1 }, "icon.skill0003"));
        view.set(
            3,
            ShortcutSlot { macro_acronym: "ABCD".into(), ..ShortcutSlot::new(SlotKind::Macro { id: 1000 }, "") },
        );
        view.set(4, ShortcutSlot::new(SlotKind::Recipe { object_id: 77 }, "icon.etc_recipe_i00"));
        view.set(12, ShortcutSlot::new(SlotKind::Action { id: 2 }, "icon.action003"));
        view
    }

    #[test]
    fn slots_are_painted_with_icon_outline_macro_letters_and_recipe_overlay() {
        let mut view = stocked();
        view.pressed = Some(1);
        view.slots[1].toggled = true;
        view.slots[0].cool = Some(CoolFrame::Running(90));
        let quads = painted(&view, 1.0);
        let potion = with_texture(&quads, "icon.etc_potion_red_i00");
        assert_eq!((potion[0].x, potion[0].y, potion[0].w), (33.0, 7.0, 32.0));
        let outline = with_texture(&quads, SLOT_OUTLINE);
        assert_eq!(outline.len(), 4);
        assert_eq!((outline[0].x, outline[0].y, outline[0].w), (32.0, 6.0, 34.0));
        let down = with_texture(&quads, SLOT_OUTLINE_DOWN);
        assert_eq!((down[0].x, down[0].y), (69.0, 6.0));
        let toggle = with_texture(&quads, TOGGLE_EFFECT);
        assert_eq!((toggle[0].x, toggle[0].y), (70.0, 7.0));
        assert!(with_texture(&quads, RECIPE_ICON).is_empty(), "a barra horizontal não desenha o ícone de receita");
        assert!(with_texture(&quads, BLACK_TEXTURE).len() == 1);
        let letters: Vec<(f32, f32)> =
            quads.iter().filter(|q| q.texture == "L2Font-e.SmallFont-e").map(|q| (q.x, q.y)).collect();
        assert_eq!(letters, vec![(148.0, 11.0), (161.0, 11.0), (148.0, 24.0), (161.0, 24.0)]);
        let cool = quads.iter().find(|q| q.texture == "L2_SkillTime.Rotation_Type.cooltime90").unwrap();
        assert_eq!((cool.x, cool.y, cool.w, cool.h), (33.0, 7.0, 32.0, 32.0));
        assert!(with_texture(&quads, "icon.action003").is_empty());
        let mut vertical = stocked();
        vertical.vertical = true;
        let quads = painted(&vertical, 1.0);
        let potion = with_texture(&quads, "icon.etc_potion_red_i00");
        assert_eq!((potion[0].x, potion[0].y), (7.0, 33.0));
        let recipe = with_texture(&quads, RECIPE_ICON);
        assert_eq!((recipe[0].x, recipe[0].y), (7.0, 33.0 + 37.0 * 4.0 + 5.0 + 0.0));
        assert_eq!((with_texture(&quads, SLOT_OUTLINE)[0].x, with_texture(&quads, SLOT_OUTLINE)[0].y), (6.0, 32.0));
    }

    #[test]
    fn f_keys_use_the_slot_of_the_page_once_until_the_key_is_released() {
        let mut view = stocked();
        assert!(!view.key_down(VK_F1));
        assert_eq!(view.take_events(), vec![BarEvent::Use(0)]);
        assert!(view.key_down(VK_F1));
        assert!(view.take_events().is_empty());
        assert!(!view.key_up(VK_F1));
        assert!(!view.key_down(VK_F1));
        assert_eq!(view.take_events(), vec![BarEvent::Use(0)]);
        assert!(!view.key_down(VK_F3));
        assert_eq!(view.take_events(), vec![BarEvent::Sound(SOUND_EMPTY_SLOT)]);
        assert!(!view.key_down(0x31));
        assert!(view.take_events().is_empty());
        view.enter_chatting = true;
        assert!(!view.key_down(0x32));
        assert_eq!(view.take_events(), vec![BarEvent::Use(1)]);
        assert!(!view.key_down(VK_0));
        assert_eq!(view.take_events(), vec![BarEvent::Sound(SOUND_EMPTY_SLOT)]);
        view.dialog_visible = true;
        assert!(!view.key_down(0x32));
        assert!(view.take_events().is_empty());
        view.set_page(1);
        assert_eq!(view.take_events(), vec![BarEvent::PageChanged(1)]);
        assert!(!view.key_down(VK_F1));
        assert_eq!(view.take_events(), vec![BarEvent::Use(12)]);
        view.set_page(10);
        assert_eq!(view.page, 1);
        view.set_page(0);
        assert!(view.armed[12]);
    }

    #[test]
    fn quick_use_items_repeat_every_200_ms_and_the_rest_wait_for_the_release_or_the_cool_time() {
        const VK_F2: u32 = VK_F1 + 1;
        assert!(is_quick_use_item(1060) && is_quick_use_item(0x1794) && !is_quick_use_item(0x1795) && !is_quick_use_item(1));
        let mut view = ShortcutView::new(false);
        let potion = ShortcutSlot { item_id: 1060, ..ShortcutSlot::new(SlotKind::Item { object_id: 10 }, "icon.potion") };
        view.set(0, potion);
        view.set(1, ShortcutSlot { item_id: 57, ..ShortcutSlot::new(SlotKind::Item { object_id: 11 }, "icon.adena") });
        view.set(2, ShortcutSlot::new(SlotKind::Skill { id: 3, level: 1 }, "icon.skill0003"));
        view.set(3, ShortcutSlot { toggle_skill: true, ..ShortcutSlot::new(SlotKind::Skill { id: 7, level: 1 }, "icon.skill0007") });
        view.set(4, ShortcutSlot::new(SlotKind::Action { id: ACTION_PICK_UP }, "icon.action"));
        assert_eq!(view.slots[0].key_mode(), KeyMode::Interval(200));
        assert_eq!(view.slots[1].key_mode(), KeyMode::OnKeyUp);
        assert_eq!(view.slots[2].key_mode(), KeyMode::OnCoolTime);
        assert_eq!(view.slots[3].key_mode(), KeyMode::Interval(1000));
        assert_eq!(view.slots[4].key_mode(), KeyMode::Interval(500));
        assert!(!view.key_down(VK_F1));
        assert_eq!(view.take_events(), vec![BarEvent::Use(0)]);
        view.tick(150);
        assert!(view.key_down(VK_F1));
        assert!(view.take_events().is_empty());
        view.tick(60);
        assert!(!view.key_down(VK_F1));
        assert_eq!(view.take_events(), vec![BarEvent::Use(0)]);
        assert!(!view.key_down(VK_F2));
        view.take_events();
        view.tick(5000);
        assert!(view.key_down(VK_F2), "item comum só volta com a tecla solta");
        view.key_up(VK_F2);
        assert!(!view.key_down(VK_F2));
        view.take_events();
        assert!(!view.key_down(VK_F3));
        view.take_events();
        view.slots[2].cool = Some(CoolFrame::Running(10));
        view.tick(5000);
        assert!(view.key_down(VK_F3), "skill espera o fim do cooltime");
        view.slots[2].cool = None;
        view.tick(1);
        assert!(!view.key_down(VK_F3));
        assert_eq!(view.take_events(), vec![BarEvent::Use(2)]);
    }

    #[test]
    fn a_click_on_a_quick_item_that_is_not_ready_does_nothing_and_a_skill_click_is_ready_at_once() {
        let mut view = ShortcutView::new(false);
        view.set(0, ShortcutSlot { item_id: 1060, ..ShortcutSlot::new(SlotKind::Item { object_id: 10 }, "icon.potion") });
        view.set(1, ShortcutSlot::new(SlotKind::Skill { id: 3, level: 1 }, "icon.skill0003"));
        view.mouse_down(40, 10);
        view.mouse_up(DropTarget::Bar(41, 12));
        assert_eq!(view.take_events(), vec![BarEvent::Use(0)]);
        view.mouse_down(40, 10);
        view.mouse_up(DropTarget::Bar(41, 12));
        assert!(view.take_events().is_empty());
        view.tick(201);
        view.mouse_down(40, 10);
        view.mouse_up(DropTarget::Bar(41, 12));
        assert_eq!(view.take_events(), vec![BarEvent::Use(0)]);
        for _ in 0..2 {
            view.mouse_down(75, 10);
            view.mouse_up(DropTarget::Bar(76, 12));
            assert_eq!(view.take_events(), vec![BarEvent::Use(1)]);
        }
    }

    #[test]
    fn the_page_buttons_step_the_page_and_wrap_like_the_button_handlers() {
        let mut view = ShortcutView::new(true);
        view.set(0, ShortcutSlot::new(SlotKind::Item { object_id: 10 }, "icon.etc_potion_red_i00"));
        assert_eq!(view.page_button_at(31, 13), Some(PageButton::Next));
        assert_eq!(view.page_button_at(44, 26), Some(PageButton::Next));
        assert_eq!(view.page_button_at(1, 13), Some(PageButton::Prev));
        assert_eq!(view.page_button_at(14, 26), Some(PageButton::Prev));
        assert_eq!(view.page_button_at(20, 20), None);
        assert_eq!(view.page_button_at(31, 40), None);
        view.press_page_button(PageButton::Prev);
        assert_eq!(view.page, 9);
        assert_eq!(view.take_events(), vec![BarEvent::PageChanged(9)]);
        view.press_page_button(PageButton::Next);
        assert_eq!(view.page, 0);
        for expected in 1..PAGES {
            view.next_page();
            assert_eq!(view.page, expected);
        }
        view.next_page();
        assert_eq!(view.page, 0);
        view.prev_page();
        assert_eq!(view.page, 9);
        view.prev_page();
        assert_eq!(view.page, 8);
        assert!(!view.key_down(VK_F1));
        assert_eq!(view.take_events().last(), Some(&BarEvent::Sound(SOUND_EMPTY_SLOT)));
        view.set_page(0);
        view.take_events();
        assert!(!view.key_down(VK_F1));
        assert_eq!(view.take_events(), vec![BarEvent::Use(0)]);
        let mut horizontal = stocked();
        assert_eq!(horizontal.page_button_at(13, 1), Some(PageButton::Next));
        assert_eq!(horizontal.page_button_at(13, 31), Some(PageButton::Prev));
        assert_eq!(horizontal.page_button_at(31, 13), None);
        horizontal.next_page();
        assert_eq!(horizontal.page, 1);
    }

    #[test]
    fn clicks_use_drag_swaps_and_dragging_off_the_bar_deletes() {
        let mut view = stocked();
        assert_eq!(view.slot_at(33, 7), Some(0));
        assert_eq!(view.slot_at(70, 38), Some(1));
        assert_eq!(view.slot_at(66, 7), None);
        assert_eq!(view.slot_at(20, 20), None);
        view.mouse_down(40, 10);
        assert_eq!(view.pressed, Some(0));
        assert!(view.dragging);
        view.mouse_up(DropTarget::Bar(41, 12));
        assert_eq!(view.take_events(), vec![BarEvent::Use(0)]);
        view.mouse_down(40, 10);
        view.mouse_up(DropTarget::Bar(75, 10));
        assert_eq!(
            view.take_events(),
            vec![
                BarEvent::Register { kind: SHORTCUT_TYPE_ITEM, slot: 1, id: 10 },
                BarEvent::Delete(0),
                BarEvent::Register { kind: SHORTCUT_TYPE_SKILL, slot: 0, id: 3 },
            ]
        );
        view.mouse_down(75, 10);
        view.mouse_up(DropTarget::Bar(110, 10));
        assert_eq!(
            view.take_events(),
            vec![BarEvent::Register { kind: SHORTCUT_TYPE_SKILL, slot: 2, id: 3 }, BarEvent::Delete(1)]
        );
        view.mouse_down(110, 10);
        view.mouse_up(DropTarget::Bar(112, 10));
        assert!(view.take_events().is_empty());
        view.slots[0].soulshot = true;
        view.mouse_down(40, 10);
        view.mouse_up(DropTarget::Outside(600, 300));
        assert_eq!(
            view.take_events(),
            vec![BarEvent::AutoSoulShot { object_id: 10, enable: false }, BarEvent::Delete(0)]
        );
        view.right_click(40, 10);
        assert_eq!(view.take_events(), vec![BarEvent::AutoSoulShot { object_id: 10, enable: true }]);
        view.slots[0].disabled = true;
        view.right_click(40, 10);
        assert_eq!(view.take_events(), vec![BarEvent::AutoSoulShot { object_id: 10, enable: false }]);
        view.right_click(75, 10);
        assert!(view.take_events().is_empty());
        view.mouse_up(DropTarget::Bar(0, 0));
        assert!(view.take_events().is_empty());
        let mut vertical = stocked();
        vertical.vertical = true;
        assert_eq!(vertical.slot_at(7, 70), Some(1));
        assert_eq!(vertical.slot_at(40, 70), None);
    }
}
