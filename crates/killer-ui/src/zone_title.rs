use crate::color::Argb;
use crate::draw::DrawList;
use crate::true_font::LineFont;
use crate::ini::IniFile;
use l2_dat::ZoneName;

pub const FONT_DECO: &str = "L2ui_ch3.etc.fontdeco";
pub const FONT_DECO_POS: (i32, i32) = (-100, -80);
pub const FONT_DECO_SIZE: i32 = 256;
pub const TITLE_RGB: u32 = 0xF0F0DC;
pub const SHADOW_RGB: u32 = 0x000000;
pub const SHADOW_OFFSET: i32 = 3;
pub const LINE2_TEXT_GAP: i32 = 5;
pub const LINE3_TEXT_GAP: i32 = 10;
pub const LINE2_SHADOW_GAP: i32 = 3;
pub const LINE3_SHADOW_GAP: i32 = 6;
pub const SHOWN_STATE: i32 = 2;
pub const TRUETYPE_FONT_TYPE: i32 = 4;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrueTypeFont {
    pub usage: String,
    pub name: String,
    pub weight: i32,
    pub size: i32,
    pub italic: bool,
    pub file: String,
}

impl TrueTypeFont {
    pub fn from_ini(ini: &IniFile, slot: u32) -> Option<TrueTypeFont> {
        let key = format!("Font{slot}");
        Some(TrueTypeFont {
            usage: ini.get("FontUsage", &key)?.to_string(),
            name: ini.get("FontName", &key)?.to_string(),
            weight: ini.int("FontWeight", &key)?,
            size: ini.int("FontSize", &key)?,
            italic: ini.bool("Italic", &key).unwrap_or(false),
            file: ini.get("FontFile", &key).unwrap_or(crate::true_font::NO_FONT_FILE).to_string(),
        })
    }

