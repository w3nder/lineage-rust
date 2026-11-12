#[path = "manor_crop_change_controls_generated.rs"]
mod change_controls;
#[path = "manor_crop_change_layout_generated.rs"]
mod change_layout;
#[path = "manor_crop_setting_controls_generated.rs"]
mod setting_controls;
#[path = "manor_crop_setting_layout_generated.rs"]
mod setting_layout;

use crate::button::TextButton;
use crate::combo::ComboBox;
use crate::draw::DrawList;
use crate::edit_box::{EditBox, FontMetrics};
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::inventory::thousands;
use crate::list_ctrl::{ListCell, ListCtrl, ListRow};
use crate::manor_info::{castle_name, join_names};
use crate::manor_setting::{ADENA_ICON, Confirm, TEXT_COLOR, TEXTBOX, app_atoi, app_itoa};
use crate::window::{Rect, WindowId};

pub const SETTING_ID: WindowId = WindowId::MANOR_CROP_SETTING;
pub const CHANGE_ID: WindowId = WindowId::MANOR_CROP_CHANGE;
pub const SETTING_SIZE: (i32, i32) = setting_layout::SIZE;
pub const CHANGE_SIZE: (i32, i32) = change_layout::SIZE;
pub const SETTING_INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 2, x: SETTING_SIZE.0, y: SETTING_SIZE.1 };
pub const CHANGE_INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 2, x: CHANGE_SIZE.0, y: CHANGE_SIZE.1 };

pub fn setting_grip() -> Rect {
    setting_layout::RECT_7C
}

pub fn change_grip() -> Rect {
    change_layout::RECT_7C
}

pub const SETTING_BACK: &str = "L2UI_CH3.Manor.manor2_back";
pub const CHANGE_BACK: &str = "L2UI_CH3.Manor.manor4_back";
pub const CASTLE_POS: (i32, i32) = (16, 47);
pub const TOTAL_LABEL: u32 = 1141;
pub const TOTAL_LABEL_POS: (i32, i32) = (16, 279);
pub const TOTAL_VALUE_POS: (i32, i32) = (130, 279);
pub const NAME_POS: (i32, i32) = (21, 40);
pub const CHANGE_LABELS: [(u32, (i32, i32)); 4] = [(1225, (31, 71)), (1226, (31, 96)), (1149, (31, 121)), (1150, (31, 146))];
pub const LIMIT_VALUE_POS: (i32, i32) = (145, 96);
pub const RANGE_MESSAGE: u32 = 0x617;
pub const AMOUNT_MESSAGE: u32 = 0x618;
pub const REWARD_ENTRIES: [&str; 2] = ["1", "2"];
pub const CELL_COUNT: usize = 12;
pub const CELL_DEFAULT_AMOUNT: usize = 1;
pub const CELL_DEFAULT_PRICE: usize = 2;
pub const CELL_DEFAULT_REWARD: usize = 3;
pub const CELL_AMOUNT: usize = 4;
pub const CELL_PRICE: usize = 5;
pub const CELL_REWARD: usize = 6;
pub const CELL_MIN_PRICE: usize = 7;
pub const CELL_MAX_PRICE: usize = 8;

pub fn setting_default_position(viewport: (i32, i32)) -> (i32, i32) {
    crate::manor_setting::setting_default_position(viewport)
}

