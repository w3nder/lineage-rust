use crate::button::TextButton;
use crate::draw::DrawList;
use crate::font::BitmapFont;
use crate::window::Rect;

pub struct TextButtonContract {
    pub rect: Rect,
    pub id: u32,
    pub sysstring: u32,
    pub normal: &'static str,
    pub down: &'static str,
}

impl TextButtonContract {
    pub const fn rect_at(&self, origin: (i32, i32)) -> Rect {
        Rect::new(self.rect.x + origin.0, self.rect.y + origin.1, self.rect.w, self.rect.h)
    }

    pub fn build(&self, origin: (i32, i32), text: impl Fn(u32) -> String) -> TextButton {
        TextButton::new(self.rect_at(origin), self.normal, self.down, &text(self.sysstring))
    }

    pub fn hit(&self, button: &TextButton, x: i32, y: i32) -> Option<u32> {
        button.contains(x, y).then_some(self.id)
    }

    pub fn paint(&self, button: &TextButton, list: &mut DrawList, origin: (i32, i32), pressed: bool, font: &BitmapFont) {
        button.paint(list, origin.0, origin.1, pressed, font);
    }
}

pub const BUTTON_EC: TextButtonContract = TextButtonContract {
    rect: Rect::new(78, 372, 76, 23),
    id: 0xcfddceb0,
    sysstring: 1,
    normal: "L2UI_CH3.Button.btn1_normal",
    down: "L2UI_CH3.Button.btn1_normalon",
};

pub const BUTTON_F0: TextButtonContract = TextButtonContract {
    rect: Rect::new(158, 372, 76, 23),
    id: 0xcfddceb1,
    sysstring: 1,
    normal: "L2UI_CH3.Button.btn1_normal",
    down: "L2UI_CH3.Button.btn1_normalon",
};

use crate::frame::FrameMode;
use crate::frame_contract::{FrameContract, FrameTitle};

pub const FRAME_7C: FrameContract = FrameContract {
    rect: Rect::new(0, 0, 310, 20),
    id: 0xcfddcea0,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(2),
    close_button: true,
    minimize_button: false,
};