    pub fn zone_title(ini: &IniFile) -> Option<TrueTypeFont> {
        (1..=4)
            .filter_map(|slot| TrueTypeFont::from_ini(ini, slot))
            .find(|font| font.usage.eq_ignore_ascii_case("zonetitle"))
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ZoneTitleView {
    pub elapsed_seconds: f32,
    pub lines: [String; 3],
    pub state: i32,
    pub alpha: u8,
    pub shadow_alpha: u8,
}

impl ZoneTitleView {
    pub fn shown(zone: &str) -> ZoneTitleView {
        ZoneTitleView {
            elapsed_seconds: 0.0,
            lines: [zone.to_string(), String::new(), String::new()],
            state: SHOWN_STATE,
            alpha: 0xFF,
            shadow_alpha: 0xFF,
        }
    }
}

pub const TERRITORY_COLOR_LIMIT: u32 = 200;
pub const TILE_WORLD_SIZE: f64 = 32768.0;
pub const TILE_ORIGIN: (i32, i32) = (20, 18);
pub const LOOKUP_ATTEMPTS: u32 = 256;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ZoneIndex {
    zones: Vec<ZoneName>,
    map: std::collections::HashMap<u32, usize>,
}

fn zone_key(color: u32, grid_x: u32, grid_y: u32) -> u32 {
    let combined = if color >= TERRITORY_COLOR_LIMIT { ((grid_y << 8 | grid_x) << 8) | color } else { color };
    combined << 8
}

impl ZoneIndex {
    pub fn new(zones: &[ZoneName]) -> ZoneIndex {
        let mut map = std::collections::HashMap::new();
        for (index, zone) in zones.iter().enumerate() {
            let mut key = zone_key(zone.color_id, zone.grid_x, zone.grid_y);
            while zone.color_id >= TERRITORY_COLOR_LIMIT && map.contains_key(&key) {
                key = key.wrapping_add(1);
            }
            map.insert(key, index);
        }
        ZoneIndex { zones: zones.to_vec(), map }
    }

    pub fn info(&self, color: u32, grid_x: u32, grid_y: u32, z: f32) -> Option<&ZoneName> {
        let mut key = zone_key(color, grid_x, grid_y);
        for _ in 0..LOOKUP_ATTEMPTS {
            let zone = &self.zones[*self.map.get(&key)?];
            if zone.bottom_z < z && z < zone.top_z {
                return Some(zone);
            }
            key = key.wrapping_add(1);
        }
        None
    }
}

pub fn map_cell(x: i32, y: i32) -> (i32, i32) {
    let cell = |value: i32, shift: f64| ((f64::from(value) + shift) / TILE_WORLD_SIZE + 9.0) as i32;
    (cell(x, 360_448.0), cell(y, 294_912.0))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneTexture {
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<u8>,
}

impl ZoneTexture {
    pub fn pixel(&self, x: f32, y: f32, cell: (i32, i32)) -> Option<u8> {
        let scale = 1.0 / TILE_WORLD_SIZE;
        let base_x = f64::from(((cell.0 - TILE_ORIGIN.0) as f64 * TILE_WORLD_SIZE) as i32);
        let base_y = f64::from(((cell.1 - TILE_ORIGIN.1) as f64 * TILE_WORLD_SIZE) as i32);
        let px = ((f64::from(x) - base_x) * f64::from(self.width) * scale) as i32;
        let py = ((f64::from(y) - base_y) * f64::from(self.height) * scale) as i32;
        if px < 0 || py < 0 {
            return None;
        }
        self.pixels.get(usize::try_from(py).ok()?.checked_mul(usize::try_from(self.width).ok()?)?.checked_add(px as usize)?).copied()
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ZoneTracker {
    previous: Option<(u32, String)>,
    timer: f32,
    pub current: Option<String>,
    pub current_id: Option<u32>,
}

impl ZoneTracker {
    pub const COOLDOWN_SECONDS: f32 = 8.0;

    pub fn reset(&mut self) {
        self.previous = None;
    }

    pub fn tick(&mut self, delta: f32, found: Option<&ZoneName>) -> Option<String> {
        self.timer += delta;
        let zone = found?;
        self.current = Some(zone.name.clone());
        self.current_id = Some(zone.id);
        if self.previous.as_ref().is_some_and(|(id, _)| *id == zone.id) || self.timer <= Self::COOLDOWN_SECONDS {
            return None;
        }
        let title = match &self.previous {
            Some((_, name)) if *name == zone.name => None,
            _ => Some(zone.name.clone()),
        };
        self.previous = Some((zone.id, zone.name.clone()));
        self.timer = 0.0;
        title
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ZoneResolver {
    index: Option<ZoneIndex>,
    textures: std::collections::HashMap<(i32, i32), Option<ZoneTexture>>,
    wanted: Vec<(i32, i32)>,
}

impl ZoneResolver {
    pub fn ensure(&mut self, zones: &[ZoneName]) {
        if self.index.is_none() {
            self.index = Some(ZoneIndex::new(zones));
        }
    }

    pub fn take_requests(&mut self) -> Vec<(i32, i32)> {
        std::mem::take(&mut self.wanted)
    }

    pub fn set_texture(&mut self, cell: (i32, i32), texture: Option<ZoneTexture>) {
        self.textures.insert(cell, texture);
    }

    pub fn lookup(&mut self, position: (f32, f32, f32)) -> Option<&ZoneName> {
        let cell = map_cell(position.0 as i32, position.1 as i32);
        if !self.textures.contains_key(&cell) && !self.wanted.contains(&cell) {
            self.wanted.push(cell);
        }
        let index = self.index.as_ref()?;
        let textures = &self.textures;
        zone_at(index, position, |cell| textures.get(&cell).and_then(Option::as_ref))
    }
}

pub fn zone_texture_name(cell: (i32, i32)) -> String {
    format!("L2ZoneName.{}_{}", cell.0, cell.1)
}

pub fn zone_at<'a>(
    index: &'a ZoneIndex,
    position: (f32, f32, f32),
    mut texture: impl FnMut((i32, i32)) -> Option<&'a ZoneTexture>,
) -> Option<&'a ZoneName> {
    let cell = map_cell(position.0 as i32, position.1 as i32);
    let texture = texture(cell)?;
    let color = u32::from(texture.pixel(position.0, position.1, cell)?);
    index.info(color, cell.0 as u32, cell.1 as u32, position.2)
}

pub fn title_width(view: &ZoneTitleView, font: &dyn LineFont) -> i32 {
    font.measure_text(&view.lines[title_line_index(view.lines.each_ref().map(|line| line.encode_utf16().count()))])
}

pub fn title_line_index(lengths: [usize; 3]) -> usize {
    if lengths[0] >= lengths[1] && lengths[0] >= lengths[2] {
        0
    } else if lengths[1] >= lengths[2] {
        1
    } else {
        2
    }
}

pub trait ZoneTitleTickBackend {
    fn first_line_empty(&mut self) -> bool;
    fn visible(&mut self) -> bool;
    fn first_line_height(&mut self) -> i32;
    fn title_width(&mut self) -> i32;
    fn screen_height(&mut self) -> i32;
    fn screen_width(&mut self) -> i32;
    fn set_rect(&mut self, rect: crate::window::Rect);
    fn show(&mut self);
    fn hide(&mut self);
    fn elapsed_seconds(&mut self) -> f32;
    fn set_elapsed_seconds(&mut self, seconds: f32);
    fn clear_lines(&mut self);
    fn set_alpha(&mut self, alpha: u8);
    fn set_shadow_alpha(&mut self, alpha: u8);
}

pub struct ZoneTitleFrame<'a> {
    pub view: &'a mut ZoneTitleView,
    pub window: &'a mut crate::window::Window,
    pub font: &'a dyn LineFont,
    pub screen: (i32, i32),
}

impl ZoneTitleTickBackend for ZoneTitleFrame<'_> {
    fn first_line_empty(&mut self) -> bool {
        self.view.lines[0].is_empty()
    }
    fn visible(&mut self) -> bool {
        self.window.visible
    }
    fn first_line_height(&mut self) -> i32 {
        self.font.height()
    }
    fn title_width(&mut self) -> i32 {
        title_width(self.view, self.font)
    }
    fn screen_height(&mut self) -> i32 {
        self.screen.1
    }
    fn screen_width(&mut self) -> i32 {
        self.screen.0
    }
    fn set_rect(&mut self, rect: crate::window::Rect) {
        self.window.rect = rect;
    }
    fn show(&mut self) {
        self.window.visible = true;
    }
    fn hide(&mut self) {
        self.window.visible = false;
    }
    fn elapsed_seconds(&mut self) -> f32 {
        self.view.elapsed_seconds
    }
    fn set_elapsed_seconds(&mut self, seconds: f32) {
        self.view.elapsed_seconds = seconds;
    }
    fn clear_lines(&mut self) {
        for line in &mut self.view.lines {
            line.clear();
        }
    }
    fn set_alpha(&mut self, alpha: u8) {
        self.view.alpha = alpha;
    }
    fn set_shadow_alpha(&mut self, alpha: u8) {
        self.view.shadow_alpha = alpha;
    }
}

fn alpha_byte(value: f64) -> u8 {
    crate::window_restore::x87_to_int(value.clamp(0.0, 255.0)) as u8
}

fn add_elapsed_seconds(delta: f32, elapsed: f32) -> f32 {
    if delta.is_nan() {
        let delta_bits = delta.to_bits();
        let delta = delta_bits | 0x00400000;
        let elapsed_bits = elapsed.to_bits();
        let elapsed = elapsed_bits | 0x00400000;
        if f32::from_bits(elapsed_bits).is_nan() && delta_bits & 0x00400000 == 0 && elapsed_bits & 0x00400000 != 0 {
            return f32::from_bits(elapsed);
        }
        let selected = if f32::from_bits(elapsed_bits).is_nan()
            && (elapsed_bits & 0x00400000 != 0 || delta_bits & 0x00400000 == 0)
        {
            match (delta & 0x7fffffff).cmp(&(elapsed & 0x7fffffff)) {
                std::cmp::Ordering::Less => elapsed,
                std::cmp::Ordering::Equal => delta.min(elapsed),
                std::cmp::Ordering::Greater => delta,
            }
        } else {
            delta
        };
        return f32::from_bits(selected);
    }
    if elapsed.is_nan() {
        return f32::from_bits(elapsed.to_bits() | 0x00400000);
    }
    let sum = (f64::from(delta) + f64::from(elapsed)) as f32;
    if sum.is_nan() { f32::from_bits(0xffc00000) } else { sum }
}

pub fn tick_zone_title(backend: &mut impl ZoneTitleTickBackend, delta_seconds: f32) {
    if backend.first_line_empty() {
        return;
    }
    if !backend.visible() {
        let height = backend.first_line_height();
        let width = backend.title_width();
        let y = backend.screen_height() / 5;
        let center_width = backend.title_width();
        let x = (backend.screen_width() / 2).wrapping_sub(center_width / 2);
        backend.set_rect(crate::window::Rect::new(x, y, width, height));
        backend.show();
    }
    let elapsed = add_elapsed_seconds(delta_seconds, backend.elapsed_seconds());
    backend.set_elapsed_seconds(elapsed);
    if elapsed > 8.0 {
        backend.clear_lines();
        backend.set_elapsed_seconds(0.0);
        if backend.visible() {
            backend.hide();
        }
        return;
    }
    if elapsed < 2.0 {
        backend.set_alpha(alpha_byte(f64::from(elapsed) * 127.5));
    } else if elapsed > 4.0 {
        backend.set_alpha(alpha_byte(f64::from((255.0 - (f64::from(elapsed) - 4.0) * 127.5) as f32)));
    }
    if elapsed < 2.0 {
        backend.set_shadow_alpha(alpha_byte(f64::from(elapsed) * 85.0));
    } else if elapsed > 4.0 {
        backend.set_shadow_alpha(alpha_byte(f64::from((255.0 - (f64::from(elapsed) - 4.0) * 170.0) as f32)));
    }
}

pub fn paint_zone_title(list: &mut DrawList, x: i32, y: i32, view: &ZoneTitleView, font: &dyn LineFont) {
    let width = title_width(view, font);
    let (dx, dy) = FONT_DECO_POS;
    let alpha = f32::from(view.alpha) / 255.0;
    list.draw_texture(
        x + dx,
        y + dy,
        FONT_DECO_SIZE,
        FONT_DECO_SIZE,
        0,
        0,
        FONT_DECO_SIZE,
        FONT_DECO_SIZE,
        FONT_DECO,
        alpha,
    );
    if view.state < SHOWN_STATE {
        return;
    }
    let text_color = Argb(u32::from(view.alpha) << 24 | TITLE_RGB);
    let shadow_color = Argb(u32::from(view.shadow_alpha) << 24 | SHADOW_RGB);
    let h = font.height();
    let ys = [
        (SHADOW_OFFSET, 0),
        (h + LINE2_SHADOW_GAP, h + LINE2_TEXT_GAP),
        (2 * h + LINE3_SHADOW_GAP, 2 * h + LINE3_TEXT_GAP),
    ];
    for (line, (shadow_y, text_y)) in view.lines.iter().zip(ys) {
        if line.is_empty() {
            continue;
        }
        let left = x + width / 2 - font.measure_text(line) / 2;
        font.draw(list, left + SHADOW_OFFSET, y + shadow_y, shadow_color, line);
        font.draw(list, left, y + text_y, text_color, line);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;
    use crate::font::{BitmapFont, Glyphs, gly};
    use crate::ini::parse_ini;

    const TTFONTINFO: &str = include_str!("../tests/fixtures/ttfontinfo.ini");

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 8, 7, 0, 14]).collect();
        BitmapFont::new("L2Font-e.LargeFont-e".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn zone(id: u32, color: u32, grid: (u32, u32), name: &str) -> ZoneName {
        ZoneName {
            id,
            color_id: color,
            grid_x: grid.0,
            grid_y: grid.1,
            top_z: 30000.0,
            bottom_z: -30000.0,
            name: name.into(),
        }
    }

    #[test]
    fn ttfontinfo_registers_tahoma_32_bold_for_the_zone_title() {
        let ini = parse_ini(TTFONTINFO);
        let font = TrueTypeFont::zone_title(&ini).unwrap();
        assert_eq!(
            font,
            TrueTypeFont { usage: "zonetitle".into(), name: "Tahoma".into(), weight: 700, size: 32, italic: false, file: "none".into() }
        );
        assert_eq!(TrueTypeFont::from_ini(&ini, 2), None);
        assert_eq!(TrueTypeFont::zone_title(&parse_ini("")), None);
    }

    #[test]
    fn the_title_is_drawn_twice_with_the_shadow_three_pixels_down_right_under_the_deco() {
        let font = font();
        let view = ZoneTitleView {
            elapsed_seconds: 0.0,
            lines: ["Giran".into(), "Castle".into(), "Town".into()],
            state: 2,
            alpha: 0xFF,
            shadow_alpha: 0x80,
        };
        let mut list = DrawList::default();
        paint_zone_title(&mut list, 651, 100, &view, &font);
        assert_eq!(title_width(&view, &font), 42);
        let deco: Vec<&Quad> = list.quads.iter().filter(|q| q.texture == FONT_DECO).collect();
        assert_eq!(
            (deco[0].x, deco[0].y, deco[0].w, deco[0].h, deco[0].uw, deco[0].color[3]),
            (551.0, 20.0, 256.0, 256.0, 256.0, 1.0)
        );
        let text: Vec<&Quad> = list.quads.iter().filter(|q| q.texture == font.atlas).collect();
        assert_eq!(text.len(), 2 * (5 + 6 + 4));
        let shadow = Argb(0x80000000).rgba();
        let title = Argb(0xFFF0F0DC).rgba();
        assert_eq!((text[0].x, text[0].y, text[0].color), (651.0 + 21.0 - 17.0 + 3.0, 103.0, shadow));
        assert_eq!((text[5].x, text[5].y, text[5].color), (651.0 + 21.0 - 17.0, 100.0, title));
        let line2: Vec<&&Quad> = text.iter().filter(|q| q.color == title && q.y == 100.0 + 14.0 + 5.0).collect();
        assert_eq!(line2.len(), 6);
        let line2_shadow: Vec<&&Quad> =
            text.iter().filter(|q| q.color == shadow && q.y == 100.0 + 14.0 + 3.0).collect();
        assert_eq!(line2_shadow.len(), 6);
        assert!(text.iter().any(|q| q.color == title && q.y == 100.0 + 28.0 + 10.0));
        assert!(text.iter().any(|q| q.color == shadow && q.y == 100.0 + 28.0 + 6.0));
    }

    #[test]
    fn title_width_10121260_selects_utf16_length_with_first_line_ties() {
        assert_eq!(title_line_index([3, 3, 2]), 0);
        assert_eq!(title_line_index([2, 3, 3]), 1);
        assert_eq!(title_line_index([1, 2, 3]), 2);
        let font = font();
        let view = ZoneTitleView { lines: ["abc".into(), "😀😀".into(), "a".into()], ..ZoneTitleView::default() };
        assert_eq!(title_width(&view, &font), font.measure("😀😀") as i32);
    }

    #[test]
    fn get_zone_name_info_1044c800_probes_consecutive_keys_with_strict_z_ranges() {
        let zones = vec![
            zone(4, 4, (0, 0), "Giran Territory"),
            zone(41, 253, (22, 22), "Arena"),
            zone(43, 255, (22, 22), "Giran Castle Town"),
            zone(154, 254, (22, 22), "Tanor Canyon"),
            ZoneName { top_z: 100.0, bottom_z: 0.0, ..zone(50, 230, (22, 22), "Upper") },
            ZoneName { top_z: 0.0, bottom_z: -100.0, ..zone(51, 230, (22, 22), "Lower") },
        ];
        let index = ZoneIndex::new(&zones);
        let name = |color, cell: (u32, u32), z| index.info(color, cell.0, cell.1, z).map(|zone| zone.name.as_str());
        assert_eq!(name(4, (9, 9), 0.0), Some("Giran Territory"));
        assert_eq!(name(255, (22, 22), 0.0), Some("Giran Castle Town"));
        assert_eq!(name(255, (23, 22), 0.0), None);
        assert_eq!(name(255, (22, 22), 50000.0), None);
        assert_eq!(name(230, (22, 22), 50.0), Some("Upper"));
        assert_eq!(name(230, (22, 22), -50.0), Some("Lower"));
        assert_eq!(name(230, (22, 22), 0.0), None);
        assert_eq!(name(230, (22, 22), 100.0), None);
    }

    #[test]
    fn a_later_record_with_the_same_territory_colour_replaces_the_earlier_one() {
        let zones = vec![zone(1, 7, (0, 0), "First"), zone(2, 7, (0, 0), "Second")];
        assert_eq!(ZoneIndex::new(&zones).info(7, 0, 0, 0.0).map(|zone| zone.name.as_str()), Some("Second"));
    }

    #[test]
    fn get_zone_name_1044c970_samples_the_tile_texture_at_the_world_position() {
        assert_eq!(map_cell(0, 0), (20, 18));
        assert_eq!(map_cell(-1, -1), (19, 17));
        assert_eq!(map_cell(82_157, 147_057), (22, 22));
        let zones = vec![zone(43, 255, (22, 22), "Giran Castle Town"), zone(9, 9, (0, 0), "Giran Territory")];
        let index = ZoneIndex::new(&zones);
        let mut pixels = vec![9u8; 4 * 4];
        pixels[2 * 4 + 1] = 255;
        let texture = ZoneTexture { width: 4, height: 4, pixels };
        let base = (2 * 32768, 4 * 32768);
        let at = |dx: f32, dy: f32| zone_at(&index, (base.0 as f32 + dx, base.1 as f32 + dy, 0.0), |cell| {
            assert_eq!(cell, (22, 22));
            Some(&texture)
        });
        assert_eq!(at(8192.0 + 100.0, 16384.0 + 100.0).map(|zone| zone.name.as_str()), Some("Giran Castle Town"));
        assert_eq!(at(100.0, 100.0).map(|zone| zone.name.as_str()), Some("Giran Territory"));
        assert!(zone_at(&index, (base.0 as f32, base.1 as f32, 0.0), |_| None).is_none());
        assert_eq!(zone_texture_name((22, 22)), "L2ZoneName.22_22");
    }

    #[test]
    fn the_controller_tick_105985f8_shows_a_title_only_on_a_new_zone_after_8_seconds() {
        let zone = |id, name: &str| ZoneName { id, ..zone(id, 1, (0, 0), name) };
        let (a, b, a2) = (zone(1, "Giran"), zone(2, "Aden"), zone(3, "Giran"));
        let mut tracker = ZoneTracker::default();
        assert_eq!(tracker.tick(4.0, Some(&a)), None);
        assert_eq!(tracker.current.as_deref(), Some("Giran"));
        assert_eq!(tracker.tick(4.1, Some(&a)), Some("Giran".to_string()));
        assert_eq!(tracker.tick(20.0, Some(&a)), None);
        assert_eq!(tracker.tick(5.0, Some(&b)), Some("Aden".to_string()));
        assert_eq!(tracker.current.as_deref(), Some("Aden"));
        assert_eq!(tracker.tick(4.0, Some(&a2)), None);
        assert_eq!(tracker.current.as_deref(), Some("Giran"));
        assert_eq!(tracker.tick(5.0, Some(&a2)), Some("Giran".to_string()));
        assert_eq!(tracker.tick(9.0, None), None);
        assert_eq!(tracker.current.as_deref(), Some("Giran"));
        tracker.reset();
        assert_eq!(tracker.tick(9.0, Some(&a2)), Some("Giran".to_string()));
    }
}
