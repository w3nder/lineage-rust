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

pub const CHILD_E8: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(32, 107, 100, 12),
    id: 0xff0f1a01,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x2a2), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[],
};

pub const CHILD_EC: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(32, 126, 200, 12),
    id: 0xff0f1a03,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x2a3), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[],
};

pub const CHILD_F0: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(32, 145, 200, 12),
    id: 0xff0f1a04,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x2a4), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[],
};

pub const CHILD_F4: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(32, 164, 200, 12),
    id: 0xff0f1a05,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x2a5), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[],
};

pub const CHILD_F8: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(142, 107, 200, 12),
    id: 0xff0f1a02,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x4ba), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[],
};

pub const CHILD_FC: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(32, 210, 200, 12),
    id: 0xff0f1a06,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x2a7), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[],
};

pub const CHILD_100: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(32, 229, 200, 12),
    id: 0xff0f1a07,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x2ac), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[],
};

pub const CHILD_104: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(32, 248, 200, 12),
    id: 0xff0f1a08,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x2ad), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[],
};

pub const CHILD_108: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(32, 294, 200, 12),
    id: 0xff0f1a09,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x2ab), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[],
};

pub const CHILD_10C: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(32, 313, 200, 12),
    id: 0xff0f1a10,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x2ac), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[],
};

pub const CHILD_110: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(32, 332, 200, 12),
    id: 0xff0f1a10,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x2ad), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[],
};

pub const CHILD_114: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(21, 87, 200, 12),
    id: 0xff0f1a15,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x2a1), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[],
};

pub const CHILD_118: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(21, 191, 200, 12),
    id: 0xff0f1a16,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x2a6), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[],
};

pub const CHILD_11C: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(21, 275, 200, 12),
    id: 0xff0f1a17,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x2aa), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[],
};

pub const BUTTON_120: TextButtonContract = TextButtonContract {
    rect: Rect::new(51, 372, 76, 23),
    id: 0xff0f1a11,
    sysstring: 140,
    normal: "L2UI_ch3.Button.btn1_normal",
    down: "L2UI_ch3.Button.btn1_normalon",
};

pub const BUTTON_124: TextButtonContract = TextButtonContract {
    rect: Rect::new(131, 372, 76, 23),
    id: 0xff0f1a11,
    sysstring: 141,
    normal: "L2UI_ch3.Button.btn1_normal",
    down: "L2UI_ch3.Button.btn1_normalon",
};

use crate::frame::FrameMode;
use crate::frame_contract::{FrameContract, FrameTitle};

pub const FRAME_7C: FrameContract = FrameContract {
    rect: Rect::new(0, 0, 256, 20),
    id: 0xff0f1b01,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(668),
    close_button: true,
    minimize_button: false,
};
