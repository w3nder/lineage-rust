#[path = "manor_crop_sell_change_controls_generated.rs"]
mod change_controls;
#[path = "manor_crop_sell_change_layout_generated.rs"]
mod change_layout;
#[path = "manor_crop_sell_controls_generated.rs"]
mod sell_controls;
#[path = "manor_crop_sell_layout_generated.rs"]
mod sell_layout;

use crate::button::TextButton;
use crate::color::Argb;
use crate::combo::ComboBox;
use crate::draw::DrawList;
use crate::edit_box::{EditBox, FontMetrics};
use crate::font::BitmapFont;
use crate::frame::FrameCtrl;
use crate::frame_contract::FrameHit;
use crate::list_ctrl::{ListCell, ListCtrl, ListRow};
use crate::manor_info::{castle_name, join_names};
use crate::manor_setting::{TEXT_COLOR, app_atoi, app_itoa};
use crate::window::{Rect, WindowId};

pub const SELL_ID: WindowId = WindowId::MANOR_CROP_SELL;
pub const CHANGE_ID: WindowId = WindowId::MANOR_CROP_SELL_CHANGE;
pub const SELL_SIZE: (i32, i32) = sell_layout::SIZE;
pub const CHANGE_SIZE: (i32, i32) = change_layout::SIZE;
pub const SELL_INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 2, x: SELL_SIZE.0, y: SELL_SIZE.1 };
pub const CHANGE_INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 2, vertical_mode: 2, x: CHANGE_SIZE.0, y: CHANGE_SIZE.1 };

pub fn sell_default_position(viewport: (i32, i32)) -> (i32, i32) {
    crate::manor_setting::setting_default_position(viewport)
}

pub fn change_default_position(viewport: (i32, i32)) -> (i32, i32) {
    ((viewport.0 as f32 * 0.5 - 168.0) as i32, (viewport.1 as f32 * 0.5 - 153.0) as i32)
}

pub fn sell_grip() -> Rect {
    sell_layout::RECT_7C
}

pub fn change_grip() -> Rect {
    change_layout::RECT_7C
}

