use crate::child_contract::{ChildContract, ChildKind};
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::header_ctrl::{HeaderCtrl, HeaderInfo};
use crate::scroll::{BUTTON_SIZE, DOWN_BTN, DOWN_ON_BTN, SCROLL_WIDTH, SliderBar, UP_BTN, UP_ON_BTN};
use crate::window::Rect;

pub const ZEBRA: &str = "L2UI_CH3.Etc.textbackline";
pub const SELECT_LEFT: &str = "L2UI_CH3.ListCtrl.TextSelect";
pub const SELECT_RIGHT: &str = "L2UI_CH3.ListCtrl.TextSelect2";
pub const BLACK: &str = "NWindow.BlackTexture";
pub const CELL_COLOR: Argb = Argb(0xffdcdcdc);
pub const HEADER_COLOR: Argb = Argb(0xffe6dcbe);
pub const NOTICE_SELECT: u32 = 0;

const STATE_ROWS: u16 = 0x23c;
const STATE_CAPACITY: u16 = 0x240;
const STATE_MAX_COLUMNS: u16 = 0x244;
const STATE_HEADER: u16 = 0x25c;
const STATE_MODE_X: u16 = 0x260;
const STATE_MODE_Y: u16 = 0x264;
const STATE_ROW_HEIGHT: u16 = 0x26c;

pub const TAB_TEXTURES: &[(u16, &str)] = &[
    (0x18, "L2UI_CH3.ListCtrl.tab1"),
    (0x818, "L2UI_CH3.ListCtrl.tab2"),
    (0x1018, "L2UI_CH3.ListCtrl.tab3"),
    (0x1818, "L2UI_CH3.ListCtrl.tab1_down"),
    (0x2018, "L2UI_CH3.ListCtrl.tab2_down"),
    (0x2818, "L2UI_CH3.ListCtrl.tab3_down"),
];

const TEXTURE_NORMAL: [u16; 3] = [0x18, 0x818, 0x1018];
const TEXTURE_HOVER: [u16; 3] = [0x1818, 0x2018, 0x2818];
const TEXTURE_FULL: u16 = 0x3018;
const TEXTURE_FULL_HOVER: u16 = 0x3818;
const ICON_SLOTS: usize = 16;
const CELL_CLIP_HEIGHT: i32 = 15;
const TEXT_GAP: i32 = 10;
const SELECT_TRIM: i32 = 49;
const WHEEL_STEP: i32 = 120;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ColumnContract {
    pub width: i32,
    pub height: i32,
    pub sysstring: i32,
    pub flag_c: u32,
    pub flag_10: u32,
    pub textures: &'static [(u16, &'static str)],
}

