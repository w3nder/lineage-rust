use crate::chat::{InputKey, TextInput};
use crate::color::Argb;
use crate::edit_box::{EditBox, FontMetrics};
use std::cell::RefCell;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::html::{self, Document, Line, WidgetKind};
use crate::window::Rect;

pub const EMPTY_EDIT_SYSMSG: u32 = 1207;
pub const EMPTY_MULTIEDIT_SYSMSG: u32 = 329;
pub const REJECTED_CHARACTER_SYSMSG: u32 = 204;
pub const INVALID_LENGTH_SYSMSG: u32 = 329;
pub const MESSAGE_COLOR: u32 = 0xffb0_9b79;
pub const MAX_BYTES: usize = 3000;
pub const NO_VALUE: &str = "-";
pub const BBS_MISSING_FIELD: &str = "";
pub const EDIT_TEXT: Argb = Argb(0xffdc_dcdc);
pub const EDIT_TEXT_POS: (i32, i32) = (2, 2);
pub const EDIT_CAP: i32 = 8;
pub const EDIT_SOURCE_HEIGHT: i32 = 17;
pub const EDIT_TEXTURES: [&str; 3] = ["L2UI_CH3.Etc.inputbox1", "L2UI_CH3.Etc.inputbox2", "L2UI_CH3.Etc.inputbox3"];
pub const MULTI_CORNER: i32 = 4;

