use crate::detail_status::{DetailStatusView, Sysstrings};
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::frame::{FrameCtrl, FrameMode};
use crate::frame_contract::{FrameContract, FrameHit, FrameTitle};
use crate::window::{Rect, WindowId};

pub const ID: WindowId = WindowId::GM_STATUS;
pub const SIZE: (i32, i32) = (256, 355);
pub const FRAME_ID: u32 = 0xf0f118;
pub const GRIP: Rect = Rect::new(0, 0, 256, 20);
pub const INITIAL_POSITION: crate::initial_position::InitialPosition =
    crate::initial_position::InitialPosition { enabled: true, horizontal_mode: 0, vertical_mode: 3, x: 0, y: 100 };

const FRAME: FrameContract = FrameContract {
    rect: GRIP,
    id: FRAME_ID,
    mode: FrameMode::Title,
    title: FrameTitle::None,
    close_button: true,
    minimize_button: false,
};

pub fn default_position(_viewport: (i32, i32)) -> (i32, i32) {
    (0, 100)
}

pub fn speed_text(speeds: [i32; 6], multiplier: f64) -> String {
    let scaled = |value: i32| (f64::from(value) * multiplier) as i32;
    format!(
        "{},{}/{},{}/{},{}",
        scaled(speeds[2]),
        scaled(speeds[3]),
        scaled(speeds[4]),
        scaled(speeds[5]),
        scaled(speeds[0]),
        scaled(speeds[1])
    )
}

#[derive(Clone, Debug)]
pub struct GmStatusView {
    pub frame: FrameCtrl,
    pub status: DetailStatusView,
    pub speeds: String,
    pressed: bool,
}

impl GmStatusView {
    pub fn new(lookup: impl Fn(u32) -> Option<String>) -> Self {
        Self {
            frame: FRAME.build(|id| lookup(id).unwrap_or_default()),
            status: DetailStatusView::default(),
            speeds: speed_text([0; 6], 0.0),
            pressed: false,
        }
    }

    pub fn set(&mut self, status: DetailStatusView, speeds: [i32; 6], multiplier: f64) {
        self.status = status;
        self.speeds = speed_text(speeds, multiplier);
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> bool {
        self.pressed = FRAME.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close);
        self.pressed
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> bool {
        std::mem::take(&mut self.pressed) && FRAME.hit(&self.frame, (0, 0), (x, y)) == Some(FrameHit::Close)
    }

    pub fn paint(&self, list: &mut DrawList, origin: (i32, i32), font: &BitmapFont, alpha: f32, lookup: &dyn Fn(u32) -> Option<String>) {
        FRAME.paint(&self.frame, list, origin, font, alpha);
        self.status.paint_gm(list, origin.0, origin.1, font, &Sysstrings(lookup), &self.speeds);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_speed_block_prints_water_air_ground_scaled_by_the_double_multiplier() {
        assert_eq!(speed_text([120, 80, 50, 40, 150, 100], 1.0), "50,40/150,100/120,80");
        assert_eq!(speed_text([100, 100, 100, 100, 100, 100], 1.5), "150,150/150,150/150,150");
    }

    #[test]
    fn the_status_always_paints_shifted_by_the_frame_bar() {
        let mut view = GmStatusView::new(|id| Some(format!("s{id}")));
        view.status.name = "Ana".into();
        let font = {
            let glyphs: Vec<[i32; 4]> = (0..95).map(|i| [i * 7, 6, 0, 13]).collect();
            BitmapFont::new("f".into(), crate::font::Glyphs::parse(&crate::font::gly(&[(0x20, &glyphs)], (1024, 128))).unwrap())
        };
        let mut list = DrawList::default();
        view.paint(&mut list, (0, 0), &font, 1.0, &|id| Some(format!("s{id}")));
        let back = list.quads.iter().find(|quad| quad.texture == crate::detail_status::MYINFO_BACK).unwrap();
        assert_eq!((back.x, back.y), (0.0, 20.0));
        assert!(list.quads.iter().any(|quad| quad.y == 20.0 + 9.0 && quad.x == 15.0));
    }
}
