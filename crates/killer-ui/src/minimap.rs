use crate::button::TextButton;
use crate::color::Argb;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, TITLE_HEIGHT};
use crate::window::Rect;

pub const MINIMAP_SIZE: (i32, i32) = (334, 383);
pub const MINIMAP_DEFAULT_Y_RISE: i32 = 252;
pub const TITLE_SYSSTRING: u32 = 447;
pub const POSITION_SYSSTRING: u32 = 471;
pub const QUEST_STATUS_SYSSTRING: u32 = 833;
pub const QUEST_PERIOD_SYSSTRINGS: [u32; 4] = [973, 974, 975, 976];
pub const MAP_BACK: &str = "L2UI_CH3.Minimap.MapBack";
pub const MINIMAP_POS: &str = "L2UI.MinimapWnd.MinimapPos";
pub const TARGET_POS: &str = "L2UI.MinimapWnd.TargetPos";
pub const MAP_ICON_MARK: &str = "L2UI_CH3.Minimap.MapIcon_Mark";
pub const PARTY_MARK: &str = "L2UI_CH3.Minimap.minimap_party";
pub const PARTY_MARK_SIZE: i32 = 0x10;
pub const PARTY_HIGHLIGHT_MS: i32 = 2000;
pub const MAP_SHADOW: &str = "L2UI_CH3.Minimap.MapShadow";
pub const MAP_MOON: &str = "L2UI_CH3.Minimap.Map_Moon";
pub const MAP_SUN: &str = "L2UI_CH3.Minimap.Map_Sun";
pub const BTN_NORMAL: &str = "L2UI_CH3.Button.Btn1_Normal";
pub const BTN_DOWN: &str = "L2UI_CH3.Button.Btn1_NormalOn";
pub const WORLD_MAP_PREFIX: &str = "L2Font-e.Minimap.ch4_worldmap";
pub const COVER_RECT: Rect = Rect::new(3, 21, 328, 328);
pub const VIEW: i32 = 0x148;
pub const TILE_TEXTURE: i32 = 0x400;
pub const MAP_COLUMNS: i32 = 2;
pub const MAP_ROWS: i32 = 3;
pub const MAP_SIZE: (i32, i32) = (TILE_TEXTURE * MAP_COLUMNS, TILE_TEXTURE * MAP_ROWS);
pub const WORLD_MAP_SCALE: f32 = 41.0 / 8192.0;
pub const WORLD_MAP_ORIGIN: (i32, i32) = (131072, 262144);
pub const GRID_UNITS: f32 = 12288.0;
pub const GRID_LABEL_BASE: i32 = 0x14;
pub const GRID_LABEL_MIN_X: i32 = 0xf;
pub const GRID_LABEL_MIN_Y: i32 = 0x14;
pub const GRID_LINE_COLOR: Argb = Argb(0x40FFFFFF);
pub const TEXT_COLOR: Argb = Argb(0xFFDCDCDC);
pub const FIXED_SPOT_X: (i32, i32) = (0x9e0b, 0xc3fc);
pub const FIXED_SPOT_Y: (i32, i32) = (0x3afad, 0x3d4e7);
pub const FIXED_SPOT_Z: (i32, i32) = (-0x1ada, -0x130a);
pub const FIXED_SPOT_MAP: (i32, i32) = (0x64, 0x536);
pub const MARKER_SIZE: i32 = 0x20;
pub const QUEST_MARK_RISE: i32 = 0x14;
pub const QUEST_MARK_HIT: i32 = 0x14;
pub const QUEST_MARK_EDGE: i32 = 5;
pub const QUEST_TIP_MARGIN: i32 = 5;
pub const QUEST_TIP_FRAME: i32 = 8;
pub const MARKER_MARGIN: i32 = 0xc;
pub const PARTY_MARK_RISE: i32 = 0x14;
pub const MAX_PARTY_MARKS: usize = 8;
pub const BOTTOM_BAR_RECT: Rect = Rect::new(0, 0x136, 0x149, 0x17);
pub const BOTTOM_TEXT_X: i32 = 0x16;
pub const POSITION_TEXT_RISE: i32 = 15;
pub const QUEST_TEXT_RISE: i32 = 31;
pub const CLOCK_ICON_RECT: Rect = Rect::new(4, 6, 0x19, 0x19);
pub const CLOCK_TEXT_POS: (i32, i32) = (0x1d, 0x13);
pub const SUN_HOURS: (i32, i32) = (6, 24);
pub const BUTTON_SIZE: (i32, i32) = (76, 23);
pub const BUTTON_Y: i32 = 355;
pub const BUTTON_XS: [i32; 3] = [89, 169, 249];
pub const BUTTON_SYSSTRINGS: [u32; 3] = [1223, 886, 887];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinimapButton {
    PartyMember,
    TargetLoc,
    CurrentLoc,
    ZoomIn(usize),
    ZoomOut,
}

pub const BUTTONS: [MinimapButton; 3] = [MinimapButton::PartyMember, MinimapButton::TargetLoc, MinimapButton::CurrentLoc];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinimapHit {
    Close,
    Title,
    Button(MinimapButton),
    Cover,
    Body,
}

impl From<MinimapButton> for MinimapHit {
    fn from(button: MinimapButton) -> MinimapHit {
        MinimapHit::Button(button)
    }
}

pub fn button_rect(index: usize) -> Rect {
    Rect::new(BUTTON_XS[index], BUTTON_Y, BUTTON_SIZE.0, BUTTON_SIZE.1)
}


