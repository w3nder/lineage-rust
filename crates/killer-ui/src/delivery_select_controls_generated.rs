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
    kind: ChildKind::ComboBox,
    rect: Rect::new(39, 61, 178, 17),
    id: 0xe433,
    vtable: 0x101a1120,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x1f4, 0x5), (0x240, 0xffffffff), (0x244, 0xffffffff), (0x248, 0x11)],
    textures: &[],
};

pub const BUTTON_F0: TextButtonContract = TextButtonContract {
    rect: Rect::new(51, 172, 76, 23),
    id: 0xe434,
    sysstring: 140,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_F4: TextButtonContract = TextButtonContract {
    rect: Rect::new(131, 172, 76, 23),
    id: 0xe435,
    sysstring: 141,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

use crate::frame::FrameMode;
use crate::frame_contract::{FrameContract, FrameTitle};

pub const FRAME_7C: FrameContract = FrameContract {
    rect: Rect::new(0, 0, 256, 20),
    id: 0xe432,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(558),
    close_button: true,
    minimize_button: false,
};