pub fn multi_texture(index: usize) -> String {
    format!("l2ui_ch3.multiedit.M_inputbox0{}", index + 1)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormField {
    pub var: String,
    pub multi: bool,
    pub input: TextInput,
    pub edit: RefCell<EditBox>,
    pub action: String,
    pub length: i32,
}

impl FormField {
    pub fn new(var: &str, multi: bool) -> FormField {
        FormField { var: var.to_string(), multi, input: TextInput::default(), edit: RefCell::new(EditBox::html(Rect::new(0, 0, 0, 0), 0, 0)), action: String::new(), length: 0 }
    }

    pub fn text(&self) -> String {
        if self.multi { self.input.text.clone() } else { self.edit.borrow().get_text().unwrap_or_default() }
    }

    pub fn set_text(&mut self, text: &str) {
        if self.multi {
            self.input.text = text.to_string();
            self.input.cursor = text.chars().count();
        } else {
            self.edit.borrow_mut().set_text(text);
        }
    }

    pub fn clear(&mut self) {
        self.input = TextInput::default();
        self.edit.borrow_mut().clear();
    }
}

pub const COMBO_ROW: i32 = 17;
pub const COMBO_TEXT_X: i32 = 5;
pub const COMBO_ARROW: &str = "L2UI_CH3.Button.downbutton";
pub const COMBO_ARROW_DOWN: &str = "L2UI_CH3.Button.downbutton_down";
pub const COMBO_ARROW_OVER: &str = "L2UI_CH3.Button.downbutton_over";
pub const COMBO_ARROW_SIZE: i32 = 15;
pub const COMBO_SELECT: &str = "L2UI_CH3.ListCtrl.TextSelect";
pub const COMBO_SELECT_END: &str = "L2UI_CH3.ListCtrl.TextSelect2";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormCombo {
    pub var: String,
    pub entries: Vec<String>,
    pub selected: i32,
    pub open: bool,
    pub hover: i32,
}

impl FormCombo {
    pub fn from_list(var: &str, list: &str, selected: &str) -> FormCombo {
        let entries: Vec<String> = list.split(';').filter(|item| !item.is_empty()).map(str::to_string).collect();
        let index = if selected.is_empty() {
            0
        } else {
            entries.iter().position(|entry| entry == selected).map_or(-1, |index| index as i32)
        };
        FormCombo { var: var.to_string(), entries: if list.is_empty() { Vec::new() } else { entries }, selected: if list.is_empty() { -1 } else { index }, open: false, hover: -1 }
    }

    pub fn text(&self) -> Option<&str> {
        usize::try_from(self.selected).ok().and_then(|index| self.entries.get(index)).map(String::as_str)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HtmlForm {
    pub fields: Vec<FormField>,
    pub focus: Option<usize>,
    pub combos: Vec<FormCombo>,
    pub pointer: Option<(i32, i32)>,
    pub arrow_pressed: Option<usize>,
}

pub fn arrow_rect(rect: Rect) -> Rect {
    Rect::new(rect.x + rect.w - COMBO_ARROW_SIZE - 1, rect.y + 1, COMBO_ARROW_SIZE, COMBO_ARROW_SIZE)
}

impl HtmlForm {
    pub fn from_document(doc: &Document) -> HtmlForm {
        let fields = doc.edits().into_iter().map(|(var, multi)| FormField::new(var, multi)).collect();
        HtmlForm { fields, focus: None, combos: Vec::new(), pointer: None, arrow_pressed: None }
    }

    pub fn attach(&mut self, lines: &[Line]) {
        self.combos = lines
            .iter()
            .flat_map(|line| line.widgets.iter())
            .filter_map(|widget| match &widget.kind {
                WidgetKind::ComboBox { var, list, selected } => Some(FormCombo::from_list(var, list, selected)),
                _ => None,
            })
            .collect();
        for widget in lines.iter().flat_map(|line| line.widgets.iter()) {
            if let WidgetKind::Edit { var, action, length, kind, multi } = &widget.kind
                && let Some(field) = self.fields.iter_mut().find(|field| field.var == *var)
            {
                field.action.clone_from(action);
                field.length = *length;
                if !*multi {
                    field.edit.borrow_mut().configure_html(Rect::new(0, 0, widget.width, widget.height), *kind as u32, *length);
                }
            }
        }
    }

    pub fn focused_action(&self) -> Option<&str> {
        let field = self.focus.and_then(|index| self.fields.get(index))?;
        Some(field.action.as_str()).filter(|action| !action.is_empty() || field.multi)
    }

    pub fn fill(&mut self, writer: &str, title: &str, content: &str) {
        let mut next = 0;
        if !writer.is_empty() {
            next = self.append_single(0, writer);
        }
        if !title.is_empty() {
            while next < self.fields.len() {
                next = self.append_single(next, title);
            }
        }
        if !content.is_empty() {
            for field in self.fields.iter_mut().filter(|field| field.multi) {
                field.set_text(content);
            }
        }
    }

    fn append_single(&mut self, from: usize, text: &str) -> usize {
        let Some(index) = self.fields.iter().enumerate().skip(from).find(|(_, field)| !field.multi).map(|(index, _)| index) else {
            return self.fields.len();
        };
        let field = &mut self.fields[index];
        let joined = field.text() + text;
        field.set_text(&joined);
        index + 1
    }

    pub fn focus_var(&mut self, var: &str) {
        self.focus = self.fields.iter().position(|field| field.var == var);
    }

    pub fn sync_focus(&self) {
        for (index, field) in self.fields.iter().enumerate().filter(|(_, field)| !field.multi) {
            let wanted = self.focus == Some(index);
            let mut edit = field.edit.borrow_mut();
            if wanted && !edit.focused {
                edit.focus();
            } else if !wanted && edit.focused {
                edit.kill_focus();
            }
        }
    }

    pub fn tick(&mut self, elapsed_ms: u32) {
        let focus = self.focus;
        for (index, field) in self.fields.iter().enumerate().filter(|(_, field)| !field.multi) {
            if focus == Some(index) {
                field.edit.borrow_mut().tick(elapsed_ms);
            }
        }
    }

    pub fn key(&mut self, key: InputKey, font: &BitmapFont) -> bool {
        self.sync_focus();
        let Some(field) = self.focus.and_then(|index| self.fields.get_mut(index)) else { return false };
        match key {
            InputKey::Enter | InputKey::Escape => {}
            _ if !field.multi => {
                let mut edit = field.edit.borrow_mut();
                edit.input_key(key, &FontMetrics(font));
                edit.notices.clear();
            }
            InputKey::Char(_) if field.length > 0 && field.input.text.chars().count() >= field.length as usize => {}
            other => {
                field.input.key(other);
            }
        }
        true
    }

    pub fn value(&self, var: &str, font: &BitmapFont) -> Result<String, Option<u32>> {
        let (text, multi) = if let Some(combo) = self.combos.iter().find(|combo| combo.var == var) {
            (combo.text().map(str::to_string), false)
        } else {
            let field = self.fields.iter().find(|field| field.var == var).ok_or(None)?;
            (Some(field.text()).filter(|text| !text.is_empty()), field.multi)
        };
        let Some(text) = text else {
            return Err(Some(if multi { EMPTY_MULTIEDIT_SYSMSG } else { EMPTY_EDIT_SYSMSG }));
        };
        if !multi && text.chars().any(|c| font.glyph(c).is_none()) {
            return Err(Some(REJECTED_CHARACTER_SYSMSG));
        }
        let bytes = text.chars().count();
        if bytes == 0 || bytes > MAX_BYTES || text.chars().all(|c| c == ' ') {
            return Err(Some(INVALID_LENGTH_SYSMSG));
        }
        Ok(text)
    }
}

fn tokens(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = text.chars().peekable();
    loop {
        while chars.peek().is_some_and(|c| c.is_whitespace()) {
            chars.next();
        }
        let Some(&first) = chars.peek() else { break };
        let mut token = String::new();
        if first == '"' {
            chars.next();
            for c in chars.by_ref() {
                if c == '"' {
                    break;
                }
                token.push(c);
            }
        } else {
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    break;
                }
                token.push(c);
                chars.next();
            }
        }
        out.push(token);
    }
    out
}

pub fn write_fields(action: &str, form: &HtmlForm, font: &BitmapFont) -> Option<Result<Box<[String; 6]>, Option<u32>>> {
    let words = tokens(action);
    if !words.first().is_some_and(|word| word.eq_ignore_ascii_case("write")) {
        return None;
    }
    let token = |index: usize| words.get(index).cloned();
    let result = (|| {
        let (Some(t1), Some(t2), Some(t3), Some(t4)) = (token(1), token(2), token(3), token(4)) else { return Err(None) };
        let fourth = form.value(&t4, font)?;
        let fifth = if ["4", "5", "7"].contains(&t1.as_str()) { NO_VALUE.to_string() } else { form.value(&token(5).ok_or(None)?, font)? };
        let sixth = if ["4", "7"].contains(&t1.as_str()) {
            NO_VALUE.to_string()
        } else {
            match token(6) {
                Some(var) => form.value(&var, font)?,
                None => BBS_MISSING_FIELD.to_string(),
            }
        };
        Ok(Box::new([t1, t2, t3, fourth, fifth, sixth]))
    })();
    Some(result)
}

pub fn expand_action(action: &str, form: &HtmlForm, font: &BitmapFont) -> (String, Vec<u32>) {
    let (message, command) = html::split_message(action);
    let mut words = command.split_whitespace();
    if !words.next().is_some_and(|word| word.eq_ignore_ascii_case("bypass")) || !command.contains('$') {
        return (action.to_string(), Vec::new());
    }
    let mut messages = Vec::new();
    let mut out = vec!["bypass".to_string()];
    let mut closes = false;
    for word in words {
        if !closes && word == "-h" {
            closes = true;
            out.push(word.to_string());
        } else if let Some(var) = word.strip_prefix('$') {
            match form.value(var, font) {
                Ok(value) => out.push(value),
                Err(Some(id)) => messages.push(id),
                Err(None) => {}
            }
        } else {
            out.push(word.to_string());
        }
    }
    let text = out.join(" ");
    match message {
        Some(message) => (format!("{message}{}{text}", html::MESSAGE_SEPARATOR), messages),
        None => (text, messages),
    }
}

pub fn paint_form(list: &mut DrawList, (cx, cy): (i32, i32), size: (i32, i32), lines: &[Line], scroll: i32, form: &HtmlForm, font: &BitmapFont) {
    form.sync_focus();
    let content = Rect::new(0, 0, size.0, size.1);
    let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
    for line in lines {
        let top = line.y - scroll;
        for widget in &line.widgets {
            let WidgetKind::Edit { var, multi, kind, .. } = &widget.kind else { continue };
            if top > content.h || top + widget.height < 0 || widget.x >= content.w {
                continue;
            }
            let (x, y, w, h) = (cx + widget.x, cy + top, widget.width, widget.height);
            let Some((index, field)) = form.fields.iter().enumerate().find(|(_, field)| field.var == *var) else {
                paint_background(&mut body, Rect::new(x, y, w, h), *multi, !*multi);
                continue;
            };
            let focused = form.focus == Some(index);
            let step = html::line_step(font);
            if !*multi {
                let mut edit = field.edit.borrow_mut();
                edit.configure_html(Rect::new(0, 0, w, h), *kind as u32, field.length);
                edit.paint_html(&mut body, (x, y), Rect::new(x, cy, w, size.1), &FontMetrics(font), font);
                continue;
            }
            paint_background(&mut body, Rect::new(x, y, w, h), *multi, *multi || focused);
            let mut text = field.input.text.clone();
            if focused {
                text.push('|');
            }
            for (row, part) in text.split('\n').enumerate() {
                let ty = y + EDIT_TEXT_POS.1 + row as i32 * step;
                if ty + step > y + h {
                    break;
                }
                body.draw_text(x + EDIT_TEXT_POS.0, ty, EDIT_TEXT, part, font);
            }
        }
    }
    list.append_clipped(body, Rect::new(cx, cy, size.0, size.1));
}

pub fn combo_rect(line_y: i32, widget: &crate::html::Widget, scroll: i32) -> Rect {
    Rect::new(widget.x, line_y - scroll, widget.width, COMBO_ROW)
}

impl HtmlForm {
    pub fn close_combos(&mut self) {
        self.combos.iter_mut().for_each(|combo| {
            combo.open = false;
            combo.hover = -1;
        });
    }

    pub fn combo_rects(&self, lines: &[Line], scroll: i32) -> Vec<Rect> {
        lines
            .iter()
            .flat_map(|line| line.widgets.iter().filter(|widget| matches!(widget.kind, WidgetKind::ComboBox { .. })).map(move |widget| combo_rect(line.y, widget, scroll)))
            .collect()
    }

    pub fn arrow_face(&self, index: usize, rect: Rect) -> &'static str {
        let inside = self.pointer.is_some_and(|(x, y)| arrow_rect(rect).contains(x, y));
        match (self.arrow_pressed == Some(index), inside) {
            (true, true) => COMBO_ARROW_DOWN,
            (true, false) => COMBO_ARROW,
            (false, true) => COMBO_ARROW_OVER,
            (false, false) => COMBO_ARROW,
        }
    }

    pub fn combo_release(&mut self, lines: &[Line], scroll: i32, x: i32, y: i32) -> bool {
        let Some(pressed) = self.arrow_pressed.take() else { return false };
        let rects = self.combo_rects(lines, scroll);
        if rects.get(pressed).is_some_and(|rect| arrow_rect(*rect).contains(x, y)) && self.combos.get(pressed).is_some_and(|combo| !combo.entries.is_empty()) {
            self.close_combos();
            self.combos[pressed].open = true;
        }
        true
    }

    pub fn combo_move(&mut self, lines: &[Line], scroll: i32, x: i32, y: i32) {
        self.pointer = Some((x, y));
        let mut index = 0;
        for line in lines {
            for widget in &line.widgets {
                if !matches!(widget.kind, WidgetKind::ComboBox { .. }) {
                    continue;
                }
                let rect = combo_rect(line.y, widget, scroll);
                if let Some(combo) = self.combos.get_mut(index).filter(|combo| combo.open) {
                    let row = (y - rect.y - COMBO_ROW).div_euclid(COMBO_ROW);
                    combo.hover = if x > rect.x && x < rect.x + rect.w && y >= rect.y + COMBO_ROW && row < combo.entries.len() as i32 { row } else { -1 };
                }
                index += 1;
            }
        }
    }

    pub fn combo_press(&mut self, lines: &[Line], scroll: i32, x: i32, y: i32) -> bool {
        let mut index = 0;
        let mut handled = false;
        let mut chosen = None;
        self.arrow_pressed = None;
        for (arrow, rect) in self.combo_rects(lines, scroll).into_iter().enumerate() {
            if arrow_rect(rect).contains(x, y) && self.combos.get(arrow).is_some() {
                self.arrow_pressed = Some(arrow);
            }
        }
        if self.arrow_pressed.is_some() {
            if self.combos.iter().all(|combo| !combo.open) {
                return true;
            }
            if self.combos.iter().enumerate().any(|(index, combo)| combo.open && self.arrow_pressed == Some(index)) {
                return true;
            }
        }
        for line in lines {
            for widget in &line.widgets {
                if !matches!(widget.kind, WidgetKind::ComboBox { .. }) {
                    continue;
                }
                let rect = combo_rect(line.y, widget, scroll);
                if let Some(combo) = self.combos.get(index) {
                    if combo.open {
                        handled = true;
                        let row = (y - rect.y - COMBO_ROW).div_euclid(COMBO_ROW);
                        if x > rect.x && x < rect.x + rect.w && y >= rect.y + COMBO_ROW && row < combo.entries.len() as i32 {
                            chosen = Some((index, row));
                        }
                    } else if rect.contains(x, y) {
                        chosen = Some((index, -1));
                        handled = true;
                    }
                }
                index += 1;
            }
        }
        match chosen {
            Some((open, -1)) => {
                self.close_combos();
                if let Some(combo) = self.combos.get_mut(open).filter(|combo| !combo.entries.is_empty()) {
                    combo.open = true;
                }
            }
            Some((picked, row)) => {
                self.combos[picked].selected = row;
                self.close_combos();
            }
            None => self.close_combos(),
        }
        handled
    }

    pub fn paint_combos(&self, list: &mut DrawList, (cx, cy): (i32, i32), size: (i32, i32), lines: &[Line], scroll: i32, font: &BitmapFont) {
        let mut index = 0;
        let mut open = Vec::new();
        let mut closed = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        let text_offset = 9 - font.line_height as i32 / 2;
        for line in lines {
            for widget in &line.widgets {
                if !matches!(widget.kind, WidgetKind::ComboBox { .. }) {
                    continue;
                }
                let combo = self.combos.get(index);
                index += 1;
                let rect = combo_rect(line.y, widget, scroll);
                if rect.y > size.1 || rect.y + COMBO_ROW < 0 || rect.x >= size.0 {
                    continue;
                }
                let (x, y) = (cx + rect.x, cy + rect.y);
                if let Some(combo) = combo.filter(|combo| combo.open) {
                    open.push((x, y, rect.w, combo, self.arrow_face(index - 1, rect)));
                    continue;
                }
                paint_three(&mut closed, Rect::new(x, y, rect.w, COMBO_ROW), true);
                if let Some(text) = combo.and_then(FormCombo::text) {
                    closed.draw_text(x + COMBO_TEXT_X, y + text_offset, EDIT_TEXT, text, font);
                }
                closed.draw_texture(x + rect.w - 16, y + 1, 15, 15, 0, 0, 15, 15, self.arrow_face(index - 1, rect), 1.0);
            }
        }
        list.append_clipped(closed, Rect::new(cx, cy, size.0, size.1));
        for (x, y, w, combo, arrow) in open {
            paint_three(list, Rect::new(x, y, w, COMBO_ROW), true);
            if let Some(text) = combo.text() {
                list.draw_text(x + COMBO_TEXT_X, y + text_offset, EDIT_TEXT, text, font);
            }
            for row in 0..=combo.entries.len() as i32 {
                paint_three(list, Rect::new(x, y + COMBO_ROW * row, w, COMBO_ROW), true);
            }
            for (row, entry) in combo.entries.iter().enumerate() {
                let row_y = y + COMBO_ROW * (row as i32 + 1);
                if combo.hover == row as i32 {
                    list.draw_texture(x + 1, row_y, w - 32, 16, 0, 0, 16, 13, COMBO_SELECT, 1.0);
                    list.draw_texture(x + w - 31, row_y, 32, 16, 0, 0, 32, 13, COMBO_SELECT_END, 1.0);
                }
                list.draw_text(x + COMBO_TEXT_X, row_y + text_offset, EDIT_TEXT, entry, font);
            }
            list.draw_texture(x + w - 16, y + 1, 15, 15, 0, 0, 15, 15, arrow, 1.0);
        }
    }
}