pub const MAP_WIDTHS: [i32; 17] = [1804, 653, 768, 606, 950, 657, 650, 659, 633, 950, 740, 821, 983, 767, 557, 993, 763];
pub const MAP_HEIGHTS: [i32; 17] = [2624, 626, 706, 647, 740, 618, 510, 623, 675, 860, 760, 971, 839, 701, 737, 699, 663];
pub const MAP_ORIGINS: [(i32, i32); 17] = [
    (131072, 262144), (-90624, 236416), (-89344, 145792), (-19968, 117504), (768, 9344), (11840, 138432),
    (11840, 165120), (75136, 48768), (39168, 42688), (73536, 140032), (111104, 70016), (139264, 16384),
    (101504, 212608), (-52736, -120832), (110144, -187968), (29760, -55808), (140256, -64320),
];
pub const TOWN_MAP_SCALE_F32: f32 = 0.05;
pub const WORLD_MAP_SCALE_EXACT: f64 = 0.0050048828125;
pub const ZOOM_IN_SPOTS: [(i32, i32); 16] = [
    (242, 2536), (269, 2079), (592, 1932), (721, 1394), (763, 2042), (739, 2156), (1086, 1595), (907, 1568),
    (1080, 2053), (1260, 1706), (1410, 1446), (1227, 2436), (438, 743), (1253, 401), (851, 1063), (1412, 1037),
];
pub const ZOOM_BUTTON_SIZE: i32 = 0x1a;
pub const ZOOM_OUT_RECT: Rect = Rect::new(0x12d, 0x12d, 0x18, 0x18);
pub const ZOOM_IN_TEXTURES: [&str; 2] = ["L2UI_CH3.Minimap.MapButton_ZoomIn1", "L2UI_CH3.Minimap.MapButton_ZoomIn2"];
pub const ZOOM_OUT_TEXTURES: [&str; 2] = ["L2UI_CH3.Minimap.MapButton_ZoomOut1", "L2UI_CH3.Minimap.MapButton_ZoomOut2"];
pub const TOWN_MAP_PREFIX: &str = "L2Font-e.TownMap.";
pub const TOWN_MAP_NAMES: [&str; 16] = [
    "map_talkin_island_village", "map_gludin_village", "map_gludio_castle_town", "map_dark_elven_village",
    "map_dion_castle_town", "map_floran_village", "map_town_of_oren", "map_elven_village", "map_giran_castle_town",
    "map_hunters_village", "map_town_of_aden", "map_heine", "map_orc_village", "map_dwarven_village",
    "map_rune_village", "map_godard_castle_town",
];

pub fn zone_map(zone_id: i32) -> usize {
    match zone_id {
        0x23 => 1,
        0x15 => 2,
        0x14 => 3,
        0x6b => 4,
        0x30 => 5,
        0x3a => 6,
        0x65 => 7,
        0x71 => 8,
        0x2b => 9,
        0x4a => 10,
        0x4d => 11,
        0x76 | 0x7a => 12,
        0x3e => 13,
        0x40 => 14,
        0x123 => 15,
        0x10d => 16,
        _ => 0,
    }
}

pub fn map_size(map: usize) -> (i32, i32) {
    (MAP_WIDTHS.get(map).copied().unwrap_or(0), MAP_HEIGHTS.get(map).copied().unwrap_or(0))
}

pub fn to_map(map: usize, x: i32, y: i32) -> (i32, i32) {
    let origin = MAP_ORIGINS[map.min(16)];
    if map == 0 {
        (
            (f64::from(x.wrapping_add(origin.0)) * WORLD_MAP_SCALE_EXACT) as i32,
            (f64::from(y.wrapping_add(origin.1)) * WORLD_MAP_SCALE_EXACT) as i32,
        )
    } else {
        (
            (f64::from(x.wrapping_sub(origin.0)) * f64::from(TOWN_MAP_SCALE_F32)) as i32,
            (f64::from(y.wrapping_sub(origin.1)) * f64::from(TOWN_MAP_SCALE_F32)) as i32,
        )
    }
}

pub fn marker_pos(map: usize, x: i32, y: i32) -> (i32, i32) {
    let inside = |v: i32, (lo, hi): (i32, i32)| lo <= v && v <= hi;
    if inside(x, FIXED_SPOT_X) && inside(y, FIXED_SPOT_Y) { FIXED_SPOT_MAP } else { to_map(map, x, y) }
}

pub fn player_pos(map: usize, x: i32, y: i32, z: i32) -> (i32, i32) {
    let inside = |v: i32, (lo, hi): (i32, i32)| lo <= v && v <= hi;
    if inside(x, FIXED_SPOT_X) && inside(y, FIXED_SPOT_Y) && inside(z, FIXED_SPOT_Z) {
        return FIXED_SPOT_MAP;
    }
    to_map(map, x, y)
}

pub fn scroll_for(map: usize, pos: (i32, i32)) -> (i32, i32) {
    let (width, height) = map_size(map);
    let axis = |v: i32, size: i32| {
        if v < 0xa4 {
            0
        } else if v > size - 0xa4 {
            size - VIEW
        } else {
            v - 0xa4
        }
    };
    (axis(pos.0, width), axis(pos.1, height))
}

pub fn grid_step() -> i32 {
    (WORLD_MAP_SCALE * GRID_UNITS) as i32
}

pub fn world_to_map(x: i32, y: i32) -> (i32, i32) {
    to_map(0, x, y)
}

pub fn player_map_pos(x: f32, y: f32, z: f32) -> (i32, i32) {
    player_pos(0, x as i32, y as i32, z as i32)
}

pub fn scroll_centered_on(map: (i32, i32)) -> (i32, i32) {
    scroll_for(0, map)
}

pub fn tile_placement(col: i32, row: i32, scroll: (i32, i32)) -> Option<(Rect, (i32, i32))> {
    let axis = |index: i32, scroll: i32| -> Option<(i32, i32, i32)> {
        let start = index * TILE_TEXTURE;
        if scroll >= start && scroll < start + TILE_TEXTURE {
            let u = scroll - start;
            Some((0, (TILE_TEXTURE - u).min(VIEW), u))
        } else if scroll >= start - TILE_TEXTURE && scroll < start {
            let size = scroll - start + VIEW;
            Some((VIEW - size, size, 0))
        } else {
            None
        }
    };
    let (dx, w, u) = axis(col, scroll.0)?;
    let (dy, h, v) = axis(row, scroll.1)?;
    (w > 0 && h > 0).then_some((Rect::new(dx, dy, w, h), (u, v)))
}

pub fn compass_rotation(yaw: i32) -> f32 {
    (f64::from(yaw) * std::f64::consts::TAU / 65536.0 + std::f64::consts::FRAC_PI_2) as f32
}

pub fn marker_visible(px: i32, py: i32) -> bool {
    px > MARKER_MARGIN - 1 && px <= VIEW - MARKER_MARGIN && py > MARKER_MARGIN - 1 && py < VIEW - MARKER_MARGIN
}

