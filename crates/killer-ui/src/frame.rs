use crate::color::TITLE;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub const FRAME_BACK_LEFT: &str = "L2UI_CH3.FrameCtrl.FrameBackLeft";
pub const FRAME_BACK_MID: &str = "L2UI_CH3.FrameCtrl.FrameBackMid";
pub const FRAME_BACK_RIGHT: &str = "L2UI_CH3.FrameCtrl.FrameBackRight";
pub const FRAME_CLOSE_BTN: &str = "L2UI_CH3.FrameCtrl.FrameCloseBtn";
pub const FRAME_MINI_BTN: &str = "L2UI_CH3.FrameCtrl.FrameMiniBtn";
pub const SMALLBAR_H1: &str = "L2UI_CH3.FrameCtrl.smallbar_h1";
pub const SMALLBAR_H2: &str = "L2UI_CH3.FrameCtrl.smallbar_h2";
pub const SMALLBAR_H3: &str = "L2UI_CH3.FrameCtrl.smallbar_h3";
pub const SMALLBAR_1: &str = "L2UI_CH3.FrameCtrl.smallbar1";
pub const SMALLBAR_2: &str = "L2UI_CH3.FrameCtrl.smallbar2";
pub const SMALLBAR_3: &str = "L2UI_CH3.FrameCtrl.smallbar3";

