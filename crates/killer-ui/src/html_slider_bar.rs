use crate::draw::DrawList;
use crate::html_push_button::{Ancestor, paint_clipped};
use crate::window::Rect;

pub const WIDTH: i32 = 15;
pub const CAP: i32 = 8;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HtmlSliderBar {
    pub rect: Rect,
    pub length: i32,
    pub offset: i32,
    pub top: Option<String>,
    pub middle: Option<String>,
    pub bottom: Option<String>,
}

impl HtmlSliderBar {
    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), ancestors: &[Ancestor]) {
        let x = origin.0 + self.rect.x;
        let y = origin.1 + self.rect.y;
        let mut body = DrawList { capture_text: list.capture_text, transparent: list.transparent, ..Default::default() };
        if self.length > 0 {
            if let Some(texture) = &self.top {
                body.draw_texture(x, y + self.offset, WIDTH, CAP, 0, 0, WIDTH, CAP, texture, 1.0);
            }
            if let Some(texture) = &self.bottom {
                body.draw_texture(x, y + self.offset - CAP + self.length, WIDTH, CAP, 0, 0, WIDTH, CAP, texture, 1.0);
            }
            if let Some(texture) = &self.middle {
                body.draw_texture(x, y + self.offset + CAP, WIDTH, self.length - 2 * CAP, 0, 0, WIDTH, CAP, texture, 1.0);
            }
        }
        paint_clipped(body, list, ancestors);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(length: i32) -> HtmlSliderBar {
        HtmlSliderBar {
            rect: Rect::new(4, 6, 15, 100),
            length,
            offset: 10,
            top: Some("top".into()),
            middle: Some("mid".into()),
            bottom: Some("bot".into()),
        }
    }

    #[test]
    fn html_slider_bar_1004a300_draws_caps_at_both_ends_and_stretches_the_middle_between() {
        let mut list = DrawList::default();
        bar(40).paint(&mut list, (100, 200), &[]);
        let rows: Vec<_> = list.quads.iter().map(|q| (q.texture.to_string(), q.x, q.y, q.w, q.h, q.uw, q.vh)).collect();
        assert_eq!(rows, vec![
            ("top".to_string(), 104.0, 216.0, 15.0, 8.0, 15.0, 8.0),
            ("bot".to_string(), 104.0, 248.0, 15.0, 8.0, 15.0, 8.0),
            ("mid".to_string(), 104.0, 224.0, 15.0, 24.0, 15.0, 8.0),
        ]);
    }

    #[test]
    fn html_slider_bar_1004a300_draws_nothing_without_a_positive_length() {
        for length in [0, -3] {
            let mut list = DrawList::default();
            bar(length).paint(&mut list, (0, 0), &[]);
            assert!(list.quads.is_empty());
        }
    }

    #[test]
    fn html_slider_bar_1004a300_is_clipped_by_the_first_non_html_ancestor() {
        let ancestors = [Ancestor { is_control: false, rect: Rect::new(0, 0, 200, 220) }];
        let mut list = DrawList::default();
        bar(40).paint(&mut list, (100, 200), &ancestors);
        assert_eq!(list.quads.len(), 1);
        assert_eq!(list.quads[0].h, 4.0);
    }
}
