#[path = "manor_seed_change_controls_generated.rs"]
mod change_controls;
#[path = "manor_seed_change_layout_generated.rs"]
mod change_layout;
#[path = "manor_seed_setting_controls_generated.rs"]
mod setting_controls;
#[path = "manor_seed_setting_layout_generated.rs"]
mod setting_layout;

use crate::button::TextButton;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::edit_box::{EditBox, FontMetrics};
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::inventory::thousands;
use crate::list_ctrl::{ListCell, ListCtrl, ListRow};
use crate::manor_info::{castle_name, join_names};
use crate::window::{Rect, WindowId};

pub const SETTING_ID: WindowId = WindowId::MANOR_SEED_SETTING;
pub const CHANGE_ID: WindowId = WindowId::MANOR_SEED_CHANGE;
pub const SETTING_SIZE: (i32, i32) = setting_layout::SIZE;
pub const CHANGE_SIZE: (i32, i32) = change_layout::SIZE;
pub const SETTING_INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 2, x: SETTING_SIZE.0, y: SETTING_SIZE.1 };
pub const CHANGE_INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 2, x: CHANGE_SIZE.0, y: CHANGE_SIZE.1 };

pub fn setting_default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 * 0.5 - 300.0) as i32, (viewport.1 as f32 * 0.5 - 163.0) as i32)
}

pub fn change_default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 * 0.5 - 128.0) as i32, (viewport.1 as f32 * 0.5 - 100.0) as i32)
}

pub fn setting_grip() -> Rect {
    setting_layout::RECT_7C
}

pub fn change_grip() -> Rect {
    change_layout::RECT_7C
}

pub const TEXT_COLOR: Argb = Argb(0xffdcdcdc);
pub const SETTING_BACK: &str = "L2UI_CH3.Manor.manor2_back";
pub const CHANGE_BACK: &str = "L2UI_CH3.Manor.manor4_back";
pub const ADENA_ICON: &str = "L2UI.WindowIcon.WindowIconAdena";
pub const TEXTBOX: [&str; 3] = ["L2UI_CH3.Etc.textbox1", "L2UI_CH3.Etc.textbox2", "L2UI_CH3.Etc.textbox3"];
pub const CASTLE_POS: (i32, i32) = (16, 47);
pub const TOTAL_LABEL: u32 = 1141;
pub const TOTAL_LABEL_POS: (i32, i32) = (16, 279);
pub const TOTAL_VALUE_POS: (i32, i32) = (130, 279);
pub const NAME_POS: (i32, i32) = (21, 40);
pub const CHANGE_LABELS: [(u32, (i32, i32)); 3] = [(1139, (31, 71)), (1224, (31, 96)), (1140, (31, 121))];
pub const LIMIT_VALUE_POS: (i32, i32) = (145, 96);
pub const RANGE_MESSAGE: u32 = 0x615;
pub const QUANTITY_MESSAGE: u32 = 0x616;
pub const RESET_DIALOG_MESSAGE: u32 = 0x640;
pub const DEFAULTS_DIALOG_MESSAGE: u32 = 0x641;
pub const RESET_DIALOG_TYPE: i32 = 0x56;
pub const DEFAULTS_DIALOG_TYPE: i32 = 0x57;
pub const CELL_COUNT: usize = 10;
pub const CELL_DEFAULT_QUANTITY: usize = 1;
pub const CELL_DEFAULT_PRICE: usize = 2;
pub const CELL_QUANTITY: usize = 3;
pub const CELL_PRICE: usize = 4;
pub const CELL_MIN_PRICE: usize = 5;
pub const CELL_MAX_PRICE: usize = 6;

const SETTING_RECT: Rect = Rect::new(0, 0, SETTING_SIZE.0, SETTING_SIZE.1);
const CHANGE_RECT: Rect = Rect::new(0, 0, CHANGE_SIZE.0, CHANGE_SIZE.1);