pub const TITLE_HEIGHT: i32 = 20;
pub const TITLE_END: i32 = 16;
pub const TITLE_MID_UW: i32 = 32;
pub const TITLE_TEXT_X: i32 = 20;
pub const TITLE_TEXT_Y_OFFSET: i32 = 2;
pub const BUTTON_SIZE: i32 = 15;
pub const BUTTON_Y: i32 = 3;
pub const CLOSE_INSET: i32 = 23;
pub const MINI_INSET: i32 = 40;
pub const THIN_BAR_HEIGHT: i32 = 12;
pub const THIN_BAR_END: i32 = 8;
pub const GRIP_WIDTH: i32 = 12;
pub const GRIP_END: i32 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FrameMode {
    Title,
    ThinBar,
    VerticalGrip,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FrameCtrl {
    pub mode: FrameMode,
    pub title: String,
    pub close_button: bool,
    pub minimize_button: bool,
}

pub fn title_text_y(font: &BitmapFont) -> i32 {
    TITLE_HEIGHT / 2 - font.line_height as i32 / 2 + TITLE_TEXT_Y_OFFSET
}

impl FrameCtrl {
    pub fn new(mode: FrameMode) -> FrameCtrl {
        FrameCtrl { mode, title: String::new(), close_button: false, minimize_button: false }
    }

    pub fn titled(title: &str, close_button: bool) -> FrameCtrl {
        FrameCtrl { mode: FrameMode::Title, title: title.to_string(), close_button, minimize_button: false }
    }

    pub fn height(&self) -> i32 {
        match self.mode {
            FrameMode::Title => TITLE_HEIGHT,
            FrameMode::ThinBar => THIN_BAR_HEIGHT,
            FrameMode::VerticalGrip => GRIP_WIDTH,
        }
    }

    pub fn grip(&self, w: i32, h: i32) -> Rect {
        match self.mode {
            FrameMode::Title | FrameMode::ThinBar => Rect::new(0, 0, w, self.height()),
            FrameMode::VerticalGrip => Rect::new(0, 0, GRIP_WIDTH, h),
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, w: i32, h: i32, font: &BitmapFont, alpha: f32) {
        match self.mode {
            FrameMode::Title => self.paint_title(list, x, y, w, font, alpha),
            FrameMode::ThinBar => list.slices3_h(
                x,
                y,
                w,
                THIN_BAR_HEIGHT,
                (SMALLBAR_H3, THIN_BAR_END),
                SMALLBAR_H2,
                (SMALLBAR_H1, THIN_BAR_END),
                THIN_BAR_HEIGHT,
                alpha,
            ),
            FrameMode::VerticalGrip => list.slices3_v(
                x,
                y,
                GRIP_WIDTH,
                h,
                (SMALLBAR_1, GRIP_END),
                SMALLBAR_2,
                (SMALLBAR_3, GRIP_END),
                GRIP_WIDTH,
                alpha,
            ),
        }
    }

    fn paint_title(&self, list: &mut DrawList, x: i32, y: i32, w: i32, font: &BitmapFont, alpha: f32) {
        let mid_w = (w - 2 * TITLE_END).max(0);
        let start = list.quads.len();
        list.draw_texture(x, y, TITLE_END, TITLE_HEIGHT, 0, 0, TITLE_END, TITLE_HEIGHT, FRAME_BACK_LEFT, alpha);
        list.draw_texture(
            x + TITLE_END,
            y,
            mid_w,
            TITLE_HEIGHT,
            0,
            0,
            TITLE_MID_UW,
            TITLE_HEIGHT,
            FRAME_BACK_MID,
            alpha,
        );
        list.draw_texture(
            x + w - TITLE_END,
            y,
            TITLE_END,
            TITLE_HEIGHT,
            0,
            0,
            TITLE_END,
            TITLE_HEIGHT,
            FRAME_BACK_RIGHT,
            alpha,
        );
        list.draw_text(x + TITLE_TEXT_X, y + title_text_y(font), TITLE, &self.title, font);
        list.clip_from(start, Rect::new(x, y, w, TITLE_HEIGHT));
        if self.minimize_button {
            paint_button(list, x + w - MINI_INSET, y + BUTTON_Y, FRAME_MINI_BTN, alpha);
        }
        if self.close_button {
            paint_button(list, x + w - CLOSE_INSET, y + BUTTON_Y, FRAME_CLOSE_BTN, alpha);
        }
    }
}

fn paint_button(list: &mut DrawList, x: i32, y: i32, texture: &str, alpha: f32) {
    list.draw_button_texture(x, y, BUTTON_SIZE, BUTTON_SIZE, 0, 0, BUTTON_SIZE, BUTTON_SIZE, texture, alpha, true);
}

pub const BODY_LEFT_X: i32 = 12;
pub const BODY_END_WIDTH: i32 = 16;
pub const BODY_END_EX_WIDTH: i32 = 32;
pub const BODY_SMALL_CONTENT: i32 = 44;
pub const BODY_SMALL2_CONTENT: i32 = 76;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BodyKind {
    Small,
    Small2,
    SmallWindow,
}

impl BodyKind {
    pub fn textures(self) -> (&'static str, &'static str, &'static str) {
        match self {
            BodyKind::Small => (
                "L2UI_CH3.SmallWnd.Smallwindow_back1",
                "L2UI_CH3.SmallWnd.Smallwindow_back2",
                "L2UI_CH3.SmallWnd.Smallwindow_back3",
            ),
            BodyKind::Small2 => (
                "L2UI_CH3.SmallWnd.SmallWindow2_Back1",
                "L2UI_CH3.SmallWnd.SmallWindow2_Back2",
                "L2UI_CH3.SmallWnd.SmallWindow2_Back3_ex",
            ),
            BodyKind::SmallWindow => (
                "L2UI_CH3.SmallWnd.SmallWindow_Back1",
                "L2UI_CH3.SmallWnd.SmallWindow_Back2",
                "L2UI_CH3.SmallWnd.SmallWindow_Back3_ex",
            ),
        }
    }

    pub fn end(self) -> (&'static str, i32) {
        let width = match self {
            BodyKind::Small => BODY_END_WIDTH,
            BodyKind::Small2 | BodyKind::SmallWindow => BODY_END_EX_WIDTH,
        };
        (self.textures().2, width)
    }

    pub fn content_height(self) -> i32 {
        match self {
            BodyKind::Small | BodyKind::SmallWindow => BODY_SMALL_CONTENT,
            BodyKind::Small2 => BODY_SMALL2_CONTENT,
        }
    }
}

pub fn paint_body(list: &mut DrawList, kind: BodyKind, x: i32, y: i32, w: i32, h: i32, alpha: f32) {
    paint_body_with_end(list, kind, x, y, w, h, kind.end(), alpha);
}

#[allow(clippy::too_many_arguments)]
pub fn paint_body_with_end(
    list: &mut DrawList,
    kind: BodyKind,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    end: (&str, i32),
    alpha: f32,
) {
    let (left, mid, _) = kind.textures();
    list.slices3_h(
        x + BODY_LEFT_X,
        y,
        w - BODY_LEFT_X,
        h,
        (left, BODY_END_WIDTH),
        mid,
        end,
        kind.content_height(),
        alpha,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;
    use crate::font::{Glyphs, gly};

    fn font() -> BitmapFont {
        let bytes = gly(&[(0x20, &[[0, 3, 0, 13], [4, 6, 0, 13]])], (1024, 128));
        BitmapFont { atlas: "L2Font-e.SmallFont-e".into(), glyphs: Glyphs::parse(&bytes).unwrap(), line_height: 13 }
    }

    fn xwu(q: &Quad) -> (f32, f32, f32) {
        (q.x, q.w, q.uw)
    }

    #[test]
    fn title_frame_is_three_slices_of_twenty_pixels_plus_the_title() {
        let mut list = DrawList::default();
        FrameCtrl::titled("!!", false).paint(&mut list, 0, 0, 200, 84, &font(), 1.0);
        assert_eq!(list.quads.len(), 5);
        assert_eq!(xwu(&list.quads[0]), (0.0, 16.0, 16.0));
        assert_eq!(xwu(&list.quads[1]), (16.0, 168.0, 32.0));
        assert_eq!(xwu(&list.quads[2]), (184.0, 16.0, 16.0));
        assert!(list.quads[..3].iter().all(|q| q.h == 20.0 && q.vh == 20.0 && q.y == 0.0));
        assert_eq!(list.quads[0].texture, FRAME_BACK_LEFT);
        assert_eq!(list.quads[1].texture, FRAME_BACK_MID);
        assert_eq!(list.quads[2].texture, FRAME_BACK_RIGHT);
        assert_eq!((list.quads[3].x, list.quads[3].y), (20.0, 6.0));
        assert_eq!(list.quads[3].color, TITLE.rgba());
        assert_eq!(list.quads[3].texture, "L2Font-e.SmallFont-e");
    }

    #[test]
    fn title_frame_uses_the_alpha_and_places_the_close_button() {
        let mut list = DrawList::default();
        let frame = FrameCtrl::titled("", true);
        frame.paint(&mut list, 10, 5, 200, 84, &font(), 0.5);
        assert_eq!(list.quads.len(), 4);
        assert!(list.quads.iter().all(|q| q.color == [1.0, 1.0, 1.0, 0.5]));
        let close = &list.quads[3];
        assert_eq!((close.x, close.y, close.w, close.h), (187.0, 8.0, 15.0, 15.0));
        assert_eq!((close.uw, close.vh), (15.0, 15.0));
        assert_eq!(close.texture, FRAME_CLOSE_BTN);
        let mut both = DrawList::default();
        FrameCtrl { minimize_button: true, ..frame.clone() }.paint(&mut both, 10, 5, 200, 84, &font(), 1.0);
        assert_eq!(both.quads.len(), 5);
        assert_eq!((both.quads[3].x, both.quads[3].y, both.quads[3].texture.as_str()), (170.0, 8.0, FRAME_MINI_BTN));
        assert_eq!(both.quads[4].texture, FRAME_CLOSE_BTN);
        assert_eq!(title_text_y(&font()), 6);
        assert_eq!(frame.height(), 20);
        assert_eq!(frame.grip(200, 84), Rect::new(0, 0, 200, 20));
    }

    #[test]
    fn a_long_title_is_clipped_to_the_frame_rectangle_like_the_setclip_of_onpaint() {
        let mut list = DrawList::default();
        let title = "!".repeat(40);
        FrameCtrl::titled(&title, false).paint(&mut list, 0, 0, 40, 84, &font(), 1.0);
        assert!(list.quads.iter().all(|q| q.x + q.w <= 40.0 && q.x >= 0.0));
        assert!(list.quads.len() > 3);
    }

    #[test]
    fn thin_bar_is_twelve_pixels_high() {
        let mut list = DrawList::default();
        let frame = FrameCtrl::new(FrameMode::ThinBar);
        frame.paint(&mut list, 0, 0, 100, 50, &font(), 1.0);
        assert_eq!(list.quads.len(), 3);
        assert!(list.quads.iter().all(|q| q.h == 12.0 && q.vh == 12.0));
        assert_eq!(xwu(&list.quads[0]), (0.0, 8.0, 8.0));
        assert_eq!(xwu(&list.quads[1]), (8.0, 84.0, 8.0));
        assert_eq!(xwu(&list.quads[2]), (92.0, 8.0, 8.0));
        assert_eq!(list.quads[1].texture, SMALLBAR_H2);
        assert_eq!(list.quads[0].texture, SMALLBAR_H3);
        assert_eq!(list.quads[2].texture, SMALLBAR_H1);
        assert_eq!(frame.height(), 12);
        assert_eq!(frame.grip(100, 50), Rect::new(0, 0, 100, 12));
    }

    #[test]
    fn vertical_grip_is_twelve_pixels_wide_with_the_middle_between_the_ends() {
        let mut list = DrawList::default();
        let frame = FrameCtrl::new(FrameMode::VerticalGrip);
        frame.paint(&mut list, 0, 0, 202, 84, &font(), 1.0);
        assert_eq!(list.quads.len(), 3);
        assert!(list.quads.iter().all(|q| q.w == 12.0 && q.uw == 12.0 && q.x == 0.0));
        assert_eq!((list.quads[0].y, list.quads[0].h), (0.0, 8.0));
        assert_eq!((list.quads[1].y, list.quads[1].h), (8.0, 68.0));
        assert_eq!((list.quads[2].y, list.quads[2].h), (76.0, 8.0));
        assert_eq!(list.quads[0].texture, SMALLBAR_1);
        assert_eq!(list.quads[2].texture, SMALLBAR_3);
        assert_eq!(frame.height(), 12);
        assert_eq!(frame.grip(202, 84), Rect::new(0, 0, 12, 84));
    }

    #[test]
    fn small2_body_starts_after_the_grip_and_stretches_seventy_six_texels() {
        let mut list = DrawList::default();
        paint_body(&mut list, BodyKind::Small2, 0, 0, 202, 84, 1.0);
        assert_eq!(list.quads.len(), 3);
        let back1 = &list.quads[0];
        assert_eq!((back1.x, back1.w, back1.h, back1.vh), (12.0, 16.0, 84.0, 76.0));
        assert_eq!(back1.texture, "L2UI_CH3.SmallWnd.SmallWindow2_Back1");
        let back2 = &list.quads[1];
        assert_eq!((back2.x, back2.w, back2.vh), (28.0, 142.0, 76.0));
        let back3 = &list.quads[2];
        assert_eq!((back3.x, back3.w, back3.uw), (170.0, 32.0, 32.0));
        assert_eq!(back3.texture, "L2UI_CH3.SmallWnd.SmallWindow2_Back3_ex");
    }

    #[test]
    fn small_body_has_forty_four_texels_and_a_sixteen_pixel_end() {
        let mut list = DrawList::default();
        paint_body(&mut list, BodyKind::Small, 5, 7, 100, 44, 1.0);
        assert_eq!((list.quads[0].x, list.quads[0].y, list.quads[0].vh), (17.0, 7.0, 44.0));
        assert_eq!((list.quads[2].x, list.quads[2].w), (89.0, 16.0));
        assert_eq!(list.quads[2].texture, "L2UI_CH3.SmallWnd.Smallwindow_back3");
        let mut target = DrawList::default();
        paint_body(&mut target, BodyKind::SmallWindow, 0, 0, 202, 44, 1.0);
        assert_eq!((target.quads[2].x, target.quads[2].w), (170.0, 32.0));
        assert_eq!(target.quads[2].texture, "L2UI_CH3.SmallWnd.SmallWindow_Back3_ex");
    }

    #[test]
    fn a_custom_end_replaces_the_third_slice() {
        let mut list = DrawList::default();
        paint_body_with_end(
            &mut list,
            BodyKind::Small2,
            0,
            0,
            202,
            84,
            ("L2UI_CH3.PlayerStatusWnd.ps_sizecontrol1", 4),
            1.0,
        );
        assert_eq!((list.quads[1].x, list.quads[1].w), (28.0, 170.0));
        assert_eq!((list.quads[2].x, list.quads[2].w, list.quads[2].uw, list.quads[2].vh), (198.0, 4.0, 4.0, 76.0));
        assert_eq!(list.quads[2].texture, "L2UI_CH3.PlayerStatusWnd.ps_sizecontrol1");
    }
}