pub fn clock_text(hour: f32) -> (String, bool) {
    let mut h = hour as i32;
    let mut m = ((f64::from(hour) - f64::from(h)) * 60.0) as i32;
    if m > 0x3b {
        m -= 0x3c;
        h += 1;
    }
    let sun = h >= SUN_HOURS.0 && h <= SUN_HOURS.1;
    let mut period = "AM";
    if h >= 12 {
        period = "PM";
        h -= 12;
    }
    if h == 0 {
        h = 12;
    }
    let text = match (h < 10, m < 10) {
        (false, true) => format!("{period} {h} : 0{m}"),
        (false, false) => format!("{period} {h} : {m}"),
        (true, true) => format!("{period} 0{h} : 0{m}"),
        (true, false) => format!("{period} 0{h} : {m}"),
    };
    (text, sun)
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct QuestMark {
    pub world: (i32, i32, i32),
    pub name: String,
    pub title: String,
    pub id: i32,
}

pub fn quest_map_pos(x: i32, y: i32) -> (i32, i32) {
    marker_pos(0, x, y)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PartyMark {
    pub id: i32,
    pub world: (i32, i32, i32),
    pub in_window: bool,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MinimapView {
    pub title: String,
    pub labels: [String; 3],
    pub position_label: String,
    pub quest_status_label: String,
    pub period_labels: [String; 4],
    pub scroll: (i32, i32),
    pub map: usize,
    pub saved_scroll: (i32, i32),
    pub player: Option<(i32, i32)>,
    pub targets: Vec<(i32, i32)>,
    pub quest_target: Option<QuestMark>,
    pub quest_hover: bool,
    pub party: Vec<PartyMark>,
    pub yaw: std::cell::Cell<i32>,
    pub highlight: Option<usize>,
    pub highlight_ms: i32,
    pub pressed: Option<MinimapButton>,
    pub zone: String,
    pub zone_id: i32,
    pub period: i32,
    pub hour: Option<f32>,
    pub drag: Option<(i32, i32)>,
    pub player_hover: bool,
    pub party_hover: Option<usize>,
    pub player_name: String,
}

impl MinimapView {
    pub fn with_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> MinimapView {
        let text = |id: u32| lookup(id).unwrap_or_else(|| format!("sysstring {id}"));
        MinimapView {
            title: text(TITLE_SYSSTRING),
            labels: [text(BUTTON_SYSSTRINGS[0]), text(BUTTON_SYSSTRINGS[1]), text(BUTTON_SYSSTRINGS[2])],
            position_label: text(POSITION_SYSSTRING),
            quest_status_label: text(QUEST_STATUS_SYSSTRING),
            period_labels: QUEST_PERIOD_SYSSTRINGS.map(text),
            ..Default::default()
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<MinimapHit> {
        let (w, h) = MINIMAP_SIZE;
        if !Rect::new(0, 0, w, h).contains(x, y) {
            return None;
        }
        if close_button().contains(x, y) {
            return Some(MinimapHit::Close);
        }
        if y < TITLE_HEIGHT {
            return Some(MinimapHit::Title);
        }
        for (i, button) in BUTTONS.iter().enumerate() {
            if button_rect(i).contains(x, y) {
                return Some(MinimapHit::Button(*button));
            }
        }
        if COVER_RECT.contains(x, y) {
            let (lx, ly) = (x - COVER_RECT.x, y - COVER_RECT.y);
            if self.map == 0 {
                if let Some(index) = (0..ZOOM_IN_SPOTS.len()).find(|index| self.zoom_in_rect(*index).contains(lx, ly)) {
                    return Some(MinimapHit::Button(MinimapButton::ZoomIn(index)));
                }
            } else if ZOOM_OUT_RECT.contains(lx, ly) {
                return Some(MinimapHit::Button(MinimapButton::ZoomOut));
            }
            return Some(MinimapHit::Cover);
        }
        Some(MinimapHit::Body)
    }

    pub fn quest_point(&self) -> Option<(i32, i32)> {
        self.quest_target.as_ref().map(|mark| marker_pos(self.map, mark.world.0, mark.world.1))
    }

    pub fn press_map(&mut self, x: i32, y: i32) -> bool {
        let (mx, my) = (x - COVER_RECT.x, y - COVER_RECT.y);
        if (1..=VIEW).contains(&mx) && (1..=VIEW).contains(&my) {
            self.drag = Some((mx, my));
            return true;
        }
        false
    }

    pub fn release_map(&mut self) {
        self.drag = None;
    }

    pub fn party_positions(&self) -> Vec<(usize, (i32, i32))> {
        let mut shown = 0;
        let mut positions = Vec::new();
        for (index, mark) in self.party.iter().enumerate() {
            if (mark.world.0 == -1 && mark.world.1 == -1) || !mark.in_window {
                continue;
            }
            let (mx, my) = marker_pos(self.map, mark.world.0, mark.world.1);
            let (px, py) = (mx - self.scroll.0, my - self.scroll.1);
            if shown <= MAX_PARTY_MARKS && marker_visible(px, py) {
                positions.push((index, (px, py)));
                shown += 1;
            }
        }
        positions
    }

    pub fn party_tooltip(&self) -> Option<(String, (i32, i32))> {
        let positions = self.party_positions();
        let index = self.party_hover.or(self.highlight)?;
        let (_, (px, py)) = positions.into_iter().find(|(i, _)| *i == index)?;
        let name = self.party.get(index)?.name.clone();
        (!name.is_empty()).then_some((name, (COVER_RECT.x + px, COVER_RECT.y + py)))
    }

    fn player_view_pos(&self) -> Option<(i32, i32)> {
        self.player.map(|(px, py)| (px - self.scroll.0, py - self.scroll.1))
    }

    pub fn mouse_move(&mut self, x: i32, y: i32) {
        let (mx, my) = (x - COVER_RECT.x, y - COVER_RECT.y);
        if let Some((click_x, click_y)) = self.drag
            && mx > 0
            && mx <= VIEW
            && my > 0
            && my <= VIEW
        {
            let (width, height) = map_size(self.map);
            let (dx, dy) = (click_x - mx, click_y - my);
            self.scroll.0 = if dx > 0 { (self.scroll.0 + dx).min(width - VIEW) } else { (self.scroll.0 - dx.abs()).max(0) };
            self.scroll.1 = if dy > 0 { (self.scroll.1 + dy).min(height - VIEW) } else { (self.scroll.1 - dy.abs()).max(0) };
        }
        self.player_hover = self.player_view_pos().is_some_and(|(px, py)| {
            (MARKER_MARGIN..=VIEW - MARKER_MARGIN).contains(&px)
                && (MARKER_MARGIN..VIEW - MARKER_MARGIN).contains(&py)
                && (px - 10..=px + 10).contains(&mx)
                && (py - 10..=py + 10).contains(&my)
        });
        if self.drag.is_some() {
            self.drag = Some((mx, my));
        }
        self.party_hover = self
            .party_positions()
            .into_iter()
            .find(|(_, (px, py))| (*px..*px + PARTY_MARK_SIZE).contains(&mx) && (*py..*py + PARTY_MARK_SIZE).contains(&my))
            .map(|(index, _)| index);
        self.quest_hover = self.quest_point().is_some_and(|point| {
            let (dx, dy) = (point.0 - self.scroll.0, point.1 - self.scroll.1);
            (QUEST_MARK_EDGE..=VIEW - QUEST_MARK_EDGE).contains(&dx)
                && (QUEST_MARK_EDGE..VIEW - QUEST_MARK_EDGE).contains(&dy)
                && (dx..=dx + QUEST_MARK_HIT).contains(&mx)
                && (dy - QUEST_MARK_HIT..=dy).contains(&my)
        });
    }

    pub fn paint_player_tooltip(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let (Some((px, py)), true) = (self.player_view_pos(), self.player_hover) else { return };
        let (cx, cy) = (x + COVER_RECT.x, y + COVER_RECT.y);
        let width = font.measure(&self.player_name) as i32;
        let height = font.line_height as i32;
        let mut tx = px;
        let mut ty = py + 10;
        if width + tx + 16 > VIEW - QUEST_TIP_MARGIN {
            tx = VIEW - QUEST_TIP_MARGIN - 16 - width;
        }
        if height + ty + 16 > VIEW - QUEST_TIP_MARGIN {
            ty = VIEW - QUEST_TIP_MARGIN - 16 - height;
        }
        let start = list.quads.len();
        crate::tooltip::paint_frame(list, cx + tx, cy + ty, width + 2 * QUEST_TIP_FRAME, height + 2 * QUEST_TIP_FRAME);
        list.draw_text(cx + tx + QUEST_TIP_FRAME, cy + ty + QUEST_TIP_FRAME, TEXT_COLOR, &self.player_name, font);
        list.clip_from(start, Rect::new(cx, cy, VIEW, VIEW));
    }

    pub fn paint_quest_tooltip(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont) {
        let (Some(mark), Some(point), true) = (self.quest_target.as_ref(), self.quest_point(), self.quest_hover) else { return };
        let (cx, cy) = (x + COVER_RECT.x, y + COVER_RECT.y);
        let (bx, by) = (point.0 - self.scroll.0, point.1 - self.scroll.1);
        let line = font.line_height as i32;
        let first = font.measure(&mark.name) as i32;
        let second = if mark.id > 0 { font.measure(&mark.title) as i32 } else { 0 };
        let width = first.max(second);
        let height = if mark.id > 0 { line * 2 + 2 } else { line + 2 };
        let mut tx = (bx - QUEST_TIP_MARGIN).max(QUEST_TIP_MARGIN);
        let mut ty = if by - QUEST_TIP_MARGIN < QUEST_TIP_MARGIN { QUEST_TIP_MARGIN } else { by + QUEST_TIP_MARGIN };
        if tx + width + 0x10 > VIEW - QUEST_TIP_MARGIN {
            tx = VIEW - width - 0x15;
        }
        if ty + height + 0x10 > VIEW - QUEST_TIP_MARGIN {
            ty = VIEW - height - 0x15;
        }
        let start = list.quads.len();
        crate::tooltip::paint_frame(list, cx + tx, cy + ty, width + 2 * QUEST_TIP_FRAME, height + 2 * QUEST_TIP_FRAME);
        list.draw_text(cx + tx + QUEST_TIP_FRAME, cy + ty + QUEST_TIP_FRAME, TEXT_COLOR, &mark.name, font);
        if mark.id > 0 {
            list.draw_text(cx + tx + QUEST_TIP_FRAME, cy + ty + 10 + line, TEXT_COLOR, &mark.title, font);
        }
        list.clip_from(start, Rect::new(cx, cy, VIEW, VIEW));
    }

    pub fn tick(&mut self, delta_ms: i32) {
        if self.highlight.is_some() {
            self.highlight_ms -= delta_ms;
            if self.highlight_ms <= 0 {
                self.highlight = None;
            }
        }
    }

    pub fn center_on(&mut self, map: (i32, i32)) {
        self.scroll = scroll_for(self.map, map);
    }

    pub fn show(&mut self, zone_id: i32, player: (i32, i32, i32)) {
        self.map = zone_map(zone_id);
        self.saved_scroll = (0, 0);
        let (width, height) = map_size(self.map);
        if width > 0 && height > 0 {
            self.center_on(player_pos(self.map, player.0, player.1, player.2));
        }
    }

    pub fn center_world(&mut self, world: (i32, i32, i32), zone_map_at: usize, keep_town: bool) {
        if keep_town {
            if self.map != zone_map_at {
                self.map = 0;
            }
        } else {
            if self.map == 0 {
                self.saved_scroll = self.scroll;
            }
            self.map = zone_map_at;
        }
        let (width, height) = map_size(self.map);
        if width > 0 && height > 0 {
            self.center_on(player_pos(self.map, world.0, world.1, world.2));
        }
    }

    pub fn zoom_in(&mut self, index: usize, player: (i32, i32)) {
        self.map = index + 1;
        self.saved_scroll = self.scroll;
        let (width, height) = map_size(self.map);
        let (tx, ty) = to_map(self.map, player.0, player.1);
        self.scroll = if (0..=width).contains(&tx) && (0..=height).contains(&ty) { scroll_for(self.map, (tx, ty)) } else { (0, 0) };
    }

    pub fn zoom_out(&mut self, player: (i32, i32, i32)) {
        if self.map == 0 {
            return;
        }
        self.scroll = self.saved_scroll;
        self.map = 0;
        self.center_on(player_pos(0, player.0, player.1, player.2));
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        let (w, h) = MINIMAP_SIZE;
        FrameCtrl::titled(&self.title, true).paint(list, x, y, w, h, font, alpha);
        let (cx, cy) = (x + COVER_RECT.x, y + COVER_RECT.y);
        list.draw_texture(cx, cy, VIEW, VIEW, 0, 0, VIEW, VIEW, MAP_BACK, alpha);
        if self.map == 0 {
            for row in 0..MAP_ROWS {
                for col in 0..MAP_COLUMNS {
                    if let Some((rect, (u, v))) = tile_placement(col, row, self.scroll) {
                        let texture = format!("{WORLD_MAP_PREFIX}{}", 1 + col + row * MAP_COLUMNS);
                        list.draw_texture(cx + rect.x, cy + rect.y, rect.w, rect.h, u, v, rect.w, rect.h, &texture, 1.0);
                    }
                }
            }
        } else if let Some(name) = TOWN_MAP_NAMES.get(self.map - 1) {
            let texture = format!("{TOWN_MAP_PREFIX}{name}");
            list.draw_texture(cx, cy, VIEW, VIEW, self.scroll.0, self.scroll.1, VIEW, VIEW, &texture, 1.0);
        }
        if self.map == 0 {
            self.paint_grid(list, cx, cy, font);
        }
        for (_, (px, py)) in self.party_positions() {
            list.draw_texture(cx + px, cy + py, PARTY_MARK_SIZE, PARTY_MARK_SIZE, 0, 0, PARTY_MARK_SIZE, PARTY_MARK_SIZE, PARTY_MARK, 1.0);
        }
        if let Some((mx, my)) = self.player {
            let (px, py) = (mx - self.scroll.0, my - self.scroll.1);
            if marker_visible(px, py) {
                let (ox, oy) = (cx + px - MARKER_SIZE / 2, cy + py - MARKER_SIZE / 2);
                list.draw_texture_rotated(ox, oy, MARKER_SIZE, MARKER_SIZE, MARKER_SIZE, MARKER_SIZE, MINIMAP_POS, compass_rotation(self.yaw.get()), 1.0);
            }
        }
        for &(wx, wy) in &self.targets {
            let (mx, my) = marker_pos(self.map, wx, wy);
            let (px, py) = (mx - self.scroll.0, my - self.scroll.1);
            if marker_visible(px, py) {
                list.draw_texture(cx + px, cy + py, MARKER_SIZE, MARKER_SIZE, 0, 0, MARKER_SIZE, MARKER_SIZE, TARGET_POS, 1.0);
            }
        }
        if let Some(point) = self.quest_point() {
            let (px, py) = (point.0 - self.scroll.0, point.1 - self.scroll.1);
            list.draw_texture(cx + px, cy + py - QUEST_MARK_RISE, MARKER_SIZE, MARKER_SIZE, 0, 0, MARKER_SIZE, MARKER_SIZE, MAP_ICON_MARK, 1.0);
        }
        self.paint_zoom_buttons(list, cx, cy);
        let bar = BOTTOM_BAR_RECT;
        list.draw_texture(cx + bar.x, cy + bar.y, bar.w, bar.h, 0, 0, bar.w, bar.h, MAP_SHADOW, 1.0);
        let position = format!("{} : {}", self.position_label, self.zone);
        list.draw_text(cx + BOTTOM_TEXT_X, cy + VIEW - POSITION_TEXT_RISE, TEXT_COLOR, &position, font);
        if let Some(label) = usize::try_from(self.period).ok().and_then(|i| self.period_labels.get(i)) {
            let status = format!("{} : {}", self.quest_status_label, label);
            list.draw_text(cx + BOTTOM_TEXT_X, cy + VIEW - QUEST_TEXT_RISE, TEXT_COLOR, &status, font);
        }
        if let Some(hour) = self.hour {
            let (text, sun) = clock_text(hour);
            let icon = CLOCK_ICON_RECT;
            let texture = if sun { MAP_SUN } else { MAP_MOON };
            list.draw_texture(cx + icon.x, cy + icon.y, icon.w, icon.h, 0, 0, icon.w, icon.h, texture, 1.0);
            list.draw_text(cx + CLOCK_TEXT_POS.0, cy + CLOCK_TEXT_POS.1, TEXT_COLOR, &text, font);
        }
        for (i, button) in BUTTONS.iter().enumerate() {
            TextButton::new(button_rect(i), BTN_NORMAL, BTN_DOWN, &self.labels[i]).paint(
                list,
                x,
                y,
                self.pressed == Some(*button),
                font,
            );
        }
    }

    pub fn zoom_in_rect(&self, index: usize) -> Rect {
        let (x, y) = ZOOM_IN_SPOTS[index];
        Rect::new(x - self.scroll.0, y - self.scroll.1, ZOOM_BUTTON_SIZE, ZOOM_BUTTON_SIZE)
    }

    fn paint_zoom_buttons(&self, list: &mut DrawList, cx: i32, cy: i32) {
        let start = list.quads.len();
        if self.map == 0 {
            for index in 0..ZOOM_IN_SPOTS.len() {
                let rect = self.zoom_in_rect(index);
                let texture = ZOOM_IN_TEXTURES[usize::from(self.pressed == Some(MinimapButton::ZoomIn(index)))];
                list.draw_texture(cx + rect.x, cy + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, texture, 1.0);
            }
        } else {
            let rect = ZOOM_OUT_RECT;
            let texture = ZOOM_OUT_TEXTURES[usize::from(self.pressed == Some(MinimapButton::ZoomOut))];
            list.draw_texture(cx + rect.x, cy + rect.y, rect.w, rect.h, 0, 0, rect.w, rect.h, texture, 1.0);
        }
        list.clip_from(start, Rect::new(cx, cy, VIEW, VIEW));
    }

    fn paint_grid(&self, list: &mut DrawList, cx: i32, cy: i32, font: &BitmapFont) {
        let step = grid_step();
        let line_height = font.line_height as i32;
        let first_x = step - self.scroll.0 % step;
        let first_col = self.scroll.0 / step + GRID_LABEL_BASE;
        for i in 0..=VIEW / step {
            let (gx, col) = (first_x + i * step, first_col + i);
            if col >= 0 && gx > GRID_LABEL_MIN_X {
                let label = col.to_string();
                let lw = font.measure(&label) as i32;
                list.draw_text(cx + gx - lw / 2, cy + 2, TEXT_COLOR, &label, font);
            }
            list.draw_vline(cx + gx, cy + line_height, VIEW - line_height, GRID_LINE_COLOR);
        }
        let first_y = step - self.scroll.1 % step;
        let first_row = self.scroll.1 / step + GRID_LABEL_BASE;
        for i in 0..=VIEW / step {
            let (gy, row) = (first_y + i * step, first_row + i);
            let mut lw = 0;
            if row >= 0 {
                let label = (row + 1).to_string();
                lw = font.measure(&label) as i32;
                if gy > GRID_LABEL_MIN_Y {
                    list.draw_text(cx, cy + gy - line_height / 2, TEXT_COLOR, &label, font);
                }
            }
            list.draw_hline(cx + lw, cy + gy, VIEW - lw, GRID_LINE_COLOR);
        }
    }
}

pub fn close_button() -> Rect {
    Rect::new(MINIMAP_SIZE.0 - crate::frame::CLOSE_INSET, crate::frame::BUTTON_Y, crate::frame::BUTTON_SIZE, crate::frame::BUTTON_SIZE)
}

pub fn grip() -> Rect {
    Rect::new(0, 0, MINIMAP_SIZE.0, TITLE_HEIGHT)
}

pub fn default_position(viewport: (i32, i32)) -> (i32, i32) {
    crate::mainwnd::default_position(viewport)
}

pub fn initial_position(viewport: (i32, i32)) -> crate::initial_position::InitialPosition {
    crate::mainwnd::initial_position(viewport)
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
    fn tile_placement_follows_fun_100b9490() {
        assert_eq!(tile_placement(0, 0, (0, 0)), Some((Rect::new(0, 0, 328, 328), (0, 0))));
        assert_eq!(tile_placement(0, 0, (900, 0)), Some((Rect::new(0, 0, 124, 328), (900, 0))));
        assert_eq!(tile_placement(1, 0, (900, 0)), Some((Rect::new(124, 0, 204, 328), (0, 0))));
        assert_eq!(tile_placement(1, 0, (0, 0)), None);
        assert_eq!(tile_placement(0, 0, (0, 1000)), Some((Rect::new(0, 0, 328, 24), (0, 1000))));
        assert_eq!(tile_placement(0, 1, (0, 1000)), Some((Rect::new(0, 24, 328, 304), (0, 0))));
        assert_eq!(tile_placement(0, 2, (0, 1000)), None);
        assert_eq!(tile_placement(0, 1, (0, 1024)), Some((Rect::new(0, 0, 328, 328), (0, 0))));
    }

    #[test]
    fn the_world_maps_through_the_loadmaps_tables_and_the_fixed_spot() {
        assert_eq!(world_to_map(-131072, -262144), (0, 0));
        assert_eq!(world_to_map(83400, 148000), (1073, 2052));
        assert_eq!(player_map_pos(83400.7, 148000.2, -3500.0), (1073, 2052));
        assert_eq!(player_map_pos(45000.0, 245000.0, -6000.0), FIXED_SPOT_MAP);
        assert_eq!(player_map_pos(45000.0, 245000.0, -3000.0), world_to_map(45000, 245000));
        assert_eq!(grid_step(), 61);
    }

    #[test]
    fn the_scroll_centers_on_the_player_within_the_map_and_the_view_hits_its_parts() {
        assert_eq!(scroll_centered_on((100, 100)), (0, 0));
        assert_eq!(scroll_centered_on((1000, 2000)), (836, 1836));
        assert_eq!(scroll_centered_on((3000, 4000)), (1804 - 328, 2624 - 328));
        let view = MinimapView::with_sysstrings(|id| Some(format!("s{id}")));
        assert_eq!(view.hit(100, 100), Some(MinimapHit::Cover));
        assert_eq!(view.hit(100, 10), Some(MinimapHit::Title));
        assert_eq!(view.hit(100, 360), Some(MinimapButton::PartyMember.into()));
        assert_eq!(view.hit(260, 360), Some(MinimapHit::Button(MinimapButton::CurrentLoc)));
        assert_eq!(view.hit(400, 10), None);
        assert_eq!(default_position((1558, 933)), (0, 933 / 2 - 252));
        assert!(marker_visible(12, 12));
        assert!(!marker_visible(11, 12));
        assert!(marker_visible(316, 315));
        assert!(!marker_visible(317, 315));
        assert!(!marker_visible(316, 316));
    }

    #[test]
    fn a_party_marker_names_its_member_on_hover_and_while_it_is_the_picked_one() {
        let mut view = MinimapView::with_sysstrings(|id| Some(format!("s{id}")));
        view.party = vec![PartyMark { id: 7, world: (71_000, 140_000, 0), in_window: true, name: "Ana".into() }];
        view.scroll = scroll_for(0, marker_pos(0, 71_000, 140_000));
        let (index, (px, py)) = view.party_positions()[0];
        assert_eq!(index, 0);
        assert_eq!(view.party_tooltip(), None);
        view.mouse_move(COVER_RECT.x + px + 3, COVER_RECT.y + py + 3);
        assert_eq!(view.party_tooltip(), Some(("Ana".to_string(), (COVER_RECT.x + px, COVER_RECT.y + py))));
        view.mouse_move(COVER_RECT.x + px + PARTY_MARK_SIZE, COVER_RECT.y + py);
        assert_eq!(view.party_hover, None);
        view.highlight = Some(0);
        assert_eq!(view.party_tooltip().map(|tip| tip.0), Some("Ana".to_string()));
    }

    #[test]
    fn the_quest_tooltip_has_one_line_when_there_is_no_tracked_quest() {
        let font = BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &(0..95).map(|i| [i * 7, 6, 0, 13]).collect::<Vec<_>>())], (1024, 128))).unwrap());
        let mut view = MinimapView::with_sysstrings(|id| Some(format!("s{id}")));
        view.quest_target = Some(QuestMark { world: (71_000, 140_000, 0), name: "Mark".into(), title: "Quest".into(), id: 0 });
        view.scroll = scroll_for(0, marker_pos(0, 71_000, 140_000));
        view.quest_hover = true;
        let mut one = DrawList::default();
        view.paint_quest_tooltip(&mut one, 0, 0, &font);
        view.quest_target.as_mut().unwrap().id = 5;
        let mut two = DrawList::default();
        view.paint_quest_tooltip(&mut two, 0, 0, &font);
        let extent = |list: &DrawList| list.quads.iter().map(|q| (q.y + q.h) as i32).max().unwrap_or(0);
        assert!(extent(&two) > extent(&one));
    }

    #[test]
    fn the_clock_text_pads_and_switches_sun_and_moon_like_onpaint() {
        assert_eq!(clock_text(7.083), ("AM 07 : 04".to_string(), true));
        assert_eq!(clock_text(13.5), ("PM 01 : 30".to_string(), true));
        assert_eq!(clock_text(0.25), ("AM 12 : 15".to_string(), false));
        assert_eq!(clock_text(23.999), ("PM 11 : 59".to_string(), true));
        assert_eq!(clock_text(5.9999), ("AM 05 : 59".to_string(), false));
        assert_eq!(clock_text(4.0), ("AM 04 : 00".to_string(), false));
    }

    #[test]
    fn paint_draws_the_tiles_grid_markers_bar_and_buttons() {
        let font = font();
        let mut view = MinimapView::with_sysstrings(|id| Some(format!("s{id}")));
        view.player = Some((1000, 2000));
        view.targets = vec![(70_730, 139_465)];
        view.party = vec![PartyMark { id: 7, world: (71_000, 140_000, 0), in_window: true, name: "Ana".into() }, PartyMark { id: 8, world: (-1, -1, 0), in_window: true, name: String::new() }, PartyMark { id: 9, world: (70_900, 139_900, 0), in_window: false, name: String::new() }];
        view.zone = "Giran".into();
        view.period = 1;
        view.hour = Some(13.5);
        view.center_on((1000, 2000));
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font, 1.0);
        let tiles: Vec<_> = list.quads.iter().filter(|q| q.texture.starts_with(WORLD_MAP_PREFIX)).collect();
        assert_eq!(tiles.len(), 4);
        let marker = list.quads.iter().find(|q| q.texture == MINIMAP_POS).unwrap();
        assert_eq!((marker.x, marker.y), ((3 + 164 - 16) as f32, (21 + 164 - 16) as f32));
        assert_eq!(marker.rotation, compass_rotation(0));
        view.yaw.set(16384);
        let mut turned = DrawList::default();
        view.paint(&mut turned, 0, 0, &font, 1.0);
        let turned = turned.quads.iter().find(|q| q.texture == MINIMAP_POS).unwrap().rotation;
        assert!((turned - std::f32::consts::PI).abs() < 1e-5);
        assert_eq!(list.quads.iter().filter(|q| q.texture == PARTY_MARK).count(), 1);
        assert!(list.quads.iter().all(|q| q.texture != MAP_ICON_MARK));
        let target = list.quads.iter().find(|q| q.texture == TARGET_POS).unwrap();
        let target_map = world_to_map(70_730, 139_465);
        assert_eq!((target.x, target.y), ((3 + target_map.0 - view.scroll.0) as f32, (21 + target_map.1 - view.scroll.1) as f32));
        let party_map = world_to_map(71_000, 140_000);
        let party = list.quads.iter().find(|q| q.texture == PARTY_MARK).unwrap();
        assert_eq!((party.x, party.y, party.w), ((3 + party_map.0 - view.scroll.0) as f32, (21 + party_map.1 - view.scroll.1) as f32, 16.0));
        assert_eq!(list.quads.iter().filter(|q| q.texture == BTN_NORMAL).count(), 3);
        assert!(list.quads.iter().any(|q| q.texture == MAP_SHADOW && q.y == (21 + 0x136) as f32));
        let sun = list.quads.iter().find(|q| q.texture == MAP_SUN).unwrap();
        assert_eq!((sun.x, sun.y, sun.w), (7.0, 27.0, 25.0));
        let lines: Vec<_> = list.quads.iter().filter(|q| q.texture == crate::draw::LINE_TEXTURE).collect();
        let vertical = lines.iter().filter(|q| q.w == 1.0).count();
        let horizontal = lines.iter().filter(|q| q.h == 1.0).count();
        assert_eq!((vertical, horizontal), (6, 6));
        let first_vertical = lines.iter().find(|q| q.w == 1.0).unwrap();
        assert_eq!(first_vertical.x, (3 + 61 - 836 % 61) as f32);
        let text: Vec<_> = list.quads.iter().filter(|q| q.texture == font.atlas).collect();
        assert!(text.iter().any(|q| q.x == (3 + 0x16) as f32 && q.y == (21 + 328 - 15) as f32));
        assert!(text.iter().any(|q| q.x == (3 + 0x16) as f32 && q.y == (21 + 328 - 31) as f32));
        assert!(text.iter().any(|q| q.x == (3 + 0x1d) as f32 && q.y == (21 + 0x13) as f32));
    }

    #[test]
    fn the_quest_mark_is_drawn_twenty_pixels_up_without_a_visibility_test_and_has_a_hover_tooltip() {
        let font = font();
        let mut view = MinimapView::with_sysstrings(|id| Some(format!("s{id}")));
        view.quest_target = Some(QuestMark { world: (-131072 + 4000, -262144 + 4000, 0), name: "Gatekeeper".into(), title: "A Quest".into(), id: 5 });
        view.center_on((0, 0));
        let point = view.quest_point().unwrap();
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font, 1.0);
        let mark = list.quads.iter().find(|q| q.texture == MAP_ICON_MARK).unwrap();
        assert_eq!((mark.x, mark.y), ((3 + point.0 - view.scroll.0) as f32, (21 + point.1 - view.scroll.1 - 20) as f32));
        view.scroll = (point.0 + 400, point.1 + 400);
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font, 1.0);
        assert!(list.quads.iter().any(|q| q.texture == MAP_ICON_MARK));
        view.scroll = (point.0 - 100, point.1 - 100);
        view.mouse_move(3 + 105, 21 + 90);
        assert!(view.quest_hover);
        view.mouse_move(3 + 140, 21 + 90);
        assert!(!view.quest_hover);
        assert_eq!(quest_map_pos(0xa000, 0x3c000), FIXED_SPOT_MAP);
    }

    #[test]
    fn zone_ids_select_the_town_maps_like_100b9670() {
        let pairs = [(0x23, 1), (0x15, 2), (0x14, 3), (0x6b, 4), (0x30, 5), (0x3a, 6), (0x65, 7), (0x71, 8), (0x2b, 9), (0x4a, 10), (0x4d, 11), (0x76, 12), (0x7a, 12), (0x3e, 13), (0x40, 14), (0x123, 15), (0x10d, 16)];
        for (zone, map) in pairs {
            assert_eq!(zone_map(zone), map, "{zone:#x}");
        }
        assert_eq!((zone_map(-1), zone_map(1), zone_map(0x4b)), (0, 0, 0));
    }

    #[test]
    fn town_map_coordinates_subtract_the_origin_and_scale_by_a_twentieth() {
        assert_eq!(to_map(9, 83400, 148000), (493, 398));
        assert_eq!(to_map(0, 0, 0), (656, 1312));
        assert_eq!(marker_pos(9, 0xa000, 0x3c000), FIXED_SPOT_MAP);
        assert_eq!(player_pos(9, 0xa000, 0x3c000, 0), to_map(9, 0xa000, 0x3c000));
        assert_eq!(scroll_for(9, (493, 398)), (493 - 0xa4, 398 - 0xa4));
        assert_eq!(scroll_for(9, (940, 850)), (950 - 328, 860 - 328));
    }

    #[test]
    fn zoom_in_and_out_switch_between_the_world_map_and_a_town_map_like_100b9b30_and_100bb290() {
        let font = font();
        let mut view = MinimapView::with_sysstrings(|id| Some(format!("s{id}")));
        view.scroll = (1000, 1900);
        let rect = view.zoom_in_rect(8);
        assert_eq!(rect, Rect::new(1080 - 1000, 2053 - 1900, 26, 26));
        assert_eq!(view.hit(COVER_RECT.x + rect.x + 2, COVER_RECT.y + rect.y + 2), Some(MinimapHit::Button(MinimapButton::ZoomIn(8))));
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font, 1.0);
        let visible = (0..16).filter(|index| {
            let r = view.zoom_in_rect(*index);
            r.x + r.w > 0 && r.x < VIEW && r.y + r.h > 0 && r.y < VIEW
        }).count();
        assert_eq!(list.quads.iter().filter(|q| q.texture == ZOOM_IN_TEXTURES[0]).count(), visible);
        view.zoom_in(8, (83400, 148000));
        assert_eq!((view.map, view.saved_scroll, view.scroll), (9, (1000, 1900), (493 - 0xa4, 398 - 0xa4)));
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 0, &font, 1.0);
        let town = list.quads.iter().find(|q| q.texture == "L2Font-e.TownMap.map_giran_castle_town").unwrap();
        assert_eq!((town.u, town.v, town.w, town.uw), ((493 - 0xa4) as f32, (398 - 0xa4) as f32, 328.0, 328.0));
        assert!(list.quads.iter().all(|q| !q.texture.starts_with(WORLD_MAP_PREFIX) && q.texture != ZOOM_IN_TEXTURES[0]));
        assert_eq!(list.quads.iter().filter(|q| q.texture == ZOOM_OUT_TEXTURES[0]).count(), 1);
        let out = ZOOM_OUT_RECT;
        assert_eq!(view.hit(COVER_RECT.x + out.x + 1, COVER_RECT.y + out.y + 1), Some(MinimapHit::Button(MinimapButton::ZoomOut)));
        view.zoom_out((83400, 148000, -3400));
        assert_eq!((view.map, view.scroll), (0, scroll_for(0, player_pos(0, 83400, 148000, -3400))));
        view.zoom_in(8, (-200_000, 0));
        assert_eq!((view.map, view.scroll), (9, (0, 0)));
    }

    #[test]
    fn showing_the_map_selects_the_town_of_the_current_zone_and_centres_on_the_player() {
        let mut view = MinimapView::with_sysstrings(|id| Some(format!("s{id}")));
        view.show(0x2b, (83400, 148000, -3400));
        assert_eq!((view.map, view.scroll), (9, (493 - 0xa4, 398 - 0xa4)));
        view.show(-1, (83400, 148000, -3400));
        assert_eq!((view.map, view.scroll), (0, scroll_for(0, to_map(0, 83400, 148000))));
        view.center_world((83400, 148000, -3400), 9, true);
        assert_eq!(view.map, 0);
        view.map = 9;
        view.center_world((83400, 148000, -3400), 9, true);
        assert_eq!((view.map, view.scroll), (9, (493 - 0xa4, 398 - 0xa4)));
    }

    #[test]
    fn dragging_the_map_moves_the_scroll_by_the_travel_and_stops_at_the_map_edges() {
        let mut view = MinimapView { map: 1, scroll: (100, 100), ..Default::default() };
        let at = |x: i32, y: i32| (COVER_RECT.x + x, COVER_RECT.y + y);
        let (px, py) = at(50, 50);
        assert!(view.press_map(px, py));
        let (mx, my) = at(30, 60);
        view.mouse_move(mx, my);
        assert_eq!(view.scroll, (120, 90));
        let (mx, my) = at(1, 1);
        view.mouse_move(mx, my);
        assert_eq!(view.scroll, (149, 149));
        let (mx, my) = at(VIEW, VIEW);
        view.mouse_move(mx, my);
        assert_eq!(view.scroll, (0, 0));
        view.scroll = (600, 600);
        let (mx, my) = at(1, 1);
        view.mouse_move(mx, my);
        assert_eq!(view.scroll, (map_size(1).0 - VIEW, map_size(1).1 - VIEW));
        view.release_map();
        let before = view.scroll;
        let (mx, my) = at(200, 200);
        view.mouse_move(mx, my);
        assert_eq!(view.scroll, before);
        assert!(!view.press_map(COVER_RECT.x, COVER_RECT.y));
        assert!(!view.press_map(COVER_RECT.x + VIEW + 1, COVER_RECT.y + 5));
    }

    #[test]
    fn hovering_ten_pixels_around_the_players_mark_shows_the_name_tooltip_kept_inside_the_view() {
        let font = font();
        let mut view = MinimapView { map: 1, scroll: (100, 100), player: Some((200, 200)), player_name: "Wender".into(), ..Default::default() };
        let at = |x: i32, y: i32| (COVER_RECT.x + x, COVER_RECT.y + y);
        let (x, y) = at(110, 90);
        view.mouse_move(x, y);
        assert!(view.player_hover);
        let (x, y) = at(111, 100);
        view.mouse_move(x, y);
        assert!(!view.player_hover);
        let (x, y) = at(100, 100);
        view.mouse_move(x, y);
        let mut list = DrawList::default();
        view.paint_player_tooltip(&mut list, 0, 0, &font);
        assert!(!list.quads.is_empty());
        view.player = Some((100 + VIEW - 5, 150));
        let (x, y) = at(VIEW - 5, 50);
        view.mouse_move(x, y);
        assert!(!view.player_hover);
        view.player = Some((100 + 8, 150));
        let (x, y) = at(8, 50);
        view.mouse_move(x, y);
        assert!(!view.player_hover, "a marca fora de 12 px da borda não tem dica");
    }
}
