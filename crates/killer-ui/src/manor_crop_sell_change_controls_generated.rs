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

use crate::list_ctrl::{ColumnContract, ListContract};

pub const CHILD_200: ChildContract = ChildContract {
    kind: ChildKind::ComboBox,
    rect: Rect::new(123, 216, 138, 17),
    id: 0xf0000a30,
    vtable: 0x101a1120,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x1f4, 0x5), (0x244, 0xffffffff), (0x248, 0x11)],
    textures: &[],
};

pub const CHILD_204: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(123, 241, 68, 17),
    id: 0xf0000a20,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x254, 0xa), (0x2b8, 0x1)],
    textures: &[],
};

pub const CHILD_208: ChildContract = ChildContract {
    kind: ChildKind::ListCtrl,
    rect: Rect::new(7, 92, 322, 109),
    id: 0xf0000a00,
    vtable: 0x101a6fa8,
    registration: (0x2, 0x0, 0x0),
    handles: &[0x230, 0x234, 0x238],
    state: &[(0x23c, 0x5), (0x240, 0x3e8), (0x244, 0x4), (0x250, 0xffffffff), (0x254, 0xffffffff), (0x258, 0xf), (0x25c, 0x13), (0x264, 0x3), (0x268, 0x1), (0x26c, 0x12)],
    textures: &[],
};

pub const COLUMNS_208: &[ColumnContract] = &[
    ColumnContract { width: 115, height: 19, sysstring: 1153, flag_c: 1, flag_10: 0, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
    ColumnContract { width: 70, height: 19, sysstring: 1154, flag_c: 1, flag_10: 1, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
    ColumnContract { width: 70, height: 19, sysstring: 1126, flag_c: 1, flag_10: 1, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
    ColumnContract { width: 67, height: 19, sysstring: 1127, flag_c: 1, flag_10: 0, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
];

pub const LIST_208: ListContract = ListContract { child: CHILD_208, columns: COLUMNS_208 };

pub const BUTTON_1F4: TextButtonContract = TextButtonContract {
    rect: Rect::new(196, 240, 66, 21),
    id: 0xf0000a10,
    sysstring: 1158,
    normal: "L2UI_CH3.Button.smallbutton2",
    down: "L2UI_CH3.Button.smallbutton2_down",
};

pub const BUTTON_1F8: TextButtonContract = TextButtonContract {
    rect: Rect::new(91, 277, 76, 23),
    id: 0xf0000a11,
    sysstring: 140,
    normal: "L2UI_CH3.Button.btn1_normal",
    down: "L2UI_CH3.Button.btn1_normalon",
};

pub const BUTTON_1FC: TextButtonContract = TextButtonContract {
    rect: Rect::new(171, 277, 76, 23),
    id: 0xf0000a12,
    sysstring: 141,
    normal: "L2UI_CH3.Button.btn1_normal",
    down: "L2UI_CH3.Button.btn1_normalon",
};

use crate::frame::FrameMode;
use crate::frame_contract::{FrameContract, FrameTitle};

pub const FRAME_7C: FrameContract = FrameContract {
    rect: Rect::new(0, 0, 336, 20),
    id: 0xf0000a40,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(1152),
    close_button: true,
    minimize_button: false,
};