pub fn change_default_position(viewport: (i32, i32)) -> (i32, i32) {
    crate::manor_setting::change_default_position(viewport)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CropRow {
    pub item: i32,
    pub second: i32,
    pub limit: i32,
    pub numbers: [i32; 6],
    pub flag: i32,
    pub tail: [i32; 2],
    pub last: i32,
    pub first_items: Vec<i32>,
    pub second_items: Vec<i32>,
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
    Submit { manor: i32, crops: Vec<(i32, i32, i32, u8)> },
}

#[derive(Clone, Debug)]
pub struct CropSettingView {
    pub frame: FrameCtrl,
    pub list: ListCtrl,
    pub buttons: [TextButton; 5],
    pub manor: i32,
    pub total: i32,
    pub data: Vec<(i32, i32)>,
    pressed: Option<SettingHit>,
}

impl CropSettingView {
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

    pub fn set_data(&mut self, manor: i32, rows: &[CropRow], item_name: &dyn Fn(i32) -> String) {
        self.manor = manor;
        self.list.clear();
        self.data.clear();
        for row in rows {
            let texts = [
                item_name(row.item),
                row.numbers[4].to_string(),
                row.numbers[5].to_string(),
                row.flag.to_string(),
                row.tail[0].to_string(),
                row.tail[1].to_string(),
                row.last.to_string(),
                row.numbers[2].to_string(),
                row.numbers[3].to_string(),
                row.second.to_string(),
                join_names(&row.first_items, item_name),
                join_names(&row.second_items, item_name),
            ];
            let cells = texts.iter().map(|text| ListCell::text(text)).collect();
            if self.list.add_item(ListRow { cells }) {
                self.data.push((row.item, row.limit));
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
        for row in 0..self.data.len() {
            total = total.wrapping_add(app_atoi(self.cell(row, CELL_AMOUNT)).wrapping_mul(app_atoi(self.cell(row, CELL_PRICE))));
        }
        self.total = total;
    }

    pub fn apply_change(&mut self, amount: i32, price: i32, reward: i32) {
        if let Some(row) = self.list.selected() {
            self.set_cell(row, CELL_AMOUNT, app_itoa(amount));
            self.set_cell(row, CELL_PRICE, app_itoa(price));
            self.set_cell(row, CELL_REWARD, app_itoa(reward));
        }
        self.recalc();
    }

    pub fn reset(&mut self) {
        for row in 0..self.list.rows.len() {
            for cell in [CELL_AMOUNT, CELL_PRICE, CELL_REWARD] {
                self.set_cell(row, cell, "0".into());
            }
        }
        self.recalc();
    }

    pub fn use_defaults(&mut self) {
        for row in 0..self.list.rows.len() {
            for (target, source) in [(CELL_AMOUNT, CELL_DEFAULT_AMOUNT), (CELL_PRICE, CELL_DEFAULT_PRICE), (CELL_REWARD, CELL_DEFAULT_REWARD)] {
                let text = self.cell(row, source).to_string();
                self.set_cell(row, target, text);
            }
        }
        self.recalc();
    }

    pub fn answer(&mut self, confirm: Confirm) {
        match confirm {
            Confirm::Reset => self.reset(),
            Confirm::Defaults => self.use_defaults(),
        }
    }

    pub fn change_crop(&self) -> Option<ChangeCrop> {
        let row = self.list.selected()?;
        let (_, limit) = *self.data.get(row)?;
        Some(ChangeCrop {
            name: self.cell(row, 0).to_string(),
            limit,
            amount: self.cell(row, CELL_AMOUNT).to_string(),
            price: self.cell(row, CELL_PRICE).to_string(),
            reward: app_atoi(self.cell(row, CELL_REWARD)),
            min_price: app_atoi(self.cell(row, CELL_MIN_PRICE)),
            max_price: app_atoi(self.cell(row, CELL_MAX_PRICE)),
        })
    }

    pub fn submit(&self) -> SettingAction {
        let crops = self
            .data
            .iter()
            .enumerate()
            .map(|(row, (item, _))| (*item, app_atoi(self.cell(row, CELL_AMOUNT)), app_atoi(self.cell(row, CELL_PRICE)), app_atoi(self.cell(row, CELL_REWARD)) as u8))
            .collect();
        SettingAction::Submit { manor: self.manor, crops }
    }

    fn button_hit(&self, x: i32, y: i32) -> Option<SettingHit> {
        let contracts = [
            (&setting_controls::BUTTON_F0, SettingHit::Change),
            (&setting_controls::BUTTON_F4, SettingHit::Defaults),
            (&setting_controls::BUTTON_F8, SettingHit::Reset),
            (&setting_controls::BUTTON_FC, SettingHit::Ok),
            (&setting_controls::BUTTON_100, SettingHit::Cancel),
        ];
        self.buttons.iter().zip(contracts).find_map(|(button, (contract, hit))| contract.hit(button, x, y).map(|_| hit))
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
        list.clip_from(start, Rect::new(x, y, SETTING_SIZE.0, SETTING_SIZE.1));
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
            contract.paint(button, list, origin, self.pressed == Some(hit), font);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeCrop {
    pub name: String,
    pub limit: i32,
    pub amount: String,
    pub price: String,
    pub reward: i32,
    pub min_price: i32,
    pub max_price: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeHit {
    Close,
    Ok,
    Cancel,
    Amount,
    Price,
    Reward,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangeOutcome {
    Close,
    Apply { amount: i32, price: i32, reward: i32 },
    Rejected { message: u32, numbers: [i32; 2] },
}

#[derive(Clone, Debug)]
pub struct CropChangeView {
    pub frame: FrameCtrl,
    pub ok: TextButton,
    pub cancel: TextButton,
    pub amount: EditBox,
    pub price: EditBox,
    pub reward: ComboBox,
    pub name: String,
    pub limit: i32,
    pub min_price: i32,
    pub max_price: i32,
    pressed: Option<ChangeHit>,
}

impl CropChangeView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        let mut reward = ComboBox::from_contract(&change_controls::CHILD_208, REWARD_ENTRIES.iter().map(|entry| entry.to_string()).collect(), 13);
        reward.selected = 0;
        Self {
            frame: change_controls::FRAME_7C.build(text),
            ok: change_controls::BUTTON_1F4.build((0, 0), text),
            cancel: change_controls::BUTTON_1F8.build((0, 0), text),
            amount: EditBox::from_contract(&change_controls::CHILD_1FC),
            price: EditBox::from_contract(&change_controls::CHILD_200),
            reward,
            name: String::new(),
            limit: 0,
            min_price: 0,
            max_price: 0,
            pressed: None,
        }
    }

    pub fn open(&mut self, crop: &ChangeCrop) {
        self.name.clone_from(&crop.name);
        self.limit = crop.limit;
        self.amount.set_text(&crop.amount);
        self.price.set_text(&crop.price);
        self.reward.selected = if crop.reward == 0 { 0 } else { crop.reward - 1 };
        self.reward.lose_focus();
        self.min_price = crop.min_price;
        self.max_price = crop.max_price;
        self.price.kill_focus();
        self.amount.focus();
    }

    pub fn blur(&mut self) {
        for edit in [&mut self.amount, &mut self.price] {
            if edit.focused {
                edit.kill_focus();
            }
        }
        self.reward.lose_focus();
    }

    pub fn focused_edit(&mut self) -> Option<&mut EditBox> {
        [&mut self.amount, &mut self.price].into_iter().find(|edit| edit.focused)
    }

    fn field_value(edit: &EditBox) -> i32 {
        app_atoi(&crate::edit_box::to_string(edit.left()))
    }

    pub fn reward_value(&self) -> i32 {
        self.reward.selected_text().map_or(0, app_atoi)
    }

    pub fn validate(&self) -> ChangeOutcome {
        let amount = Self::field_value(&self.amount);
        if amount < 0 || self.limit < amount {
            return ChangeOutcome::Rejected { message: AMOUNT_MESSAGE, numbers: [0, self.limit] };
        }
        let price = Self::field_value(&self.price);
        if amount != 0 && (self.min_price > price || self.max_price < price) {
            return ChangeOutcome::Rejected { message: RANGE_MESSAGE, numbers: [self.min_price, self.max_price] };
        }
        ChangeOutcome::Apply { amount, price, reward: self.reward_value() }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<ChangeHit> {
        if change_controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(ChangeHit::Close);
        }
        if self.reward.expanded_rect().contains(x, y) {
            return Some(ChangeHit::Reward);
        }
        if change_controls::BUTTON_1F4.hit(&self.ok, x, y).is_some() {
            return Some(ChangeHit::Ok);
        }
        if change_controls::BUTTON_1F8.hit(&self.cancel, x, y).is_some() {
            return Some(ChangeHit::Cancel);
        }
        if self.amount.rect.contains(x, y) {
            return Some(ChangeHit::Amount);
        }
        self.price.rect.contains(x, y).then_some(ChangeHit::Price)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, font: &BitmapFont) -> Option<ChangeHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        let metrics = FontMetrics(font);
        if hit == Some(ChangeHit::Reward) {
            for edit in [&mut self.amount, &mut self.price] {
                if edit.focused {
                    edit.kill_focus();
                }
            }
            self.reward.mouse_move(x, y);
            self.reward.mouse_down();
            return hit;
        }
        self.reward.lose_focus();
        let target = match hit {
            Some(ChangeHit::Amount) => Some((&mut self.amount, &mut self.price)),
            Some(ChangeHit::Price) => Some((&mut self.price, &mut self.amount)),
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

    pub fn mouse_move(&mut self, x: i32, y: i32, font: &BitmapFont) {
        let metrics = FontMetrics(font);
        self.reward.mouse_move(x, y);
        if let Some(edit) = self.focused_edit() {
            edit.mouse_move(x, &metrics);
            edit.layout(&metrics);
        }
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<ChangeOutcome> {
        for edit in [&mut self.amount, &mut self.price] {
            edit.mouse_up();
        }
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            ChangeHit::Close | ChangeHit::Cancel => Some(ChangeOutcome::Close),
            ChangeHit::Ok => Some(self.validate()),
            ChangeHit::Amount | ChangeHit::Price | ChangeHit::Reward => None,
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
        for edit in [&mut self.amount, &mut self.price] {
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
        list.clip_from(start, Rect::new(x, y, CHANGE_SIZE.0, CHANGE_SIZE.1));
        let metrics = FontMetrics(font);
        self.amount.paint(list, origin, &metrics, font);
        self.price.paint(list, origin, &metrics, font);
        change_controls::BUTTON_1F4.paint(&self.ok, list, origin, self.pressed == Some(ChangeHit::Ok), font);
        change_controls::BUTTON_1F8.paint(&self.cancel, list, origin, self.pressed == Some(ChangeHit::Cancel), font);
        self.reward.paint(list, origin, font, true, false);
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

    fn row(item: i32) -> CropRow {
        CropRow {
            item,
            second: 9,
            limit: 50,
            numbers: [50, 2, 10, 20, 13, 14],
            flag: 3,
            tail: [15, 16],
            last: 2,
            first_items: vec![1, 2],
            second_items: vec![3],
        }
    }

    fn view() -> CropSettingView {
        let mut view = CropSettingView::new(lookup);
        view.set_data(3, &[row(100), row(101)], &item);
        view
    }

    #[test]
    fn windows_use_the_native_sizes_and_columns() {
        assert_eq!(SETTING_SIZE, (600, 326));
        assert_eq!(CHANGE_SIZE, (256, 201));
        assert_eq!((SETTING_ID, CHANGE_ID), (WindowId(70), WindowId(74)));
        assert_eq!(view().list.columns.len(), 7);
        assert_eq!(setting_grip(), Rect::new(0, 0, 600, 20));
        assert_eq!(change_grip(), Rect::new(0, 0, 256, 20));
    }

    #[test]
    fn rows_carry_twelve_cells_and_the_total_multiplies_amount_by_price() {
        let view = view();
        let cells: Vec<_> = view.list.rows[0].cells.iter().map(|cell| cell.text.clone()).collect();
        assert_eq!(cells, ["item100", "13", "14", "3", "15", "16", "2", "10", "20", "9", "item1,item2", "item3"]);
        assert_eq!(view.list.rows[0].cells.len(), CELL_COUNT);
        assert_eq!(view.data, vec![(100, 50), (101, 50)]);
        assert_eq!(view.total, 2 * 15 * 16);
    }

    #[test]
    fn apply_reset_and_defaults_rewrite_the_three_cells() {
        let mut view = view();
        view.list.select(1);
        view.apply_change(4, 5, 1);
        assert_eq!((view.cell(1, CELL_AMOUNT), view.cell(1, CELL_PRICE), view.cell(1, CELL_REWARD)), ("4", "5", "1"));
        assert_eq!(view.total, 15 * 16 + 20);
        view.reset();
        assert_eq!((view.cell(0, CELL_AMOUNT), view.cell(0, CELL_REWARD), view.total), ("0", "0", 0));
        view.use_defaults();
        assert_eq!((view.cell(0, CELL_AMOUNT), view.cell(0, CELL_PRICE), view.cell(0, CELL_REWARD)), ("13", "14", "3"));
        assert_eq!(view.total, 2 * 13 * 14);
    }

    #[test]
    fn change_crop_and_submit_follow_the_native_cells() {
        let mut view = view();
        assert_eq!(view.change_crop(), None);
        view.list.select(0);
        assert_eq!(
            view.change_crop(),
            Some(ChangeCrop { name: "item100".into(), limit: 50, amount: "15".into(), price: "16".into(), reward: 2, min_price: 10, max_price: 20 })
        );
        assert_eq!(view.submit(), SettingAction::Submit { manor: 3, crops: vec![(100, 15, 16, 2), (101, 15, 16, 2)] });
    }

    fn open_change(limit: i32, amount: &str, price: &str, reward: i32) -> CropChangeView {
        let mut change = CropChangeView::new(lookup);
        change.open(&ChangeCrop { name: "n".into(), limit, amount: amount.into(), price: price.into(), reward, min_price: 10, max_price: 20 });
        change
    }

    #[test]
    fn reward_combo_holds_one_and_two_and_selects_reward_minus_one() {
        assert_eq!(open_change(5, "1", "15", 0).reward_value(), 1);
        assert_eq!(open_change(5, "1", "15", 1).reward_value(), 1);
        assert_eq!(open_change(5, "1", "15", 2).reward_value(), 2);
        assert_eq!(open_change(5, "1", "15", 0).reward.entries, ["1", "2"]);
    }

    #[test]
    fn change_validation_follows_the_native_ok_handler() {
        assert_eq!(open_change(50, "5", "15", 2).validate(), ChangeOutcome::Apply { amount: 5, price: 15, reward: 2 });
        assert_eq!(open_change(50, "51", "15", 1).validate(), ChangeOutcome::Rejected { message: 0x618, numbers: [0, 50] });
        assert_eq!(open_change(50, "5", "9", 1).validate(), ChangeOutcome::Rejected { message: 0x617, numbers: [10, 20] });
        assert_eq!(open_change(50, "0", "999", 1).validate(), ChangeOutcome::Apply { amount: 0, price: 999, reward: 1 });
    }

    #[test]
    fn clicking_the_reward_combo_opens_it_and_picking_an_entry_selects_it() {
        let font = font();
        let mut change = open_change(50, "5", "15", 1);
        let combo = change.reward.rect;
        assert_eq!(change.mouse_down(combo.x + 2, combo.y + 2, &font), Some(ChangeHit::Reward));
        assert!(change.reward.open && !change.amount.focused);
        let second = (combo.x + 5, combo.y + combo.h + change.reward.row_height + 3);
        change.mouse_move(second.0, second.1, &font);
        assert_eq!(change.mouse_down(second.0, second.1, &font), Some(ChangeHit::Reward));
        assert_eq!(change.reward_value(), 2);
        assert!(!change.reward.open);
    }

    #[test]
    fn typing_edits_the_focused_field_and_ok_reports_the_outcome() {
        let font = font();
        let mut change = open_change(50, "5", "15", 1);
        assert!(change.key(InputKey::Backspace, &font));
        assert!(change.key(InputKey::Char('7'), &font));
        let ok = change_controls::BUTTON_1F4.rect;
        change.mouse_down(ok.x + 2, ok.y + 2, &font);
        assert_eq!(change.mouse_up(ok.x + 2, ok.y + 2), Some(ChangeOutcome::Apply { amount: 7, price: 15, reward: 1 }));
        let cancel = change_controls::BUTTON_1F8.rect;
        change.mouse_down(cancel.x + 2, cancel.y + 2, &font);
        assert_eq!(change.mouse_up(cancel.x + 2, cancel.y + 2), Some(ChangeOutcome::Close));
    }

    #[test]
    fn double_clicking_a_row_opens_the_change_window() {
        let mut view = view();
        assert_eq!(view.double_click(1, 1), None);
        let rect = view.list.rect;
        let y = rect.y + view.list.header_height + 2;
        assert_eq!(view.double_click(rect.x + 3, y), Some(SettingAction::OpenChange));
    }

    #[test]
    fn setting_buttons_open_the_change_window_with_a_selection_and_confirm_the_dialogs() {
        let mut view = view();
        let change = setting_controls::BUTTON_F0.rect;
        view.mouse_down(change.x + 2, change.y + 2);
        assert_eq!(view.mouse_up(change.x + 2, change.y + 2), None);
        view.list.select(0);
        view.mouse_down(change.x + 2, change.y + 2);
        assert_eq!(view.mouse_up(change.x + 2, change.y + 2), Some(SettingAction::OpenChange));
        let reset = setting_controls::BUTTON_F8.rect;
        view.mouse_down(reset.x + 2, reset.y + 2);
        assert_eq!(view.mouse_up(reset.x + 2, reset.y + 2), Some(SettingAction::Confirm(Confirm::Reset)));
        let ok = setting_controls::BUTTON_FC.rect;
        view.mouse_down(ok.x + 2, ok.y + 2);
        assert!(matches!(view.mouse_up(ok.x + 2, ok.y + 2), Some(SettingAction::Submit { manor: 3, .. })));
    }

    #[test]
    fn paints_draw_the_native_textures() {
        let font = font();
        let mut list = DrawList::default();
        view().paint(&mut list, (10, 20), &font, 1.0, &lookup);
        for expected in [SETTING_BACK, ADENA_ICON, TEXTBOX[0]] {
            assert!(list.quads.iter().any(|quad| quad.texture == expected), "{expected}");
        }
        let mut list = DrawList::default();
        open_change(50, "5", "15", 1).paint(&mut list, (10, 20), &font, 1.0, &lookup);
        assert!(list.quads.iter().any(|quad| quad.texture == CHANGE_BACK));
    }
}
