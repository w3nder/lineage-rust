#[path = "tutorial_layout_generated.rs"]
mod layout;

use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode, TITLE_HEIGHT};
use crate::html::{self, Line};
use crate::npc_viewer::paint_html_lines;
use crate::window::Rect;

pub const VIEWER_WIDTH: i32 = layout::SIZE.0;
pub const VIEWER_CREATE_HEIGHT: i32 = layout::SIZE.1;
pub const VIEWER_RIGHT_MARGIN: i32 = 20;
pub const VIEWER_Y: i32 = 20;
pub const VIEWER_STYLE: u32 = 0x5002;
pub const VIEWER_MIN_HTML: i32 = 0x100;
pub const VIEWER_MAX_HTML: i32 = 0x2a0;
pub const VIEWER_HTML_PAD: i32 = 8;
pub const HTML_POS: (i32, i32) = (8, 32);
pub const HTML_INSET: (i32, i32) = (8 + 7, 12 + 9 + 20);
pub const BACK1: &str = "l2ui_ch3.tutorialwnd.tutorial_back1";
pub const BACK2: &str = "l2ui_ch3.tutorialwnd.tutorial_back2";
pub const BACK3: &str = "l2ui_ch3.tutorialwnd.tutorial_back3";
pub const BACK_TOP_H: i32 = 12;
pub const BACK_ROW_H: i32 = 12;
pub const BACK_BOTTOM_H: i32 = 9;
pub const HTML_WHEEL_STEP: i32 = crate::npc_viewer::HTML_WHEEL_STEP;

pub const BUTTON_X: i32 = 5;
pub const BUTTON_SIZE: i32 = 32;
pub const BUTTON_STYLE: u32 = 0x80002;
pub const BUTTON_NORMAL: &str = "L2UI_ch3.ConsoleBtn.TutorialBtn";
pub const BUTTON_DOWN: &str = "L2UI_ch3.ConsoleBtn.TutorialBtn_down";
pub const BUTTON_LIGHT: &str = "L2UI_ch3.ConsoleBtn.TutorialBtn_light";
pub const QUEST_BUTTON_NORMAL: &str = "L2UI_ch3.ConsoleBtn.QuestBtn";
pub const QUEST_BUTTON_DOWN: &str = "L2UI_ch3.ConsoleBtn.QuestBtn_down";
pub const QUEST_BUTTON_LIGHT: &str = "L2UI_ch3.ConsoleBtn.QuestBtn_light";
pub const QUEST_BUTTON_X: i32 = 42;
pub const MAIL_BUTTON_NORMAL: &str = "L2UI_ch3.ConsoleBtn.MailBtn";
pub const MAIL_BUTTON_DOWN: &str = "L2UI_ch3.ConsoleBtn.MailBtn_down";
pub const MAIL_BUTTON_LIGHT: &str = "L2UI_ch3.ConsoleBtn.MailBtn_light";
pub const MAIL_BUTTON_X: i32 = 79;
pub const MAIL_BOARD_TAB: usize = 5;
pub const GLOW1: &str = "l2ui_ch3.etc.glowbtn1";
pub const GLOW2: &str = "l2ui_ch3.etc.glowbtn2";
pub const GLOW_SIZE: i32 = 128;
pub const GLOW_OFFSET: i32 = -48;
pub const BLINK_MS: u32 = 0x1f4;
pub const FADE_MS: u32 = 10;
pub const GROW_MS: u32 = 0x32;
pub const FADE_STEP: i32 = 15;
pub const GROW_STEP: i32 = 5;
pub const GROW_START: i32 = 0x32;

pub const EVENT_MOVE: i32 = 0x1;
pub const EVENT_CAMERA_ROTATE: i32 = 0x2;
pub const EVENT_ZOOM: i32 = 0x4;
pub const EVENT_DEFAULT_CAMERA: i32 = 0x8;
pub const EVENT_TURN_BACK: i32 = 0x10;
pub const EVENT_QUEST_TAB: i32 = 0x2000;
pub const CAMERA_EVENT_MIN_UNITS: i32 = 100;

pub fn viewer_position(viewport: (i32, i32)) -> (i32, i32) {
    (viewport.0 - VIEWER_WIDTH - VIEWER_RIGHT_MARGIN, VIEWER_Y)
}

pub const VIEWER_INITIAL_POSITION: crate::initial_position::InitialPosition = crate::initial_position::InitialPosition {
    enabled: true, horizontal_mode: 1, vertical_mode: 3, x: VIEWER_WIDTH + VIEWER_RIGHT_MARGIN, y: VIEWER_Y,
};
pub const BUTTON_BOTTOM: i32 = 187 + 32 + 5;
pub const BUTTON_INITIAL_POSITION: crate::initial_position::InitialPosition = crate::initial_position::InitialPosition {
    enabled: true, horizontal_mode: 0, vertical_mode: 4, x: BUTTON_X, y: BUTTON_BOTTOM,
};

