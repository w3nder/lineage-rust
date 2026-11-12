use crate::chat::wrap_line;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::inventory::thousands;

pub const FRAME_TEXTURES: [&str; 9] = [
    "L2UI_ch3.Tooltip.Tooltip1",
    "L2UI_ch3.Tooltip.Tooltip2",
    "L2UI_ch3.Tooltip.Tooltip3",
    "L2UI_ch3.Tooltip.Tooltip4",
    "L2UI_ch3.Tooltip.Tooltip5",
    "L2UI_ch3.Tooltip.Tooltip6",
    "L2UI_ch3.Tooltip.Tooltip7",
    "L2UI_ch3.Tooltip.Tooltip8",
    "L2UI_ch3.Tooltip.Tooltip9",
];
pub const CORNER: i32 = 8;
pub const PADDING: i32 = 5;
pub const LINE_GAP: i32 = 6;
pub const BELOW_CURSOR: i32 = 32;
const GRADE_SYMBOL: i32 = 12;
pub const NAME_COLOR: Argb = Argb(0xFFDCDCDC);
pub const ENCHANT_COLOR: Argb = Argb(0xFFB09B79);
pub const ADD_NAME_COLOR: Argb = Argb(0xFFFFD969);
pub const LABEL_COLOR: Argb = Argb(0xFFA3A3A3);
pub const VALUE_COLOR: Argb = Argb(0xFFB09B79);
pub const DESCRIPTION_COLOR: Argb = Argb(0xFFB2BECF);
pub const SEPARATOR: &str = " : ";
pub const LV_SYSSTRING: u32 = 88;
pub const MP_SYSSTRING: u32 = 91;
pub const WEIGHT_SYSSTRING: u32 = 52;
pub const ITEM_MIN_WIDTH: i32 = 144;
pub const ROW_WIDTH: i32 = 174;
pub const ROW_TWO_LINES_STYLE: u32 = 0x10000;
pub const ROW_TWO_LINES_THRESHOLD: i32 = 0x90;
pub const ROW_TWO_LINES_MARGIN: i32 = 0x1e;
pub const BOX_MARGIN: i32 = 2 * PADDING;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TooltipLine {
    pub label: String,
    pub value: String,
    pub color: Argb,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Tooltip {
    pub enchant: i32,
    pub name: String,
    pub add_name: String,
    pub grade: Option<String>,
    pub count: Option<i64>,
    pub level: Option<(String, i32)>,
    pub lines: Vec<TooltipLine>,
    pub description: String,
    pub item: bool,
    pub row: bool,
    pub row_style: u32,
    pub compact: bool,
    pub anchor: Option<(crate::window::WindowId, (i32, i32))>,
}

impl Tooltip {
    pub fn header_head(&self) -> Vec<(String, Argb)> {
        let mut parts = Vec::new();
        if self.enchant > 0 {
            parts.push((format!("+{} ", self.enchant), ENCHANT_COLOR));
        }
        parts.push((self.name.clone(), NAME_COLOR));
        if !self.add_name.is_empty() {
            parts.push((format!(" {}", self.add_name), ADD_NAME_COLOR));
        }
        parts
    }

    pub fn header_tail(&self) -> Vec<(String, Argb)> {
        let mut parts = Vec::new();
        if let Some((label, level)) = &self.level {
            parts.push((format!(" {label} "), LABEL_COLOR));
            parts.push((level.to_string(), VALUE_COLOR));
        }
        if let Some(count) = self.count {
            parts.push((format!(" ({})", thousands(count)), NAME_COLOR));
        }
        parts
    }

    pub fn header(&self) -> Vec<(String, Argb)> {
        let mut parts = self.header_head();
        parts.extend(self.header_tail());
        parts
    }

    fn grade_width(&self, font: &BitmapFont) -> i32 {
        if self.grade.is_some() { GRADE_SYMBOL + font.measure(" ") as i32 } else { 0 }
    }

    pub fn description_rows(&self, font: &BitmapFont) -> Vec<String> {
        if self.description.is_empty() {
            return Vec::new();
        }
        self.item_description_rows(self.item_width(font), font)
    }

    fn row_description_rows(&self, font: &BitmapFont) -> Vec<String> {
        if self.description.is_empty() {
            return Vec::new();
        }
        self.item_description_rows(self.row_width(font), font)
    }

    pub fn row_width(&self, font: &BitmapFont) -> i32 {
        if self.row_style != ROW_TWO_LINES_STYLE {
            return ROW_WIDTH;
        }
        let widest = self
            .lines
            .iter()
            .take(2)
            .map(|line| font.measure(&format!("{}{}{}", line.label, SEPARATOR, line.value)) as i32)
            .max()
            .unwrap_or(0);
        if widest < ROW_TWO_LINES_THRESHOLD { ROW_WIDTH } else { widest + ROW_TWO_LINES_MARGIN }
    }

    fn header_width(&self, font: &BitmapFont) -> i32 {
        self.header().iter().map(|(text, _)| font.measure(text) as i32).sum::<i32>() + self.grade_width(font)
    }

    fn line_width(&self, line: &TooltipLine, font: &BitmapFont) -> i32 {
        if line.label.is_empty() {
            font.measure(&line.value) as i32
        } else {
            font.measure(&line.label) as i32 + font.measure(SEPARATOR) as i32 + font.measure(&line.value) as i32
        }
    }

    pub fn item_width(&self, font: &BitmapFont) -> i32 {
        let mut width = self.header_width(font).max(ITEM_MIN_WIDTH);
        if let Some(first) = self.lines.first() {
            width = width.max(self.line_width(first, font));
        }
        width
    }

    fn item_description_rows(&self, width: i32, font: &BitmapFont) -> Vec<String> {
        self.description
            .replace("\\n", "\r\n")
            .split('\n')
            .flat_map(|line| wrap_line(line.trim_end_matches('\r'), width, 0, font).into_iter().map(|(_, row)| row))
            .collect()
    }

    pub fn size(&self, font: &BitmapFont) -> (i32, i32) {
        let line_height = font.line_height as i32;
        if self.row {
            let mut height = (line_height + LINE_GAP) * self.lines.len() as i32;
            let rows = self.row_description_rows(font);
            if !rows.is_empty() {
                height += rows.len() as i32 * line_height + LINE_GAP;
            }
            let width = self.row_width(font) + BOX_MARGIN;
            return (width, if height > 0 { height + BOX_MARGIN } else { 0 });
        }
        if self.item && self.compact {
            return (self.header_width(font) + 2 * PADDING, line_height + 2 * PADDING);
        }
        let width = self.item_width(font);
        let mut height = line_height + (line_height + LINE_GAP) * self.lines.len() as i32;
        let rows = self.description_rows(font);
        if !rows.is_empty() {
            height += rows.len() as i32 * line_height + LINE_GAP;
        }
        (width + 2 * PADDING, height + 2 * PADDING)
    }

    fn paint_lines(&self, list: &mut DrawList, tx: i32, mut ty: i32, font: &BitmapFont) -> i32 {
        for line in &self.lines {
            ty += LINE_GAP;
            let mut lx = tx;
            if line.label.is_empty() {
                list.draw_text(lx, ty, line.color, &line.value, font);
            } else {
                lx = list.draw_text(lx, ty, LABEL_COLOR, &line.label, font);
                lx = list.draw_text(lx, ty, NAME_COLOR, SEPARATOR, font);
                list.draw_text(lx, ty, line.color, &line.value, font);
            }
            ty += font.line_height as i32;
        }
        ty
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let (w, h) = self.size(font);
        paint_frame(list, x, y, w, h);
        let (tx, mut ty) = (x + PADDING, y + PADDING);
        let line_height = font.line_height as i32;
        if self.row {
            let mut ty = self.paint_lines(list, tx, ty, font);
            let rows = self.row_description_rows(font);
            if !rows.is_empty() {
                ty += LINE_GAP;
                for row in rows {
                    list.draw_text(tx, ty, DESCRIPTION_COLOR, &row, font);
                    ty += line_height;
                }
            }
            return;
        }
        let mut hx = tx;
        for (text, color) in self.header_head() {
            hx = list.draw_text(hx, ty, color, &text, font);
        }
        if let Some(texture) = &self.grade {
            hx += font.measure(" ") as i32;
            list.draw_texture(hx, ty, GRADE_SYMBOL, GRADE_SYMBOL, 0, 0, GRADE_SYMBOL, GRADE_SYMBOL, texture, 1.0);
            if let Some(quad) = list.quads.last_mut() { quad.color = NAME_COLOR.rgba(); }
            hx += GRADE_SYMBOL;
        }
        for (text, color) in self.header_tail() {
            hx = list.draw_text(hx, ty, color, &text, font);
        }
        ty += line_height;
        if self.item && self.compact {
            return;
        }
        ty = self.paint_lines(list, tx, ty, font);
        let rows = self.description_rows(font);
        if !rows.is_empty() {
            ty += LINE_GAP;
            for row in rows {
                list.draw_text(tx, ty, DESCRIPTION_COLOR, &row, font);
                ty += line_height;
            }
        }
    }
}

pub fn place_slot(slot: (i32, i32), size: (i32, i32), screen: (i32, i32), compact: bool) -> (i32, i32) {
    let (w, h) = size;
    let mut x = slot.0;
    if x + w > screen.0 {
        x -= x + w - screen.0;
    } else if x < 0 {
        x = 0;
    }
    let mut y = slot.1 - h;
    if y < 0 {
        y = if compact { slot.1 } else { slot.1 + BELOW_CURSOR };
    }
    (x, y)
}

pub fn place(cursor: (i32, i32), size: (i32, i32), screen: (i32, i32)) -> (i32, i32) {
    let (w, h) = size;
    let mut x = cursor.0;
    if x + w > screen.0 {
        x -= x + w - screen.0;
    } else if x < 0 {
        x = 0;
    }
    let mut y = cursor.1 - h;
    if y < 0 {
        y = cursor.1 + BELOW_CURSOR;
    }
    (x, y)
}

pub fn paint_frame(list: &mut DrawList, x: i32, y: i32, w: i32, h: i32) {
    let c = CORNER;
    let [tl, top, tr, left, center, right, bl, bottom, br] = FRAME_TEXTURES;
    list.draw_texture(x, y, c, c, 0, 0, c, c, tl, 1.0);
    list.draw_texture(x + c, y, w - 2 * c, c, 0, 0, c, c, top, 1.0);
    list.draw_texture(x + w - c, y, c, c, 0, 0, c, c, tr, 1.0);
    list.draw_texture(x, y + c, c, h - 2 * c, 0, 0, c, c, left, 1.0);
    list.draw_texture(x + c, y + c, w - 2 * c, h - 2 * c, 0, 0, c, c, center, 1.0);
    list.draw_texture(x + w - c, y + c, c, h - 2 * c, 0, 0, c, c, right, 1.0);
    list.draw_texture(x, y + h - c, c, c, 0, 0, c, c, bl, 1.0);
    list.draw_texture(x + c, y + h - c, w - 2 * c, c, 0, 0, c, c, bottom, 1.0);
    list.draw_texture(x + w - c, y + h - c, c, c, 0, 0, c, c, br, 1.0);
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

    #[test]
    fn the_box_sits_above_the_cursor_and_flips_below_or_slides_left_at_the_edges() {
        assert_eq!(place((100, 200), (50, 40), (800, 600)), (100, 160));
        assert_eq!(place((100, 30), (50, 40), (800, 600)), (100, 62));
        assert_eq!(place((780, 200), (50, 40), (800, 600)), (750, 160));
        assert_eq!(place((-3, 200), (50, 40), (800, 600)), (0, 160));
    }

    #[test]
    fn the_grade_symbol_is_drawn_after_the_add_name_and_widens_the_header() {
        let font = font();
        let plain = Tooltip { name: "Bow".into(), add_name: "Focus".into(), ..Default::default() };
        let graded = Tooltip { grade: Some("symbol.grade_s".into()), ..plain.clone() };
        assert_eq!(graded.header_width(&font), plain.header_width(&font) + GRADE_SYMBOL + font.measure(" ") as i32);
        let mut list = DrawList::default();
        graded.paint(&mut list, 0, 0, &font);
        let symbol = list.quads.iter().find(|quad| quad.texture == "symbol.grade_s").unwrap();
        let name_end = font.measure("Bow Focus") as i32 + font.measure(" ") as i32;
        assert_eq!((symbol.w as i32, symbol.h as i32), (GRADE_SYMBOL, GRADE_SYMBOL));
        assert_eq!(symbol.x as i32, PADDING + name_end);
    }

    #[test]
    fn the_header_carries_enchant_name_add_name_grade_level_and_count_in_their_colors() {
        let tip = Tooltip {
            enchant: 3,
            name: "Sword".into(),
            add_name: "Focus".into(),
            grade: Some("symbol.grade_d".into()),
            count: Some(1500),
            ..Default::default()
        };
        let header = tip.header();
        assert_eq!(header[0], ("+3 ".to_string(), ENCHANT_COLOR));
        assert_eq!(header[1], ("Sword".to_string(), NAME_COLOR));
        assert_eq!(header[2], (" Focus".to_string(), ADD_NAME_COLOR));
        assert_eq!(header[3], (" (1,500)".to_string(), NAME_COLOR));
        let skill = Tooltip { name: "Wind Strike".into(), level: Some(("Lv".into(), 3)), ..Default::default() };
        assert_eq!(skill.header()[1], (" Lv ".to_string(), LABEL_COLOR));
        assert_eq!(skill.header()[2], ("3".to_string(), VALUE_COLOR));
    }

    #[test]
    fn size_and_paint_stack_header_lines_and_description_with_the_frame_around() {
        let font = font();
        let tip = Tooltip {
            name: "Adena".into(),
            lines: vec![TooltipLine { label: "Weight".into(), value: "0".into(), color: VALUE_COLOR }],
            description: "Money.".into(),
            ..Default::default()
        };
        let (w, h) = tip.size(&font);
        assert_eq!(w, ITEM_MIN_WIDTH + 10);
        assert_eq!(h, 13 + 6 + 13 + 6 + 13 + 10);
        let mut list = DrawList::default();
        tip.paint(&mut list, 100, 50, &font);
        let frame: Vec<_> = list.quads.iter().filter(|q| q.texture.starts_with("L2UI_ch3.Tooltip")).collect();
        assert_eq!(frame.len(), 9);
        assert_eq!((frame[0].x, frame[0].y, frame[0].w, frame[0].h), (100.0, 50.0, 8.0, 8.0));
        assert_eq!((frame[4].x, frame[4].y, frame[4].w, frame[4].h), (108.0, 58.0, (w - 16) as f32, (h - 16) as f32));
        assert_eq!((frame[8].x, frame[8].y), ((100 + w - 8) as f32, (50 + h - 8) as f32));
        let text: Vec<_> = list.quads.iter().filter(|q| q.texture == font.atlas).collect();
        assert_eq!((text[0].x, text[0].y, text[0].color), (105.0, 55.0, NAME_COLOR.rgba()));
        let weight = text.iter().find(|q| q.color == LABEL_COLOR.rgba()).unwrap();
        assert_eq!((weight.x, weight.y), (105.0, (55 + 13 + 6) as f32));
        let value = text.iter().find(|q| q.color == VALUE_COLOR.rgba() && q.y == (55 + 13 + 6) as f32).unwrap();
        assert_eq!(value.x, (105 + 6 * 6 + 6 * 3) as f32);
        let desc = text.iter().find(|q| q.color == DESCRIPTION_COLOR.rgba()).unwrap();
        assert_eq!((desc.x, desc.y), (105.0, (55 + 13 + 6 + 13 + 6) as f32));
    }

    #[test]
    fn the_item_tooltip_is_compact_unless_selected_and_widens_only_by_the_header_and_first_line() {
        let font = font();
        let mut tip = Tooltip {
            name: "Sword".into(),
            item: true,
            compact: true,
            lines: vec![
                TooltipLine { label: String::new(), value: "Sword / One Handed".into(), color: VALUE_COLOR },
                TooltipLine { label: "P. Atk.".into(), value: "x".repeat(60), color: VALUE_COLOR },
            ],
            description: "one two".into(),
            ..Default::default()
        };
        assert_eq!(tip.size(&font), (5 * 6 + 10, 13 + 10));
        tip.compact = false;
        let (w, h) = tip.size(&font);
        assert_eq!(w, 144 + 10);
        assert_eq!(h, 13 + (13 + 6) * 2 + 13 + 6 + 10);
        assert_eq!(place_slot((10, 5), (50, 40), (800, 600), true), (10, 5));
        assert_eq!(place_slot((10, 5), (50, 40), (800, 600), false), (10, 37));
        assert_eq!(place_slot((790, 200), (50, 40), (800, 600), false), (750, 160));
    }

    fn row_tip(style: u32, lines: Vec<TooltipLine>, description: &str) -> Tooltip {
        Tooltip { row: true, row_style: style, lines, description: description.into(), ..Default::default() }
    }

    fn line(label: &str, value: &str) -> TooltipLine {
        TooltipLine { label: label.into(), value: value.into(), color: VALUE_COLOR }
    }

    #[test]
    fn layout_1002eac0_row_styles_are_174_wide_plus_the_ten_pixel_frame_and_wrap_the_description_at_174() {
        let font = font();
        let tip = row_tip(0x200000, vec![line("", "a"), line("L", "v")], "");
        assert_eq!(tip.size(&font), (174 + 10, (13 + 6) * 2 + 10));
        let empty = row_tip(0x8000, vec![], "");
        assert_eq!(empty.size(&font), (174 + 10, 0));
        let words = "w".repeat(40);
        let long = row_tip(0x200, vec![line("L", "v")], &format!("{words} {words}"));
        assert_eq!(long.row_width(&font), 174);
        let rows = long.row_description_rows(&font);
        assert!(rows.len() >= 2 && rows.iter().all(|row| font.measure(row) as i32 <= 174));
        assert_eq!(long.size(&font).1, 13 + 6 + rows.len() as i32 * 13 + 6 + 10);
    }

    #[test]
    fn layout_1002f1d3_the_two_line_style_widens_to_the_wider_of_the_first_two_lines_plus_thirty() {
        let font = font();
        let narrow = row_tip(ROW_TWO_LINES_STYLE, vec![line("a", "b"), line("c", "d")], "");
        assert_eq!(narrow.row_width(&font), 174);
        let value = "x".repeat(30);
        let wide = row_tip(ROW_TWO_LINES_STYLE, vec![line("a", "b"), line("c", &value), line("e", &"y".repeat(80))], "");
        let measured = font.measure(&format!("c : {value}")) as i32;
        assert!(measured >= 0x90);
        assert_eq!(wide.row_width(&font), measured + 30);
        assert_eq!(wide.size(&font), (measured + 30 + 10, (13 + 6) * 3 + 10));
    }
}
