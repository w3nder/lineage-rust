use crate::color::Argb;
use crate::font::BitmapFont;
use crate::html_tokens::{
    KIND_A, KIND_BODY, KIND_BR, KIND_BR1, KIND_BUTTON, KIND_CENTER, KIND_COMBOBOX, KIND_EDIT, KIND_HTML, KIND_IMG, KIND_MULTIEDIT,
    KIND_TABLE, KIND_TD, KIND_TEXT, KIND_TR, Token,
};
use crate::text_align;

const LANGUAGE: i32 = 1;
const LINE_GAP: i32 = 6;
const BREAK_HEIGHT: i32 = 9;
const TEXT_COLOR: Argb = Argb(0xFFDCDCDC);
const LINK_COLOR: Argb = Argb(0xFF6699FF);
const LEVEL_COLOR: Argb = Argb(0xFFFFCC00);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LeafKind {
    Text,
    Image { src: String },
    Break,
    BreakNoSpace,
    Button { value: String, action: String, back: String, fore: String },
    MultiEdit { var: String },
    Edit { var: String, action: String, kind: i32, length: i32 },
    ComboBox { var: String, list: String, selected: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leaf {
    pub kind: LeafKind,
    pub is_text: bool,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub min_w: i32,
    pub wrap_w: i32,
    pub align: i32,
    pub start: i32,
    pub end: i32,
    pub subtype: i32,
    pub text: String,
    pub colors: String,
    pub action: String,
    pub message: String,
}

impl Leaf {
    fn new(kind: LeafKind) -> Leaf {
        Leaf { kind, is_text: false, x: 0, y: 0, w: 0, h: 0, min_w: 0, wrap_w: 0, align: 0, start: 0, end: 0, subtype: 0, text: String::new(), colors: String::new(), action: String::new(), message: String::new() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    pub kind: u32,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub min_w: i32,
    pub align: i32,
    pub valign: i32,
    pub y_offset: i32,
    pub border_flag: bool,
    pub width_attr: i32,
    pub percent: bool,
    pub fix_width: i32,
    pub children: Vec<Child>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Child {
    Leaf(Box<Leaf>),
    Container(Box<Container>),
    Table(Box<Table>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub min_w: i32,
    pub align: i32,
    pub padding: i32,
    pub spacing: i32,
    pub border: i32,
    pub width_attr: i32,
    pub percent: bool,
    pub background: Option<u32>,
    pub rows: Vec<Vec<Container>>,
    pub column_widths: Vec<i32>,
    pub column_minimums: Vec<i32>,
    pub column_positions: Vec<i32>,
    pub row_heights: Vec<i32>,
}

struct Builder<'a> {
    tokens: &'a [Token],
    position: usize,
    font: &'a BitmapFont,
    align_stack: Vec<i32>,
    valign_stack: Vec<i32>,
    subtype: i32,
    link_action: String,
    link_message: String,
}

fn attribute_int(token: &Token, id: usize) -> Option<i32> {
    token.attribute(id).map(crate::html::atoi_text)
}

impl<'a> Builder<'a> {
    fn top(stack: &[i32]) -> i32 {
        stack.last().copied().unwrap_or(0)
    }

    fn glyph_a(&self) -> (i32, i32) {
        self.font.glyph('A').map_or((0, 0), |glyph| (glyph.width as i32, glyph.height as i32))
    }

    fn container(&mut self, kind: u32, border_flag: bool) -> Container {
        let align = Self::top(&self.align_stack);
        let valign = Self::top(&self.valign_stack);
        let mut container = Container {
            kind,
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            min_w: 0,
            align,
            valign,
            y_offset: 0,
            border_flag,
            width_attr: 0,
            percent: false,
            fix_width: -1,
            children: Vec::new(),
        };
        if kind == KIND_TD {
            self.cell_attributes(&mut container);
        }
        while let Some(token) = self.tokens.get(self.position) {
            if token.kind == kind && !token.open {
                if kind == KIND_TD {
                    if container.fix_width > 0 {
                        container.w = container.fix_width;
                    }
                    self.align_stack.pop();
                    self.position += 1;
                }
                break;
            }
            let leaf = match token.kind {
                KIND_TEXT if token.text.is_empty() => {
                    self.position += 1;
                    continue;
                }
                KIND_TEXT => self.text_leaf(token),
                KIND_IMG => {
                    self.subtype = 7;
                    self.simple_leaf(token)
                }
                KIND_A => {
                    self.anchor(token);
                    self.position += 1;
                    continue;
                }
                KIND_TABLE if !token.open => {
                    self.position += 1;
                    continue;
                }
                KIND_TABLE => {
                    let table = self.table();
                    let (width, min_width) = (table.w, table.min_w);
                    if container.w < width {
                        container.w = width;
                    }
                    if container.min_w < min_width {
                        container.min_w = min_width;
                    }
                    container.children.push(Child::Table(Box::new(table)));
                    continue;
                }
                KIND_BR | KIND_BR1 => self.simple_leaf(token),
                KIND_CENTER => {
                    if token.open {
                        self.align_stack.push(1);
                    } else {
                        self.align_stack.pop();
                    }
                    self.position += 1;
                    continue;
                }
                KIND_BUTTON => {
                    self.subtype = 3;
                    self.simple_leaf(token)
                }
                KIND_MULTIEDIT => {
                    self.subtype = 4;
                    self.simple_leaf(token)
                }
                KIND_EDIT => {
                    self.subtype = 5;
                    self.simple_leaf(token)
                }
                KIND_COMBOBOX => {
                    self.subtype = 8;
                    self.simple_leaf(token)
                }
                _ => {
                    self.position += 1;
                    continue;
                }
            };
            let (width, min_width) = (leaf.w, leaf.min_w);
            self.subtype = 0;
            if container.w < width {
                container.w = width;
            }
            if container.min_w < min_width {
                container.min_w = min_width;
            }
            container.children.push(Child::Leaf(Box::new(leaf)));
        }
        container
    }

    fn table(&mut self) -> Table {
        let token = &self.tokens[self.position];
        let mut table = Table {
            x: 0,
            y: 0,
            w: 0,
            h: 0,
            min_w: 0,
            align: Self::top(&self.align_stack),
            padding: 1,
            spacing: 2,
            border: 0,
            width_attr: 0,
            percent: false,
            background: None,
            rows: Vec::new(),
            column_widths: Vec::new(),
            column_minimums: Vec::new(),
            column_positions: Vec::new(),
            row_heights: Vec::new(),
        };
        if let Some(border) = attribute_int(token, 4) {
            table.border = border;
        }
        if let Some(value) = token.attribute(11) {
            table.width_attr = crate::html::atoi_text(value);
            table.percent = value.ends_with('%');
        }
        if let Some(height) = attribute_int(token, 10) {
            table.h = height;
        }
        if let Some(color) = token.attribute(3) {
            table.background = Some(crate::html::font_color(color).0);
        }
        if let Some(padding) = attribute_int(token, 8) {
            table.padding = padding;
        }
        if let Some(spacing) = attribute_int(token, 9) {
            table.spacing = spacing;
        }
        let mut column = 0usize;
        let mut have_row = false;
        while let Some(token) = self.tokens.get(self.position) {
            if token.kind == KIND_TABLE && !token.open {
                self.position += 1;
                break;
            }
            match token.kind {
                KIND_TR => {
                    if token.open {
                        table.rows.push(Vec::new());
                        have_row = true;
                    }
                    column = 0;
                    self.position += 1;
                }
                KIND_TD if have_row => {
                    let cell = self.container(KIND_TD, table.border > 0);
                    let width = cell.w;
                    let minimum = cell.min_w;
                    if let Some(row) = table.rows.last_mut() {
                        row.push(cell);
                    }
                    if column < table.column_widths.len() {
                        if table.column_widths[column] < width {
                            table.column_widths[column] = width;
                        }
                        if table.column_minimums[column] < minimum {
                            table.column_minimums[column] = minimum;
                        }
                    } else {
                        table.column_widths.push(width - table.padding * 2);
                        table.column_minimums.push(minimum - table.padding * 2);
                    }
                    column += 1;
                }
                _ => self.position += 1,
            }
        }
        for index in 0..table.column_widths.len() {
            table.w += table.column_widths[index] + table.spacing + (table.border + table.padding) * 2;
            table.min_w += table.column_minimums[index];
        }
        let edge = table.spacing + table.border * 2;
        table.w += edge;
        table.min_w += edge;
        table
    }

    fn cell_attributes(&mut self, container: &mut Container) {
        let Some(token) = self.tokens.get(self.position) else { return };
        match token.attribute(1) {
            None => self.align_stack.push(0),
            Some(value) => {
                let align = if value.eq_ignore_ascii_case("CENTER") {
                    1
                } else if value.eq_ignore_ascii_case("RIGHT") {
                    3
                } else if value.eq_ignore_ascii_case("LEFT") {
                    2
                } else {
                    0
                };
                self.align_stack.push(align);
            }
        }
        container.valign = match token.attribute(7) {
            None => 1,
            Some(value) if value.eq_ignore_ascii_case("TOP") => 2,
            Some(value) if value.eq_ignore_ascii_case("BOTTOM") => 3,
            Some(value) => i32::from(value.eq_ignore_ascii_case("CENTER")),
        };
        if let Some(value) = token.attribute(8) {
            container.width_attr = crate::html::atoi_text(value);
            container.percent = value.ends_with('%');
        }
        if let Some(height) = attribute_int(token, 6) {
            container.h = height;
        }
        container.fix_width = attribute_int(token, 5).unwrap_or(-1);
        if !container.percent && container.width_attr > 0 {
            container.w = container.width_attr;
        }
    }

    fn anchor(&mut self, token: &Token) {
        if !token.open {
            self.subtype = 0;
            self.link_action.clear();
            self.link_message.clear();
            return;
        }
        if let Some(path) = token.attribute(4) {
            self.subtype = 1;
            self.link_action = path.to_string();
        } else if let Some(action) = token.attribute(1) {
            self.subtype = 2;
            self.link_action = action.to_string();
            self.link_message = token.attribute(5).unwrap_or_default().to_string();
        } else if let Some(command) = token.attribute(2) {
            self.subtype = 9;
            self.link_action = command.to_string();
        } else {
            self.subtype = 0;
            self.link_action.clear();
        }
        if self.subtype != 2 {
            self.link_message.clear();
        }
    }

    fn text_leaf(&mut self, token: &Token) -> Leaf {
        let (a_width, _) = self.glyph_a();
        let mut leaf = Leaf::new(LeafKind::Text);
        leaf.is_text = true;
        leaf.text = token.text.clone();
        leaf.colors = token.colors.clone();
        leaf.subtype = self.subtype;
        if matches!(self.subtype, 1 | 2 | 9) {
            leaf.action = self.link_action.clone();
        }
        if self.subtype == 2 {
            leaf.message = self.link_message.clone();
        }
        leaf.align = Self::top(&self.align_stack);
        let extent = text_align::extent(&leaf.text, 0, self.font, LANGUAGE, 0, false);
        leaf.w = extent.width;
        leaf.h = extent.height + LINE_GAP;
        leaf.min_w = a_width;
        self.position += 1;
        leaf
    }

    fn simple_leaf(&mut self, token: &Token) -> Leaf {
        let mut leaf;
        match token.kind {
            KIND_IMG => {
                leaf = Leaf::new(LeafKind::Image { src: token.attribute(2).unwrap_or_default().to_string() });
                let width = attribute_int(token, 4).unwrap_or(0);
                leaf.w = width;
                leaf.min_w = width;
                leaf.h = attribute_int(token, 1).unwrap_or(0);
            }
            KIND_BR => {
                leaf = Leaf::new(LeafKind::Break);
                leaf.w = 1;
                leaf.min_w = 1;
                leaf.h = BREAK_HEIGHT;
            }
            KIND_BR1 => {
                leaf = Leaf::new(LeafKind::BreakNoSpace);
                leaf.w = 1;
                leaf.min_w = 1;
                leaf.h = 0;
            }
            KIND_BUTTON => {
                let width = attribute_int(token, 6).unwrap_or(10);
                let height = attribute_int(token, 4).unwrap_or(10);
                leaf = Leaf::new(LeafKind::Button {
                    value: token.attribute(5).unwrap_or_default().to_string(),
                    action: token.attribute(1).unwrap_or_default().to_string(),
                    back: token.attribute(2).unwrap_or_default().to_string(),
                    fore: token.attribute(3).unwrap_or_default().to_string(),
                });
                leaf.w = width;
                leaf.min_w = width;
                leaf.h = height;
            }
            KIND_MULTIEDIT => {
                let width = attribute_int(token, 3).unwrap_or(50);
                let height = attribute_int(token, 1).unwrap_or(40);
                leaf = Leaf::new(LeafKind::MultiEdit { var: token.attribute(2).unwrap_or_default().to_string() });
                leaf.w = width;
                leaf.min_w = width;
                leaf.h = height + LINE_GAP;
            }
            KIND_EDIT => {
                let width = attribute_int(token, 6).unwrap_or(50);
                let height = attribute_int(token, 2).unwrap_or(15);
                leaf = Leaf::new(LeafKind::Edit {
                    var: token.attribute(5).unwrap_or_default().to_string(),
                    action: token.attribute(1).unwrap_or_default().to_string(),
                    kind: edit_kind(token.attribute(4)),
                    length: attribute_int(token, 3).unwrap_or(0),
                });
                leaf.w = width;
                leaf.min_w = width;
                leaf.h = height + LINE_GAP;
            }
            _ => {
                let width = attribute_int(token, 4).unwrap_or(30);
                leaf = Leaf::new(LeafKind::ComboBox {
                    var: token.attribute(3).unwrap_or_default().to_string(),
                    list: token.attribute(1).unwrap_or_default().to_string(),
                    selected: token.attribute(2).unwrap_or_default().to_string(),
                });
                leaf.w = width;
                leaf.min_w = width;
                leaf.h = 21;
            }
        }
        leaf.subtype = self.subtype;
        leaf.align = Self::top(&self.align_stack);
        self.position += 1;
        leaf
    }
}

fn edit_kind(value: Option<&str>) -> i32 {
    match value {
        None => 0,
        Some("normal") => 0,
        Some("password") => 2,
        Some("date") => 4,
        Some("time") => 5,
        Some("number") => 6,
        Some(_) => 3,
    }
}

pub struct Tree {
    pub root: Container,
}

pub fn build(tokens: &[Token], font: &BitmapFont) -> Option<Tree> {
    let start = tokens.iter().position(|token| token.kind == KIND_HTML || token.kind == KIND_BODY)?;
    let mut builder = Builder {
        tokens,
        position: start,
        font,
        align_stack: vec![0],
        valign_stack: vec![0],
        subtype: 0,
        link_action: String::new(),
        link_message: String::new(),
    };
    let kind = tokens[start].kind;
    builder.position += 1;
    let root = builder.container(kind, false);
    Some(Tree { root })
}

pub fn glyph_a(font: &BitmapFont) -> (i32, i32) {
    font.glyph('A').map_or((0, 0), |glyph| (glyph.width as i32, glyph.height as i32))
}

fn layout_leaf(leaf: &mut Leaf, width: i32, font: &BitmapFont) {
    if !leaf.is_text {
        return;
    }
    let width = width.max(leaf.min_w);
    if leaf.align == 1 {
        leaf.start = 0;
    }
    let extent = text_align::extent(&leaf.text, width, font, LANGUAGE, leaf.start, true);
    leaf.w = extent.width;
    leaf.wrap_w = width;
    leaf.end = extent.end_x;
    leaf.h = extent.height + LINE_GAP;
    if matches!(leaf.subtype, 1 | 2 | 9) {
        let _ = leaf.end;
    }
}

fn layout_container(container: &mut Container, width: i32, font: &BitmapFont) {
    let (_, a_height) = glyph_a(font);
    let width = width.max(container.min_w);
    container.w = width;
    let count = container.children.len();
    let mut y = 0;
    let mut last = 0;
    for index in 0..count {
        let next_is_text = index + 1 < count && child_is_text(&container.children[index + 1]);
        let next_centered = index + 1 < count && child_align(&container.children[index + 1]) == 1;
        let is_text = child_is_text(&container.children[index]);
        let centered = child_align(&container.children[index]) == 1;
        let mut previous_centered = false;
        if index > 0 && is_text {
            let (previous_align, previous_end) = match &container.children[index - 1] {
                Child::Leaf(leaf) => (leaf.align, leaf.end),
                Child::Container(inner) => (inner.align, 0),
                Child::Table(table) => (table.align, 0),
            };
            previous_centered = previous_align == 1;
            if let Child::Leaf(leaf) = &mut container.children[index] {
                leaf.start = previous_end;
                if centered || previous_centered {
                    leaf.start = 0;
                }
            }
        }
        match &mut container.children[index] {
            Child::Leaf(leaf) => layout_leaf(leaf, width, font),
            Child::Container(inner) => layout_container(inner, width, font),
            Child::Table(table) => layout_table(table, width, font),
        }
        let child = &mut container.children[index];
        if !is_text || previous_centered {
            set_position(child, 0, y);
        } else {
            let start = match child {
                Child::Leaf(leaf) => leaf.start,
                _ => 0,
            };
            set_position(child, start, y);
        }
        let height = child_height(child);
        last = if !is_text || !next_is_text || next_centered || centered {
            y + height
        } else {
            y - LINE_GAP + (height - a_height)
        };
        if container.w < child_width(child) {
            container.w = child_width(child);
        }
        y = last;
    }
    if container.h < last {
        container.h = last;
    }
}

fn child_is_text(child: &Child) -> bool {
    matches!(child, Child::Leaf(leaf) if leaf.is_text)
}

fn child_align(child: &Child) -> i32 {
    match child {
        Child::Leaf(leaf) => leaf.align,
        Child::Container(inner) => inner.align,
        Child::Table(table) => table.align,
    }
}

fn child_height(child: &Child) -> i32 {
    match child {
        Child::Leaf(leaf) => leaf.h,
        Child::Container(inner) => inner.h,
        Child::Table(table) => table.h,
    }
}

fn child_width(child: &Child) -> i32 {
    match child {
        Child::Leaf(leaf) => leaf.w,
        Child::Container(inner) => inner.w,
        Child::Table(table) => table.w,
    }
}

fn set_position(child: &mut Child, x: i32, y: i32) {
    match child {
        Child::Leaf(leaf) => {
            leaf.x = x;
            leaf.y = y;
        }
        Child::Container(inner) => {
            inner.x = x;
            inner.y = y;
        }
        Child::Table(table) => {
            table.x = x;
            table.y = y;
        }
    }
}

fn layout_table(table: &mut Table, available: i32, font: &BitmapFont) {
    table.row_heights.clear();
    let (border, padding, spacing) = (table.border, table.padding, table.spacing);
    if table.percent {
        table.width_attr = table.width_attr * available / 100;
    }
    if available <= table.w && table.width_attr == 0 {
        table.width_attr = available;
    }
    let natural = available > table.w && table.width_attr == 0;
    let mut y = padding + border * 2 + spacing;
    let mut widest = 0;
    let mut sum_heights = 0;
    let natural_width = table.w;
    let width_attr = table.width_attr;
    let widths = table.column_widths.clone();
    table.column_positions.clear();
    for row in &mut table.rows {
        let mut row_height = 0;
        let mut x = padding + border * 2 + spacing;
        for (column, cell) in row.iter_mut().enumerate() {
            let cell_width = widths.get(column).copied().unwrap_or(0);
            if natural {
                layout_container(cell, cell_width, font);
            } else if natural_width < 1 {
                layout_container(cell, cell_width * width_attr, font);
            } else {
                layout_container(cell, cell_width * width_attr / natural_width, font);
            }
            cell.y = y;
            cell.x = x;
            cell.h -= LINE_GAP;
            row_height = row_height.max(cell.h);
            x += cell.w + (border + padding) * 2 + spacing;
            if column < table.column_positions.len() {
                if table.column_positions[column] < cell.w {
                    table.column_positions[column] = cell.w;
                }
            } else {
                table.column_positions.push(cell.w);
            }
        }
        for cell in row.iter_mut() {
            if cell.valign == 1 {
                cell.y_offset = row_height / 2 - cell.h / 2;
            } else if cell.valign == 3 {
                cell.y_offset = row_height - cell.h;
            }
        }
        table.row_heights.push(row_height);
        sum_heights += row_height;
        widest = widest.max(x - padding);
        y += spacing + (border + padding) * 2 + row_height;
    }
    if table.h <= y - padding {
        table.h = y - padding;
        table.w = widest;
        return;
    }
    let count = table.rows.len() as i32;
    let mut y = padding + border * 2 + spacing;
    for (index, row) in table.rows.iter_mut().enumerate() {
        let share = table.row_heights[index] * ((table.h - ((padding + border) * count + border) * 2) - (count + 1) * spacing) / sum_heights.max(1);
        for cell in row.iter_mut() {
            if cell.valign == 1 {
                cell.y_offset = share / 2 - cell.h / 2;
            } else if cell.valign == 3 {
                cell.y_offset = share - cell.h;
            }
            cell.h = share;
            cell.y = y;
        }
        table.row_heights[index] = row.first().map_or(0, |cell| cell.h);
        y += table.row_heights[index] + (border + padding) * 2 + spacing;
    }
    table.w = widest;
}

fn place_table(table: &mut Table, width: i32) {
    if table.align == 1 {
        table.x = width / 2 - table.w / 2;
    } else if table.align == 3 {
        table.x = width - table.w;
    }
    let (border, padding, spacing) = (table.border, table.padding, table.spacing);
    for (row_index, row) in table.rows.iter_mut().enumerate() {
        let mut x = spacing + border * 2 + padding;
        for (column, cell) in row.iter_mut().enumerate() {
            cell.x = x;
            let column_width = table.column_positions.get(column).copied().unwrap_or(0);
            x += spacing + (border + padding) * 2 + column_width;
            place(cell, column_width, table.row_heights.get(row_index).copied().unwrap_or(0));
        }
    }
}

fn place(container: &mut Container, width: i32, height: i32) {
    if container.h < height {
        container.h = height;
    }
    if container.w < width {
        container.w = width;
    }
    let (w, h) = (container.w, container.h);
    for child in &mut container.children {
        match child {
            Child::Leaf(leaf) => {
                if leaf.align == 1 {
                    leaf.x = w / 2 - leaf.w / 2;
                } else if leaf.align == 3 {
                    leaf.x = w - leaf.w;
                }
            }
            Child::Container(inner) => place(inner, w, h),
            Child::Table(table) => place_table(table, w),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextDraw {
    pub x: i32,
    pub y: i32,
    pub margin: i32,
    pub text: String,
    pub base: Argb,
    pub segments: Vec<(usize, usize, Argb)>,
    pub link: bool,
    pub action: String,
    pub message: String,
    pub subtype: i32,
    pub width: i32,
    pub glyphs: Vec<(i32, i32, char, Argb)>,
    pub underline: Vec<(i32, i32, i32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeafBox {
    pub kind: LeafKind,
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
    pub action: String,
    pub message: String,
    pub subtype: i32,
    pub end: i32,
    pub wrap: i32,
    pub parent_x: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Primitive {
    Fill { x: i32, y: i32, w: i32, h: i32, color: u32 },
    Line { x0: i32, y0: i32, x1: i32, y1: i32 },
    Text(TextDraw),
    Image { x: i32, y: i32, w: i32, h: i32, src: String },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Painted {
    pub primitives: Vec<Primitive>,
    pub boxes: Vec<LeafBox>,
    pub height: i32,
}

pub fn color_segments(display: &str, colors: &str) -> Vec<(usize, usize, Argb)> {
    let pieces: Vec<&str> = colors.split('\u{1e}').collect();
    let display: Vec<char> = display.chars().collect();
    let mut segments = Vec::new();
    let mut index = 1;
    while index + 1 < pieces.len() {
        let needle: Vec<char> = pieces[index + 1].chars().collect();
        let start = find_chars(&display, &needle);
        let color = if pieces[index] == "LEVEL" { LEVEL_COLOR } else { crate::html::font_color(pieces[index]) };
        segments.push((start, start.wrapping_add(needle.len()).wrapping_sub(1), color));
        index += 2;
    }
    segments
}

fn find_chars(haystack: &[char], needle: &[char]) -> usize {
    if needle.is_empty() {
        return 0;
    }
    if haystack.len() < needle.len() {
        return usize::MAX;
    }
    (0..=haystack.len() - needle.len()).find(|&start| haystack[start..start + needle.len()] == *needle).unwrap_or(usize::MAX)
}

fn text_primitive(leaf: &Leaf, x: i32, y: i32, margin: i32, font: &BitmapFont) -> TextDraw {
    let link = matches!(leaf.subtype, 1 | 2 | 9);
    let base = if link { LINK_COLOR } else { TEXT_COLOR };
    let segments = color_segments(&leaf.text, &leaf.colors);
    let start = x - margin;
    let cells = text_align::glyph_cells(&leaf.text, leaf.wrap_w, font, LANGUAGE, start, true, true);
    let chars: Vec<char> = leaf.text.chars().collect();
    let a_height = font.glyph('A').map_or(0, |glyph| glyph.height as i32);
    let mut glyphs = Vec::new();
    let mut underline = Vec::new();
    let mut cursor = 0;
    for cell in cells {
        while cursor < chars.len() && chars[cursor] != cell.c {
            cursor += 1;
        }
        if link {
            underline.push((margin + cell.x, margin + cell.x + cell.advance, y + cell.y + a_height));
        }
        if cell.c != ' ' {
            let color = segments.iter().find(|(from, to, _)| *from <= cursor && cursor <= *to).map_or(base, |(_, _, color)| *color);
            glyphs.push((margin + cell.x, y + cell.y, cell.c, color));
        }
        cursor += 1;
    }
    TextDraw { x, y, margin, text: leaf.text.clone(), base, segments, link, action: leaf.action.clone(), message: leaf.message.clone(), subtype: leaf.subtype, width: leaf.wrap_w, glyphs, underline }
}

fn paint_container(container: &Container, origin: (i32, i32), font: &BitmapFont, out: &mut Painted) {
    let origin = (origin.0 + container.x, origin.1 + container.y);
    for child in &container.children {
        match child {
            Child::Leaf(leaf) => {
                let (x, y) = (origin.0 + leaf.x, origin.1 + leaf.y + container.y_offset);
                if leaf.is_text {
                    out.primitives.push(Primitive::Text(text_primitive(leaf, x, y, origin.0, font)));
                } else if let LeafKind::Image { src } = &leaf.kind {
                    out.primitives.push(Primitive::Image { x, y, w: leaf.w, h: leaf.h, src: src.clone() });
                }
                if !leaf.is_text || matches!(leaf.subtype, 1 | 2 | 9) {
                    out.boxes.push(LeafBox { kind: leaf.kind.clone(), x, y: y - container.y_offset, w: leaf.w, h: leaf.h, action: leaf.action.clone(), message: leaf.message.clone(), subtype: leaf.subtype, end: leaf.end, wrap: leaf.wrap_w, parent_x: origin.0 });
                }
            }
            Child::Container(inner) => paint_container(inner, origin, font, out),
            Child::Table(table) => paint_table(table, origin, font, out),
        }
    }
}

fn paint_table(table: &Table, origin: (i32, i32), font: &BitmapFont, out: &mut Painted) {
    let (x, y) = (origin.0 + table.x, origin.1 + table.y);
    if let Some(color) = table.background {
        out.primitives.push(Primitive::Fill { x, y, w: table.w, h: table.h, color });
    }
    for level in 0..table.border.max(0) {
        let (x0, x1) = (x + level, x + table.w - level - 1);
        let (y0, y1) = (y + level, y + table.h - level - 1);
        out.primitives.push(Primitive::Line { x0, y0, x1, y1: y0 });
        out.primitives.push(Primitive::Line { x0, y0: y1, x1, y1 });
        out.primitives.push(Primitive::Line { x0, y0, x1: x0, y1 });
        out.primitives.push(Primitive::Line { x0: x1, y0, x1, y1 });
    }
    for row in &table.rows {
        for cell in row {
            paint_cell(cell, (x, y), table, font, out);
        }
    }
}

fn paint_cell(cell: &Container, origin: (i32, i32), table: &Table, font: &BitmapFont, out: &mut Painted) {
    let (x, y) = (origin.0 + cell.x, origin.1 + cell.y);
    if cell.border_flag && table.border > 0 {
        for level in 0..table.border {
            let left = cell.x - table.padding - 1 + level + origin.0;
            let top = cell.y - table.padding - 1 + level + origin.1;
            let right = cell.w - level + table.padding + cell.x + origin.0;
            let bottom = cell.h - level + cell.y + table.padding + origin.1;
            out.primitives.push(Primitive::Line { x0: left, y0: top, x1: right, y1: top });
            out.primitives.push(Primitive::Line { x0: left, y0: bottom, x1: right, y1: bottom });
            out.primitives.push(Primitive::Line { x0: left, y0: top, x1: left, y1: bottom });
            out.primitives.push(Primitive::Line { x0: right, y0: top, x1: right, y1: bottom });
        }
    }
    for child in &cell.children {
        match child {
            Child::Leaf(leaf) => {
                let (lx, ly) = (x + leaf.x, y + leaf.y + cell.y_offset);
                if leaf.is_text {
                    out.primitives.push(Primitive::Text(text_primitive(leaf, lx, ly, x, font)));
                } else if let LeafKind::Image { src } = &leaf.kind {
                    out.primitives.push(Primitive::Image { x: lx, y: ly, w: leaf.w, h: leaf.h, src: src.clone() });
                }
                if !leaf.is_text || matches!(leaf.subtype, 1 | 2 | 9) {
                    out.boxes.push(LeafBox { kind: leaf.kind.clone(), x: lx, y: ly - cell.y_offset, w: leaf.w, h: leaf.h, action: leaf.action.clone(), message: leaf.message.clone(), subtype: leaf.subtype, end: leaf.end, wrap: leaf.wrap_w, parent_x: x });
                }
            }
            Child::Container(inner) => paint_container(inner, (x, y), font, out),
            Child::Table(inner) => paint_table(inner, (x, y), font, out),
        }
    }
}

pub fn layout_tree(tokens: &[Token], width: i32, font: &BitmapFont) -> Option<Tree> {
    let mut tree = build(tokens, font)?;
    layout_container(&mut tree.root, width, font);
    let height = tree.root.h;
    place(&mut tree.root, width, height);
    Some(tree)
}

impl Tree {
    pub fn leaves(&self) -> Vec<&Leaf> {
        fn walk<'a>(container: &'a Container, out: &mut Vec<&'a Leaf>) {
            for child in &container.children {
                match child {
                    Child::Leaf(leaf) => out.push(leaf),
                    Child::Container(inner) => walk(inner, out),
                    Child::Table(table) => {
                        for row in &table.rows {
                            for cell in row {
                                walk(cell, out);
                            }
                        }
                    }
                }
            }
        }
        let mut out = Vec::new();
        walk(&self.root, &mut out);
        out
    }
}

pub fn run(tokens: &[Token], width: i32, font: &BitmapFont) -> Painted {
    let Some(tree) = layout_tree(tokens, width, font) else { return Painted::default() };
    let mut out = Painted { height: tree.root.h, ..Painted::default() };
    paint_container(&tree.root, (0, 0), font, &mut out);
    out
}

pub const COLOR_TEXT: Argb = TEXT_COLOR;
pub const COLOR_LINK: Argb = LINK_COLOR;
pub const COLOR_LEVEL: Argb = LEVEL_COLOR;
const _: () = {
    let _ = KIND_TABLE;
    let _ = KIND_TR;
};
