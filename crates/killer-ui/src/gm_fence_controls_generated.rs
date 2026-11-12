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

pub const CHILD_204: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(115, 32, 80, 16),
    id: 0xffffd001,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x2b8, 0x1)],
    textures: &[],
};

pub const CHILD_208: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(215, 32, 80, 16),
    id: 0xffffd001,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x2b8, 0x1)],
    textures: &[],
};

pub const CHILD_20C: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(315, 32, 80, 16),
    id: 0xffffd001,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x2b8, 0x1)],
    textures: &[],
};

pub const CHILD_210: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(55, 68, 80, 16),
    id: 0xffffd001,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x2b8, 0x1)],
    textures: &[],
};

pub const CHILD_214: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(180, 68, 80, 16),
    id: 0xffffd001,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x2b8, 0x1)],
    textures: &[],
};

pub const BUTTON_1F8: TextButtonContract = TextButtonContract {
    rect: Rect::new(150, 100, 76, 23),
    id: 0xffffe001,
    sysstring: 140,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_1FC: TextButtonContract = TextButtonContract {
    rect: Rect::new(250, 100, 76, 23),
    id: 0xffffe002,
    sysstring: 141,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_200: TextButtonContract = TextButtonContract {
    rect: Rect::new(415, 30, 76, 23),
    id: 0xffffe003,
    sysstring: 1064,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

use crate::frame::FrameMode;
use crate::frame_contract::{FrameContract, FrameTitle};

pub const FRAME_7C: FrameContract = FrameContract {
    rect: Rect::new(0, 0, 500, 20),
    id: 0xeffff001,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(1105),
    close_button: true,
    minimize_button: false,
};
