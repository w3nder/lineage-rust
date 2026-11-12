use crate::ini::IniFile;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlyphRect {
    pub x: i32,
    pub width: u32,
    pub y: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Glyphs {
    pub tex_width: u32,
    pub tex_height: u32,
    pub glyphs: HashMap<char, GlyphRect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontError {
    Truncated,
    NegativeMetric,
    InvalidChar(u32),
}

impl fmt::Display for FontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => write!(f, ".gly truncado"),
            Self::NegativeMetric => write!(f, ".gly com métrica negativa"),
            Self::InvalidChar(c) => write!(f, ".gly com caractere inválido {c:#x}"),
        }
    }
}

impl std::error::Error for FontError {}

impl Glyphs {
    pub fn parse(bytes: &[u8]) -> Result<Glyphs, FontError> {
        let mut reader = Reader { bytes, pos: 0 };
        let tex_width = reader.u32()?;
        let tex_height = reader.u32()?;
        let ranges = reader.u32()?;
        let mut glyphs = HashMap::new();
        for _ in 0..ranges {
            let first = reader.u32()?;
            let count = reader.u32()?;
            for index in 0..count {
                let code = first.checked_add(index).ok_or(FontError::InvalidChar(first))?;
                let c = char::from_u32(code).ok_or(FontError::InvalidChar(code))?;
                let raw = [reader.u32()? as i32, reader.u32()? as i32, reader.u32()? as i32, reader.u32()? as i32];
                if raw == [-1; 4] {
                    continue;
                }
                let [x, width, y, height] = raw;
                let [width, y, height] = [width, y, height].map(|v| u32::try_from(v).map_err(|_| FontError::NegativeMetric));
                glyphs.insert(c, GlyphRect { x, width: width?, y: y?, height: height? });
            }
        }
        Ok(Glyphs { tex_width, tex_height, glyphs })
    }
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn u32(&mut self) -> Result<u32, FontError> {
        let end = self.pos + 4;
        let slice = self.bytes.get(self.pos..end).ok_or(FontError::Truncated)?;
        self.pos = end;
        Ok(u32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]]))
    }

}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitmapFont {
    pub atlas: String,
    pub glyphs: Glyphs,
    pub line_height: u32,
}

impl BitmapFont {
    pub fn new(atlas: String, glyphs: Glyphs) -> BitmapFont {
        let line_height = glyphs.glyphs.values().map(|g| g.height).max().unwrap_or(0);
        BitmapFont { atlas, glyphs, line_height }
    }

    pub fn measure(&self, text: &str) -> u32 {
        text.lines().map(|line| line.chars().filter_map(|c| self.glyph(c)).map(|g| g.width).sum()).max().unwrap_or(0)
    }

    pub fn glyph(&self, c: char) -> Option<&GlyphRect> {
        self.glyphs.glyphs.get(&c)
    }

    pub fn has_glyph(&self, c: char) -> bool {
        self.glyph(c).is_some_and(|g| g.width > 0 && g.height > 0)
    }

