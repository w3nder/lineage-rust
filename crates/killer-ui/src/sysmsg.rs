use crate::chat::{
    CHAT_LINE, CHAT_WIDTH, ChatEntry, LINE_INDENT, LINE_WRAP_INSET, SYSTEM_GROUP_DAMAGE, SYSTEM_GROUP_USE_ITEMS,
    wrap_line,
};
use crate::color::{Argb, CHAT_SYSTEM};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::ini::IniFile;
use crate::window::Rect;

pub const SYSTEM_MSG_SIZE: (i32, i32) = (CHAT_WIDTH, 125);
pub const INITIAL_POSITION: crate::initial_position::InitialPosition = crate::initial_position::DISABLED_INITIAL_POSITION;
pub const SYSTEM_MSG_RISE: i32 = 130;
pub const SYSTEM_MSG_BACK: &str = "L2UI_CH3.ChatWnd.chatting_system";
pub const SYSTEM_MSG_BACK_VH: i32 = 128;
pub const LINE_TEXT_X: i32 = 23;
pub const LINE_TEXT_Y: i32 = 14;
pub const LIST_BOTTOM_INSET: i32 = 10;
pub const SCROLL_UP_RECT: Rect = Rect::new(5, 5, 15, 15);
pub const SCROLL_TRACK_Y: i32 = 20;
pub const CAPACITY: usize = 200;
pub const SYSTEM_GROUP_CENTER_WARNING: u32 = 6;
pub const SYSTEM_GROUP_SKIPPED: std::ops::RangeInclusive<u32> = 0xb..=0x10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SystemMsgOptions {
    pub shown: bool,
    pub damage: bool,
    pub expendable_items: bool,
}

impl Default for SystemMsgOptions {
    fn default() -> Self {
        SystemMsgOptions { shown: false, damage: true, expendable_items: true }
    }
}

impl SystemMsgOptions {
    pub fn from_ini(ini: &IniFile) -> SystemMsgOptions {
        SystemMsgOptions {
            shown: ini.bool("Game", "SystemMsgWnd").unwrap_or(false),
            damage: ini.bool("Game", "SystemMsgWndDamage").unwrap_or(true),
            expendable_items: ini.bool("Game", "SystemMsgWndExpendableItem").unwrap_or(true),
        }
    }

