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

pub const CHILD_EC: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(25, 50, 150, 16),
    id: 0xffffe001,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x254, 0xa), (0x2b8, 0x1)],
    textures: &[],
};

pub const CHILD_F0: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(265, 50, 150, 16),
    id: 0xffffe002,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x254, 0xa), (0x2b8, 0x1)],
    textures: &[],
};

pub const CHILD_F4: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(100, 418, 25, 16),
    id: 0xffffe009,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x254, 0x1), (0x2b8, 0x1)],
    textures: &[],
};

pub const CHILD_F8: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(170, 418, 25, 16),
    id: 0xffffe00a,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x254, 0x1), (0x2b8, 0x1)],
    textures: &[],
};

pub const CHILD_FC: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(25, 165, 250, 16),
    id: 0xffffe003,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x2b8, 0x1)],
    textures: &[],
};

pub const CHILD_100: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(350, 165, 250, 16),
    id: 0xffffe004,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x2b8, 0x1)],
    textures: &[],
};

pub const CHILD_104: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(25, 105, 150, 16),
    id: 0xffffe005,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x2b8, 0x1)],
    textures: &[],
};

pub const CHILD_108: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(265, 105, 150, 16),
    id: 0xffffe006,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x2b8, 0x1)],
    textures: &[],
};

pub const CHILD_10C: ChildContract = ChildContract {
    kind: ChildKind::EditBox,
    rect: Rect::new(80, 448, 500, 16),
    id: 0xffffe008,
    vtable: 0x101a2730,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x21c, 0x1), (0x220, 0x1), (0x224, 0x1), (0x238, 0x1), (0x248, 0x108), (0x254, 0x32), (0x2b8, 0x1)],
    textures: &[],
};

pub const CHILD_148: ChildContract = ChildContract {
    kind: ChildKind::ListCtrl,
    rect: Rect::new(25, 250, 300, 150),
    id: 0xffffc001,
    vtable: 0x101a6fa8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x1f4, 0x1), (0x23c, 0x9), (0x240, 0x9), (0x244, 0x3), (0x250, 0xffffffff), (0x254, 0xffffffff), (0x258, 0xf), (0x25c, 0x13), (0x264, 0x3), (0x268, 0x1), (0x26c, 0x11)],
    textures: &[],
};

pub const COLUMNS_148: &[ColumnContract] = &[
    ColumnContract { width: 120, height: 19, sysstring: 872, flag_c: 1, flag_10: 0, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
    ColumnContract { width: 60, height: 19, sysstring: 537, flag_c: 1, flag_10: 0, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
    ColumnContract { width: 120, height: 19, sysstring: 391, flag_c: 1, flag_10: 0, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
];

pub const LIST_148: ListContract = ListContract { child: CHILD_148, columns: COLUMNS_148 };

pub const CHILD_14C: ChildContract = ChildContract {
    kind: ChildKind::ListCtrl,
    rect: Rect::new(355, 250, 300, 150),
    id: 0xffffc002,
    vtable: 0x101a6fa8,
    registration: (0x2, 0x0, 0x0),
    handles: &[],
    state: &[(0x1f4, 0x1), (0x23c, 0x9), (0x240, 0x9), (0x244, 0x3), (0x250, 0xffffffff), (0x254, 0xffffffff), (0x258, 0xf), (0x25c, 0x13), (0x264, 0x3), (0x268, 0x1), (0x26c, 0x11)],
    textures: &[],
};

pub const COLUMNS_14C: &[ColumnContract] = &[
    ColumnContract { width: 120, height: 19, sysstring: 872, flag_c: 1, flag_10: 0, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
    ColumnContract { width: 60, height: 19, sysstring: 537, flag_c: 1, flag_10: 0, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
    ColumnContract { width: 120, height: 19, sysstring: 391, flag_c: 1, flag_10: 0, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
];

pub const LIST_14C: ListContract = ListContract { child: CHILD_14C, columns: COLUMNS_14C };

pub const BUTTON_110: TextButtonContract = TextButtonContract {
    rect: Rect::new(25, 470, 76, 23),
    id: 0xffffd003,
    sysstring: 1079,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_114: TextButtonContract = TextButtonContract {
    rect: Rect::new(177, 90, 76, 23),
    id: 0xffffd004,
    sysstring: 1070,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_118: TextButtonContract = TextButtonContract {
    rect: Rect::new(177, 114, 76, 23),
    id: 0xffffd005,
    sysstring: 1071,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_11C: TextButtonContract = TextButtonContract {
    rect: Rect::new(417, 90, 76, 23),
    id: 0xffffd006,
    sysstring: 1070,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_120: TextButtonContract = TextButtonContract {
    rect: Rect::new(417, 114, 76, 23),
    id: 0xffffd007,
    sysstring: 1071,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_124: TextButtonContract = TextButtonContract {
    rect: Rect::new(590, 445, 76, 23),
    id: 0xffffd009,
    sysstring: 1078,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_128: TextButtonContract = TextButtonContract {
    rect: Rect::new(580, 190, 76, 23),
    id: 0xffffd00a,
    sysstring: 1075,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_12C: TextButtonContract = TextButtonContract {
    rect: Rect::new(495, 30, 76, 23),
    id: 0xffffd00b,
    sysstring: 1068,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_130: TextButtonContract = TextButtonContract {
    rect: Rect::new(495, 190, 76, 23),
    id: 0xffffd00c,
    sysstring: 1073,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_134: TextButtonContract = TextButtonContract {
    rect: Rect::new(200, 415, 76, 23),
    id: 0xffffd00d,
    sysstring: 1077,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_138: TextButtonContract = TextButtonContract {
    rect: Rect::new(110, 470, 76, 23),
    id: 0xffffd00e,
    sysstring: 1080,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_13C: TextButtonContract = TextButtonContract {
    rect: Rect::new(550, 102, 76, 23),
    id: 0xffffd00f,
    sysstring: 1081,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_140: TextButtonContract = TextButtonContract {
    rect: Rect::new(195, 470, 76, 23),
    id: 0xffffd010,
    sysstring: 1083,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

pub const BUTTON_144: TextButtonContract = TextButtonContract {
    rect: Rect::new(280, 470, 76, 23),
    id: 0xffffd011,
    sysstring: 1084,
    normal: "L2UI_ch3.button.btn1_normal",
    down: "L2UI_ch3.button.btn1_normalon",
};

use crate::frame::FrameMode;
use crate::frame_contract::{FrameContract, FrameTitle};

pub const FRAME_7C: FrameContract = FrameContract {
    rect: Rect::new(0, 0, 700, 20),
    id: 0xfffff001,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(1104),
    close_button: true,
    minimize_button: true,
};
