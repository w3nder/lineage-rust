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

use crate::child_contract::{ChildContract, ChildKind};

pub const CHILD_EC: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(7, 357, 296, 17),
    id: 0xf0a0b2,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x214, 0x1), (0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x254, 0x400), (0x2b8, 0x1)],
    textures: &[],
};

pub const BUTTON_F8: TextButtonContract = TextButtonContract {
    rect: Rect::new(80, 383, 76, 23),
    id: 0xf0a0b4,
    sysstring: 473,
    normal: "L2ui_ch3.button.btn1_normal",
    down: "L2ui_ch3.button.btn1_normalon",
};

pub const BUTTON_FC: TextButtonContract = TextButtonContract {
    rect: Rect::new(160, 383, 76, 23),
    id: 0xf0abb6,
    sysstring: 849,
    normal: "L2ui_ch3.button.btn1_normal",
    down: "L2ui_ch3.button.btn1_normalon",
};

use crate::frame::FrameMode;
use crate::frame_contract::{FrameContract, FrameTitle};

pub const FRAME_7C: FrameContract = FrameContract {
    rect: Rect::new(0, 0, 310, 20),
    id: 0xf0a0b5,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(470),
    close_button: true,
    minimize_button: false,
};
