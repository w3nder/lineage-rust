use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

pub const LOGICAL_DPI: i32 = 96;
pub const SYNTHETIC_BOLD_BELOW: u16 = 600;
pub const SYNTHETIC_BOLD_FROM: i32 = 600;
pub const SYNTHETIC_ITALIC_SLANT: f32 = 0.2126;
pub const TEXTURE_PREFIX: &str = "truefont:";

#[derive(Debug)]
pub enum FontError {
    Parse(String),
}

impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(why) => write!(f, "fonte TrueType inválida: {why}"),
        }
    }
}

impl std::error::Error for FontError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Style {
    pub ppem: u32,
    pub weight: i32,
    pub italic: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bitmap {
    pub width: u32,
    pub height: u32,
    pub alpha: Vec<u8>,
}

pub struct Face {
    font: fontdue::Font,
    units_per_em: i32,
    win_ascent: i32,
    win_descent: i32,
    weight: u16,
    family: String,
}

pub fn mul_div(number: i32, numerator: i32, denominator: i32) -> i32 {
    let product = i64::from(number) * i64::from(numerator);
    let half = i64::from(denominator) / 2;
    let rounded = if product >= 0 { product + half } else { product - half };
    (rounded / i64::from(denominator)) as i32
}

pub fn ppem_for_size(size: i32, logical_dpi: i32) -> u32 {
    mul_div(size, logical_dpi, 72).unsigned_abs()
}

fn scaled(value: i32, ppem: u32, units_per_em: i32) -> i32 {
    mul_div(value, ppem as i32, units_per_em)
}

impl Face {
    pub fn parse(bytes: &[u8]) -> Result<Face, FontError> {
        let parsed = ttf_parser::Face::parse(bytes, 0).map_err(|why| FontError::Parse(why.to_string()))?;
        let units_per_em = i32::from(parsed.units_per_em().max(1));
        let (win_ascent, win_descent, weight) = match parsed.tables().os2 {
            Some(os2) => (i32::from(os2.windows_ascender()), i32::from(os2.windows_descender()).abs(), parsed.weight().to_number()),
            None => (i32::from(parsed.ascender()), i32::from(parsed.descender()).abs(), parsed.weight().to_number()),
        };
        let family = parsed
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::FAMILY && name.is_unicode())
            .and_then(|name| name.to_string())
            .unwrap_or_default();
        let font = fontdue::Font::from_bytes(bytes, fontdue::FontSettings::default()).map_err(|why| FontError::Parse(why.to_string()))?;
        Ok(Face { font, units_per_em, win_ascent, win_descent, weight, family })
    }

    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn ascent(&self, style: Style) -> i32 {
        scaled(self.win_ascent, style.ppem, self.units_per_em)
    }

    pub fn descent(&self, style: Style) -> i32 {
        scaled(self.win_descent, style.ppem, self.units_per_em)
    }

    pub fn line_height(&self, style: Style) -> i32 {
        self.ascent(style) + self.descent(style)
    }

    fn synthetic_bold(&self, style: Style) -> bool {
        style.weight >= SYNTHETIC_BOLD_FROM && self.weight < SYNTHETIC_BOLD_BELOW
    }

    pub fn advance(&self, style: Style, c: char) -> i32 {
        let metrics = self.font.metrics(c, style.ppem as f32);
        metrics.advance_width.round() as i32 + i32::from(self.synthetic_bold(style))
    }

    pub fn char_size(&self, style: Style, c: char) -> (i32, i32) {
        (self.advance(style, c), self.line_height(style))
    }

    pub fn text_width(&self, style: Style, text: &str) -> i32 {
        text.lines().map(|line| line.chars().map(|c| self.advance(style, c)).sum()).max().unwrap_or(0)
    }

    fn slant(&self, style: Style) -> i32 {
        if style.italic { (self.ascent(style) as f32 * SYNTHETIC_ITALIC_SLANT).ceil() as i32 } else { 0 }
    }

    pub fn bitmap_size(&self, style: Style, line: &str) -> (i32, i32) {
        (self.text_width(style, line) + self.slant(style), self.line_height(style))
    }

