use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::ini::IniFile;
use crate::zone_title::TrueTypeFont;
use killer_font::{Face, LOGICAL_DPI, Style};
use std::sync::Arc;

pub const NO_FONT_FILE: &str = "none";
pub const FONT_SLOTS: usize = 4;
pub const DEFAULT_FONT_TYPE: i32 = 4;
pub const CLIENT_FONT_FILES: [&str; 1] = ["asiahm-medium.ttf"];

pub fn font_type_for_usage(usage: &str) -> i32 {
    let usage = usage.trim();
    if usage.eq_ignore_ascii_case("zonetitle") {
        4
    } else if usage.eq_ignore_ascii_case("benchmark") {
        5
    } else if usage.eq_ignore_ascii_case("broadcast1") {
        6
    } else if usage.eq_ignore_ascii_case("broadcast2") {
        7
    } else {
        DEFAULT_FONT_TYPE
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TrueFontTable {
    pub slots: [Option<TrueTypeFont>; FONT_SLOTS],
}

impl TrueFontTable {
    pub fn from_ini(ini: &IniFile) -> TrueFontTable {
        let mut slots: [Option<TrueTypeFont>; FONT_SLOTS] = Default::default();
        for (index, slot) in slots.iter_mut().enumerate() {
            *slot = TrueTypeFont::from_ini(ini, index as u32 + 1);
        }
        TrueFontTable { slots }
    }

    pub fn slot_for_type(&self, font_type: i32) -> usize {
        (0..FONT_SLOTS)
            .find(|&index| self.slots[index].as_ref().is_some_and(|font| font_type_for_usage(&font.usage) == font_type))
            .unwrap_or(0)
    }

    pub fn font_for_type(&self, font_type: i32) -> Option<(usize, &TrueTypeFont)> {
        let index = self.slot_for_type(font_type);
        self.slots[index].as_ref().map(|font| (index, font))
    }
}

pub fn style_of(font: &TrueTypeFont) -> Style {
    Style { ppem: killer_font::ppem_for_size(font.size, LOGICAL_DPI), weight: font.weight, italic: font.italic }
}

pub fn resolve_face(font: &TrueTypeFont, mut load: impl FnMut(&str) -> Option<Vec<u8>>) -> Option<Arc<Face>> {
    if !font.file.eq_ignore_ascii_case(NO_FONT_FILE)
        && let Some(face) = load(&font.file).and_then(|bytes| Face::parse(&bytes).ok())
    {
        return Some(Arc::new(face));
    }
    let shipped: Vec<Arc<Face>> = CLIENT_FONT_FILES.iter().filter_map(|file| load(file)).filter_map(|bytes| Face::parse(&bytes).ok()).map(Arc::new).collect();
    let by_name = shipped.iter().find(|face| face.family().eq_ignore_ascii_case(&font.name)).cloned();
    by_name.or_else(|| shipped.into_iter().next())
}

pub trait LineFont {
    fn measure_text(&self, text: &str) -> i32;
    fn char_width(&self, c: char) -> Option<i32>;
    fn height(&self) -> i32;
    fn draw(&self, list: &mut DrawList, x: i32, y: i32, color: Argb, text: &str) -> i32;
}

impl LineFont for BitmapFont {
    fn measure_text(&self, text: &str) -> i32 {
        self.measure(text) as i32
    }

    fn char_width(&self, c: char) -> Option<i32> {
        self.glyph(c).map(|glyph| glyph.width as i32)
    }

    fn height(&self) -> i32 {
        self.line_height as i32
    }

    fn draw(&self, list: &mut DrawList, x: i32, y: i32, color: Argb, text: &str) -> i32 {
        list.draw_text(x, y, color, text, self)
    }
}

#[derive(Clone)]
pub struct TrueFont {
    pub slot: u8,
    pub face: Arc<Face>,
    pub style: Style,
}

impl TrueFont {
    pub fn install(slot: u8, face: Arc<Face>, style: Style) -> TrueFont {
        killer_font::install(slot, face.clone(), style);
        TrueFont { slot, face, style }
    }

    pub fn char_size(&self, c: char) -> (i32, i32) {
        self.face.char_size(self.style, c)
    }
}

impl LineFont for TrueFont {
    fn measure_text(&self, text: &str) -> i32 {
        self.face.text_width(self.style, text)
    }

    fn char_width(&self, c: char) -> Option<i32> {
        Some(self.face.char_size(self.style, c).0)
    }

    fn height(&self) -> i32 {
        self.face.line_height(self.style)
    }

    fn draw(&self, list: &mut DrawList, x: i32, y: i32, color: Argb, text: &str) -> i32 {
        list.draw_true_text(x, y, color, text, self)
    }
}

#[derive(Clone, Default)]
pub struct TrueFonts {
    pub table: TrueFontTable,
    pub fonts: [Option<TrueFont>; FONT_SLOTS],
}

impl TrueFonts {
    pub fn load(table: TrueFontTable, mut load: impl FnMut(&str) -> Option<Vec<u8>>) -> TrueFonts {
        let mut fonts: [Option<TrueFont>; FONT_SLOTS] = Default::default();
        for (index, slot) in table.slots.iter().enumerate() {
            let Some(config) = slot else { continue };
            if let Some(face) = resolve_face(config, &mut load) {
                fonts[index] = Some(TrueFont::install(index as u8, face, style_of(config)));
            }
        }
        TrueFonts { table, fonts }
    }

    pub fn for_type(&self, font_type: i32) -> Option<&TrueFont> {
        self.fonts[self.table.slot_for_type(font_type)].as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ini::parse_ini;

    const TTFONTINFO: &str = include_str!("../tests/fixtures/ttfontinfo.ini");

    #[test]
    fn usage_names_map_to_the_types_of_d3ddrv_10003af0() {
        assert_eq!(font_type_for_usage("zonetitle"), 4);
        assert_eq!(font_type_for_usage("BenchMark"), 5);
        assert_eq!(font_type_for_usage("broadcast1"), 6);
        assert_eq!(font_type_for_usage("broadcast2"), 7);
        assert_eq!(font_type_for_usage("outro"), 4);
    }

    #[test]
    fn unconfigured_types_fall_back_to_slot_zero_as_in_d3ddrv_10003df0() {
        let table = TrueFontTable::from_ini(&parse_ini(TTFONTINFO));
        let (slot, font) = table.font_for_type(6).unwrap();
        assert_eq!((slot, font.name.as_str(), font.weight, font.size, font.file.as_str()), (0, "Tahoma", 700, 32, NO_FONT_FILE));
        assert_eq!(table.slot_for_type(4), 0);
        assert_eq!(table.slot_for_type(7), 0);
    }

    #[test]
    fn configured_slots_are_found_by_their_type() {
        let ini = parse_ini(
            "[FontUsage]\nFont1=zonetitle\nFont3=broadcast1\n[FontName]\nFont1=Tahoma\nFont3=Asia\n[FontWeight]\nFont1=700\nFont3=400\n[FontSize]\nFont1=32\nFont3=22\n[FontFile]\nFont3=AsiaHM-Medium.TTF\n",
        );
        let table = TrueFontTable::from_ini(&ini);
        assert_eq!(table.slot_for_type(6), 2);
        assert_eq!(table.slot_for_type(7), 0);
        assert_eq!(table.font_for_type(6).unwrap().1.file, "AsiaHM-Medium.TTF");
    }

    #[test]
    fn height_is_the_negated_muldiv_of_the_logical_dpi_as_in_d3ddrv_10004414() {
        let table = TrueFontTable::from_ini(&parse_ini(TTFONTINFO));
        assert_eq!(style_of(table.slots[0].as_ref().unwrap()), Style { ppem: 43, weight: 700, italic: false });
    }

    #[test]
    #[ignore]
    fn the_client_font_gives_gdi_style_integer_metrics_and_the_installed_slot_rasterizes() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR") else { return };
        let bytes = std::fs::read(std::path::Path::new(&root).join("system/asiahm-medium.ttf")).unwrap();
        let table = TrueFontTable::from_ini(&parse_ini(TTFONTINFO));
        let fonts = TrueFonts::load(table, |name| (name == "asiahm-medium.ttf").then(|| bytes.clone()));
        let font = fonts.for_type(6).unwrap();
        let (w, h) = font.char_size('A');
        assert!(w > 0 && h == font.height());
        assert_eq!(font.measure_text("AB"), font.char_size('A').0 + font.char_size('B').0);
        assert_eq!(font.measure_text(""), 0);
        let bitmap = font.face.raster_line(font.style, "Red Team").unwrap();
        assert_eq!((bitmap.width as i32, bitmap.height as i32), (font.measure_text("Red Team"), font.height()));
        assert!(bitmap.alpha.iter().filter(|&&a| a > 128).count() > 100);
        let mut list = DrawList::default();
        let end = font.draw(&mut list, 10, 20, Argb(0xFFFF0000), "Red Team");
        assert_eq!(end, 10 + font.measure_text("Red Team"));
        assert_eq!(list.quads.len(), 1);
        assert_eq!(list.quads[0].texture, killer_font::texture_name(0, "Red Team"));
        let (tw, th, rgba) = killer_font::rgba_for_texture(&list.quads[0].texture).unwrap();
        assert_eq!((tw as usize * th as usize * 4), rgba.len());
    }
}