pub fn app_atoi(text: &str) -> i32 {
    let mut chars = text.chars().peekable();
    while chars.peek().is_some_and(|c| matches!(c, ' ' | '\t' | '\n' | '\u{b}' | '\u{c}' | '\r')) {
        chars.next();
    }
    let negative = match chars.peek() {
        Some('-') => {
            chars.next();
            true
        }
        Some('+') => {
            chars.next();
            false
        }
        _ => false,
    };
    let mut total: u32 = 0;
    while let Some(digit) = chars.peek().and_then(|c| c.to_digit(10)) {
        total = total.wrapping_mul(10).wrapping_add(digit);
        chars.next();
    }
    if negative { (total as i32).wrapping_neg() } else { total as i32 }
}

pub fn app_itoa(value: i32) -> String {
    value.to_string()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Confirm {
    Reset,
    Defaults,
}

impl Confirm {
    pub const fn message(self) -> u32 {
        match self {
            Confirm::Reset => RESET_DIALOG_MESSAGE,
            Confirm::Defaults => DEFAULTS_DIALOG_MESSAGE,
        }
    }

    pub const fn dialog_type(self) -> i32 {
        match self {
            Confirm::Reset => RESET_DIALOG_TYPE,
            Confirm::Defaults => DEFAULTS_DIALOG_TYPE,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettingRow {
    pub item: i32,
    pub values: i32,
    pub limit: i32,
    pub unit: i32,
    pub numbers: [i32; 8],
    pub first: Vec<i32>,
    pub second: Vec<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingHit {
    Close,
    Change,
    Defaults,
    Reset,
    Ok,
    Cancel,
    List,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingAction {
    Close,
    OpenChange,
    Confirm(Confirm),
    Submit { manor: i32, seeds: Vec<(i32, i32, i32)> },
}

#[derive(Clone, Debug)]
pub struct SeedSettingView {
    pub frame: FrameCtrl,
    pub list: ListCtrl,
    pub buttons: [TextButton; 5],
    pub manor: i32,
    pub total: i32,
    pub data: Vec<(i32, i32, i32)>,
    pressed: Option<SettingHit>,
}

impl SeedSettingView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Self {
            frame: setting_controls::FRAME_7C.build(text),
            list: ListCtrl::from_contract(&setting_controls::LIST_E8),
            buttons: [
                setting_controls::BUTTON_F0.build((0, 0), text),
                setting_controls::BUTTON_F4.build((0, 0), text),
                setting_controls::BUTTON_F8.build((0, 0), text),
                setting_controls::BUTTON_FC.build((0, 0), text),
                setting_controls::BUTTON_100.build((0, 0), text),
            ],
            manor: 0,
            total: 0,
            data: Vec::new(),
            pressed: None,
        }
    }

    pub fn set_data(&mut self, manor: i32, rows: &[SettingRow], item_name: &dyn Fn(i32) -> String) {
        self.manor = manor;
        self.list.clear();
        self.data.clear();
        for row in rows {
            let [t2, t3, t4, t5, t6, t7] = [row.numbers[2], row.numbers[3], row.numbers[4], row.numbers[5], row.numbers[6], row.numbers[7]];
            let texts = [
                item_name(row.item),
                t4.to_string(),
                t5.to_string(),
                t6.to_string(),
                t7.to_string(),
                t2.to_string(),
                t3.to_string(),
                row.values.to_string(),
                join_names(&row.first, item_name),
                join_names(&row.second, item_name),
            ];
            let cells = texts.iter().map(|text| ListCell::text(text)).collect();
            if self.list.add_item(ListRow { cells }) {
                self.data.push((row.item, row.limit, row.unit));
            }
        }
        self.recalc();
    }

    pub fn cell(&self, row: usize, index: usize) -> &str {
        self.list.rows.get(row).and_then(|row| row.cells.get(index)).map_or("", |cell| cell.text.as_str())
    }

    fn set_cell(&mut self, row: usize, index: usize, text: String) {
        if let Some(cell) = self.list.rows.get_mut(row).and_then(|row| row.cells.get_mut(index)) {
            cell.text = text;
        }
    }

    pub fn recalc(&mut self) {
        let mut total = 0i32;
        for (row, (_, _, unit)) in self.data.iter().enumerate() {
            total = total.wrapping_add(app_atoi(self.cell(row, CELL_QUANTITY)).wrapping_mul(*unit));
        }
        self.total = total;
    }

    pub fn apply_change(&mut self, quantity: i32, price: i32) -> bool {
        let Some(row) = self.list.selected() else { return false };
        self.set_cell(row, CELL_QUANTITY, app_itoa(quantity));
        self.set_cell(row, CELL_PRICE, app_itoa(price));
        self.recalc();
        true
    }

    pub fn reset(&mut self) {
        for row in 0..self.list.rows.len() {
            self.set_cell(row, CELL_QUANTITY, "0".into());
            self.set_cell(row, CELL_PRICE, "0".into());
        }
        self.recalc();
    }

    pub fn use_defaults(&mut self) {
        for row in 0..self.list.rows.len() {
            let quantity = self.cell(row, CELL_DEFAULT_QUANTITY).to_string();
            let price = self.cell(row, CELL_DEFAULT_PRICE).to_string();
            self.set_cell(row, CELL_QUANTITY, quantity);
            self.set_cell(row, CELL_PRICE, price);
        }
        self.recalc();
    }

    pub fn answer(&mut self, confirm: Confirm) {
        match confirm {
            Confirm::Reset => self.reset(),
            Confirm::Defaults => self.use_defaults(),
        }
    }

    pub fn change_seed(&self) -> Option<ChangeSeed> {
        let row = self.list.selected()?;
        let (_, limit, _) = *self.data.get(row)?;
        Some(ChangeSeed {
            name: self.cell(row, 0).to_string(),
            limit,
            quantity: self.cell(row, CELL_QUANTITY).to_string(),
            price: self.cell(row, CELL_PRICE).to_string(),
            min_price: app_atoi(self.cell(row, CELL_MIN_PRICE)),
            max_price: app_atoi(self.cell(row, CELL_MAX_PRICE)),
        })
    }

    pub fn submit(&self) -> SettingAction {
        let seeds = self
            .data
            .iter()
            .enumerate()
            .map(|(row, (item, _, _))| (*item, app_atoi(self.cell(row, CELL_QUANTITY)), app_atoi(self.cell(row, CELL_PRICE))))
            .collect();
        SettingAction::Submit { manor: self.manor, seeds }
    }

    fn button_hit(&self, x: i32, y: i32) -> Option<SettingHit> {
        let contracts = [
            (&setting_controls::BUTTON_F0, SettingHit::Change),
            (&setting_controls::BUTTON_F4, SettingHit::Defaults),
            (&setting_controls::BUTTON_F8, SettingHit::Reset),
            (&setting_controls::BUTTON_FC, SettingHit::Ok),
            (&setting_controls::BUTTON_100, SettingHit::Cancel),
        ];
        self.buttons
            .iter()
            .zip(contracts)
            .find_map(|(button, (contract, hit))| contract.hit(button, x, y).map(|_| hit))
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<SettingHit> {
        if setting_controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(SettingHit::Close);
        }
        if let Some(hit) = self.button_hit(x, y) {
            return Some(hit);
        }
        self.list.owns(x, y).then_some(SettingHit::List)
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        self.list.mouse_move(x, y);
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<SettingHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        if hit == Some(SettingHit::List) {
            self.list.mouse_down(x, y);
        }
        hit
    }

    pub fn double_click(&mut self, x: i32, y: i32) -> Option<SettingAction> {
        if self.hit(x, y) != Some(SettingHit::List) || self.list.hit(x, y) < 0 {
            return None;
        }
        self.list.double_click(x, y);
        self.list.notices.clear();
        self.list.selected().map(|_| SettingAction::OpenChange)
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<SettingAction> {
        self.list.mouse_up(x, y);
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            SettingHit::Close | SettingHit::Cancel => Some(SettingAction::Close),
            SettingHit::Change => self.list.selected().map(|_| SettingAction::OpenChange),
            SettingHit::Defaults => Some(SettingAction::Confirm(Confirm::Defaults)),
            SettingHit::Reset => Some(SettingAction::Confirm(Confirm::Reset)),
            SettingHit::Ok => Some(self.submit()),
            SettingHit::List => None,
        }
    }

    pub fn wheel(&mut self, delta: i16) {
        self.list.wheel(delta);
    }

    pub fn pressed(&self, hit: SettingHit) -> bool {
        self.pressed == Some(hit)
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32, lookup: &dyn Fn(u32) -> Option<String>) {
        let (x, y) = origin;
        self.frame.paint(list, x, y, SETTING_SIZE.0, SETTING_SIZE.1, font, alpha);
        let start = list.quads.len();
        list.draw_texture(x, y + 20, 600, 306, 0, 0, 600, 306, SETTING_BACK, alpha);
        list.draw_texture(x + 107, y + 278, 16, 12, 0, 0, 16, 12, ADENA_ICON, alpha);
        list.draw_texture(x + 126, y + 276, 8, 17, 0, 0, 8, 17, TEXTBOX[0], alpha);
        list.draw_texture(x + 134, y + 276, 70, 17, 0, 0, 8, 17, TEXTBOX[1], alpha);
        list.draw_texture(x + 204, y + 276, 8, 17, 0, 0, 8, 17, TEXTBOX[2], alpha);
        let castle = castle_name(self.manor, lookup).unwrap_or_default();
        list.draw_text(x + CASTLE_POS.0, y + CASTLE_POS.1, TEXT_COLOR, &castle, font);
        list.draw_text(x + TOTAL_LABEL_POS.0, y + TOTAL_LABEL_POS.1, TEXT_COLOR, &lookup(TOTAL_LABEL).unwrap_or_default(), font);
        list.draw_text(x + TOTAL_VALUE_POS.0, y + TOTAL_VALUE_POS.1, TEXT_COLOR, &thousands(i64::from(self.total)), font);
        list.clip_from(start, Rect::new(x + SETTING_RECT.x, y + SETTING_RECT.y, SETTING_RECT.w, SETTING_RECT.h));
        self.list.paint_header(list, origin, font, &|id| lookup(id as u32));
        self.list.paint(list, origin, font);
        let contracts = [
            &setting_controls::BUTTON_F0,
            &setting_controls::BUTTON_F4,
            &setting_controls::BUTTON_F8,
            &setting_controls::BUTTON_FC,
            &setting_controls::BUTTON_100,
        ];
        let hits = [SettingHit::Change, SettingHit::Defaults, SettingHit::Reset, SettingHit::Ok, SettingHit::Cancel];
        for ((button, contract), hit) in self.buttons.iter().zip(contracts).zip(hits) {
            contract.paint(button, list, origin, self.pressed(hit), font);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeSeed {
    pub name: String,
    pub limit: i32,
    pub quantity: String,
    pub price: String,
    pub min_price: i32,
    pub max_price: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeHit {
    Close,
    Ok,
    Cancel,
    Quantity,
    Price,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangeOutcome {
    Close,
    Apply { quantity: i32, price: i32 },
    Rejected { message: u32, numbers: [i32; 2] },
}

#[derive(Clone, Debug)]
pub struct SeedChangeView {
    pub frame: FrameCtrl,
    pub ok: TextButton,
    pub cancel: TextButton,
    pub quantity: EditBox,
    pub price: EditBox,
    pub name: String,
    pub limit: i32,
    pub min_price: i32,
    pub max_price: i32,
    pressed: Option<ChangeHit>,
}

impl SeedChangeView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Self {
            frame: change_controls::FRAME_7C.build(text),
            ok: change_controls::BUTTON_1F4.build((0, 0), text),
            cancel: change_controls::BUTTON_1F8.build((0, 0), text),
            quantity: EditBox::from_contract(&change_controls::CHILD_1FC),
            price: EditBox::from_contract(&change_controls::CHILD_200),
            name: String::new(),
            limit: 0,
            min_price: 0,
            max_price: 0,
            pressed: None,
        }
    }

    pub fn open(&mut self, seed: &ChangeSeed) {
        self.name.clone_from(&seed.name);
        self.limit = seed.limit;
        self.quantity.set_text(&seed.quantity);
        self.price.set_text(&seed.price);
        self.min_price = seed.min_price;
        self.max_price = seed.max_price;
        self.price.kill_focus();
        self.quantity.focus();
    }

    pub fn blur(&mut self) {
        for edit in [&mut self.quantity, &mut self.price] {
            if edit.focused {
                edit.kill_focus();
            }
        }
    }

    pub fn focused_edit(&mut self) -> Option<&mut EditBox> {
        [&mut self.quantity, &mut self.price].into_iter().find(|edit| edit.focused)
    }

    fn field_text(edit: &EditBox) -> i32 {
        app_atoi(&crate::edit_box::to_string(edit.left()))
    }

    pub fn validate(&self) -> ChangeOutcome {
        let quantity = Self::field_text(&self.quantity);
        if quantity < 0 || self.limit < quantity {
            return ChangeOutcome::Rejected { message: QUANTITY_MESSAGE, numbers: [0, self.limit] };
        }
        let price = Self::field_text(&self.price);
        if quantity != 0 && (self.min_price > price || self.max_price < price) {
            return ChangeOutcome::Rejected { message: RANGE_MESSAGE, numbers: [self.min_price, self.max_price] };
        }
        ChangeOutcome::Apply { quantity, price }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<ChangeHit> {
        if change_controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(ChangeHit::Close);
        }
        if change_controls::BUTTON_1F4.hit(&self.ok, x, y).is_some() {
            return Some(ChangeHit::Ok);
        }
        if change_controls::BUTTON_1F8.hit(&self.cancel, x, y).is_some() {
            return Some(ChangeHit::Cancel);
        }
        if self.quantity.rect.contains(x, y) {
            return Some(ChangeHit::Quantity);
        }
        self.price.rect.contains(x, y).then_some(ChangeHit::Price)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, font: &BitmapFont) -> Option<ChangeHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        let metrics = FontMetrics(font);
        let target = match hit {
            Some(ChangeHit::Quantity) => Some((&mut self.quantity, &mut self.price)),
            Some(ChangeHit::Price) => Some((&mut self.price, &mut self.quantity)),
            _ => None,
        };
        match target {
            Some((edit, other)) => {
                if other.focused {
                    other.kill_focus();
                }
                if !edit.focused {
                    edit.focus();
                }
                edit.mouse_down(x, &metrics);
                edit.layout(&metrics);
            }
            None => self.blur(),
        }
        hit
    }

    pub fn mouse_move(&mut self, x: i32, font: &BitmapFont) {
        let metrics = FontMetrics(font);
        if let Some(edit) = self.focused_edit() {
            edit.mouse_move(x, &metrics);
            edit.layout(&metrics);
        }
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<ChangeOutcome> {
        for edit in [&mut self.quantity, &mut self.price] {
            edit.mouse_up();
        }
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            ChangeHit::Close | ChangeHit::Cancel => Some(ChangeOutcome::Close),
            ChangeHit::Ok => Some(self.validate()),
            ChangeHit::Quantity | ChangeHit::Price => None,
        }
    }

    pub fn key(&mut self, key: crate::chat::InputKey, font: &BitmapFont) -> bool {
        let metrics = FontMetrics(font);
        let Some(edit) = self.focused_edit() else { return false };
        match edit.input_key(key, &metrics) {
            crate::edit_box::EditInput::Escape => edit.kill_focus(),
            crate::edit_box::EditInput::Handled => {}
        }
        true
    }

    pub fn shift(&mut self, pressed: bool) -> bool {
        let Some(edit) = self.focused_edit() else { return false };
        if pressed {
            edit.key_down_shift();
        } else {
            edit.key_up(crate::edit_box::KEY_SHIFT);
        }
        true
    }

    pub fn tick(&mut self, elapsed_ms: u32) {
        for edit in [&mut self.quantity, &mut self.price] {
            edit.tick(elapsed_ms);
        }
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32, lookup: &dyn Fn(u32) -> Option<String>) {
        let (x, y) = origin;
        self.frame.paint(list, x, y, CHANGE_SIZE.0, CHANGE_SIZE.1, font, alpha);
        let start = list.quads.len();
        list.draw_texture(x, y + 20, 256, 201, 0, 0, 256, 201, CHANGE_BACK, alpha);
        list.draw_text(x + NAME_POS.0, y + NAME_POS.1, TEXT_COLOR, &self.name, font);
        for (sysstring, position) in CHANGE_LABELS {
            list.draw_text(x + position.0, y + position.1, TEXT_COLOR, &lookup(sysstring).unwrap_or_default(), font);
        }
        list.draw_text(x + LIMIT_VALUE_POS.0, y + LIMIT_VALUE_POS.1, TEXT_COLOR, &thousands(i64::from(self.limit)), font);
        list.clip_from(start, Rect::new(x + CHANGE_RECT.x, y + CHANGE_RECT.y, CHANGE_RECT.w, CHANGE_RECT.h));
        let metrics = FontMetrics(font);
        self.quantity.paint(list, origin, &metrics, font);
        self.price.paint(list, origin, &metrics, font);
        change_controls::BUTTON_1F4.paint(&self.ok, list, origin, self.pressed == Some(ChangeHit::Ok), font);
        change_controls::BUTTON_1F8.paint(&self.cancel, list, origin, self.pressed == Some(ChangeHit::Cancel), font);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chat::InputKey;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("font".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn lookup(id: u32) -> Option<String> {
        Some(format!("s{id}"))
    }

    fn item(id: i32) -> String {
        format!("item{id}")
    }

    fn row(item: i32, limit: i32, unit: i32) -> SettingRow {
        SettingRow { item, values: 9, limit, unit, numbers: [limit, unit, 11, 12, 13, 14, 15, 16], first: vec![1, 2], second: vec![3] }
    }

    fn view() -> SeedSettingView {
        let mut view = SeedSettingView::new(lookup);
        view.set_data(3, &[row(100, 50, 2), row(101, 70, 3)], &item);
        view
    }

    #[test]
    fn atoi_follows_the_c_runtime_prefix_rules() {
        assert_eq!(app_atoi("123"), 123);
        assert_eq!(app_atoi("  -45x"), -45);
        assert_eq!(app_atoi("+7"), 7);
        assert_eq!(app_atoi("1,234"), 1);
        assert_eq!(app_atoi(""), 0);
        assert_eq!(app_atoi("abc"), 0);
        assert_eq!(app_atoi("4294967297"), 1);
        assert_eq!(app_itoa(-5), "-5");
    }

    #[test]
    fn windows_use_the_native_sizes_registration_and_default_positions() {
        assert_eq!(SETTING_SIZE, (600, 326));
        assert_eq!(CHANGE_SIZE, (256, 201));
        assert_eq!(setting_default_position((1024, 768)), (212, 221));
        assert_eq!(change_default_position((1024, 768)), (384, 284));
        assert_eq!(setting_grip(), Rect::new(0, 0, 600, 20));
        assert_eq!(change_grip(), Rect::new(0, 0, 256, 20));
        assert_eq!((SETTING_ID, CHANGE_ID), (WindowId(69), WindowId(73)));
    }

    #[test]
    fn rows_carry_ten_cells_in_the_order_of_the_packet_and_the_total_uses_the_quantity_cell() {
        let view = view();
        let cells: Vec<_> = view.list.rows[0].cells.iter().map(|cell| cell.text.clone()).collect();
        assert_eq!(cells, ["item100", "13", "14", "15", "16", "11", "12", "9", "item1,item2", "item3"]);
        assert_eq!(view.list.rows[0].cells.len(), CELL_COUNT);
        assert_eq!(view.data, vec![(100, 50, 2), (101, 70, 3)]);
        assert_eq!(view.total, 15 * 2 + 15 * 3);
        assert_eq!(view.list.columns.len(), 5);
    }

    #[test]
    fn applying_a_change_rewrites_the_selected_row_and_recomputes_the_total() {
        let mut view = view();
        assert!(!view.apply_change(4, 5));
        view.list.select(1);
        assert!(view.apply_change(4, 5));
        assert_eq!(view.cell(1, CELL_QUANTITY), "4");
        assert_eq!(view.cell(1, CELL_PRICE), "5");
        assert_eq!(view.total, 15 * 2 + 4 * 3);
    }

    #[test]
    fn reset_zeroes_both_columns_and_defaults_copy_the_suggested_values() {
        let mut view = view();
        view.reset();
        assert_eq!((view.cell(0, CELL_QUANTITY), view.cell(0, CELL_PRICE), view.total), ("0", "0", 0));
        view.use_defaults();
        assert_eq!((view.cell(0, CELL_QUANTITY), view.cell(0, CELL_PRICE)), ("13", "14"));
        assert_eq!(view.total, 13 * 2 + 13 * 3);
        view.answer(Confirm::Reset);
        assert_eq!(view.total, 0);
        view.answer(Confirm::Defaults);
        assert_eq!(view.cell(1, CELL_PRICE), "14");
    }

    #[test]
    fn dialogs_use_the_native_types_and_messages() {
        assert_eq!((Confirm::Reset.dialog_type(), Confirm::Reset.message()), (0x56, 1600));
        assert_eq!((Confirm::Defaults.dialog_type(), Confirm::Defaults.message()), (0x57, 1601));
    }

    #[test]
    fn change_seed_reads_the_selected_row_and_the_price_bounds() {
        let mut view = view();
        assert_eq!(view.change_seed(), None);
        view.list.select(0);
        assert_eq!(
            view.change_seed(),
            Some(ChangeSeed { name: "item100".into(), limit: 50, quantity: "15".into(), price: "16".into(), min_price: 11, max_price: 12 })
        );
    }

    fn open_change(limit: i32, quantity: &str, price: &str) -> SeedChangeView {
        let mut change = SeedChangeView::new(lookup);
        change.open(&ChangeSeed { name: "n".into(), limit, quantity: quantity.into(), price: price.into(), min_price: 10, max_price: 20 });
        change
    }

    #[test]
    fn change_validation_follows_the_native_ok_handler() {
        assert_eq!(open_change(50, "5", "15").validate(), ChangeOutcome::Apply { quantity: 5, price: 15 });
        assert_eq!(open_change(50, "51", "15").validate(), ChangeOutcome::Rejected { message: 0x616, numbers: [0, 50] });
        assert_eq!(open_change(50, "-1", "15").validate(), ChangeOutcome::Rejected { message: 0x616, numbers: [0, 50] });
        assert_eq!(open_change(50, "5", "9").validate(), ChangeOutcome::Rejected { message: 0x615, numbers: [10, 20] });
        assert_eq!(open_change(50, "5", "21").validate(), ChangeOutcome::Rejected { message: 0x615, numbers: [10, 20] });
        assert_eq!(open_change(50, "0", "999").validate(), ChangeOutcome::Apply { quantity: 0, price: 999 });
        assert_eq!(open_change(50, "", "").validate(), ChangeOutcome::Apply { quantity: 0, price: 0 });
    }

    #[test]
    fn opening_the_change_window_focuses_the_quantity_field_and_typing_edits_it() {
        let font = font();
        let mut change = open_change(50, "5", "15");
        assert!(change.quantity.focused && !change.price.focused);
        assert!(change.key(InputKey::Backspace, &font));
        assert!(change.key(InputKey::Char('7'), &font));
        assert_eq!(change.validate(), ChangeOutcome::Apply { quantity: 7, price: 15 });
        change.blur();
        assert!(!change.key(InputKey::Char('1'), &font));
    }

    #[test]
    fn clicking_a_field_moves_the_focus_and_clicking_elsewhere_drops_it() {
        let font = font();
        let mut change = open_change(50, "5", "15");
        let price = change.price.rect;
        assert_eq!(change.mouse_down(price.x + 2, price.y + 2, &font), Some(ChangeHit::Price));
        assert!(change.price.focused && !change.quantity.focused);
        assert_eq!(change.mouse_down(1, 150, &font), None);
        assert!(!change.price.focused && !change.quantity.focused);
    }

    #[test]
    fn change_buttons_close_or_report_the_validation() {
        let font = font();
        let mut change = open_change(50, "5", "15");
        let ok = change_controls::BUTTON_1F4.rect;
        assert_eq!(change.mouse_down(ok.x + 2, ok.y + 2, &font), Some(ChangeHit::Ok));
        assert_eq!(change.mouse_up(ok.x + 2, ok.y + 2), Some(ChangeOutcome::Apply { quantity: 5, price: 15 }));
        let cancel = change_controls::BUTTON_1F8.rect;
        change.mouse_down(cancel.x + 2, cancel.y + 2, &font);
        assert_eq!(change.mouse_up(cancel.x + 2, cancel.y + 2), Some(ChangeOutcome::Close));
        change.mouse_down(ok.x + 2, ok.y + 2, &font);
        assert_eq!(change.mouse_up(cancel.x + 2, cancel.y + 2), None);
    }

    #[test]
    fn double_clicking_a_row_opens_the_change_window_like_the_native_list_slot() {
        let mut view = view();
        assert_eq!(view.double_click(1, 1), None);
        let rect = view.list.rect;
        let y = rect.y + view.list.header_height + 2;
        assert_eq!(view.double_click(rect.x + 3, y), Some(SettingAction::OpenChange));
        assert_eq!(view.list.selected(), Some(0));
    }

    #[test]
    fn setting_buttons_open_the_change_window_only_with_a_selection_and_submit_every_row() {
        let mut view = view();
        let change = setting_controls::BUTTON_F0.rect;
        let (cx, cy) = (change.x + 2, change.y + 2);
        assert_eq!(view.mouse_down(cx, cy), Some(SettingHit::Change));
        assert_eq!(view.mouse_up(cx, cy), None);
        view.list.select(0);
        view.mouse_down(cx, cy);
        assert_eq!(view.mouse_up(cx, cy), Some(SettingAction::OpenChange));
        let defaults = setting_controls::BUTTON_F4.rect;
        view.mouse_down(defaults.x + 2, defaults.y + 2);
        assert_eq!(view.mouse_up(defaults.x + 2, defaults.y + 2), Some(SettingAction::Confirm(Confirm::Defaults)));
        let reset = setting_controls::BUTTON_F8.rect;
        view.mouse_down(reset.x + 2, reset.y + 2);
        assert_eq!(view.mouse_up(reset.x + 2, reset.y + 2), Some(SettingAction::Confirm(Confirm::Reset)));
        let ok = setting_controls::BUTTON_FC.rect;
        view.mouse_down(ok.x + 2, ok.y + 2);
        assert_eq!(
            view.mouse_up(ok.x + 2, ok.y + 2),
            Some(SettingAction::Submit { manor: 3, seeds: vec![(100, 15, 16), (101, 15, 16)] })
        );
        let cancel = setting_controls::BUTTON_100.rect;
        view.mouse_down(cancel.x + 2, cancel.y + 2);
        assert_eq!(view.mouse_up(cancel.x + 2, cancel.y + 2), Some(SettingAction::Close));
    }

    #[test]
    fn setting_paint_draws_the_background_icon_text_boxes_and_texts() {
        let font = font();
        let mut list = DrawList::default();
        view().paint(&mut list, (10, 20), &font, 1.0, &lookup);
        let textures: Vec<_> = list.quads.iter().map(|quad| quad.texture.as_str()).collect();
        for expected in [SETTING_BACK, ADENA_ICON, TEXTBOX[0], TEXTBOX[1], TEXTBOX[2]] {
            assert!(textures.contains(&expected), "{expected}");
        }
    }

    #[test]
    fn change_paint_draws_the_background_and_the_edit_fields() {
        let font = font();
        let mut list = DrawList::default();
        open_change(50, "5", "15").paint(&mut list, (10, 20), &font, 1.0, &lookup);
        assert!(list.quads.iter().any(|quad| quad.texture == CHANGE_BACK));
    }
}