    pub fn raster_line(&self, style: Style, line: &str) -> Option<Bitmap> {
        let (width, height) = self.bitmap_size(style, line);
        if self.text_width(style, line) <= 0 || height <= 0 {
            return None;
        }
        let bold = self.synthetic_bold(style);
        let (width, height) = (width as usize, height as usize);
        let mut alpha = vec![0u8; width * height];
        let baseline = self.ascent(style);
        let mut pen = 0i32;
        for c in line.chars() {
            let (metrics, coverage) = self.font.rasterize(c, style.ppem as f32);
            let top = baseline - metrics.ymin - metrics.height as i32;
            for row in 0..metrics.height {
                let y = top + row as i32;
                if y < 0 || y as usize >= height {
                    continue;
                }
                let shear = if style.italic { ((baseline - y) as f32 * SYNTHETIC_ITALIC_SLANT).round() as i32 } else { 0 };
                for col in 0..metrics.width {
                    let value = coverage[row * metrics.width + col];
                    if value == 0 {
                        continue;
                    }
                    for extra in 0..=i32::from(bold) {
                        let x = pen + metrics.xmin + col as i32 + shear + extra;
                        if x < 0 || x as usize >= width {
                            continue;
                        }
                        let cell = &mut alpha[y as usize * width + x as usize];
                        *cell = (*cell).max(value);
                    }
                }
            }
            pen += self.advance(style, c);
        }
        Some(Bitmap { width: width as u32, height: height as u32, alpha })
    }
}

pub fn texture_name(slot: u8, text: &str) -> String {
    let mut name = format!("{TEXTURE_PREFIX}{slot}:");
    for unit in text.encode_utf16() {
        name.push_str(&format!("{unit:04x}"));
    }
    name
}

pub fn parse_texture_name(name: &str) -> Option<(u8, String)> {
    let rest = name.strip_prefix(TEXTURE_PREFIX)?;
    let (slot, hex) = rest.split_once(':')?;
    if hex.len() % 4 != 0 {
        return None;
    }
    let units: Option<Vec<u16>> = (0..hex.len() / 4).map(|i| u16::from_str_radix(&hex[i * 4..i * 4 + 4], 16).ok()).collect();
    Some((slot.parse().ok()?, String::from_utf16(&units?).ok()?))
}

#[derive(Clone)]
pub struct Installed {
    pub face: Arc<Face>,
    pub style: Style,
}

fn registry() -> &'static Mutex<BTreeMap<u8, Installed>> {
    static REGISTRY: OnceLock<Mutex<BTreeMap<u8, Installed>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(BTreeMap::new()))
}

pub fn install(slot: u8, face: Arc<Face>, style: Style) {
    if let Ok(mut map) = registry().lock() {
        map.insert(slot, Installed { face, style });
    }
}

pub fn installed(slot: u8) -> Option<Installed> {
    registry().lock().ok()?.get(&slot).cloned()
}

pub fn rgba_for_texture(name: &str) -> Option<(u32, u32, Vec<u8>)> {
    let (slot, text) = parse_texture_name(name)?;
    let installed = installed(slot)?;
    let bitmap = installed.face.raster_line(installed.style, &text)?;
    let mut rgba = Vec::with_capacity(bitmap.alpha.len() * 4);
    for alpha in bitmap.alpha {
        rgba.extend([255, 255, 255, alpha]);
    }
    Some((bitmap.width, bitmap.height, rgba))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul_div_rounds_like_win32() {
        assert_eq!(mul_div(32, 96, 72), 43);
        assert_eq!(mul_div(22, 96, 72), 29);
        assert_eq!(mul_div(10, 96, 72), 13);
        assert_eq!(ppem_for_size(32, LOGICAL_DPI), 43);
    }

    #[test]
    fn texture_name_round_trips_any_text() {
        let name = texture_name(2, "Lv85 \u{d55c}\u{ae00} \u{1f600}");
        assert_eq!(parse_texture_name(&name), Some((2, "Lv85 \u{d55c}\u{ae00} \u{1f600}".to_string())));
        assert_eq!(name, name.to_lowercase());
        assert_eq!(parse_texture_name("outra"), None);
    }
}