    pub fn accepts(&self, entry: &ChatEntry) -> bool {
        if entry.kind != CHAT_SYSTEM {
            return false;
        }
        match entry.group {
            SYSTEM_GROUP_DAMAGE => self.damage,
            SYSTEM_GROUP_USE_ITEMS => self.expendable_items,
            SYSTEM_GROUP_CENTER_WARNING => false,
            group if SYSTEM_GROUP_SKIPPED.contains(&group) => false,
            _ => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemMsgView {
    pub options: SystemMsgOptions,
    pub scroll: usize,
    pub scroll_pressed: Option<crate::list_scroll::ListScrollHit>,
    pub last_rows: std::cell::Cell<usize>,
    pub scroll_grab: Option<i32>,
    pub rows_cache: crate::chat::RowsCache,
    pub list: Vec<ChatEntry>,
    pub stored: bool,
}

impl SystemMsgView {
    pub fn new(options: SystemMsgOptions) -> SystemMsgView {
        SystemMsgView { options, scroll: 0, scroll_pressed: None, last_rows: std::cell::Cell::new(0), scroll_grab: None, rows_cache: crate::chat::RowsCache::default(), list: Vec::new(), stored: false }
    }

    pub fn ingest(&mut self, entry: &ChatEntry) {
        if self.options.accepts(entry) {
            self.list.push(entry.clone());
            if self.list.len() > CAPACITY {
                self.list.remove(0);
            }
        }
    }

    fn entries<'a>(&'a self, lines: &'a [ChatEntry]) -> impl Iterator<Item = &'a ChatEntry> {
        let source = if self.stored { self.list.as_slice() } else { lines };
        source.iter().filter(|entry| self.stored || self.options.accepts(entry))
    }

    pub fn visible_lines(&self) -> usize {
        ((SYSTEM_MSG_SIZE.1 - LIST_BOTTOM_INSET) / CHAT_LINE).max(0) as usize
    }

    pub fn cached_rows(&self, lines: &[ChatEntry], font: &BitmapFont) -> std::sync::Arc<Vec<crate::chat::ChatRow>> {
        let key = crate::chat::rows_hash(self.entries(lines), font);
        self.rows_cache.get_or_build(key, || self.rows(lines, font))
    }

    pub fn rows(&self, lines: &[ChatEntry], font: &BitmapFont) -> Vec<(i32, String, Argb)> {
        let width = CHAT_WIDTH - LINE_WRAP_INSET;
        self.entries(lines)
            .flat_map(|entry| {
                let color = entry.color();
                wrap_line(&entry.line(), width, LINE_INDENT, font)
                    .into_iter()
                    .map(move |(indent, text)| (indent, text, color))
            })
            .collect()
    }

    pub fn scroll_press(&mut self, x: i32, y: i32) -> bool {
        let Some(hit) = crate::list_scroll::hit(SYSTEM_MSG_SIZE.1, x, y) else { return false };
        self.scroll_pressed = Some(hit);
        let max = self.last_rows.get().saturating_sub(self.visible_lines());
        let step = match hit {
            crate::list_scroll::ListScrollHit::Up => 1,
            crate::list_scroll::ListScrollHit::Down => -1,
            crate::list_scroll::ListScrollHit::Track => 0,
        };
        self.scroll = (self.scroll as i64 + step).clamp(0, max as i64) as usize;
        if hit == crate::list_scroll::ListScrollHit::Track {
            self.scroll_grab = crate::list_scroll::press_thumb(SYSTEM_MSG_SIZE.1, self.last_rows.get(), self.visible_lines(), self.scroll, y - crate::list_scroll::TRACK_Y);
        }
        true
    }

    pub fn scroll_drag_to(&mut self, y: i32) -> bool {
        let Some(grab) = self.scroll_grab else { return false };
        if let Some(offset) = crate::list_scroll::drag(SYSTEM_MSG_SIZE.1, self.last_rows.get(), self.visible_lines(), self.scroll, grab, y - crate::list_scroll::TRACK_Y) {
            self.scroll = offset;
        }
        true
    }

    pub fn scroll_by(&mut self, lines: &[ChatEntry], font: &BitmapFont, delta: i32) {
        let rows = self.cached_rows(lines, font).len();
        let max = rows.saturating_sub(self.visible_lines());
        self.scroll = (self.scroll as i64 + i64::from(delta)).clamp(0, max as i64) as usize;
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, lines: &[ChatEntry], font: &BitmapFont, alpha: f32) {
        let (w, h) = SYSTEM_MSG_SIZE;
        list.draw_texture(x, y, w, h, 0, 0, w, SYSTEM_MSG_BACK_VH, SYSTEM_MSG_BACK, alpha);
        let rows = self.cached_rows(lines, font);
        self.last_rows.set(rows.len());
        let end = rows.len().saturating_sub(self.scroll);
        let start = end.saturating_sub(self.visible_lines());
        for (i, (indent, text, color)) in rows[start..end].iter().enumerate() {
            let row_y = y + LINE_TEXT_Y + CHAT_LINE * i as i32;
            list.draw_text_clipped(x + LINE_TEXT_X + indent, row_y, w - LINE_TEXT_X - indent, *color, text, font);
        }
        crate::list_scroll::paint(list, x, y, h, crate::list_scroll::ListScrollState { total: rows.len(), visible: self.visible_lines(), offset: self.scroll, pressed: self.scroll_pressed });
    }
}

pub fn position_over_chat(chat: Rect) -> (i32, i32) {
    (chat.x, chat.y - SYSTEM_MSG_RISE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;
    use crate::font::{Glyphs, gly};
    use crate::ini::parse_ini;

    const TEXTURES: &str = include_str!("../tests/fixtures/nwindow-texturas-sysmsg.txt");
    const OPTION: &str = include_str!("../tests/fixtures/option.ini");

    fn font() -> BitmapFont {
        let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, if i == 0 { 3 } else { 6 }, 0, 13]).collect();
        BitmapFont::new("f".into(), Glyphs::parse(&gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
    }

    fn system(text: &str, group: u32) -> ChatEntry {
        ChatEntry { kind: CHAT_SYSTEM, name: String::new(), text: text.into(), group, color: Some(Argb(0xFFB09B79)) }
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    #[test]
    fn cached_rows_follow_the_accepted_lines() {
        let font = font();
        let view = SystemMsgView::new(SystemMsgOptions { shown: true, damage: false, expendable_items: true });
        let mut lines = vec![system("Welcome.", 0), system("You hit for 12.", 3)];
        let first = view.cached_rows(&lines, &font);
        assert_eq!(*first, view.rows(&lines, &font));
        assert!(std::sync::Arc::ptr_eq(&first, &view.cached_rows(&lines, &font)));
        lines[1].text = "You hit for 99.".into();
        assert!(std::sync::Arc::ptr_eq(&first, &view.cached_rows(&lines, &font)));
        lines.push(system("A new announcement.", 0));
        let next = view.cached_rows(&lines, &font);
        assert!(!std::sync::Arc::ptr_eq(&first, &next));
        assert_eq!(*next, view.rows(&lines, &font));
    }

    #[test]
    fn options_come_from_the_game_section_and_route_the_groups() {
        let options = SystemMsgOptions::from_ini(&parse_ini(OPTION));
        assert_eq!(options, SystemMsgOptions { shown: true, damage: true, expendable_items: true });
        let off = SystemMsgOptions { shown: true, damage: false, expendable_items: false };
        assert!(off.accepts(&system("Welcome.", 0)));
        assert!(off.accepts(&system("x", 2)));
        assert!(!off.accepts(&system("You hit for 12.", 3)));
        assert!(!off.accepts(&system("Soulshot used.", 7)));
        assert!(!off.accepts(&system("warning", 6)));
        assert!(!off.accepts(&system("x", 0xc)));
        assert!(SystemMsgOptions::default().accepts(&system("You hit for 12.", 3)));
        assert!(!off.accepts(&ChatEntry::new(0, "a", "b")));
        assert_eq!(SystemMsgOptions::from_ini(&parse_ini("")), SystemMsgOptions::default());
    }

    #[test]
    fn the_panel_is_the_chat_width_by_125_above_the_chat_with_seven_lines_from_23_14() {
        let font = font();
        let view = SystemMsgView::new(SystemMsgOptions::default());
        assert_eq!(view.visible_lines(), 7);
        assert_eq!(position_over_chat(Rect::new(0, 746, 348, 187)), (0, 616));
        let lines: Vec<ChatEntry> = (0..9).map(|i| system(&format!("m{i}"), 0)).collect();
        let mut list = DrawList::default();
        view.paint(&mut list, 0, 616, &lines, &font, 0.5);
        let back = with_texture(&list.quads, SYSTEM_MSG_BACK);
        assert_eq!(
            (back[0].x, back[0].y, back[0].w, back[0].h, back[0].uw, back[0].vh, back[0].color[3]),
            (0.0, 616.0, 348.0, 125.0, 348.0, 128.0, 0.5)
        );
        let text: Vec<&Quad> = list.quads.iter().filter(|q| q.texture == "f").collect();
        assert_eq!(text.len(), 14);
        assert_eq!((text[0].x, text[0].y), (23.0, 616.0 + 14.0));
        assert_eq!(text[12].y, 616.0 + 14.0 + 15.0 * 6.0);
        assert!(text.iter().all(|q| q.color == Argb(0xFFB09B79).rgba()));
        assert_eq!(text[1].u, (('2' as i32 - 0x20) * 7) as f32);
    }

    #[test]
    fn long_lines_wrap_with_the_chat_indent_and_the_wheel_scrolls_within_the_rows() {
        let font = font();
        let mut view = SystemMsgView::new(SystemMsgOptions::default());
        let lines = vec![system(&"word ".repeat(80), 0), system("short", 0)];
        let rows = view.rows(&lines, &font);
        assert!(rows.len() > 8);
        assert_eq!(rows[1].0, LINE_INDENT);
        assert_eq!(rows.last().unwrap().1, "short");
        view.scroll_by(&lines, &font, 100);
        assert_eq!(view.scroll, rows.len() - 7);
        view.scroll_by(&lines, &font, -1);
        assert_eq!(view.scroll, rows.len() - 8);
        view.scroll_by(&lines, &font, -100);
        assert_eq!(view.scroll, 0);
        let mut list = DrawList::default();
        view.scroll = rows.len() - 7;
        view.paint(&mut list, 0, 0, &lines, &font, 1.0);
        assert!(list.quads.iter().filter(|q| q.texture == "f").all(|q| q.x + q.w <= 348.0));
    }

    #[test]
    fn every_texture_exists_in_the_client_list() {
        let known: Vec<String> = TEXTURES.lines().map(|l| l.trim().to_lowercase()).collect();
        let mut list = DrawList::default();
        SystemMsgView::new(SystemMsgOptions::default()).paint(&mut list, 0, 0, &[], &font(), 1.0);
        for texture in list.quads.iter().map(|q| q.texture.as_str()) {
            assert!(known.contains(&texture.to_lowercase()), "textura desconhecida {texture}");
        }
    }
}
