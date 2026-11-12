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

pub const CHILD_F4: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(21, 52, 100, 12),
    id: 0xff00a1,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[0x274, 0x278],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x352), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[(0x274, "L2UI_CH3.RadioCtrl.RadioButton2"), (0x278, "L2UI_CH3.RadioCtrl.RadioButton1")],
};

pub const CHILD_F8: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(21, 71, 100, 12),
    id: 0xff00a2,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[0x274, 0x278],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x353), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[(0x274, "L2UI_CH3.RadioCtrl.RadioButton2"), (0x278, "L2UI_CH3.RadioCtrl.RadioButton1")],
};

pub const CHILD_FC: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(21, 90, 100, 12),
    id: 0xff00a3,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[0x274, 0x278],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x354), (0x298, 0x1), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[(0x274, "L2UI_CH3.RadioCtrl.RadioButton2"), (0x278, "L2UI_CH3.RadioCtrl.RadioButton1")],
};

pub const CHILD_100: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(21, 109, 100, 12),
    id: 0xff00a4,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[0x274, 0x278],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x355), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[(0x274, "L2UI_CH3.RadioCtrl.RadioButton2"), (0x278, "L2UI_CH3.RadioCtrl.RadioButton1")],
};

pub const CHILD_104: ChildContract = ChildContract {
    kind: ChildKind::CheckBox,
    rect: Rect::new(21, 128, 100, 12),
    id: 0xff00a5,
    vtable: 0x101a0ce8,
    registration: (0x2, 0x0, 0x0),
    handles: &[0x274, 0x278],
    state: &[(0x20c, 0xffffffff), (0x224, 0xffffffff), (0x290, 0x356), (0x2a4, 0xc), (0x2a8, 0xc)],
    textures: &[(0x274, "L2UI_CH3.RadioCtrl.RadioButton2"), (0x278, "L2UI_CH3.RadioCtrl.RadioButton1")],
};

pub const BUTTON_EC: TextButtonContract = TextButtonContract {
    rect: Rect::new(90, 372, 76, 23),
    id: 0xff00a7,
    sysstring: 140,
    normal: "L2UI_CH3.BUTTON.Btn1_normal",
    down: "L2UI_CH3.BUTTON.Btn1_normalon",
};

use crate::frame::FrameMode;
use crate::frame_contract::{FrameContract, FrameTitle};

pub const FRAME_7C: FrameContract = FrameContract {
    rect: Rect::new(0, 0, 256, 21),
    id: 0xff00a6,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(848),
    close_button: true,
    minimize_button: false,
};