pub fn paint_background(list: &mut DrawList, rect: Rect, multi: bool, enabled: bool) {
    if multi {
        paint_nine(list, rect, enabled);
    } else {
        paint_three(list, rect, enabled);
    }
}

fn paint_three(list: &mut DrawList, Rect { x, y, w, h }: Rect, enabled: bool) {
    for (index, (px, width)) in [(x, EDIT_CAP), (x + EDIT_CAP, w - 2 * EDIT_CAP), (x + w - EDIT_CAP, EDIT_CAP)].into_iter().enumerate() {
        let texture = if enabled { EDIT_TEXTURES[index].to_string() } else { format!("{}_disable", EDIT_TEXTURES[index]) };
        list.draw_texture(px, y, width, h, 0, 0, EDIT_CAP, EDIT_SOURCE_HEIGHT, &texture, 1.0);
    }
}

fn paint_nine(list: &mut DrawList, Rect { x, y, w, h }: Rect, enabled: bool) {
    let c = MULTI_CORNER;
    let (mw, mh) = (w - 2 * c, h - 2 * c);
    let columns = [(x, c), (x + c, mw), (x + c + mw, c)];
    let rows = [(y, c), (y + c, mh), (y + c + mh, c)];
    for (row, (py, ph)) in rows.into_iter().enumerate() {
        for (column, (px, pw)) in columns.into_iter().enumerate() {
            let mut texture = multi_texture(row * 3 + column);
            if !enabled {
                texture.push_str("_disable");
            }
            list.draw_texture(px, py, pw, ph, 0, 0, c, c, &texture, 1.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn form() -> HtmlForm {
        let doc = html::parse("<html><body><edit var=\"Name\" width=100 height=15><edit var=\"Title\" width=300 height=15><multiedit var=\"Content\" width=400 height=100></body></html>");
        HtmlForm::from_document(&doc)
    }

    fn typed(form: &mut HtmlForm, var: &str, text: &str) {
        form.focus_var(var);
        for c in text.chars() {
            form.key(InputKey::Char(c), &font());
        }
    }

    #[test]
    fn edits_are_collected_in_document_order() {
        let form = form();
        assert_eq!(form.fields.iter().map(|f| (f.var.as_str(), f.multi)).collect::<Vec<_>>(), [("Name", false), ("Title", false), ("Content", true)]);
    }

    #[test]
    fn value_1003c5e0_reports_the_original_messages() {
        let mut form = form();
        let font = font();
        assert_eq!(form.value("Missing", &font), Err(None));
        assert_eq!(form.value("Title", &font), Err(Some(EMPTY_EDIT_SYSMSG)));
        assert_eq!(form.value("Content", &font), Err(Some(EMPTY_MULTIEDIT_SYSMSG)));
        typed(&mut form, "Title", "   ");
        assert_eq!(form.value("Title", &font), Err(Some(INVALID_LENGTH_SYSMSG)));
        typed(&mut form, "Name", "\u{7f}");
        assert_eq!(form.value("Name", &font), Err(Some(REJECTED_CHARACTER_SYSMSG)));
        typed(&mut form, "Content", "é ok");
        assert_eq!(form.value("Content", &font), Ok("é ok".into()));
        assert_eq!(form.value("title", &font), Err(None));
    }

    #[test]
    fn write_1005cc20_builds_the_six_fields_like_the_probe() {
        let mut form = form();
        let font = font();
        typed(&mut form, "Title", "Ola");
        typed(&mut form, "Content", "Texto");
        let fields = |action: &str| write_fields(action, &form, &font);
        assert_eq!(fields("bypass _bbshome"), None);
        assert_eq!(fields("Write Notice Set _ Title Content Content"), Some(Ok(Box::new(["Notice", "Set", "_", "Ola", "Texto", "Texto"].map(String::from)))));
        assert_eq!(fields("write 4 1 2 Title Content Content"), Some(Ok(Box::new(["4", "1", "2", "Ola", "-", "-"].map(String::from)))));
        assert_eq!(fields("Write 7 1 2 Title"), Some(Ok(Box::new(["7", "1", "2", "Ola", "-", "-"].map(String::from)))));
        assert_eq!(fields("Write 1 2 3 Title Content"), Some(Ok(Box::new(["1", "2", "3", "Ola", "Texto", ""].map(String::from)))));
        assert_eq!(fields("Write 1 2 3 Name Content Content"), Some(Err(Some(EMPTY_EDIT_SYSMSG))));
        assert_eq!(fields("Write 1 2"), Some(Err(None)));
    }

    #[test]
    fn mode_3_fill_10041de0_puts_writer_title_and_content_in_order() {
        let mut form = form();
        form.fill("Autor", "Titulo", "Corpo");
        assert_eq!(form.fields.iter().map(|f| f.text()).collect::<Vec<_>>(), ["Autor", "Titulo", "Corpo"]);
        let mut only_title = super::HtmlForm::from_document(&html::parse("<edit var=\"Title\" width=10 height=10>"));
        only_title.fill("", "Titulo", "");
        assert_eq!(only_title.fields[0].text(), "Titulo");
    }

    #[test]
    fn a_single_line_edit_scrolls_horizontally_and_keeps_the_caret_blinking_while_focused() {
        let doc = html::parse("<html><body><edit var=\"Name\" width=100 height=15 length=40></body></html>");
        let lines = html::layout(&doc, 300, &font());
        let mut form = HtmlForm::from_document(&doc);
        form.attach(&lines);
        form.focus_var("Name");
        for c in "abcdefghijklmnopqrst".chars() {
            form.key(InputKey::Char(c), &font());
        }
        assert!(form.fields[0].edit.borrow().scroll() > 0);
        assert_eq!(form.fields[0].text(), "abcdefghijklmnopqrst");
        form.key(InputKey::Home, &font());
        assert_eq!(form.fields[0].edit.borrow().right().len(), 20);
        let blink = form.fields[0].edit.borrow().caret_blink;
        form.tick(crate::edit_box::BLINK_PERIOD_MS);
        assert_ne!(form.fields[0].edit.borrow().caret_blink, blink);
        form.focus = None;
        form.sync_focus();
        assert!(!form.fields[0].edit.borrow().focused);
        assert_eq!(form.fields[0].edit.borrow().right().len(), 0);
    }

    #[test]
    fn fill_1003f9e0_adds_the_writer_to_the_first_edit_and_the_title_to_all_the_ones_after_it() {
        let doc = html::parse("<html><body><edit var=\"a\" width=100 height=15><edit var=\"b\" width=100 height=15><edit var=\"c\" width=100 height=15><multiedit var=\"m\" width=400 height=100><multiedit var=\"n\" width=400 height=100></body></html>");
        let mut form = HtmlForm::from_document(&doc);
        form.fill("W", "T", "C");
        let texts: Vec<_> = form.fields.iter().map(|f| f.text()).collect();
        assert_eq!(texts, ["W", "T", "T", "C", "C"]);
        let mut form = HtmlForm::from_document(&doc);
        form.fill("", "T", "");
        let texts: Vec<_> = form.fields.iter().map(|f| f.text()).collect();
        assert_eq!(texts, ["T", "T", "T", "", ""]);
    }
}
