use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::html::{self, Line};
use crate::npc_viewer::paint_html_lines;

pub const VIEWER_SIZE: (i32, i32) = (400, 550);
pub const BACKGROUND_SIZE: (i32, i32) = (1024, 768);
pub const HTML_PREFIX: &str = "..\\L2Text\\";
pub const FONT_TEXTURE: &str = "L2UI.CreditWnd.CreditFont";
pub const FONT_FILE: &str = "CreditFont.gly";
pub const ESCAPE: char = '\u{1b}';

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditEntry {
    pub id: u32,
    pub html: String,
    pub image: String,
    pub time: u32,
    pub align: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreditEvent {
    None,
    Closed,
}

pub fn viewer_origin(screen: (i32, i32), align: u32) -> (i32, i32) {
    let (x, y) = screen;
    let left = 0x1e;
    let right = x - 0x1ae;
    let center_x = x / 2 - 200;
    let top = 0x1e;
    let middle = y / 2 - 0x113;
    let bottom = y - 0x244;
    match align {
        2 => (center_x, top),
        3 => (right, top),
        4 => (left, middle),
        5 => (center_x, middle),
        6 => (right, middle),
        7 => (left, bottom),
        8 => (center_x, bottom),
        9 => (right, bottom),
        _ => (left, top),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreditScreen {
    pub screen: (i32, i32),
    pub entries: Vec<CreditEntry>,
    pub visible: bool,
    pub index: u32,
    pub elapsed: f32,
    pub lines: Vec<Line>,
    pub html_request: Option<String>,
    pub back: Option<String>,
}

impl CreditScreen {
    pub fn new(screen: (i32, i32), entries: Vec<CreditEntry>) -> CreditScreen {
        CreditScreen { screen, entries, visible: false, index: 0, elapsed: 0.0, lines: Vec::new(), html_request: None, back: None }
    }

    pub fn entry(&self) -> Option<&CreditEntry> {
        self.entries.iter().find(|entry| entry.id == self.index)
    }

    pub fn show(&mut self) {
        self.visible = true;
        self.index = 0;
        self.elapsed = 0.0;
        self.prepare();
    }

    fn prepare(&mut self) {
        self.lines.clear();
        let found = self.entry().map(|entry| (format!("{HTML_PREFIX}{}", entry.html), entry.image.clone()));
        self.html_request = found.as_ref().map(|(html, _)| html.clone());
        if let Some((_, image)) = found.filter(|(_, image)| image.chars().count() > 1) {
            self.back = Some(image);
        }
    }

    pub fn set_html(&mut self, text: &str, font: &BitmapFont) {
        self.set_html_with(text, font, &|_, _| None);
    }

    pub fn set_html_with(&mut self, text: &str, font: &BitmapFont, resolve: &dyn Fn(html::Marker, i32) -> Option<String>) {
        self.lines = html::layout(&html::parse_with(text, resolve), VIEWER_SIZE.0, font);
    }

    fn leave(&mut self) -> CreditEvent {
        self.visible = false;
        self.index = 0;
        self.elapsed = 0.0;
        self.lines.clear();
        self.html_request = None;
        CreditEvent::Closed
    }

    pub fn next_entry(&mut self) -> CreditEvent {
        self.elapsed = 0.0;
        self.index += 1;
        if (self.index as usize) < self.entries.len() {
            self.prepare();
            CreditEvent::None
        } else {
            self.leave()
        }
    }

    pub fn tick(&mut self, dt: f32) -> CreditEvent {
        if !self.visible {
            return CreditEvent::None;
        }
        self.elapsed += dt;
        match self.entry() {
            Some(entry) if (entry.time as f32) < self.elapsed => self.next_entry(),
            _ => CreditEvent::None,
        }
    }

    pub fn click(&mut self) -> CreditEvent {
        if self.visible { self.next_entry() } else { CreditEvent::None }
    }

    pub fn key(&mut self, key: char) -> CreditEvent {
        if self.visible && key == ESCAPE { self.leave() } else { CreditEvent::None }
    }

    pub fn paint(&self, list: &mut DrawList, font: &BitmapFont) {
        if !self.visible {
            return;
        }
        let Some(entry) = self.entry() else { return };
        if let Some(image) = &self.back {
            list.draw_texture(0, 0, self.screen.0, self.screen.1, 0, 0, BACKGROUND_SIZE.0, BACKGROUND_SIZE.1, image, 1.0);
        }
        let (x, y) = viewer_origin(self.screen, entry.align);
        paint_html_lines(list, x, y, VIEWER_SIZE, &self.lines, 0, font, 1.0);
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

    fn entries() -> Vec<CreditEntry> {
        [(0u32, 5u32), (1, 3), (2, 9)]
            .into_iter()
            .map(|(id, align)| CreditEntry { id, html: format!("credit_{id}.htm"), image: format!("L2UI_CH3.credit_{id}"), time: 10, align })
            .collect()
    }

    #[test]
    fn credits_html_runs_the_marker_pass_like_the_other_viewers() {
        let font = font();
        let mut screen = CreditScreen::new((1024, 768), entries());
        let mut expected = CreditScreen::new((1024, 768), entries());
        screen.set_html_with("<html><body>&$7; and &@30045;</body></html>", &font, &|marker, id| Some(format!("{marker:?}{id}")));
        expected.set_html("<html><body>SysString7 and NpcName30045</body></html>", &font);
        assert_eq!(screen.lines, expected.lines);
    }

    #[test]
    fn the_viewer_origin_follows_the_nine_alignments_of_the_original_switch() {
        let screen = (1024, 768);
        assert_eq!(viewer_origin(screen, 1), (30, 30));
        assert_eq!(viewer_origin(screen, 0), (30, 30));
        assert_eq!(viewer_origin(screen, 2), (312, 30));
        assert_eq!(viewer_origin(screen, 3), (594, 30));
        assert_eq!(viewer_origin(screen, 4), (30, 109));
        assert_eq!(viewer_origin(screen, 5), (312, 109));
        assert_eq!(viewer_origin(screen, 6), (594, 109));
        assert_eq!(viewer_origin(screen, 7), (30, 188));
        assert_eq!(viewer_origin(screen, 8), (312, 188));
        assert_eq!(viewer_origin(screen, 9), (594, 188));
    }

    #[test]
    fn showing_asks_for_the_first_file_and_the_timer_advances_after_the_entry_time_has_passed() {
        let mut credits = CreditScreen::new((1024, 768), entries());
        assert_eq!(credits.tick(5.0), CreditEvent::None);
        assert!(credits.html_request.is_none());
        credits.show();
        assert_eq!(credits.html_request.as_deref(), Some("..\\L2Text\\credit_0.htm"));
        assert_eq!(credits.tick(10.0), CreditEvent::None);
        assert_eq!(credits.index, 0);
        assert_eq!(credits.tick(0.1), CreditEvent::None);
        assert_eq!((credits.index, credits.elapsed), (1, 0.0));
        assert_eq!(credits.html_request.as_deref(), Some("..\\L2Text\\credit_1.htm"));
    }

    #[test]
    fn a_click_skips_to_the_next_entry_and_the_last_one_closes_the_window() {
        let mut credits = CreditScreen::new((1024, 768), entries());
        credits.show();
        assert_eq!(credits.click(), CreditEvent::None);
        assert_eq!(credits.click(), CreditEvent::None);
        assert_eq!(credits.index, 2);
        assert_eq!(credits.click(), CreditEvent::Closed);
        assert!(!credits.visible);
        assert_eq!(credits.index, 0);
        assert_eq!(credits.click(), CreditEvent::None);
    }

    #[test]
    fn escape_closes_at_once_and_other_keys_do_nothing() {
        let mut credits = CreditScreen::new((1024, 768), entries());
        credits.show();
        assert_eq!(credits.key('a'), CreditEvent::None);
        assert!(credits.visible);
        assert_eq!(credits.key(ESCAPE), CreditEvent::Closed);
        assert!(!credits.visible);
    }

    #[test]
    fn paint_draws_the_image_over_1024_by_768_and_the_html_at_the_aligned_origin() {
        let mut credits = CreditScreen::new((1024, 768), entries());
        let mut list = DrawList { capture_text: true, ..Default::default() };
        credits.paint(&mut list, &font());
        assert!(list.quads.is_empty());
        credits.show();
        credits.set_html("<html><body>Production</body></html>", &font());
        credits.paint(&mut list, &font());
        let back = &list.quads[0];
        assert_eq!((back.texture.as_str(), back.w, back.h), ("L2UI_CH3.credit_0", 1024.0, 768.0));
        assert!(list.text_runs.iter().any(|run| run.text.contains("Production") && run.rect[0] >= 312.0));
    }

    #[test]
    fn a_page_without_picture_keeps_the_previous_one_and_the_picture_fills_the_window() {
        let mut items = entries();
        items[1].image = String::new();
        let mut credits = CreditScreen::new((800, 600), items);
        credits.show();
        assert_eq!(credits.back.as_deref(), Some("L2UI_CH3.credit_0"));
        credits.click();
        assert_eq!(credits.back.as_deref(), Some("L2UI_CH3.credit_0"));
        let mut list = DrawList { capture_text: true, ..Default::default() };
        credits.paint(&mut list, &font());
        let back = &list.quads[0];
        assert_eq!((back.texture.as_str(), back.w, back.h, back.uw, back.vh), ("L2UI_CH3.credit_0", 800.0, 600.0, 1024.0, 768.0));
        credits.click();
        assert_eq!(credits.back.as_deref(), Some("L2UI_CH3.credit_2"));
    }
}
