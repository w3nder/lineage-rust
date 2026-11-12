use crate::draw::DrawList;

pub const MINIBAR_BACK: &str = "L2UI_ch3.dialog.minibar_back";
pub const MINIBAR_TIME: &str = "L2UI_ch3.dialog.minibar_time1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressCtrl {
    pub back: &'static str,
    pub fill: &'static str,
    pub inset: i32,
}

pub const DIALOG_TIMER: ProgressCtrl = ProgressCtrl { back: MINIBAR_BACK, fill: MINIBAR_TIME, inset: 1 };

impl ProgressCtrl {
    pub fn remaining_width(&self, w: i32, elapsed: i32, total: i32) -> i32 {
        let inner = w - 2 * self.inset;
        if total <= 0 {
            return 0;
        }
        let filled = (elapsed as u32).wrapping_mul(inner as u32) / total as u32;
        inner.wrapping_sub(filled as i32)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn paint(&self, list: &mut DrawList, x: i32, y: i32, w: i32, h: i32, elapsed: i32, total: i32) {
        list.draw_texture(x, y, w, h, 0, 0, w, h, self.back, 1.0);
        let remaining = self.remaining_width(w, elapsed, total);
        if remaining > 0 {
            list.draw_texture(
                x + self.inset,
                y + self.inset,
                remaining,
                h - 2 * self.inset,
                0,
                0,
                w,
                h,
                self.fill,
                1.0,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::draw::Quad;

    fn with_texture<'a>(quads: &'a [Quad], texture: &str) -> Vec<&'a Quad> {
        quads.iter().filter(|q| q.texture == texture).collect()
    }

    #[test]
    fn the_fill_shrinks_from_the_inset_with_the_elapsed_share_over_the_full_texture_uv() {
        let mut list = DrawList::default();
        DIALOG_TIMER.paint(&mut list, 17, 61, 222, 6, 2500, 10000);
        let back = with_texture(&list.quads, MINIBAR_BACK);
        assert_eq!(
            (back[0].x, back[0].y, back[0].w, back[0].h, back[0].uw, back[0].vh),
            (17.0, 61.0, 222.0, 6.0, 222.0, 6.0)
        );
        let fill = with_texture(&list.quads, MINIBAR_TIME);
        assert_eq!((fill[0].x, fill[0].y, fill[0].w, fill[0].h), (18.0, 62.0, 165.0, 4.0));
        assert_eq!((fill[0].uw, fill[0].vh), (222.0, 6.0));
        assert_eq!(DIALOG_TIMER.remaining_width(222, 0, 10000), 220);
        assert_eq!(DIALOG_TIMER.remaining_width(222, 10000, 10000), 0);
        assert_eq!(DIALOG_TIMER.remaining_width(222, 20000, 10000), -220);
        assert!(DIALOG_TIMER.remaining_width(222, -1000, 10000) < 0);
        assert_eq!(DIALOG_TIMER.remaining_width(222, 5, 0), 0);
    }

    #[test]
    fn a_finished_timer_paints_only_the_background() {
        let timer: ProgressCtrl = DIALOG_TIMER;
        let mut list = DrawList::default();
        timer.paint(&mut list, 0, 0, 222, 6, 10000, 10000);
        assert_eq!(list.quads.len(), 1);
        assert_eq!(list.quads[0].texture, MINIBAR_BACK);
    }
}
