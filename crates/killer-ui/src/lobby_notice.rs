use crate::button::TextButton;
use crate::color::TEXT;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::pixel_scroll::PixelScroll;
use crate::window::Rect;

pub const SIZE: (i32, i32) = (546, 425);
pub const BACK: &str = "L2UI_ch3.LobbyWnd.lobby3_back";
pub const ANNOUNCE_BACK: &str = "L2UI_ch3.LobbyWnd.Announce_cn";
pub const STRIPE: &str = "l2ui_ch3.etc.textbackline";
pub const READER_RECT: Rect = Rect::new(6, 6, 536, 375);
pub const STRIPE_STEP: i32 = 18;
pub const STRIPE_HEIGHT: i32 = 15;
pub const STRIPE_UV_WIDTH: i32 = 8;
pub const STRIPE_ROWS_FACTOR: f32 = 0.06666667;
pub const SCROLL_BAR_WIDTH: i32 = 15;
pub const TEXT_POS: (i32, i32) = (1, 2);
pub const TEXT_WRAP_INSET: i32 = 16 + 4;
pub const AGREE_RECT: Rect = Rect::new(196, 396, 76, 23);
pub const DISAGREE_RECT: Rect = Rect::new(276, 396, 76, 23);
pub const OK_RECT: Rect = Rect::new(235, 396, 76, 23);
pub const BUTTON_NORMAL: &str = "L2UI_ch3.button.btn1_normal";
pub const BUTTON_DOWN: &str = "L2UI_ch3.button.btn1_normalon";
pub const SYSSTRINGS: [u32; 3] = [599, 600, 140];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Eula,
    Warning,
    Announce,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hit {
    Agree,
    Disagree,
    Ok,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeAction {
    Agree,
    Disagree,
    Ok,
    Announce,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NoticeLabels {
    pub agree: String,
    pub disagree: String,
    pub ok: String,
}

impl NoticeLabels {
    pub fn from_sysstrings(lookup: impl Fn(u32) -> Option<String>) -> NoticeLabels {
        let text = |id: u32| lookup(id).unwrap_or_default();
        NoticeLabels { agree: text(SYSSTRINGS[0]), disagree: text(SYSSTRINGS[1]), ok: text(SYSSTRINGS[2]) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub screen: (i32, i32),
    pub kind: Kind,
    pub labels: NoticeLabels,
    pub text: String,
    pub scroll: PixelScroll,
    pub pressed: Option<Hit>,
}

impl Notice {
    pub fn new(screen: (i32, i32), kind: Kind, labels: NoticeLabels) -> Notice {
        Notice { screen, kind, labels, text: String::new(), scroll: PixelScroll::new((READER_RECT.w, READER_RECT.h)), pressed: None }
    }

    pub fn origin(&self) -> (i32, i32) {
        let height = self.screen.1 - crate::lobby_login::MESSAGE_BAR_HEIGHT;
        ((self.screen.0 as f32 / 2.0 - 273.0) as i32, (height as f32 / 2.0 - 212.0) as i32)
    }

    pub fn set_text(&mut self, text: &str) {
        if !text.is_empty() {
            self.text = text.to_string();
        }
    }

    pub fn clear_text(&mut self) {
        self.text.clear();
    }

    pub fn stripe_rows() -> i32 {
        (READER_RECT.h as f32 * STRIPE_ROWS_FACTOR) as i32
    }

    fn wrap_width() -> i32 {
        READER_RECT.w - TEXT_WRAP_INSET
    }

    fn reads(&self) -> bool {
        self.kind != Kind::Announce && !self.text.is_empty()
    }

    pub fn prepare(&mut self, font: &BitmapFont) {
        if self.reads() {
            let height = crate::text_align::measure_height(&self.text, Self::wrap_width(), font, 1, false, true);
            self.scroll.set_content_height(height);
        }
    }

    pub fn wheel(&mut self, delta: i32) {
        if self.kind != Kind::Announce {
            self.scroll.wheel(delta);
        }
    }

    pub fn mouse_move(&mut self, y: i32) -> bool {
        let (_, oy) = self.origin();
        self.kind != Kind::Announce && self.scroll.mouse_move(y - oy - READER_RECT.y)
    }

    fn buttons(&self) -> Vec<(Rect, Hit)> {
        match self.kind {
            Kind::Eula => vec![(AGREE_RECT, Hit::Agree), (DISAGREE_RECT, Hit::Disagree)],
            Kind::Warning | Kind::Announce => vec![(OK_RECT, Hit::Ok)],
        }
    }

    pub fn hit(&self, x: i32, y: i32) -> Option<Hit> {
        let (ox, oy) = self.origin();
        self.buttons().into_iter().find(|(rect, _)| rect.contains(x - ox, y - oy)).map(|(_, hit)| hit)
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) {
        let (ox, oy) = self.origin();
        if self.kind != Kind::Announce && self.scroll.mouse_down(x - ox - READER_RECT.x, y - oy - READER_RECT.y) {
            return;
        }
        self.pressed = self.hit(x, y);
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Option<NoticeAction> {
        self.scroll.mouse_up();
        let pressed = self.pressed.take()?;
        if self.hit(x, y) != Some(pressed) {
            return None;
        }
        Some(self.action(pressed))
    }

    fn action(&self, hit: Hit) -> NoticeAction {
        match (hit, self.kind) {
            (Hit::Agree, _) => NoticeAction::Agree,
            (Hit::Disagree, _) => NoticeAction::Disagree,
            (Hit::Ok, Kind::Announce) => NoticeAction::Announce,
            (Hit::Ok, _) => NoticeAction::Ok,
        }
    }

    pub fn enter(&self) -> NoticeAction {
        match self.kind {
            Kind::Eula => NoticeAction::Agree,
            Kind::Warning => NoticeAction::Ok,
            Kind::Announce => NoticeAction::Announce,
        }
    }

    pub fn paint(&self, list: &mut DrawList, font: &BitmapFont) {
        let (ox, oy) = self.origin();
        let back = if self.kind == Kind::Announce { ANNOUNCE_BACK } else { BACK };
        list.draw_texture(ox, oy, SIZE.0, SIZE.1, 0, 0, SIZE.0, SIZE.1, back, 1.0);
        if self.kind != Kind::Announce {
            let (rx, ry) = (ox + READER_RECT.x, oy + READER_RECT.y);
            let start = list.quads.len();
            for row in 0..Self::stripe_rows() {
                if row % 2 == 1 {
                    list.draw_texture(rx, ry + row * STRIPE_STEP, READER_RECT.w - SCROLL_BAR_WIDTH, STRIPE_HEIGHT, 0, 0, STRIPE_UV_WIDTH, STRIPE_HEIGHT, STRIPE, 1.0);
                }
            }
            if !self.text.is_empty() {
                let lines = crate::text_align::bitmap_lines(&self.text, Self::wrap_width(), font, 1, false);
                crate::text_align::paint(list, (rx + TEXT_POS.0, ry + TEXT_POS.1 + self.scroll.offset), &lines, TEXT, font, true);
            }
            list.clip_from(start, READER_RECT.offset(ox, oy));
            let mut scroll = self.scroll.clone();
            if !self.text.is_empty() {
                scroll.set_content_height(crate::text_align::measure_height(&self.text, Self::wrap_width(), font, 1, false, true));
            }
            scroll.paint_bar(list, rx, ry);
        }
        for (rect, hit) in self.buttons() {
            let label = match hit {
                Hit::Agree => &self.labels.agree,
                Hit::Disagree => &self.labels.disagree,
                Hit::Ok => &self.labels.ok,
            };
            TextButton::new(rect, BUTTON_NORMAL, BUTTON_DOWN, label).paint(list, ox, oy, self.pressed == Some(hit), font);
        }
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

    fn eula() -> Notice {
        Notice::new((1024, 768), Kind::Eula, NoticeLabels::from_sysstrings(|id| Some(format!("s{id}"))))
    }

    #[test]
    fn the_window_is_centered_in_the_login_window_that_leaves_the_bar_out() {
        assert_eq!(eula().origin(), (1024 / 2 - 273, (716.0f32 / 2.0 - 212.0) as i32));
    }

    #[test]
    fn eula_has_agree_and_disagree_and_the_warning_a_single_ok_and_enter_takes_the_main_button() {
        let mut notice = eula();
        let (ox, oy) = notice.origin();
        let point = |rect: Rect| (ox + rect.x + 1, oy + rect.y + 1);
        let (x, y) = point(AGREE_RECT);
        notice.mouse_down(x, y);
        assert_eq!(notice.mouse_up(x, y), Some(NoticeAction::Agree));
        let (x, y) = point(DISAGREE_RECT);
        notice.mouse_down(x, y);
        assert_eq!(notice.mouse_up(x, y), Some(NoticeAction::Disagree));
        assert_eq!(notice.enter(), NoticeAction::Agree);
        let mut warning = Notice::new((1024, 768), Kind::Warning, NoticeLabels::default());
        let (x, y) = point(OK_RECT);
        warning.mouse_down(x, y);
        assert_eq!(warning.mouse_up(x, y), Some(NoticeAction::Ok));
        let (x, y) = point(AGREE_RECT);
        assert_eq!(warning.hit(x, y), None);
        assert_eq!(warning.enter(), NoticeAction::Ok);
    }

    #[test]
    fn the_reader_stripes_the_odd_window_rows_with_a_scrolling_text_and_a_pixel_scroll_bar() {
        let mut notice = eula();
        notice.set_text(&"palavra ".repeat(900));
        notice.prepare(&font());
        assert!(notice.scroll.minimum < 0);
        notice.wheel(-120);
        assert_eq!(notice.scroll.offset, -45);
        let mut list = DrawList { capture_text: true, ..Default::default() };
        notice.paint(&mut list, &font());
        let (ox, oy) = notice.origin();
        let stripes: Vec<_> = list.quads.iter().filter(|quad| quad.texture == STRIPE).collect();
        assert_eq!(stripes.len(), 10);
        assert_eq!(
            (stripes[0].x, stripes[0].y, stripes[0].w, stripes[0].h, stripes[0].uw, stripes[0].vh),
            ((ox + 6) as f32, (oy + 6 + 18) as f32, 521.0, 15.0, 8.0, 15.0)
        );
        assert_eq!(stripes[9].y, (oy + 6 + 19 * 18) as f32);
        notice.wheel(120 * 10);
        assert_eq!(notice.scroll.offset, 0);
        let mut top = DrawList { capture_text: true, ..Default::default() };
        notice.paint(&mut top, &font());
        let first = top.quads.iter().find(|quad| quad.texture == "f").unwrap();
        assert_eq!((first.x, first.y), ((ox + 6 + 1) as f32, (oy + 6 + 2) as f32));
        assert!(list.quads.iter().filter(|quad| quad.texture == "f").all(|quad| quad.y >= (oy + 6) as f32));
    }

    #[test]
    fn the_scroll_bar_buttons_move_the_text_and_an_empty_text_leaves_the_old_one() {
        let mut notice = eula();
        notice.set_text(&"palavra ".repeat(900));
        notice.set_text("");
        assert!(notice.text.starts_with("palavra"));
        notice.prepare(&font());
        let (ox, oy) = notice.origin();
        let down = crate::scroll::scroll_rects(READER_RECT.w, READER_RECT.h).down;
        let (x, y) = (ox + READER_RECT.x + down.x + 1, oy + READER_RECT.y + down.y + 1);
        notice.mouse_down(x, y);
        assert_eq!(notice.scroll.offset, -15);
        assert_eq!(notice.mouse_up(x, y), None);
        notice.clear_text();
        assert!(notice.text.is_empty());
    }

    #[test]
    fn the_announce_paints_its_own_back_without_the_reader_and_ok_answers_as_the_announce() {
        let mut announce = Notice::new((1024, 768), Kind::Announce, NoticeLabels::from_sysstrings(|id| Some(format!("s{id}"))));
        announce.set_text(&"palavra ".repeat(900));
        let mut list = DrawList { capture_text: true, ..Default::default() };
        announce.paint(&mut list, &font());
        assert!(list.quads.iter().any(|quad| quad.texture == ANNOUNCE_BACK));
        assert!(list.quads.iter().all(|quad| quad.texture != BACK && quad.texture != STRIPE));
        let (ox, oy) = announce.origin();
        let (x, y) = (ox + OK_RECT.x + 1, oy + OK_RECT.y + 1);
        announce.mouse_down(x, y);
        assert_eq!(announce.mouse_up(x, y), Some(NoticeAction::Announce));
        assert_eq!(announce.enter(), NoticeAction::Announce);
        let (x, y) = (ox + AGREE_RECT.x + 1, oy + AGREE_RECT.y + 1);
        assert_eq!(announce.hit(x, y), None);
    }
}