pub fn viewer_height(frame_max_height: i32) -> i32 {
    frame_max_height.clamp(VIEWER_MIN_HTML, VIEWER_MAX_HTML) + VIEWER_HTML_PAD
}

pub fn html_size(height: i32) -> (i32, i32) {
    (VIEWER_WIDTH - HTML_INSET.0, height - HTML_INSET.1)
}

pub fn viewer_grip() -> Rect {
    Rect::new(0, 0, VIEWER_WIDTH, TITLE_HEIGHT)
}

pub fn viewer_frame() -> FrameCtrl {
    FrameCtrl::new(FrameMode::Title)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TutorialViewer {
    pub lines: Vec<Line>,
    pub doc_height: i32,
    pub height: i32,
    pub scroll: i32,
    pub html_bounds: Rect,
    pub body_bounds: Rect,
    pub bottom_y: i32,
}

impl Default for TutorialViewer {
    fn default() -> Self {
        TutorialViewer { lines: Vec::new(), doc_height: 0, height: VIEWER_CREATE_HEIGHT, scroll: 0,
            html_bounds: layout::RECT_E8,
            body_bounds: Rect::new(0, 32, VIEWER_WIDTH, VIEWER_CREATE_HEIGHT - 41),
            bottom_y: VIEWER_CREATE_HEIGHT - 9,
        }
    }
}

impl TutorialViewer {
    pub fn load(&mut self, html_text: &str, font: &BitmapFont) -> bool {
        self.load_with(html_text, font, &|_, _| None)
    }

    pub fn load_with(&mut self, html_text: &str, font: &BitmapFont, resolve: &dyn Fn(html::Marker, i32) -> Option<String>) -> bool {
        if html_text.is_empty() {
            return false;
        }
        let doc = html::parse_with(html_text, resolve);
        self.lines = html::layout(&doc, html_size(self.height).0, font);
        self.doc_height = html::height(&self.lines, font);
        self.height = viewer_height(self.doc_height);
        self.html_bounds.h = self.height - 41;
        self.body_bounds.h = self.height - 41;
        self.bottom_y = self.height - 9;
        self.scroll = 0;
        true
    }

    pub fn max_scroll(&self) -> i32 {
        (self.doc_height - self.html_bounds.h).max(0)
    }

    pub fn scroll_by(&mut self, dy: i32) {
        self.scroll = (self.scroll + dy).clamp(0, self.max_scroll());
    }

    pub fn html_rect(&self) -> Rect {
        self.html_bounds
    }

    pub fn link_message_at(&self, x: i32, y: i32) -> Option<&str> {
        let rect = self.html_rect();
        if !rect.contains(x, y) {
            return None;
        }
        html::link_message_at(&self.lines, x - rect.x, y - rect.y + self.scroll)
    }

    pub fn link_at(&self, x: i32, y: i32, font: &BitmapFont) -> Option<&str> {
        let rect = self.html_rect();
        if !rect.contains(x, y) {
            return None;
        }
        html::link_at(&self.lines, x - rect.x, y - rect.y + self.scroll, font)
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, font: &BitmapFont, alpha: f32) {
        let w = VIEWER_WIDTH;
        viewer_frame().paint(list, x, y, w, TITLE_HEIGHT, font, alpha);
        list.draw_texture(x, y + TITLE_HEIGHT, w, BACK_TOP_H, 0, 0, w, BACK_TOP_H, BACK1, alpha);
        let body = self.body_bounds.h;
        let rows = body / BACK_ROW_H;
        let top = self.body_bounds.y;
        for row in 0..rows {
            list.draw_texture(x, y + top + row * BACK_ROW_H, w, BACK_ROW_H, 0, 0, w, BACK_ROW_H, BACK2, alpha);
        }
        let rest = body - rows * BACK_ROW_H;
        list.draw_texture(x, y + top + rows * BACK_ROW_H, w, rest, 0, 0, w, rest, BACK2, alpha);
        list.draw_texture(x, y + self.bottom_y, w, BACK_BOTTOM_H, 0, 0, w, BACK_BOTTOM_H, BACK3, alpha);
        let rect = self.html_rect();
        paint_html_lines(list, x + rect.x, y + rect.y, (rect.w, rect.h), &self.lines, self.scroll, font, alpha);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EffectButton {
    pub mark_id: i32,
    pub blink: bool,
    pub effect: bool,
    pub rising: bool,
    pub alpha: i32,
    pub scale: i32,
    blink_ms: u32,
    fade_ms: u32,
    grow_ms: u32,
}

impl EffectButton {
    pub fn begin_effect(&mut self, mark_id: i32) {
        self.mark_id = mark_id;
        self.scale = GROW_START;
        self.alpha = 0;
        self.rising = true;
        self.effect = true;
        self.blink_ms = 0;
        self.fade_ms = 0;
        self.grow_ms = 0;
    }

    pub fn tick(&mut self, dt_ms: u32) {
        self.blink_ms += dt_ms;
        while self.blink_ms >= BLINK_MS {
            self.blink_ms -= BLINK_MS;
            self.blink = !self.blink;
        }
        if !self.effect {
            return;
        }
        self.grow_ms += dt_ms;
        while self.grow_ms >= GROW_MS {
            self.grow_ms -= GROW_MS;
            self.scale += GROW_STEP;
        }
        self.fade_ms += dt_ms;
        while self.effect && self.fade_ms >= FADE_MS {
            self.fade_ms -= FADE_MS;
            self.fade();
        }
    }

    fn fade(&mut self) {
        if self.rising {
            self.alpha += FADE_STEP;
            if self.alpha > 0xff {
                self.alpha = 0xff;
                self.rising = false;
            }
            return;
        }
        self.alpha -= FADE_STEP;
        if self.alpha < 0 {
            self.alpha = 0;
            self.rising = true;
            self.effect = false;
        }
    }

    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, pressed: bool, hovered: bool, alpha: f32) {
        self.paint_textures(list, (x, y), (pressed, hovered), alpha, [BUTTON_NORMAL, BUTTON_DOWN, BUTTON_LIGHT]);
    }

    pub fn paint_quest(&self, list: &mut DrawList, x: i32, y: i32, pressed: bool, hovered: bool, alpha: f32) {
        self.paint_textures(list, (x, y), (pressed, hovered), alpha, [QUEST_BUTTON_NORMAL, QUEST_BUTTON_DOWN, QUEST_BUTTON_LIGHT]);
    }

    pub fn paint_mail(&self, list: &mut DrawList, x: i32, y: i32, pressed: bool, hovered: bool, alpha: f32) {
        self.paint_textures(list, (x, y), (pressed, hovered), alpha, [MAIL_BUTTON_NORMAL, MAIL_BUTTON_DOWN, MAIL_BUTTON_LIGHT]);
    }

    fn paint_textures(&self, list: &mut DrawList, (x, y): (i32, i32), (pressed, hovered): (bool, bool), alpha: f32, [normal, down, light]: [&str; 3]) {
        let base = if pressed { down } else if hovered { light } else { normal };
        list.draw_texture(x, y, BUTTON_SIZE, BUTTON_SIZE, 0, 0, BUTTON_SIZE, BUTTON_SIZE, base, alpha);
        if self.effect {
            let glow = alpha * self.alpha as f32 / 255.0;
            list.draw_texture(x + GLOW_OFFSET, y + GLOW_OFFSET, GLOW_SIZE, GLOW_SIZE, 0, 0, GLOW_SIZE, GLOW_SIZE, GLOW1, glow);
            let size = (self.scale << 7) / 100;
            let at = 16 - size / 2;
            list.draw_texture(x + at, y + at, size, size, 0, 0, GLOW_SIZE, GLOW_SIZE, GLOW2, glow);
        } else if self.blink {
            list.draw_texture(x, y, BUTTON_SIZE, BUTTON_SIZE, 0, 0, BUTTON_SIZE, BUTTON_SIZE, light, alpha);
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClientEvents {
    pub mask: i32,
}

impl ClientEvents {
    pub fn enable(&mut self, mask: i32) {
        self.mask = mask;
    }

    pub fn fire(&mut self, event: i32) -> Option<i32> {
        if self.mask & event == 0 {
            return None;
        }
        self.mask ^= event;
        Some(event)
    }
}

pub fn camera_rotate_units(radians: f32) -> i32 {
    (radians * 65536.0 / std::f32::consts::TAU) as i32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;
    use crate::font::Glyphs;

    const SMALL: &[u8] = include_bytes!("../tests/fixtures/smallfont-e.gly");

    fn font() -> BitmapFont {
        BitmapFont::new("L2Font-e.SmallFont-e".into(), Glyphs::parse(SMALL).unwrap())
    }

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    #[test]
    fn the_viewer_is_created_at_the_right_edge_and_sized_by_the_html() {
        assert_eq!(viewer_position((1024, 768)), (694, 20));
        assert_eq!(viewer_height(10), 264);
        assert_eq!(viewer_height(400), 408);
        assert_eq!(viewer_height(5000), 680);
        assert_eq!(html_size(401), (295, 360));
    }

    #[test]
    fn the_tutorial_html_runs_the_marker_pass_with_the_client_tables() {
        let font = font();
        let mut plain = TutorialViewer::default();
        let mut marked = TutorialViewer::default();
        assert!(plain.load_with("<html><body>&@30045; sells</body></html>", &font, &|_, _| Some("Malcom".to_string())));
        assert!(marked.load("<html><body>Malcom sells</body></html>", &font));
        assert_eq!(plain.lines, marked.lines);
    }

    #[test]
    fn the_viewer_paints_the_title_back_rows_and_bottom() {
        let font = font();
        let mut viewer = TutorialViewer::default();
        assert!(!viewer.load("", &font));
        assert!(viewer.load("<html><body>Hello <a action=\"link tutorial_1.htm\">next</a></body></html>", &font));
        assert_eq!(viewer.height, 264);
        let mut list = DrawList::default();
        viewer.paint(&mut list, 100, 50, &font, 1.0);
        let top = with_texture(&list.quads, BACK1);
        assert_eq!((top[0].x, top[0].y, top[0].w, top[0].h), (100.0, 70.0, 310.0, 12.0));
        let rows = with_texture(&list.quads, BACK2);
        assert_eq!(rows.len(), (264 - 41) as usize / 12 + 1);
        assert_eq!((rows[0].y, rows[0].h), (82.0, 12.0));
        let last = rows.last().unwrap();
        assert_eq!((last.y, last.h), (82.0 + 18.0 * 12.0, 7.0));
        let bottom = with_texture(&list.quads, BACK3);
        assert_eq!((bottom[0].y, bottom[0].h), (50.0 + 255.0, 9.0));
        assert!(with_texture(&list.quads, crate::frame::FRAME_CLOSE_BTN).is_empty());
        let link = viewer.lines.iter().flat_map(|l| l.spans.iter().map(move |s| (l.y, s))).find(|(_, s)| s.action.is_some()).unwrap();
        assert_eq!(viewer.link_at(HTML_POS.0 + link.1.x + 1, HTML_POS.1 + link.0 + 1, &font), Some("link tutorial_1.htm"));
    }

    #[test]
    fn the_effect_button_fades_in_and_out_and_keeps_blinking() {
        let mut button = EffectButton::default();
        button.begin_effect(7);
        assert_eq!((button.mark_id, button.scale, button.alpha, button.effect), (7, 50, 0, true));
        button.tick(170);
        assert_eq!((button.alpha, button.rising), (255, true));
        button.tick(10);
        assert_eq!((button.alpha, button.rising), (255, false));
        assert_eq!(button.scale, 50 + 3 * 5);
        button.tick(180);
        assert!(!button.effect);
        assert_eq!(button.alpha, 0);
        assert!(!button.blink);
        button.tick(150);
        assert!(button.blink);
        let mut list = DrawList::default();
        button.paint(&mut list, 5, 500, false, false, 1.0);
        assert_eq!(with_texture(&list.quads, BUTTON_LIGHT).len(), 1);
    }

    #[test]
    fn the_glow_is_drawn_around_the_button_while_the_effect_runs() {
        let mut button = EffectButton::default();
        button.begin_effect(1);
        button.tick(50);
        let mut list = DrawList::default();
        button.paint(&mut list, 5, 500, false, false, 1.0);
        let glow1 = with_texture(&list.quads, GLOW1);
        assert_eq!((glow1[0].x, glow1[0].y, glow1[0].w), (-43.0, 452.0, 128.0));
        let glow2 = with_texture(&list.quads, GLOW2);
        assert_eq!((glow2[0].w, glow2[0].x), (70.0, 5.0 + 16.0 - 35.0));
    }

    #[test]
    fn client_events_fire_once_per_enabled_bit() {
        let mut events = ClientEvents::default();
        assert_eq!(events.fire(EVENT_MOVE), None);
        events.enable(EVENT_MOVE | EVENT_ZOOM);
        assert_eq!(events.fire(EVENT_ZOOM), Some(EVENT_ZOOM));
        assert_eq!(events.fire(EVENT_ZOOM), None);
        assert_eq!(events.fire(EVENT_MOVE), Some(EVENT_MOVE));
        assert_eq!(events.mask, 0);
        assert_eq!(camera_rotate_units(std::f32::consts::TAU / 4.0), 16384);
    }
}