    pub fn strip_missing_glyphs(&self, text: &str) -> String {
        text.chars().filter(|&c| self.has_glyph(c) || c == '\r' || c == '\n').collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FontSet {
    pub small: (String, String),
    pub large: (String, String),
}

impl FontSet {
    pub fn from_localization(ini: &IniFile) -> Option<FontSet> {
        let language = ini.get("LanguageSet", "Language")?;
        let country = ini.get("LanguageSet", language)?;
        let entry = |suffix: &str| ini.get(country, &format!("{country}_{suffix}")).map(str::to_string);
        Some(FontSet { small: (entry("Font")?, entry("Glyph")?), large: (entry("Font2")?, entry("Glyph2")?) })
    }
}

#[cfg(test)]
pub(crate) fn gly(ranges: &[(u32, &[[i32; 4]])], tex: (u32, u32)) -> Vec<u8> {
    let mut b = Vec::new();
    b.extend(tex.0.to_le_bytes());
    b.extend(tex.1.to_le_bytes());
    b.extend((ranges.len() as u32).to_le_bytes());
    for (first, glyphs) in ranges {
        b.extend(first.to_le_bytes());
        b.extend((glyphs.len() as u32).to_le_bytes());
        for g in *glyphs {
            for v in g {
                b.extend(v.to_le_bytes());
            }
        }
    }
    b.extend([0u8; 20]);
    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ini::parse_ini;

    #[test]
    fn parses_ranges_into_glyph_rects() {
        let bytes = gly(&[(0x20, &[[0, 3, 0, 13], [4, 6, 0, 13]])], (1024, 128));
        let g = Glyphs::parse(&bytes).unwrap();
        assert_eq!((g.tex_width, g.tex_height), (1024, 128));
        assert_eq!(g.glyphs[&' '].width, 3);
        assert_eq!(g.glyphs[&'!'].x, 4);
        assert!(Glyphs::parse(&bytes[..20]).is_err());
    }

    #[test]
    fn a_negative_x_is_kept_as_a_signed_origin_like_the_interlude_row_starts() {
        let bytes = gly(&[(0x7e, &[[-1, 8, 15, 13], [7, 7, 1, 13]])], (1024, 128));
        let g = Glyphs::parse(&bytes).unwrap();
        assert_eq!(g.glyphs[&'~'], GlyphRect { x: -1, width: 8, y: 15, height: 13 });
        assert_eq!(g.glyphs[&'\u{7f}'].x, 7);
        let mut bad = bytes.clone();
        bad[24 + 8..24 + 12].copy_from_slice(&(-1i32).to_le_bytes());
        assert_eq!(Glyphs::parse(&bad), Err(FontError::NegativeMetric));
    }

    #[test]
    #[ignore]
    fn the_real_interlude_fonts_parse_with_signed_row_origins() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR") else { return };
        let dir = std::path::Path::new(&root).join("system");
        let Ok(bytes) = std::fs::read(dir.join("SmallFont-r.gly")) else { return };
        let small = Glyphs::parse(&bytes).unwrap();
        assert_eq!((small.tex_width, small.tex_height, small.glyphs.len()), (1024, 128, 343));
        assert_eq!(small.glyphs[&'~'], GlyphRect { x: -1, width: 8, y: 15, height: 13 });
        let large = Glyphs::parse(&std::fs::read(dir.join("LargeFont-r.gly")).unwrap()).unwrap();
        assert_eq!(large.glyphs.len(), 350);
        assert!(large.glyphs.values().all(|g| g.height == 17));
        let font = BitmapFont::new("L2Font-r.SmallFont-r".into(), small);
        assert!(font.measure("Lineage") > 0 && font.has_glyph('\u{0410}'));
    }

    #[test]
    fn a_record_of_minus_one_is_an_absent_glyph_and_other_negatives_still_fail() {
        let mut bytes = gly(&[(0x20, &[[0, 3, 0, 13], [-1; 4]])], (1024, 128));
        let g = Glyphs::parse(&bytes).unwrap();
        assert!(g.glyphs.contains_key(&' ') && !g.glyphs.contains_key(&'!'));
        let at = bytes.len() - 20 - 12;
        bytes[at..at + 4].copy_from_slice(&(-2i32).to_le_bytes());
        assert!(Glyphs::parse(&bytes).is_err());
    }

    #[test]
    #[ignore]
    fn the_real_credit_font_parses_with_absent_glyphs() {
        let Some(root) = std::env::var_os("KILLER_CLIENT_DIR") else { return };
        let bytes = std::fs::read(std::path::Path::new(&root).join("system/CreditFont.gly")).unwrap();
        let g = Glyphs::parse(&bytes).unwrap();
        assert_eq!((g.tex_width, g.tex_height), (1024, 1024));
        assert!(g.glyphs.contains_key(&'A') && !g.glyphs.contains_key(&'\u{ac02}'));
    }

    const LOCALIZATION: &str = include_str!("../tests/fixtures/localization.ini");

    #[test]
    fn new_takes_the_line_height_from_the_tallest_glyph() {
        let bytes = gly(&[(0x20, &[[0, 3, 0, 13], [4, 6, 0, 14]])], (1024, 128));
        let font = BitmapFont::new("L2Font-e.LargeFont-e".into(), Glyphs::parse(&bytes).unwrap());
        assert_eq!(font.line_height, 14);
        assert_eq!(font.atlas, "L2Font-e.LargeFont-e");
        let empty = BitmapFont::new("x".into(), Glyphs::parse(&gly(&[], (1, 1))).unwrap());
        assert_eq!(empty.line_height, 0);
    }

    #[test]
    fn font_set_follows_the_language_of_localization_ini() {
        let set = FontSet::from_localization(&parse_ini(LOCALIZATION)).unwrap();
        assert_eq!(set.small, ("L2Font-e.SmallFont-e".to_string(), "SmallFont-e.gly".to_string()));
        assert_eq!(set.large, ("L2Font-e.LargeFont-e".to_string(), "LargeFont-e.gly".to_string()));
        let korea = FontSet::from_localization(&parse_ini(&LOCALIZATION.replace("Language=1", "Language=0"))).unwrap();
        assert_eq!(korea.small.0, "L2Font.SmallFont");
        assert_eq!(korea.large.1, "LargeFont.gly");
    }

    #[test]
    fn font_set_is_none_without_a_complete_language_section() {
        assert!(FontSet::from_localization(&parse_ini("")).is_none());
        assert!(FontSet::from_localization(&parse_ini("[LanguageSet]\nLanguage=7\n1=English\n")).is_none());
        let partial = "[LanguageSet]\nLanguage=1\n1=English\n[English]\nEnglish_Font=a\nEnglish_Glyph=b\n";
        assert!(FontSet::from_localization(&parse_ini(partial)).is_none());
    }

    #[test]
    fn measure_sums_widths_and_space() {
        let bytes = gly(&[(0x20, &[[0, 3, 0, 13], [4, 6, 0, 13]])], (1024, 128));
        let font = BitmapFont {
            atlas: "L2Font-e.SmallFont-e".into(),
            glyphs: Glyphs::parse(&bytes).unwrap(),
            line_height: 13,
        };
        assert_eq!(font.measure("! !"), 6 + 3 + 6);
        assert_eq!(font.measure("!?"), 6);
    }

    #[test]
    fn nl2font_100884a0_accepts_only_glyphs_with_positive_width_and_height() {
        let bytes = gly(&[(0x20, &[[0, 3, 0, 13], [4, 0, 0, 13], [8, 5, 0, 0]])], (1024, 128));
        let font = BitmapFont { atlas: "f".into(), glyphs: Glyphs::parse(&bytes).unwrap(), line_height: 13 };
        assert!(font.has_glyph(' '));
        assert!(!font.has_glyph('!'));
        assert!(!font.has_glyph('"'));
        assert!(!font.has_glyph('a'));
        assert_eq!(font.strip_missing_glyphs(" !a\r\"\n "), " \r\n ");
    }
}