pub const SELL_BACK: &str = "L2UI_CH3.Manor.manor2_back";
pub const CHANGE_BACK: &str = "L2UI_CH3.Manor.manor3_back";
pub const VALUE_COLOR: Argb = Argb(0xffb09b79);
pub const CHANGE_LABELS: [(u32, (i32, i32)); 3] = [(1124, (15, 38)), (1134, (15, 54)), (1135, (15, 70))];
pub const CHANGE_VALUE_X: i32 = 76;
pub const CHANGE_FOOTERS: [(u32, (i32, i32)); 2] = [(1153, (75, 219)), (1156, (75, 244))];
pub const SELL_CELL_COUNT: usize = 10;
pub const CELL_CASTLE: usize = 1;
pub const CELL_PRICE_A: usize = 2;
pub const CELL_PRICE_B: usize = 3;
pub const CELL_PRICE_C: usize = 4;
pub const CELL_HAVE: usize = 5;
pub const CELL_SELL: usize = 6;
pub const CELL_LIST_A: usize = 8;
pub const CELL_LIST_B: usize = 9;
pub const BYPASS_DETAIL: &str = "manor_menu_select?ask=9&state=";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SellRow {
    pub object: i32,
    pub crop: i32,
    pub value2: i32,
    pub first: Vec<i32>,
    pub second: Vec<i32>,
    pub manor: i32,
    pub tail: [i32; 2],
    pub flag: i32,
    pub have: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SellHit {
    Close,
    Change,
    Ok,
    Cancel,
    List,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SellAction {
    Close,
    OpenChange(ChangeOpen),
    Submit(Vec<(i32, i32, i32, i32)>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChangeOpen {
    pub bypass: String,
    pub name: String,
    pub first: String,
    pub second: String,
}

#[derive(Clone, Debug)]
pub struct CropSellView {
    pub frame: FrameCtrl,
    pub list: ListCtrl,
    pub buttons: [TextButton; 3],
    pub data: Vec<(i32, i32, i32)>,
    pressed: Option<SellHit>,
}

impl CropSellView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Self {
            frame: sell_controls::FRAME_7C.build(text),
            list: ListCtrl::from_contract(&sell_controls::LIST_E8),
            buttons: [
                sell_controls::BUTTON_F0.build((0, 0), text),
                sell_controls::BUTTON_F4.build((0, 0), text),
                sell_controls::BUTTON_F8.build((0, 0), text),
            ],
            data: Vec::new(),
            pressed: None,
        }
    }

    pub fn set_data(&mut self, rows: &[SellRow], item_name: &dyn Fn(i32) -> String, lookup: &dyn Fn(u32) -> Option<String>) {
        self.list.clear();
        self.data.clear();
        for row in rows {
            let texts = [
                item_name(row.crop),
                castle_name(row.manor, lookup).unwrap_or_default(),
                row.tail[0].to_string(),
                row.tail[1].to_string(),
                row.flag.to_string(),
                row.have.to_string(),
                "0".to_string(),
                row.value2.to_string(),
                join_names(&row.first, item_name),
                join_names(&row.second, item_name),
            ];
            let cells = texts.iter().map(|text| ListCell::text(text)).collect();
            if self.list.add_item(ListRow { cells }) {
                self.data.push((row.object, row.crop, row.manor));
            }
        }
    }

    pub fn cell(&self, row: usize, index: usize) -> &str {
        self.list.rows.get(row).and_then(|row| row.cells.get(index)).map_or("", |cell| cell.text.as_str())
    }

    fn set_cell(&mut self, row: usize, index: usize, text: String) {
        if let Some(cell) = self.list.rows.get_mut(row).and_then(|row| row.cells.get_mut(index)) {
            cell.text = text;
        }
    }

    pub fn have(&self) -> i32 {
        self.list.selected().map_or(0, |row| app_atoi(self.cell(row, CELL_HAVE)))
    }

    pub fn apply_item(&mut self, count: i32, manor: i32, values: [i32; 3], lookup: &dyn Fn(u32) -> Option<String>) {
        let Some(row) = self.list.selected() else { return };
        if let Some(data) = self.data.get_mut(row) {
            data.2 = manor;
        }
        self.set_cell(row, CELL_CASTLE, castle_name(manor, lookup).unwrap_or_default());
        for (cell, value) in [CELL_PRICE_A, CELL_PRICE_B, CELL_PRICE_C].into_iter().zip(values) {
            self.set_cell(row, cell, app_itoa(value));
        }
        self.set_cell(row, CELL_SELL, app_itoa(count));
    }

    pub fn change_open(&self) -> Option<ChangeOpen> {
        let row = self.list.selected()?;
        let (_, crop, _) = *self.data.get(row)?;
        Some(ChangeOpen {
            bypass: format!("{BYPASS_DETAIL}{crop}&time=0"),
            name: self.cell(row, 0).to_string(),
            first: self.cell(row, CELL_LIST_A).to_string(),
            second: self.cell(row, CELL_LIST_B).to_string(),
        })
    }

    pub fn submit(&self) -> Vec<(i32, i32, i32, i32)> {
        self.data
            .iter()
            .enumerate()
            .filter_map(|(row, (object, crop, manor))| {
                let count = app_atoi(self.cell(row, CELL_SELL));
                (count != 0).then_some((*object, *crop, *manor, count))
            })
            .collect()
    }

    fn button_hit(&self, x: i32, y: i32) -> Option<SellHit> {
        let contracts = [
            (&sell_controls::BUTTON_F0, SellHit::Change),
            (&sell_controls::BUTTON_F4, SellHit::Ok),
            (&sell_controls::BUTTON_F8, SellHit::Cancel),
        ];
        self.buttons.iter().zip(contracts).find_map(|(button, (contract, hit))| contract.hit(button, x, y).map(|_| hit))
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<SellHit> {
        if sell_controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(SellHit::Close);
        }
        if let Some(hit) = self.button_hit(x, y) {
            return Some(hit);
        }
        self.list.owns(x, y).then_some(SellHit::List)
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        self.list.mouse_move(x, y);
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Option<SellHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        if hit == Some(SellHit::List) {
            self.list.mouse_down(x, y);
        }
        hit
    }

    pub fn double_click(&mut self, x: i32, y: i32) -> Option<SellAction> {
        if self.hit(x, y) != Some(SellHit::List) || self.list.hit(x, y) < 0 {
            return None;
        }
        self.list.double_click(x, y);
        self.list.notices.clear();
        self.change_open().map(SellAction::OpenChange)
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<SellAction> {
        self.list.mouse_up(x, y);
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            SellHit::Close | SellHit::Cancel => Some(SellAction::Close),
            SellHit::Change => self.change_open().map(SellAction::OpenChange),
            SellHit::Ok => Some(SellAction::Submit(self.submit())),
            SellHit::List => None,
        }
    }

    pub fn wheel(&mut self, delta: i16) {
        self.list.wheel(delta);
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32, lookup: &dyn Fn(u32) -> Option<String>) {
        let (x, y) = origin;
        self.frame.paint(list, x, y, SELL_SIZE.0, SELL_SIZE.1, font, alpha);
        let start = list.quads.len();
        list.draw_texture(x, y + 20, 600, 306, 0, 0, 600, 306, SELL_BACK, alpha);
        list.clip_from(start, Rect::new(x, y, SELL_SIZE.0, SELL_SIZE.1));
        self.list.paint_header(list, origin, font, &|id| lookup(id as u32));
        self.list.paint(list, origin, font);
        let contracts = [&sell_controls::BUTTON_F0, &sell_controls::BUTTON_F4, &sell_controls::BUTTON_F8];
        let hits = [SellHit::Change, SellHit::Ok, SellHit::Cancel];
        for ((button, contract), hit) in self.buttons.iter().zip(contracts).zip(hits) {
            contract.paint(button, list, origin, self.pressed == Some(hit), font);
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetailRow {
    pub manor: i32,
    pub values: [i32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChangeHit {
    Close,
    Max,
    Ok,
    Cancel,
    Count,
    Manor,
    List,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ChangeAction {
    Close,
    Apply { count: i32, manor: i32, values: [i32; 3] },
}

#[derive(Clone, Debug)]
pub struct ProcureChangeView {
    pub frame: FrameCtrl,
    pub list: ListCtrl,
    pub combo: ComboBox,
    pub combo_data: Vec<i32>,
    pub count: EditBox,
    pub buttons: [TextButton; 3],
    pub name: String,
    pub first: String,
    pub second: String,
    pub rows: Vec<DetailRow>,
    pressed: Option<ChangeHit>,
}

impl ProcureChangeView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        let text = |id: u32| lookup(id).unwrap_or_default();
        Self {
            frame: change_controls::FRAME_7C.build(text),
            list: ListCtrl::from_contract(&change_controls::LIST_208),
            combo: ComboBox::from_contract(&change_controls::CHILD_200, Vec::new(), 13),
            combo_data: Vec::new(),
            count: EditBox::from_contract(&change_controls::CHILD_204),
            buttons: [
                change_controls::BUTTON_1F4.build((0, 0), text),
                change_controls::BUTTON_1F8.build((0, 0), text),
                change_controls::BUTTON_1FC.build((0, 0), text),
            ],
            name: String::new(),
            first: String::new(),
            second: String::new(),
            rows: Vec::new(),
            pressed: None,
        }
    }

    pub fn set_texts(&mut self, open: &ChangeOpen) {
        self.name.clone_from(&open.name);
        self.first.clone_from(&open.first);
        self.second.clone_from(&open.second);
    }

    pub fn set_data(&mut self, rows: &[DetailRow], lookup: &dyn Fn(u32) -> Option<String>) {
        self.list.clear();
        self.combo.clear_entries();
        self.combo_data.clear();
        self.rows.clear();
        for row in rows {
            let name = castle_name(row.manor, lookup).unwrap_or_default();
            let cells = [name.clone(), row.values[0].to_string(), row.values[1].to_string(), row.values[2].to_string()];
            if self.list.add_item(ListRow { cells: cells.iter().map(|text| ListCell::text(text)).collect() }) {
                self.rows.push(row.clone());
            }
            if !name.is_empty() {
                self.combo.entries.push(name);
                self.combo_data.push(row.manor);
            }
        }
        self.count.set_text("");
        self.combo.selected = 0;
        self.combo.lose_focus();
        self.count.focus();
    }

    pub fn selected_manor(&self) -> i32 {
        usize::try_from(self.combo.selected).ok().and_then(|index| self.combo_data.get(index).copied()).unwrap_or(0)
    }

    fn selected_row(&self) -> Option<&DetailRow> {
        let manor = self.selected_manor();
        self.rows.iter().find(|row| row.manor == manor)
    }

    pub fn max(&mut self, have: i32) {
        let Some(row) = self.selected_row() else { return };
        let limit = row.values[0];
        self.count.set_text(&app_itoa(limit.min(have)));
    }

    pub fn apply(&self) -> ChangeAction {
        let values = self.selected_row().map_or([0; 3], |row| row.values);
        let count = app_atoi(&crate::edit_box::to_string(self.count.left()));
        ChangeAction::Apply { count, manor: self.selected_manor(), values }
    }

    pub fn blur(&mut self) {
        if self.count.focused {
            self.count.kill_focus();
        }
        self.combo.lose_focus();
    }

    fn button_hit(&self, x: i32, y: i32) -> Option<ChangeHit> {
        let contracts = [
            (&change_controls::BUTTON_1F4, ChangeHit::Max),
            (&change_controls::BUTTON_1F8, ChangeHit::Ok),
            (&change_controls::BUTTON_1FC, ChangeHit::Cancel),
        ];
        self.buttons.iter().zip(contracts).find_map(|(button, (contract, hit))| contract.hit(button, x, y).map(|_| hit))
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<ChangeHit> {
        if change_controls::FRAME_7C.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close) {
            return Some(ChangeHit::Close);
        }
        if self.combo.expanded_rect().contains(x, y) {
            return Some(ChangeHit::Manor);
        }
        if let Some(hit) = self.button_hit(x, y) {
            return Some(hit);
        }
        if self.count.rect.contains(x, y) {
            return Some(ChangeHit::Count);
        }
        self.list.owns(x, y).then_some(ChangeHit::List)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32, font: &BitmapFont) -> Option<ChangeHit> {
        let hit = self.hit(x, y);
        self.pressed = hit;
        let metrics = FontMetrics(font);
        match hit {
            Some(ChangeHit::Manor) => {
                if self.count.focused {
                    self.count.kill_focus();
                }
                self.combo.mouse_move(x, y);
                self.combo.mouse_down();
            }
            Some(ChangeHit::Count) => {
                self.combo.lose_focus();
                if !self.count.focused {
                    self.count.focus();
                }
                self.count.mouse_down(x, &metrics);
                self.count.layout(&metrics);
            }
            Some(ChangeHit::List) => {
                self.combo.lose_focus();
                self.list.mouse_down(x, y);
            }
            _ => {
                self.combo.lose_focus();
                if self.count.focused {
                    self.count.kill_focus();
                }
            }
        }
        hit
    }

    pub fn mouse_move(&mut self, x: i32, y: i32, font: &BitmapFont) {
        let metrics = FontMetrics(font);
        self.combo.mouse_move(x, y);
        self.list.mouse_move(x, y);
        if self.count.focused {
            self.count.mouse_move(x, &metrics);
            self.count.layout(&metrics);
        }
    }

    pub fn mouse_up(&mut self, x: i32, y: i32, have: i32) -> Option<ChangeAction> {
        self.list.mouse_up(x, y);
        self.count.mouse_up();
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        match pressed {
            ChangeHit::Close | ChangeHit::Cancel => Some(ChangeAction::Close),
            ChangeHit::Max => {
                self.max(have);
                None
            }
            ChangeHit::Ok => Some(self.apply()),
            ChangeHit::Count | ChangeHit::Manor | ChangeHit::List => None,
        }
    }

    pub fn key(&mut self, key: crate::chat::InputKey, font: &BitmapFont) -> bool {
        if !self.count.focused {
            return false;
        }
        let metrics = FontMetrics(font);
        if self.count.input_key(key, &metrics) == crate::edit_box::EditInput::Escape {
            self.count.kill_focus();
        }
        true
    }

    pub fn shift(&mut self, pressed: bool) -> bool {
        if !self.count.focused {
            return false;
        }
        if pressed {
            self.count.key_down_shift();
        } else {
            self.count.key_up(crate::edit_box::KEY_SHIFT);
        }
        true
    }

    pub fn tick(&mut self, elapsed_ms: u32) {
        self.count.tick(elapsed_ms);
    }

    pub fn wheel(&mut self, delta: i16) {
        self.list.wheel(delta);
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32, lookup: &dyn Fn(u32) -> Option<String>) {
        let (x, y) = origin;
        self.frame.paint(list, x, y, CHANGE_SIZE.0, CHANGE_SIZE.1, font, alpha);
        let start = list.quads.len();
        list.draw_texture(x, y + 20, 336, 286, 0, 0, 336, 286, CHANGE_BACK, alpha);
        let values = [&self.name, &self.first, &self.second];
        for ((sysstring, position), value) in CHANGE_LABELS.into_iter().zip(values) {
            list.draw_text(x + position.0, y + position.1, TEXT_COLOR, &lookup(sysstring).unwrap_or_default(), font);
            list.draw_text(x + CHANGE_VALUE_X, y + position.1, VALUE_COLOR, value, font);
        }
        for (sysstring, position) in CHANGE_FOOTERS {
            list.draw_text(x + position.0, y + position.1, TEXT_COLOR, &lookup(sysstring).unwrap_or_default(), font);
        }
        list.clip_from(start, Rect::new(x, y, CHANGE_SIZE.0, CHANGE_SIZE.1));
        self.list.paint_header(list, origin, font, &|id| lookup(id as u32));
        self.list.paint(list, origin, font);
        self.count.paint(list, origin, &FontMetrics(font), font);
        let contracts = [&change_controls::BUTTON_1F4, &change_controls::BUTTON_1F8, &change_controls::BUTTON_1FC];
        let hits = [ChangeHit::Max, ChangeHit::Ok, ChangeHit::Cancel];
        for ((button, contract), hit) in self.buttons.iter().zip(contracts).zip(hits) {
            contract.paint(button, list, origin, self.pressed == Some(hit), font);
        }
        self.combo.paint(list, origin, font, true, false);
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

    fn sell_row(object: i32, crop: i32) -> SellRow {
        SellRow { object, crop, value2: 7, first: vec![1, 2], second: vec![3], manor: 1, tail: [11, 12], flag: 1, have: 40 }
    }

    fn sell() -> CropSellView {
        let mut view = CropSellView::new(lookup);
        view.set_data(&[sell_row(10, 100), sell_row(11, 101)], &item, &lookup);
        view
    }

    #[test]
    fn windows_use_the_native_sizes_columns_and_positions() {
        assert_eq!((SELL_SIZE, CHANGE_SIZE), ((600, 326), (336, 306)));
        assert_eq!((SELL_ID, CHANGE_ID), (WindowId(71), WindowId(72)));
        assert_eq!(sell().list.columns.len(), 7);
        assert_eq!(ProcureChangeView::new(lookup).list.columns.len(), 4);
        assert_eq!(change_default_position((1024, 768)), (344, 231));
        assert_eq!(sell_grip(), Rect::new(0, 0, 600, 20));
        assert_eq!(change_grip(), Rect::new(0, 0, 336, 20));
    }

    #[test]
    fn rows_carry_ten_cells_with_the_castle_name_and_a_zero_sell_count() {
        let view = sell();
        let cells: Vec<_> = view.list.rows[0].cells.iter().map(|cell| cell.text.clone()).collect();
        assert_eq!(cells, ["item100", "s1049", "11", "12", "1", "40", "0", "7", "item1,item2", "item3"]);
        assert_eq!(view.list.rows[0].cells.len(), SELL_CELL_COUNT);
        assert_eq!(view.data, vec![(10, 100, 1), (11, 101, 1)]);
        assert!(view.submit().is_empty());
    }

    #[test]
    fn change_open_asks_for_the_detail_of_the_selected_crop_and_copies_the_texts() {
        let mut view = sell();
        assert_eq!(view.change_open(), None);
        view.list.select(1);
        assert_eq!(
            view.change_open(),
            Some(ChangeOpen {
                bypass: "manor_menu_select?ask=9&state=101&time=0".into(),
                name: "item101".into(),
                first: "item1,item2".into(),
                second: "item3".into(),
            })
        );
        assert_eq!(view.have(), 40);
    }

    #[test]
    fn apply_item_rewrites_the_selected_row_and_submit_sends_only_rows_with_a_count() {
        let mut view = sell();
        view.apply_item(5, 1, [1, 2, 3], &lookup);
        assert_eq!(view.cell(0, CELL_SELL), "0");
        view.list.select(1);
        view.apply_item(5, 2, [1, 2, 3], &lookup);
        assert_eq!(
            (view.cell(1, CELL_CASTLE), view.cell(1, CELL_PRICE_A), view.cell(1, CELL_PRICE_B), view.cell(1, CELL_PRICE_C), view.cell(1, CELL_SELL)),
            ("s1052", "1", "2", "3", "5")
        );
        assert_eq!(view.submit(), vec![(11, 101, 2, 5)]);
    }

    #[test]
    fn sell_buttons_open_change_submit_and_close() {
        let mut view = sell();
        let change = sell_controls::BUTTON_F0.rect;
        view.mouse_down(change.x + 2, change.y + 2);
        assert_eq!(view.mouse_up(change.x + 2, change.y + 2), None);
        view.list.select(0);
        view.mouse_down(change.x + 2, change.y + 2);
        assert!(matches!(view.mouse_up(change.x + 2, change.y + 2), Some(SellAction::OpenChange(_))));
        let ok = sell_controls::BUTTON_F4.rect;
        view.mouse_down(ok.x + 2, ok.y + 2);
        assert_eq!(view.mouse_up(ok.x + 2, ok.y + 2), Some(SellAction::Submit(Vec::new())));
        let cancel = sell_controls::BUTTON_F8.rect;
        view.mouse_down(cancel.x + 2, cancel.y + 2);
        assert_eq!(view.mouse_up(cancel.x + 2, cancel.y + 2), Some(SellAction::Close));
    }

    #[test]
    fn double_clicking_a_row_selects_it_and_opens_the_change_window() {
        let mut view = sell();
        assert_eq!(view.double_click(1, 1), None);
        let row = view.list.rect;
        let y = row.y + view.list.header_height + 2;
        let action = view.double_click(row.x + 3, y);
        assert!(matches!(action, Some(SellAction::OpenChange(open)) if open.name == "item100"));
        assert_eq!(view.list.selected(), Some(0));
    }

    fn detail_rows() -> Vec<DetailRow> {
        vec![DetailRow { manor: 1, values: [30, 11, 12] }, DetailRow { manor: 2, values: [5, 21, 22] }]
    }

    fn change() -> ProcureChangeView {
        let mut view = ProcureChangeView::new(lookup);
        view.set_data(&detail_rows(), &lookup);
        view
    }

    #[test]
    fn detail_fills_the_list_and_the_combo_with_the_castle_data() {
        let view = change();
        assert_eq!(view.combo.entries, ["s1049", "s1052"]);
        assert_eq!(view.combo_data, vec![1, 2]);
        assert_eq!(view.selected_manor(), 1);
        assert!(view.count.focused);
        let cells: Vec<_> = view.list.rows[1].cells.iter().map(|cell| cell.text.clone()).collect();
        assert_eq!(cells, ["s1052", "5", "21", "22"]);
    }

    #[test]
    fn max_fills_the_smaller_of_the_row_limit_and_what_the_player_has() {
        let mut view = change();
        view.max(100);
        assert_eq!(crate::edit_box::to_string(view.count.left()), "30");
        view.max(8);
        assert_eq!(crate::edit_box::to_string(view.count.left()), "8");
        view.combo.selected = 1;
        view.max(100);
        assert_eq!(crate::edit_box::to_string(view.count.left()), "5");
    }

    #[test]
    fn ok_reports_the_count_the_manor_and_the_row_values_without_validating() {
        let font = font();
        let mut view = change();
        assert!(view.key(InputKey::Char('9'), &font));
        assert!(view.key(InputKey::Char('9'), &font));
        assert_eq!(view.apply(), ChangeAction::Apply { count: 99, manor: 1, values: [30, 11, 12] });
        view.combo.selected = 1;
        assert_eq!(view.apply(), ChangeAction::Apply { count: 99, manor: 2, values: [5, 21, 22] });
    }

    #[test]
    fn buttons_and_combo_follow_the_native_handlers() {
        let font = font();
        let mut view = change();
        let ok = change_controls::BUTTON_1F8.rect;
        view.mouse_down(ok.x + 2, ok.y + 2, &font);
        assert_eq!(view.mouse_up(ok.x + 2, ok.y + 2, 0), Some(ChangeAction::Apply { count: 0, manor: 1, values: [30, 11, 12] }));
        let cancel = change_controls::BUTTON_1FC.rect;
        view.mouse_down(cancel.x + 2, cancel.y + 2, &font);
        assert_eq!(view.mouse_up(cancel.x + 2, cancel.y + 2, 0), Some(ChangeAction::Close));
        let max = change_controls::BUTTON_1F4.rect;
        view.mouse_down(max.x + 2, max.y + 2, &font);
        assert_eq!(view.mouse_up(max.x + 2, max.y + 2, 7), None);
        assert_eq!(crate::edit_box::to_string(view.count.left()), "7");
        let combo = view.combo.rect;
        assert_eq!(view.mouse_down(combo.x + 2, combo.y + 2, &font), Some(ChangeHit::Manor));
        let second = (combo.x + 5, combo.y + combo.h + view.combo.row_height + 3);
        view.mouse_move(second.0, second.1, &font);
        view.mouse_down(second.0, second.1, &font);
        assert_eq!(view.selected_manor(), 2);
    }

    #[test]
    fn paints_draw_the_native_backgrounds() {
        let font = font();
        let mut list = DrawList::default();
        sell().paint(&mut list, (10, 20), &font, 1.0, &lookup);
        assert!(list.quads.iter().any(|quad| quad.texture == SELL_BACK));
        let mut list = DrawList::default();
        let mut view = change();
        view.set_texts(&ChangeOpen { bypass: String::new(), name: "n".into(), first: "a".into(), second: "b".into() });
        view.paint(&mut list, (10, 20), &font, 1.0, &lookup);
        assert!(list.quads.iter().any(|quad| quad.texture == CHANGE_BACK));
    }
}
