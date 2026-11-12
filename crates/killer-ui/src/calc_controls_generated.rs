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
    rect: Rect::new(34, 40, 188, 17),
    id: 0xff01ff02,
    vtable: 0x101d17e0,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x214, 0x3), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x2b8, 0x1)],
    textures: &[],
};

pub const BUTTON_13C: TextButtonContract = TextButtonContract {
    rect: Rect::new(91, 182, 76, 23),
    id: 0xff01ff00,
    sysstring: 646,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

use crate::frame::FrameMode;
use crate::frame_contract::{FrameContract, FrameTitle};

pub const FRAME_7C: FrameContract = FrameContract {
    rect: Rect::new(0, 0, 256, 20),
    id: 0x0,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(669),
    close_button: true,
    minimize_button: false,
};