impl ColumnContract {
    fn texture(&self, offset: u16) -> Option<&'static str> {
        self.textures.iter().find(|(at, _)| *at == offset).map(|(_, name)| *name)
    }

    fn named(&self, offset: u16) -> String {
        self.texture(offset).unwrap_or_default().to_string()
    }

    pub fn header_info(&self) -> HeaderInfo {
        HeaderInfo {
            width: self.width,
            height: self.height,
            label: self.sysstring,
            field_22c: self.flag_c as i32,
            field_230: self.flag_10 as i32,
            single: self.named(TEXTURE_FULL),
            single_hover: self.named(TEXTURE_FULL_HOVER),
            parts: TEXTURE_NORMAL.map(|offset| self.named(offset)),
            parts_hover: TEXTURE_HOVER.map(|offset| self.named(offset)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListContract {
    pub child: ChildContract,
    pub columns: &'static [ColumnContract],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pressed {
    Header(usize),
    Up,
    Down,
    Slider,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CellIcon {
    pub texture: String,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub u: i32,
    pub v: i32,
    pub uw: i32,
    pub vh: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BigIcon {
    pub texture: String,
    pub w: i32,
    pub h: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListCell {
    pub text: String,
    pub color: Argb,
    pub text_enabled: bool,
    pub icons: Vec<CellIcon>,
    pub big_icon: Option<BigIcon>,
}

impl Default for ListCell {
    fn default() -> Self {
        Self { text: String::new(), color: CELL_COLOR, text_enabled: true, icons: Vec::new(), big_icon: None }
    }
}

impl ListCell {
    pub fn text(text: &str) -> Self {
        Self { text: text.to_string(), ..Self::default() }
    }

    pub fn colored(text: &str, color: Argb) -> Self {
        Self { text: text.to_string(), color, ..Self::default() }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ListRow {
    pub cells: Vec<ListCell>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListNotice {
    pub id: u32,
    pub index: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListCtrl {
    pub id: u32,
    pub rect: Rect,
    pub columns: Vec<ColumnContract>,
    pub rows: Vec<ListRow>,
    pub rows_visible: i32,
    pub capacity: i32,
    pub max_columns: i32,
    pub header_height: i32,
    pub mode: (i32, i32),
    pub row_height: i32,
    pub headers: Vec<HeaderCtrl>,
    pub sort_forward: Vec<bool>,
    pub sortable: bool,
    origin: Vec<usize>,
    pub slider: SliderBar,
    pub pressed: Option<Pressed>,
    pub left_held: bool,
    pub sounds: Vec<usize>,
    first: i32,
    selected: i32,
    hover: i32,
    pub notices: Vec<ListNotice>,
}

fn float_trunc(value: f32) -> i32 {
    value as i64 as i32
}

impl ListCtrl {
    pub fn from_contract(contract: &ListContract) -> Self {
        debug_assert_eq!(contract.child.kind, ChildKind::ListCtrl);
        let child = &contract.child;
        let header = contract.columns.last().map_or(child.field(STATE_HEADER) as i32, |column| column.height);
        let mut x = 0;
        let headers: Vec<HeaderCtrl> = contract
            .columns
            .iter()
            .enumerate()
            .map(|(index, column)| {
                let mut header = HeaderCtrl::new(Rect::new(x, 0, column.width, column.height));
                header.set_info(&column.header_info());
                header.notify_id = index as i32;
                x += column.width;
                header
            })
            .collect();
        let mut ctrl = Self {
            id: child.id,
            rect: child.rect,
            columns: contract.columns.to_vec(),
            rows: Vec::new(),
            rows_visible: child.field(STATE_ROWS) as i32,
            capacity: child.field(STATE_CAPACITY) as i32,
            max_columns: child.field(STATE_MAX_COLUMNS) as i32,
            header_height: header,
            mode: (child.field(STATE_MODE_X) as i32, child.field(STATE_MODE_Y) as i32),
            row_height: child.field(STATE_ROW_HEIGHT) as i32,
            sort_forward: vec![true; headers.len()],
            sortable: true,
            origin: Vec::new(),
            headers,
            slider: SliderBar::default(),
            pressed: None,
            left_held: false,
            sounds: Vec::new(),
            first: 0,
            selected: -1,
            hover: -1,
            notices: Vec::new(),
        };
        ctrl.slider = SliderBar::new(ctrl.scroll_rects().slider.h);
        ctrl.sync_slider();
        ctrl
    }

    pub fn scroll_rects(&self) -> crate::scroll::ScrollRects {
        let top = self.columns.first().map_or(0, |column| column.height);
        let x = self.rect.w - SCROLL_WIDTH;
        crate::scroll::ScrollRects {
            up: Rect::new(x, top, BUTTON_SIZE, BUTTON_SIZE),
            down: Rect::new(x, self.rect.h - BUTTON_SIZE, BUTTON_SIZE, BUTTON_SIZE),
            slider: Rect::new(x, top + BUTTON_SIZE, SCROLL_WIDTH, self.rect.h - 2 * BUTTON_SIZE - top),
        }
    }

    fn sync_slider(&mut self) {
        self.slider.set_range(self.count(), self.rows_visible);
        self.slider.set_pos(self.first);
    }

    fn sync_position(&mut self) {
        self.slider.set_pos(self.first);
    }

    pub fn count(&self) -> i32 {
        self.rows.len() as i32
    }

    pub fn first_row(&self) -> i32 {
        self.first
    }

    pub fn selected(&self) -> Option<usize> {
        usize::try_from(self.selected).ok().filter(|&index| index < self.rows.len())
    }

    pub fn selected_raw(&self) -> i32 {
        self.selected
    }

    pub fn selected_origin(&self) -> Option<usize> {
        self.selected().and_then(|index| self.origin.get(index).copied())
    }

    pub fn hovered(&self) -> i32 {
        self.hover
    }

    pub fn row_tip(&self, style: u32, description: &str, text: &dyn Fn(i32) -> String) -> Option<crate::tooltip::Tooltip> {
        match_row_tooltip(style, self.tip_row()?, description, text)
    }

    pub fn tip_row(&self) -> Option<&ListRow> {
        if self.hover >= 0 && self.hover < self.capacity && self.hover == self.selected {
            self.rows.get(self.hover as usize)
        } else {
            None
        }
    }

    pub fn add_item(&mut self, row: ListRow) -> bool {
        if self.capacity <= self.count() {
            return false;
        }
        self.origin.push(self.rows.len());
        self.rows.push(row);
        self.sync_slider();
        true
    }

    pub fn clear(&mut self) {
        self.rows.clear();
        self.origin.clear();
        self.first = 0;
        self.selected = -1;
        self.hover = -1;
        self.slider.total = 0;
        self.slider.visible = 0;
        self.slider.thumb = 0;
        self.slider.thumb_y = 0;
        self.slider.pos = 0;
        self.slider.shown = false;
    }

    pub fn select(&mut self, index: i32) {
        if (0..self.count()).contains(&index) {
            self.selected = index;
        }
    }

    pub fn set_mode(&mut self, x: i32, y: i32) {
        self.mode = (x, y);
    }

    pub fn hit(&self, x: i32, y: i32) -> i32 {
        let px = x - self.rect.x;
        let py = y - self.rect.y;
        if !(px > -1 && px <= self.rect.w && self.header_height <= py && py <= self.rect.h) {
            return -1;
        }
        let end = self.first + self.rows_visible;
        let mut row = self.first;
        let mut top = self.header_height;
        let mut found = -1;
        if row < end {
            loop {
                found = -1;
                let keep = row < self.count()
                    && (self.rect.w <= px || py < top || {
                        found = row;
                        top + self.row_height <= py
                    });
                if !keep {
                    break;
                }
                row += 1;
                top += self.row_height;
                if end <= row {
                    return -1;
                }
            }
        }
        found
    }

    pub fn child_at(&self, x: i32, y: i32) -> Option<Pressed> {
        let (px, py) = (x - self.rect.x, y - self.rect.y);
        let inside = |rect: Rect| rect.x <= px && px <= rect.x + rect.w && rect.y <= py && py <= rect.y + rect.h;
        let rects = self.scroll_rects();
        if inside(rects.slider) {
            return Some(Pressed::Slider);
        }
        if inside(rects.down) {
            return Some(Pressed::Down);
        }
        if inside(rects.up) {
            return Some(Pressed::Up);
        }
        self.headers.iter().position(|header| header.contains(px, py)).map(Pressed::Header)
    }

    pub fn owns(&self, x: i32, y: i32) -> bool {
        self.child_at(x, y).is_some() || self.hit(x, y) > -1
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) -> bool {
        let (px, py) = (x - self.rect.x, y - self.rect.y);
        let left = self.left_held;
        for header in &mut self.headers {
            header.on_mouse_move(px, py, left);
        }
        if self.pressed == Some(Pressed::Slider) {
            let top = self.scroll_rects().slider.y;
            if let Some(position) = self.slider.mouse_move(py - top) {
                self.first = position;
            }
        }
        self.hover = self.hit(x, y);
        true
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> bool {
        self.left_held = true;
        let (px, py) = (x - self.rect.x, y - self.rect.y);
        match self.child_at(x, y) {
            Some(Pressed::Header(index)) => {
                let events = self.headers[index].on_lbutton_down(px, py);
                self.sounds.extend(events.sound);
                self.pressed = Some(Pressed::Header(index));
            }
            Some(Pressed::Up) => {
                self.pressed = Some(Pressed::Up);
                self.line_up();
            }
            Some(Pressed::Down) => {
                self.pressed = Some(Pressed::Down);
                self.line_down();
            }
            Some(Pressed::Slider) => {
                self.pressed = Some(Pressed::Slider);
                let top = self.scroll_rects().slider.y;
                self.slider.mouse_down(py - top);
            }
            None => {
                let index = self.hit(x, y);
                self.selected = index;
                self.hover = index;
            }
        }
        true
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> bool {
        self.left_held = false;
        let (px, py) = (x - self.rect.x, y - self.rect.y);
        match self.pressed.take() {
            Some(Pressed::Header(_)) => {
                if let Some(index) = self.headers.iter().position(|header| header.contains(px, py)) {
                    let events = self.headers[index].on_lbutton_up(px, py);
                    if let Some(column) = events.notify
                        && self.sortable
                    {
                        self.sort(column as usize);
                    }
                }
            }
            Some(Pressed::Slider) => self.slider.mouse_up(),
            Some(Pressed::Up | Pressed::Down) | None => {}
        }
        true
    }

    pub fn sort(&mut self, column: usize) {
        let Some(contract) = self.columns.get(column) else { return };
        let numeric = contract.flag_10 != 0;
        let forward = self.sort_forward[column];
        let key = |row: &ListRow| row.cells.get(column).map_or(String::new(), |cell| cell.text.clone());
        let keys: Vec<String> = self.rows.iter().map(key).collect();
        let mut order: Vec<usize> = (0..self.rows.len()).collect();
        let count = order.len();
        for pass in 0..count {
            for at in 0..count.saturating_sub(pass).min(count.saturating_sub(1)) {
                let (a, b) = (&keys[order[at]], &keys[order[at + 1]]);
                let ordering = if numeric { wtoi(a).cmp(&wtoi(b)) } else { wcsicmp(a, b) };
                let swap = if forward { ordering == std::cmp::Ordering::Greater } else { ordering == std::cmp::Ordering::Less };
                if swap {
                    order.swap(at, at + 1);
                }
            }
        }
        let selected = self.selected();
        self.rows = order.iter().map(|&index| self.rows[index].clone()).collect();
        self.origin = order.iter().map(|&index| self.origin[index]).collect();
        self.sort_forward[column] = !forward;
        if let Some(previous) = selected
            && let Some(index) = order.iter().position(|&candidate| candidate == previous)
        {
            self.selected = index as i32;
        }
    }

    pub fn origins(&self) -> &[usize] {
        &self.origin
    }

    pub fn reset_origins(&mut self) {
        self.origin = (0..self.rows.len()).collect();
    }

    pub fn replace_rows(&mut self, rows: Vec<ListRow>) {
        self.origin = (0..rows.len()).collect();
        self.rows = rows;
        self.first = self.first.min((self.count() - self.rows_visible).max(0));
        self.hover = -1;
        self.sync_slider();
    }

    pub fn double_click(&mut self, x: i32, y: i32) -> bool {
        let index = self.hit(x, y);
        self.selected = index;
        self.hover = index;
        if index > -1 && index < self.count() {
            self.notices.push(ListNotice { id: self.id, index });
        }
        true
    }

    pub fn wheel(&mut self, delta: i16) -> bool {
        let steps = i32::from(delta) / WHEEL_STEP;
        let count = self.count();
        let rows = self.rows_visible;
        if steps < 1 {
            if count < rows {
                self.first = if rows < (count - self.first) + steps { self.first - steps } else { 0 };
            } else {
                self.first = if rows < (count - self.first) + steps { self.first - steps } else { count - rows };
            }
        } else if steps < self.first {
            self.first -= steps;
        } else {
            self.first = 0;
        }
        self.sync_position();
        true
    }

    pub fn line_up(&mut self) -> bool {
        if self.rows_visible < self.count() {
            self.first = (self.first - 1).max(0);
        }
        self.sync_position();
        true
    }

    pub fn line_down(&mut self) -> bool {
        if self.rows_visible < self.count() {
            self.first = (self.first + 1).min(self.count() - self.rows_visible);
        }
        self.sync_position();
        true
    }

    pub fn column_x(&self, index: usize) -> i32 {
        self.columns.iter().take(index).map(|column| column.width).sum()
    }

    pub fn paint_header(
        &self,
        list: &mut DrawList,
        origin: (i32, i32),
        font: &BitmapFont,
        lookup: &dyn Fn(i32) -> Option<String>,
    ) {
        let origin = (origin.0 + self.rect.x, origin.1 + self.rect.y);
        for header in &self.headers {
            let label = if header.label >= 0 { lookup(header.label) } else { None };
            header.paint(list, origin, label.as_deref(), font);
        }
    }

    fn paint_scroll(&self, list: &mut DrawList, origin: (i32, i32)) {
        let rects = self.scroll_rects();
        let (x, y) = (origin.0 + self.rect.x, origin.1 + self.rect.y);
        let up = if self.pressed == Some(Pressed::Up) { UP_ON_BTN } else { UP_BTN };
        let down = if self.pressed == Some(Pressed::Down) { DOWN_ON_BTN } else { DOWN_BTN };
        let size = BUTTON_SIZE;
        list.draw_texture(x + rects.up.x, y + rects.up.y, size, size, 0, 0, size, size, up, 1.0);
        list.draw_texture(x + rects.down.x, y + rects.down.y, size, size, 0, 0, size, size, down, 1.0);
        self.slider.paint(list, x + rects.slider.x, y + rects.slider.y);
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont) {
        let (x, y) = (origin.0 + self.rect.x, origin.1 + self.rect.y);
        let (width, height) = (self.rect.w, self.rect.h);
        let whole = Rect::new(x, y, width, height);
        let start = list.quads.len();
        for index in 0..self.rows_visible + 1 {
            if index % 2 == 1 {
                list.draw_texture(x, y + self.row_height * index - 1 + self.header_height, width, self.row_height, 0, 0, 8, 15, ZEBRA, 1.0);
            }
        }
        list.clip_from(start, whole);
        let body = Rect::new(x, y + self.header_height, width, float_trunc(height as f32 - self.header_height as f32));
        let mut slot = 0;
        let mut row = self.first;
        while row < self.first + self.rows_visible && row < self.count() {
            let top = y + self.row_height * slot - 1 + self.header_height;
            if row == self.selected {
                let begin = list.quads.len();
                list.draw_texture(x + 1, top, float_trunc(width as f32 - SELECT_TRIM as f32), self.row_height, 0, 0, 16, 13, SELECT_LEFT, 1.0);
                list.draw_texture(x + float_trunc(width as f32 + 1.0 - 47.0 - 2.0), top, 32, self.row_height, 0, 0, 32, 13, SELECT_RIGHT, 1.0);
                list.clip_from(begin, body);
            }
            self.paint_row(list, (x, y), slot, &self.rows[row as usize], font);
            slot += 1;
            row += 1;
        }
        self.paint_scroll(list, origin);
    }

    fn paint_row(&self, list: &mut DrawList, origin: (i32, i32), slot: i32, row: &ListRow, font: &BitmapFont) {
        let mut column_x = self.mode.0;
        let row_y = slot * self.row_height + self.mode.1 + self.header_height;
        for (index, column) in self.columns.iter().enumerate() {
            let Some(cell) = row.cells.get(index) else {
                column_x += column.width;
                continue;
            };
            let start = list.quads.len();
            let mut icons_width = 0;
            for icon in cell.icons.iter().take(ICON_SLOTS) {
                let uw = if icon.uw == 0 { icon.w } else { icon.uw };
                let vh = if icon.vh == 0 { icon.w } else { icon.vh };
                list.draw_texture(origin.0 + icon.x + column_x, origin.1 + icon.y + row_y, icon.w, icon.h, icon.u, icon.v, uw, vh, &icon.texture, 1.0);
                icons_width += icon.w;
            }
            if let Some(big) = &cell.big_icon {
                let x = column.width / 2 - big.w / 2 + column_x;
                let y = slot * self.row_height - big.h / 2 + self.header_height + self.row_height / 2;
                list.draw_texture(origin.0 + x, origin.1 + y, big.w, big.h, 0, 0, big.w, big.h, &big.texture, 1.0);
            }
            if cell.text_enabled {
                let offset = cell.big_icon.as_ref().map_or(0, |big| big.w) + icons_width + TEXT_GAP + column_x;
                list.draw_text(origin.0 + offset, origin.1 + row_y, cell.color, &cell.text, font);
            }
            let clip = Rect::new(origin.0 + column_x, origin.1 + row_y - 1, column.width, CELL_CLIP_HEIGHT);
            list.clip_from(start, clip);
            column_x += column.width;
        }
    }
}

pub const ROW_TIP_QUEST: u32 = 0x10000;
pub const ROW_TIP_SEED_INFO: u32 = 0x20000;
pub const ROW_TIP_CROP_INFO: u32 = 0x40000;
pub const ROW_TIP_SEED_SETTING: u32 = 0x80000;
pub const ROW_TIP_CROP_SETTING: u32 = 0x100000;
pub const ROW_TIP_DEFAULT_INFO: u32 = 0x200000;
pub const ROW_TIP_CROP_SELL: u32 = 0x400000;

pub fn match_row_tooltip(style: u32, row: &ListRow, description: &str, text: &dyn Fn(i32) -> String) -> Option<crate::tooltip::Tooltip> {
    use crate::tooltip::{Tooltip, TooltipLine, VALUE_COLOR};
    let cell = |index: usize| row.cells.get(index).map_or_else(String::new, |cell| cell.text.clone());
    let line = |label: String, value: String| TooltipLine { label, value, color: VALUE_COLOR };
    let cells = match style {
        ROW_TIP_SEED_SETTING | ROW_TIP_CROP_SELL => Some((7, 8, 9)),
        ROW_TIP_SEED_INFO => Some((4, 5, 6)),
        ROW_TIP_CROP_INFO => Some((5, 6, 7)),
        ROW_TIP_CROP_SETTING => Some((9, 10, 11)),
        ROW_TIP_DEFAULT_INFO => Some((1, 4, 5)),
        _ => None,
    };
    if let Some((first, second, third)) = cells {
        return Some(Tooltip {
            row: true,
            row_style: style,
            lines: vec![
                line(String::new(), cell(0)),
                line(text(537), cell(first)),
                line(text(1134), cell(second)),
                line(text(1135), cell(third)),
            ],
            ..Default::default()
        });
    }
    if style == ROW_TIP_QUEST {
        let kind = wtoi(&cell(3));
        let mut last = cell(2);
        let tail = match kind {
            0 | 2 => Some(text(861)),
            1 | 3 => Some(text(862)),
            _ => None,
        };
        if let Some(tail) = tail {
            last = tail;
        }
        return Some(Tooltip {
            row: true,
            row_style: style,
            lines: vec![
                line(text(1200), cell(0)),
                line(text(1201), cell(1)),
                line(text(922), cell(2)),
                line(text(1202), last),
            ],
            description: description.to_string(),
            ..Default::default()
        });
    }
    None
}

pub fn wcsicmp(a: &str, b: &str) -> std::cmp::Ordering {
    let fold = |unit: u16| if (0x41..=0x5a).contains(&unit) { unit + 0x20 } else { unit };
    a.encode_utf16().map(fold).cmp(b.encode_utf16().map(fold))
}

pub fn wtoi(text: &str) -> i32 {
    let mut units = text.encode_utf16().skip_while(|unit| matches!(*unit, 0x09..=0x0d | 0x20)).peekable();
    let negative = units.peek() == Some(&0x2d);
    if matches!(units.peek(), Some(0x2d | 0x2b)) {
        units.next();
    }
    let mut value: i32 = 0;
    for unit in units {
        let Some(digit) = char::from_u32(u32::from(unit)).and_then(|c| c.to_digit(10)) else { break };
        value = value.wrapping_mul(10).wrapping_add(digit as i32);
    }
    if negative { value.wrapping_neg() } else { value }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    const COLUMNS: &[ColumnContract] = &[
        ColumnContract { width: 169, height: 19, sysstring: 1124, flag_c: 1, flag_10: 0, textures: &[] },
        ColumnContract { width: 50, height: 19, sysstring: 537, flag_c: 1, flag_10: 1, textures: &[] },
    ];

    const CONTRACT: ListContract = ListContract {
        child: ChildContract {
            kind: ChildKind::ListCtrl,
            rect: Rect::new(7, 42, 219, 199),
            id: 0xf0000001,
            vtable: 0x101a6fa8,
            registration: (2, 0, 0),
            handles: &[],
            state: &[(0x23c, 10), (0x240, 1000), (0x244, 6), (0x250, 0xffff_ffff), (0x254, 0xffff_ffff), (0x258, 15), (0x25c, 19), (0x264, 3), (0x268, 1), (0x26c, 18)],
            textures: &[],
        },
        columns: COLUMNS,
    };

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("font".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn filled(count: usize) -> ListCtrl {
        let mut ctrl = ListCtrl::from_contract(&CONTRACT);
        for index in 0..count {
            ctrl.add_item(ListRow { cells: vec![ListCell::text(&format!("row{index}")), ListCell::text("1")] });
        }
        ctrl
    }

    #[test]
    fn contract_state_initializes_geometry_and_limits() {
        let ctrl = ListCtrl::from_contract(&CONTRACT);
        assert_eq!((ctrl.rows_visible, ctrl.capacity, ctrl.max_columns), (10, 1000, 6));
        assert_eq!((ctrl.header_height, ctrl.row_height, ctrl.mode), (19, 18, (0, 3)));
        assert_eq!((ctrl.selected(), ctrl.hovered(), ctrl.first_row(), ctrl.count()), (None, -1, 0, 0));
        assert_eq!(ctrl.column_x(0), 0);
        assert_eq!(ctrl.column_x(1), 169);
        assert_eq!(ctrl.column_x(2), 219);
    }

    #[test]
    fn add_item_stops_at_the_capacity_and_clear_resets_selection_and_scroll() {
        let mut ctrl = ListCtrl::from_contract(&CONTRACT);
        ctrl.capacity = 2;
        assert!(ctrl.add_item(ListRow::default()));
        assert!(ctrl.add_item(ListRow::default()));
        assert!(!ctrl.add_item(ListRow::default()));
        ctrl.select(1);
        assert_eq!(ctrl.selected(), Some(1));
        ctrl.select(2);
        ctrl.select(-1);
        assert_eq!(ctrl.selected(), Some(1));
        ctrl.clear();
        assert_eq!((ctrl.count(), ctrl.selected(), ctrl.hovered(), ctrl.first_row()), (0, None, -1, 0));
    }

    #[test]
    fn hit_maps_rows_below_the_header_and_rejects_the_blank_area_and_the_right_edge() {
        let ctrl = filled(3);
        let (ox, oy) = (7, 42);
        assert_eq!(ctrl.hit(ox + 5, oy + 18), -1);
        assert_eq!(ctrl.hit(ox + 5, oy + 19), 0);
        assert_eq!(ctrl.hit(ox + 5, oy + 19 + 18), 1);
        assert_eq!(ctrl.hit(ox + 5, oy + 19 + 17 + 18 + 18), 2);
        assert_eq!(ctrl.hit(ox + 5, oy + 19 + 18 * 3), -1);
        assert_eq!(ctrl.hit(ox + 219, oy + 30), -1);
        assert_eq!(ctrl.hit(ox - 1, oy + 30), -1);
        assert_eq!(ctrl.hit(ox + 5, oy + 200), -1);
    }

    #[test]
    fn mouse_events_track_hover_selection_and_notify_only_on_double_click() {
        let mut ctrl = filled(3);
        ctrl.mouse_move(7 + 5, 42 + 19 + 20);
        assert_eq!((ctrl.hovered(), ctrl.selected()), (1, None));
        ctrl.mouse_down(7 + 5, 42 + 19 + 40);
        assert_eq!((ctrl.hovered(), ctrl.selected()), (2, Some(2)));
        assert!(ctrl.notices.is_empty());
        ctrl.double_click(7 + 5, 42 + 19 + 2);
        assert_eq!(ctrl.notices, vec![ListNotice { id: 0xf0000001, index: 0 }]);
        ctrl.double_click(7 + 5, 42 + 150);
        assert_eq!(ctrl.notices.len(), 1);
        assert_eq!(ctrl.selected(), None);
    }

    #[test]
    fn rows_beyond_the_page_are_reached_by_wheel_and_line_steps_within_bounds() {
        let mut ctrl = filled(25);
        ctrl.wheel(-120);
        assert_eq!(ctrl.first_row(), 1);
        ctrl.wheel(-360);
        assert_eq!(ctrl.first_row(), 4);
        ctrl.wheel(-120 * 40);
        assert_eq!(ctrl.first_row(), 15);
        ctrl.wheel(120);
        assert_eq!(ctrl.first_row(), 14);
        ctrl.wheel(120 * 40);
        assert_eq!(ctrl.first_row(), 0);
        ctrl.line_up();
        assert_eq!(ctrl.first_row(), 0);
        for _ in 0..30 {
            ctrl.line_down();
        }
        assert_eq!(ctrl.first_row(), 15);
        ctrl.line_up();
        assert_eq!(ctrl.first_row(), 14);
        let mut short = filled(4);
        short.wheel(-120);
        short.line_down();
        assert_eq!(short.first_row(), 0);
    }

    #[test]
    fn hit_uses_the_scrolled_rows() {
        let mut ctrl = filled(25);
        ctrl.line_down();
        ctrl.line_down();
        assert_eq!(ctrl.hit(7 + 5, 42 + 19 + 1), 2);
        assert_eq!(ctrl.hit(7 + 5, 42 + 19 + 18 * 9 + 1), 11);
    }

    #[test]
    fn paint_draws_zebra_on_odd_slots_the_selection_and_clipped_cell_text() {
        let mut ctrl = filled(3);
        ctrl.select(1);
        let mut list = DrawList::default();
        ctrl.paint(&mut list, (100, 200), &font());
        let zebra: Vec<_> = list.quads.iter().filter(|q| q.texture == ZEBRA).collect();
        assert_eq!(zebra.len(), 5);
        assert_eq!((zebra[0].x, zebra[0].y, zebra[0].w, zebra[0].h), (107.0, 200.0 + 42.0 + 18.0 - 1.0 + 19.0, 219.0, 18.0));
        let left = list.quads.iter().find(|q| q.texture == SELECT_LEFT).unwrap();
        assert_eq!((left.x, left.y, left.w, left.h), (108.0, 242.0 + 18.0 - 1.0 + 19.0, 170.0, 18.0));
        let right = list.quads.iter().find(|q| q.texture == SELECT_RIGHT).unwrap();
        assert_eq!((right.x, right.w), (107.0 + 171.0, 32.0));
        assert!(list.quads.iter().any(|q| q.texture == "font"));
    }

    #[test]
    fn header_draws_black_parts_per_column_and_the_centered_sysstring() {
        let ctrl = filled(0);
        let mut list = DrawList::default();
        ctrl.paint_header(&mut list, (0, 0), &font(), &|id| (id == 1124).then(|| "abcd".to_string()));
        let black: Vec<_> = list.quads.iter().filter(|q| q.texture == BLACK).collect();
        assert_eq!(black.len(), 6);
        assert_eq!((black[0].x, black[0].w, black[1].x, black[1].w, black[2].x), (7.0, 8.0, 15.0, 153.0, 168.0));
        assert_eq!(black[3].x, 7.0 + 169.0);
        assert_eq!(list.quads.iter().filter(|q| q.texture == "font").count(), 4);
    }

    #[test]
    fn list_ctrl_10023210_places_the_scroll_children_from_the_first_column_height() {
        let ctrl = filled(0);
        let rects = ctrl.scroll_rects();
        assert_eq!(rects.up, Rect::new(204, 19, 15, 15));
        assert_eq!(rects.down, Rect::new(204, 184, 15, 15));
        assert_eq!(rects.slider, Rect::new(204, 34, 15, 199 - 30 - 19));
    }

    #[test]
    fn list_ctrl_scroll_buttons_step_the_first_row_on_press_and_the_slider_follows_it() {
        let mut ctrl = filled(25);
        let (ox, oy) = (7, 42);
        ctrl.mouse_down(ox + 210, oy + 190);
        assert_eq!((ctrl.first_row(), ctrl.pressed, ctrl.slider.pos), (1, Some(Pressed::Down), 1));
        ctrl.mouse_up(ox + 210, oy + 190);
        assert_eq!(ctrl.pressed, None);
        ctrl.mouse_down(ox + 210, oy + 25);
        assert_eq!((ctrl.first_row(), ctrl.pressed), (0, Some(Pressed::Up)));
        ctrl.mouse_up(ox + 210, oy + 25);
        assert_eq!(ctrl.selected(), None);
        ctrl.wheel(-360);
        assert_eq!(ctrl.slider.pos, 3);
    }

    #[test]
    fn list_ctrl_slider_drag_sets_the_first_row_to_the_reported_position() {
        let mut ctrl = filled(40);
        let (ox, oy) = (7, 42);
        let track = ctrl.scroll_rects().slider;
        ctrl.mouse_down(ox + track.x + 5, oy + track.y + 2);
        assert_eq!(ctrl.pressed, Some(Pressed::Slider));
        assert!(ctrl.slider.dragging);
        ctrl.mouse_move(ox + track.x + 5, oy + track.y + 2 + 40);
        assert!(ctrl.first_row() > 0);
        let first = ctrl.first_row();
        ctrl.mouse_up(ox + track.x + 5, oy + track.y + 42);
        assert!(!ctrl.slider.dragging);
        ctrl.mouse_move(ox + track.x + 5, oy + track.y + 100);
        assert_eq!(ctrl.first_row(), first);
    }

    #[test]
    fn list_ctrl_paints_the_scroll_buttons_and_the_thumb_only_when_rows_overflow() {
        let paint = |ctrl: &ListCtrl| {
            let mut list = DrawList::default();
            ctrl.paint(&mut list, (0, 0), &font());
            list
        };
        let few = paint(&filled(3));
        assert!(few.quads.iter().any(|q| q.texture == crate::scroll::UP_BTN && (q.x, q.y) == (211.0, 61.0)));
        assert!(few.quads.iter().any(|q| q.texture == crate::scroll::DOWN_BTN && (q.x, q.y) == (211.0, 226.0)));
        assert!(!few.quads.iter().any(|q| q.texture == crate::scroll::SLIDER_TOP));
        let mut many = filled(30);
        many.mouse_down(7 + 210, 42 + 25);
        let list = paint(&many);
        assert!(list.quads.iter().any(|q| q.texture == crate::scroll::UP_ON_BTN));
        assert!(list.quads.iter().any(|q| q.texture == crate::scroll::SLIDER_TOP));
    }

    #[test]
    fn nc_header_ctrl_10021e40_tracks_hover_and_press_per_column() {
        let mut ctrl = filled(2);
        let (ox, oy) = (7, 42);
        ctrl.mouse_move(ox + 10, oy + 5);
        assert_eq!(ctrl.headers.iter().map(|h| h.state).collect::<Vec<_>>(), vec![2, 0]);
        ctrl.mouse_down(ox + 10, oy + 5);
        assert_eq!(ctrl.headers[0].state, 1);
        assert_eq!(ctrl.sounds, vec![crate::interface_sound::CLICK]);
        assert_eq!(ctrl.selected(), None);
        ctrl.mouse_move(ox + 180, oy + 5);
        assert_eq!(ctrl.headers.iter().map(|h| h.state).collect::<Vec<_>>(), vec![0, 1]);
        ctrl.mouse_up(ox + 180, oy + 5);
        assert_eq!(ctrl.headers.iter().map(|h| h.state).collect::<Vec<_>>(), vec![0, 2]);
        ctrl.mouse_move(ox + 100, oy + 100);
        assert_eq!(ctrl.headers.iter().map(|h| h.state).collect::<Vec<_>>(), vec![0, 0]);
    }

    #[test]
    fn nc_header_ctrl_100227e0_paints_the_texture_of_the_column_state() {
        const TEXTURED: &[ColumnContract] = &[ColumnContract {
            width: 80,
            height: 19,
            sysstring: -1,
            flag_c: 1,
            flag_10: 0,
            textures: &[(0x3018, "col.n"), (0x3818, "col.h")],
        }];
        let contract = ListContract { child: CONTRACT.child, columns: TEXTURED };
        let mut ctrl = ListCtrl::from_contract(&contract);
        let painted = |ctrl: &ListCtrl| {
            let mut list = DrawList::default();
            ctrl.paint_header(&mut list, (0, 0), &font(), &|_| None);
            list.quads.iter().map(|q| q.texture.to_string()).collect::<Vec<_>>()
        };
        assert_eq!(painted(&ctrl), vec!["col.n"]);
        ctrl.mouse_move(7 + 10, 42 + 5);
        assert_eq!(painted(&ctrl), vec!["col.n_over"]);
        ctrl.mouse_down(7 + 10, 42 + 5);
        assert_eq!(painted(&ctrl), vec!["col.h"]);
        ctrl.mouse_up(7 + 10, 42 + 5);
        ctrl.mouse_move(7 + 150, 42 + 100);
        assert_eq!(painted(&ctrl), vec!["col.n"]);
    }

    #[test]
    fn nc_list_ctrl_10022fc0_sorts_by_the_clicked_column_and_flips_the_direction() {
        let mut ctrl = ListCtrl::from_contract(&CONTRACT);
        for (name, number) in [("b", "10"), ("A", "9"), ("c", "-3"), ("a", "9")] {
            ctrl.add_item(ListRow { cells: vec![ListCell::text(name), ListCell::text(number)] });
        }
        ctrl.select(2);
        let names = |ctrl: &ListCtrl| ctrl.rows.iter().map(|r| r.cells[0].text.clone()).collect::<Vec<_>>();
        ctrl.sort(0);
        assert_eq!(names(&ctrl), vec!["A", "a", "b", "c"]);
        assert_eq!(ctrl.selected(), Some(3));
        ctrl.sort(0);
        assert_eq!(names(&ctrl), vec!["c", "b", "A", "a"]);
        assert_eq!(ctrl.selected(), Some(0));
        ctrl.sort(1);
        assert_eq!(names(&ctrl), vec!["c", "A", "a", "b"]);
        ctrl.sort(1);
        assert_eq!(names(&ctrl), vec!["b", "A", "a", "c"]);
    }

    #[test]
    fn nc_header_ctrl_10021f70_click_on_a_sortable_header_sorts_on_release_inside() {
        let mut ctrl = ListCtrl::from_contract(&CONTRACT);
        for name in ["z", "m", "a"] {
            ctrl.add_item(ListRow { cells: vec![ListCell::text(name), ListCell::text("0")] });
        }
        ctrl.mouse_down(7 + 10, 42 + 5);
        ctrl.mouse_up(7 + 300, 42 + 5);
        assert_eq!(ctrl.rows[0].cells[0].text, "z");
        ctrl.mouse_down(7 + 10, 42 + 5);
        ctrl.mouse_up(7 + 10, 42 + 5);
        assert_eq!(ctrl.rows.iter().map(|r| r.cells[0].text.as_str()).collect::<Vec<_>>(), vec!["a", "m", "z"]);
    }

    #[test]
    fn wtoi_and_wcsicmp_follow_the_crt_rules() {
        assert_eq!((wtoi(" 12x"), wtoi("-7"), wtoi("+5"), wtoi("abc"), wtoi("")), (12, -7, 5, 0, 0));
        assert_eq!(wcsicmp("ABC", "abd"), std::cmp::Ordering::Less);
        assert_eq!(wcsicmp("Zz", "zZ"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn cell_icons_default_their_source_size_and_advance_the_text() {
        let mut ctrl = ListCtrl::from_contract(&CONTRACT);
        let icon = CellIcon { texture: "icon.a".into(), x: 2, y: 1, w: 16, h: 16, u: 0, v: 0, uw: 0, vh: 0 };
        let cell = ListCell { icons: vec![icon], text: "x".into(), ..ListCell::default() };
        ctrl.add_item(ListRow { cells: vec![cell, ListCell::default()] });
        let mut list = DrawList::default();
        ctrl.paint(&mut list, (0, 0), &font());
        let quad = list.quads.iter().find(|q| q.texture == "icon.a").unwrap();
        assert_eq!((quad.x, quad.y, quad.w, quad.h, quad.uw, quad.vh), (9.0, 42.0 + 19.0 + 3.0 + 1.0, 16.0, 13.0, 16.0, 13.0));
        let text = list.quads.iter().find(|q| q.texture == "font").map(|q| q.x);
        assert_eq!(text, Some(7.0 + 16.0 + 10.0));
    }

    fn text(id: i32) -> String {
        format!("s{id}")
    }

    fn cells(count: usize) -> ListRow {
        ListRow { cells: (0..count).map(|i| ListCell::text(&format!("c{i}"))).collect() }
    }

    #[test]
    fn nclistctrl_10023dd0_the_tip_row_needs_the_hovered_row_to_be_the_selected_one() {
        let mut ctrl = filled(3);
        assert!(ctrl.tip_row().is_none());
        ctrl.mouse_move(7 + 5, 42 + 19 + 20);
        assert!(ctrl.tip_row().is_none());
        ctrl.mouse_down(7 + 5, 42 + 19 + 20);
        assert_eq!(ctrl.tip_row().map(|row| row.cells[0].text.as_str()), Some("row1"));
        ctrl.mouse_move(7 + 5, 42 + 19 + 2 * 18 + 3);
        assert!(ctrl.tip_row().is_none());
    }

    #[test]
    fn nctooltipinfo_10031b10_fills_the_cell_styles_and_the_quest_style() {
        for (style, expected) in [
            (ROW_TIP_SEED_SETTING, (7, 8, 9)),
            (ROW_TIP_CROP_SELL, (7, 8, 9)),
            (ROW_TIP_SEED_INFO, (4, 5, 6)),
            (ROW_TIP_CROP_INFO, (5, 6, 7)),
            (ROW_TIP_CROP_SETTING, (9, 10, 11)),
            (ROW_TIP_DEFAULT_INFO, (1, 4, 5)),
        ] {
            let tip = match_row_tooltip(style, &cells(12), "", &text).unwrap();
            let lines: Vec<_> = tip.lines.iter().map(|l| (l.label.as_str(), l.value.clone())).collect();
            assert_eq!(
                lines,
                vec![
                    ("", "c0".to_string()),
                    ("s537", format!("c{}", expected.0)),
                    ("s1134", format!("c{}", expected.1)),
                    ("s1135", format!("c{}", expected.2)),
                ]
            );
            assert!(tip.row);
        }
        let mut row = cells(5);
        row.cells[3].text = "1".into();
        let tip = match_row_tooltip(ROW_TIP_QUEST, &row, "summary", &text).unwrap();
        let lines: Vec<_> = tip.lines.iter().map(|l| (l.label.as_str(), l.value.as_str())).collect();
        assert_eq!(lines, vec![("s1200", "c0"), ("s1201", "c1"), ("s922", "c2"), ("s1202", "s862")]);
        assert_eq!(tip.description, "summary");
        row.cells[3].text = "2".into();
        assert_eq!(match_row_tooltip(ROW_TIP_QUEST, &row, "", &text).unwrap().lines[3].value, "s861");
        row.cells[3].text = "9".into();
        assert_eq!(match_row_tooltip(ROW_TIP_QUEST, &row, "", &text).unwrap().lines[3].value, "c2");
        assert!(match_row_tooltip(0x8000, &row, "", &text).is_none());
    }

    #[test]
    fn nclistctrl_10023000_sorts_stably_with_the_stock_pass_bounds_and_follows_the_selection() {
        let mut ctrl = filled(0);
        for (name, n) in [("b", "3"), ("a", "10"), ("c", "2"), ("a", "1")] {
            ctrl.add_item(ListRow { cells: vec![ListCell::text(name), ListCell::text(n)] });
        }
        ctrl.select(0);
        ctrl.sort(0);
        let names: Vec<_> = ctrl.rows.iter().map(|r| (r.cells[0].text.clone(), r.cells[1].text.clone())).collect();
        assert_eq!(names, vec![("a".into(), "10".into()), ("a".into(), "1".into()), ("b".into(), "3".into()), ("c".into(), "2".into())]);
        assert_eq!(ctrl.selected(), Some(2));
        ctrl.sort(1);
        assert_eq!(ctrl.rows.iter().map(|r| r.cells[1].text.as_str()).collect::<Vec<_>>(), vec!["1", "2", "3", "10"]);
        ctrl.sort(1);
        assert_eq!(ctrl.rows.iter().map(|r| r.cells[1].text.as_str()).collect::<Vec<_>>(), vec!["10", "3", "2", "1"]);
    }
}
