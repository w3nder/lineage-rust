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

pub const CHILD_E8: ChildContract = ChildContract {
    kind: ChildKind::ListCtrl,
    rect: Rect::new(7, 49, 586, 237),
    id: 0xf00a1,
    vtable: 0x101a6fa8,
    registration: (0x22, 0x0, 0x0),
    handles: &[0x230, 0x234, 0x238],
    state: &[(0x23c, 0xd), (0x240, 0x7d0), (0x244, 0x5), (0x250, 0xffffffff), (0x254, 0xffffffff), (0x258, 0xf), (0x25c, 0x13), (0x264, 0x3), (0x268, 0x1), (0x26c, 0x11)],
    textures: &[],
};

pub const COLUMNS_E8: &[ColumnContract] = &[
    ColumnContract { width: 212, height: 19, sysstring: 1200, flag_c: 1, flag_10: 0, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
    ColumnContract { width: 130, height: 19, sysstring: 1201, flag_c: 1, flag_10: 0, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
    ColumnContract { width: 52, height: 19, sysstring: 922, flag_c: 1, flag_10: 1, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
    ColumnContract { width: 40, height: 19, sysstring: 1202, flag_c: 1, flag_10: 0, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
    ColumnContract { width: 152, height: 19, sysstring: 1203, flag_c: 1, flag_10: 0, textures: &[(0x18, "L2UI_CH3.ListCtrl.tab1"), (0x818, "L2UI_CH3.ListCtrl.tab2"), (0x1018, "L2UI_CH3.ListCtrl.tab3"), (0x1818, "L2UI_CH3.ListCtrl.tab1_down"), (0x2018, "L2UI_CH3.ListCtrl.tab2_down"), (0x2818, "L2UI_CH3.ListCtrl.tab3_down")] },
];

pub const LIST_E8: ListContract = ListContract { child: CHILD_E8, columns: COLUMNS_E8 };

pub const BUTTON_F0: TextButtonContract = TextButtonContract {
    rect: Rect::new(518, 297, 76, 23),
    id: 0xf00a0,
    sysstring: 860,
    normal: "L2UI_CH3.Button.btn1_normal",
    down: "L2UI_CH3.Button.btn1_normalon",
};

use crate::frame::FrameMode;
use crate::frame_contract::{FrameContract, FrameTitle};

pub const FRAME_7C: FrameContract = FrameContract {
    rect: Rect::new(0, 0, 600, 20),
    id: 0x0,
    mode: FrameMode::Title,
    title: FrameTitle::Sysstring(1204),
    close_button: true,
    minimize_button: false,
};
